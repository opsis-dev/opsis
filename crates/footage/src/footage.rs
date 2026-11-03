use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use image::{DynamicImage, RgbImage};
use jpeg_encoder::{ColorType, Encoder, SamplingFactor};
use opsis_image::ops::redact;

use crate::FootageError;
use crate::chain::{FootageSource, JUMP_US, MATCH_TOLERANCE_US, SEEK_MARGIN_US, file_time};
use crate::decoder::{Decoder, FootageTools, RawFrame};
use crate::terminal::{TerminalFootage, TerminalState};

const QUALITY: u8 = 92;
const KEPT_PICTURES: usize = 6;
const KEPT_FRAMES: usize = 6;
pub const IDLE: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct Shown {
    pub pts_us: i64,
    pub image: RgbImage,
}

#[derive(Debug)]
struct SourceState {
    source: FootageSource,
    decoder: Option<Decoder>,
    seen: VecDeque<RawFrame>,
    ahead: Option<RawFrame>,
    first_pts: Option<i64>,
    ended: bool,
    pictures: VecDeque<(i64, Arc<Vec<u8>>)>,
    last_used: Instant,
}

fn redacted(source: &FootageSource, frame: &RawFrame) -> RgbImage {
    let covering: Vec<_> = source
        .redactions
        .iter()
        .filter(|r| r.covers(frame.pts_us))
        .collect();
    if covering.is_empty() {
        return frame.image.clone();
    }
    covering
        .into_iter()
        .fold(
            DynamicImage::ImageRgb8(frame.image.clone()),
            |picture, r| redact(&picture, &r.redact),
        )
        .into_rgb8()
}

pub fn jpeg(image: &RgbImage) -> Result<Vec<u8>, FootageError> {
    let too_big = || FootageError::Encode("The frame is too big for a picture.".to_owned());
    let width = u16::try_from(image.width()).map_err(|_| too_big())?;
    let height = u16::try_from(image.height()).map_err(|_| too_big())?;
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, QUALITY);
    encoder.set_sampling_factor(SamplingFactor::R_4_4_4);
    encoder
        .encode(image.as_raw(), width, height, ColorType::Rgb)
        .map_err(|e| FootageError::Encode(e.to_string()))?;
    Ok(out)
}

impl SourceState {
    fn new(source: FootageSource) -> Self {
        Self {
            source,
            decoder: None,
            seen: VecDeque::new(),
            ahead: None,
            first_pts: None,
            ended: false,
            pictures: VecDeque::new(),
            last_used: Instant::now(),
        }
    }

    fn stop(&mut self) {
        self.decoder = None;
        self.seen.clear();
        self.ahead = None;
        self.first_pts = None;
        self.ended = false;
    }

    fn start_at(
        &mut self,
        tools: &FootageTools,
        starts: &AtomicU32,
        start_us: i64,
    ) -> Result<(), FootageError> {
        self.stop();
        self.decoder = Some(Decoder::start(tools, &self.source, start_us)?);
        starts.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn oldest_is_first(&self) -> bool {
        matches!((self.seen.front(), self.first_pts), (Some(f), Some(first)) if f.pts_us == first)
    }

    fn needs_restart(&self, target: i64) -> bool {
        let Some(decoder) = &self.decoder else {
            return true;
        };
        let before_kept = self.seen.front().is_some_and(|f| f.pts_us > target)
            && !(decoder.start_us == 0 && self.oldest_is_first());
        let newest = self.ahead.as_ref().or_else(|| self.seen.back());
        let jumped = newest.is_some_and(|f| target > f.pts_us + JUMP_US);
        before_kept || jumped
    }

    fn keep(&mut self, frame: RawFrame) {
        if self.seen.len() == KEPT_FRAMES {
            self.seen.pop_front();
        }
        self.seen.push_back(frame);
    }

    fn advance(&mut self, target: i64) -> Result<(), FootageError> {
        loop {
            if let Some(ahead) = &self.ahead {
                if ahead.pts_us > target {
                    return Ok(());
                }
                if let Some(frame) = self.ahead.take() {
                    self.keep(frame);
                }
                continue;
            }
            if self.ended {
                return Ok(());
            }
            let Some(decoder) = self.decoder.as_mut() else {
                return Ok(());
            };
            match decoder.next_frame()? {
                Some(frame) => {
                    self.first_pts.get_or_insert(frame.pts_us);
                    self.ahead = Some(frame);
                }
                None => self.ended = true,
            }
        }
    }

    fn pick(&self, target: i64) -> Option<i64> {
        self.seen
            .iter()
            .rev()
            .find(|f| f.pts_us <= target)
            .or_else(|| self.seen.front())
            .or(self.ahead.as_ref())
            .map(|f| f.pts_us)
    }

    fn locate(
        &mut self,
        tools: &FootageTools,
        starts: &AtomicU32,
        time_us: i64,
    ) -> Result<i64, FootageError> {
        let target = file_time(&self.source, time_us) + MATCH_TOLERANCE_US;
        if self.needs_restart(target) {
            self.start_at(tools, starts, (target - SEEK_MARGIN_US).max(0))?;
        }
        self.advance(target)?;
        let overshot =
            self.seen.is_empty() && self.decoder.as_ref().is_some_and(|d| d.start_us > 0);
        if overshot {
            self.start_at(tools, starts, 0)?;
            self.advance(target)?;
        }
        self.pick(target).ok_or(FootageError::NoFrames)
    }

    fn frame_at(&self, pts_us: i64) -> Result<&RawFrame, FootageError> {
        self.seen
            .iter()
            .chain(self.ahead.iter())
            .find(|f| f.pts_us == pts_us)
            .ok_or(FootageError::NoFrames)
    }

    fn shown(
        &mut self,
        tools: &FootageTools,
        starts: &AtomicU32,
        time_us: i64,
    ) -> Result<Shown, FootageError> {
        let pts_us = self.locate(tools, starts, time_us)?;
        let image = redacted(&self.source, self.frame_at(pts_us)?);
        Ok(Shown { pts_us, image })
    }

    fn picture(
        &mut self,
        tools: &FootageTools,
        starts: &AtomicU32,
        time_us: i64,
    ) -> Result<Arc<Vec<u8>>, FootageError> {
        let pts_us = self.locate(tools, starts, time_us)?;
        if let Some((_, bytes)) = self.pictures.iter().find(|(at, _)| *at == pts_us) {
            return Ok(Arc::clone(bytes));
        }
        let bytes = Arc::new(jpeg(&redacted(&self.source, self.frame_at(pts_us)?))?);
        if self.pictures.len() == KEPT_PICTURES {
            self.pictures.pop_front();
        }
        self.pictures.push_back((pts_us, Arc::clone(&bytes)));
        Ok(bytes)
    }
}

#[derive(Debug)]
pub struct Footage {
    tools: FootageTools,
    sources: HashMap<String, Mutex<SourceState>>,
    terminals: HashMap<String, TerminalState>,
    starts: AtomicU32,
}

impl Footage {
    pub fn new(
        tools: FootageTools,
        sources: impl IntoIterator<Item = (String, FootageSource)>,
    ) -> Self {
        Self {
            tools,
            sources: sources
                .into_iter()
                .map(|(id, source)| (id, Mutex::new(SourceState::new(source))))
                .collect(),
            terminals: HashMap::new(),
            starts: AtomicU32::new(0),
        }
    }

    #[must_use]
    pub fn with_terminals(
        mut self,
        terminals: impl IntoIterator<Item = (String, TerminalFootage)>,
    ) -> Self {
        self.terminals.extend(
            terminals
                .into_iter()
                .map(|(id, source)| (id, TerminalState::new(source))),
        );
        self
    }

    pub fn terminal_frames(&self, id: &str) -> Result<Arc<Vec<u8>>, FootageError> {
        self.terminals
            .get(id)
            .ok_or_else(|| FootageError::Unknown(id.to_owned()))?
            .frames()
    }

    pub fn decoder_starts(&self) -> u32 {
        self.starts.load(Ordering::Relaxed)
    }

    pub fn running_decoders(&self) -> usize {
        self.sources
            .values()
            .filter(|state| state.lock().is_ok_and(|s| s.decoder.is_some()))
            .count()
    }

    pub fn end_idle(&self, idle: Duration) {
        for state in self.sources.values() {
            if let Ok(mut state) = state.try_lock()
                && state.decoder.is_some()
                && state.last_used.elapsed() >= idle
            {
                state.stop();
            }
        }
    }

    fn with_source<T>(
        &self,
        id: &str,
        work: impl FnOnce(&mut SourceState) -> Result<T, FootageError>,
    ) -> Result<T, FootageError> {
        self.end_idle(IDLE);
        let state = self
            .sources
            .get(id)
            .ok_or_else(|| FootageError::Unknown(id.to_owned()))?;
        let mut state = state
            .lock()
            .map_err(|_| FootageError::Decode("An earlier frame failed.".to_owned()))?;
        state.last_used = Instant::now();
        let result = work(&mut state);
        if result.is_err() {
            state.stop();
        }
        result
    }

    pub fn shown(&self, id: &str, time_us: i64) -> Result<Shown, FootageError> {
        self.with_source(id, |state| state.shown(&self.tools, &self.starts, time_us))
    }

    pub fn picture(&self, id: &str, time_us: i64) -> Result<Arc<Vec<u8>>, FootageError> {
        self.with_source(id, |state| {
            state.picture(&self.tools, &self.starts, time_us)
        })
    }
}
