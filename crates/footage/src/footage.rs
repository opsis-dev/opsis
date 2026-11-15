use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use image::{DynamicImage, RgbImage};
use opsis_image::ops::redact;

use crate::FootageError;
use crate::chain::{FootageSource, JUMP_US, MATCH_TOLERANCE_US, SEEK_MARGIN_US, file_time};
use crate::decoder::{Decoder, FootageTools, RawFrame};
use crate::terminal::{TerminalFootage, TerminalState};

const KEPT_FRAMES: usize = 6;
const KEPT_DECODERS: usize = 4;
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

impl SourceState {
    fn new(source: FootageSource) -> Self {
        Self {
            source,
            decoder: None,
            seen: VecDeque::new(),
            ahead: None,
            first_pts: None,
            ended: false,
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

    fn newest_pts(&self) -> Option<i64> {
        self.ahead
            .as_ref()
            .or_else(|| self.seen.back())
            .map(|f| f.pts_us)
    }

    fn reach(&self, target: i64) -> Option<i64> {
        if self.needs_restart(target) {
            return None;
        }
        Some(
            self.newest_pts()
                .map_or(0, |newest| (target - newest).max(0)),
        )
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
}

#[derive(Debug)]
struct Pool {
    source: FootageSource,
    idle: Mutex<Vec<SourceState>>,
}

impl Pool {
    const fn new(source: FootageSource) -> Self {
        Self {
            source,
            idle: Mutex::new(Vec::new()),
        }
    }

    fn check_out(&self, time_us: i64) -> Result<SourceState, FootageError> {
        let target = file_time(&self.source, time_us) + MATCH_TOLERANCE_US;
        let mut idle = self
            .idle
            .lock()
            .map_err(|_| FootageError::Decode("An earlier frame failed.".to_owned()))?;
        let closest = idle
            .iter()
            .enumerate()
            .filter_map(|(index, state)| state.reach(target).map(|reach| (reach, index)))
            .min()
            .map(|(_, index)| index);
        Ok(match closest {
            Some(index) => idle.swap_remove(index),
            None => SourceState::new(self.source.clone()),
        })
    }

    fn check_in(&self, state: SourceState) {
        let Ok(mut idle) = self.idle.lock() else {
            return;
        };
        idle.push(state);
        if idle.len() > KEPT_DECODERS
            && let Some(oldest) = idle
                .iter()
                .enumerate()
                .min_by_key(|(_, s)| s.last_used)
                .map(|(index, _)| index)
        {
            idle.swap_remove(oldest);
        }
    }
}

#[derive(Debug)]
pub struct Footage {
    tools: FootageTools,
    sources: HashMap<String, Pool>,
    terminals: HashMap<String, TerminalState>,
    starts: AtomicU32,
    stalled: AtomicBool,
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
                .map(|(id, source)| (id, Pool::new(source)))
                .collect(),
            terminals: HashMap::new(),
            starts: AtomicU32::new(0),
            stalled: AtomicBool::new(false),
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
            .frames(&self.tools)
    }

    pub fn decoder_starts(&self) -> u32 {
        self.starts.load(Ordering::Relaxed)
    }

    pub fn running_decoders(&self) -> usize {
        self.sources
            .values()
            .map(|pool| {
                pool.idle.lock().map_or(0, |idle| {
                    idle.iter().filter(|s| s.decoder.is_some()).count()
                })
            })
            .sum()
    }

    pub fn end_idle(&self, idle: Duration) {
        for pool in self.sources.values() {
            if let Ok(mut states) = pool.idle.try_lock() {
                states.retain(|state| state.last_used.elapsed() < idle);
            }
        }
    }

    fn with_source<T>(
        &self,
        id: &str,
        time_us: i64,
        work: impl FnOnce(&mut SourceState) -> Result<T, FootageError>,
    ) -> Result<T, FootageError> {
        self.end_idle(IDLE);
        let pool = self
            .sources
            .get(id)
            .ok_or_else(|| FootageError::Unknown(id.to_owned()))?;
        let mut state = pool.check_out(time_us)?;
        state.last_used = Instant::now();
        let result = work(&mut state);
        if result.is_ok() {
            pool.check_in(state);
        }
        result
    }

    pub fn shown(&self, id: &str, time_us: i64) -> Result<Shown, FootageError> {
        let shown = self.with_source(id, time_us, |state| {
            state.shown(&self.tools, &self.starts, time_us)
        });
        if matches!(shown, Err(FootageError::Stalled)) {
            self.stalled.store(true, Ordering::Relaxed);
        }
        shown
    }

    pub fn stalled(&self) -> bool {
        self.stalled.load(Ordering::Relaxed)
    }
}
