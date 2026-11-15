use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use image::{DynamicImage, RgbImage};
use opsis_image::ops::redact;

use crate::FootageError;
use crate::chain::{
    FootageSource, JUMP_US, MATCH_TOLERANCE_US, PixelLayout, SEEK_MARGIN_US, file_time,
    snapped_time, window_frames, window_span_us,
};
use crate::decoder::{Decoder, FootageTools, RawFrame};
use crate::terminal::{TerminalFootage, TerminalState};

pub const IDLE: Duration = Duration::from_secs(30);
const STILL_FPS: f64 = 30.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.pixels
    }
}

impl Frame {
    pub fn rgb(&self) -> RgbImage {
        let rgb = rgb_of(&self.pixels);
        RgbImage::from_raw(self.width, self.height, rgb).unwrap_or_default()
    }
}

fn rgb_of(bgra: &[u8]) -> Vec<u8> {
    bgra.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[b, g, r, _]| [*r, *g, *b])
        .collect()
}

fn bgra_of(rgb: &[u8]) -> Vec<u8> {
    rgb.as_chunks::<3>()
        .0
        .iter()
        .flat_map(|[r, g, b]| [*b, *g, *r, u8::MAX])
        .collect()
}

#[derive(Debug, Clone)]
pub struct Shown {
    pub pts_us: i64,
    pub frame: Arc<Frame>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    pub ahead_frames: usize,
    pub film_fps: f64,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            ahead_frames: 0,
            film_fps: STILL_FPS,
        }
    }
}

fn redacted(source: &FootageSource, raw: RawFrame) -> Frame {
    let covering: Vec<_> = source
        .redactions
        .iter()
        .filter(|r| r.covers(raw.pts_us))
        .collect();
    if covering.is_empty() {
        return Frame {
            width: raw.width,
            height: raw.height,
            pixels: raw.pixels,
        };
    }
    let Some(plain) = RgbImage::from_raw(raw.width, raw.height, rgb_of(&raw.pixels)) else {
        return Frame {
            width: raw.width,
            height: raw.height,
            pixels: raw.pixels,
        };
    };
    let hidden = covering
        .into_iter()
        .fold(DynamicImage::ImageRgb8(plain), |picture, r| {
            redact(&picture, &r.redact)
        })
        .into_rgb8();
    Frame {
        width: raw.width,
        height: raw.height,
        pixels: bgra_of(hidden.as_raw()),
    }
}

#[derive(Debug)]
struct Reader {
    decoder: Option<Decoder>,
    frames: BTreeMap<i64, Arc<Frame>>,
    first_pts: Option<i64>,
    newest: Option<i64>,
    ended: bool,
    earliest: i64,
    floor: i64,
    asked: BTreeMap<u32, i64>,
    last_asker: Option<u32>,
    last_used: Instant,
}

impl Reader {
    fn new() -> Self {
        Self {
            decoder: None,
            frames: BTreeMap::new(),
            first_pts: None,
            newest: None,
            ended: false,
            earliest: i64::MAX,
            floor: i64::MIN,
            asked: BTreeMap::new(),
            last_asker: None,
            last_used: Instant::now(),
        }
    }

    fn start_at(
        &mut self,
        tools: &FootageTools,
        source: &FootageSource,
        starts: &AtomicU32,
        start_us: i64,
    ) -> Result<(), FootageError> {
        *self = Self::new();
        self.decoder = Some(Decoder::start(tools, source, start_us, PixelLayout::Bgra)?);
        starts.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn oldest(&self) -> Option<i64> {
        self.frames.keys().next().copied()
    }

    const fn pending(&self) -> bool {
        !self.asked.is_empty()
    }

    fn reach(&self, target: i64, jump: i64) -> Option<i64> {
        let decoder = self.decoder.as_ref()?;
        let from_start = decoder.start_us == 0 && self.oldest() == self.first_pts;
        let start = self.oldest().unwrap_or(decoder.start_us);
        if start > target && !from_start {
            return None;
        }
        let newest = self.newest.unwrap_or(decoder.start_us);
        if self.ended || target <= newest {
            return Some(0);
        }
        (target - newest <= jump).then_some(target - newest)
    }

    fn prune(&mut self, cut: i64) {
        let kept = self.frames.range(..=cut).next_back().map(|(pts, _)| *pts);
        if let Some(kept) = kept {
            self.frames = self.frames.split_off(&kept);
        }
    }

    fn cut(&self, target: i64, span: i64) -> i64 {
        self.floor
            .max(self.earliest.saturating_sub(span))
            .min(target)
    }

    fn advance(
        &mut self,
        source: &FootageSource,
        target: i64,
        span: i64,
    ) -> Result<(), FootageError> {
        while !self.ended && self.newest.is_none_or(|newest| newest <= target) {
            let Some(decoder) = self.decoder.as_mut() else {
                return Ok(());
            };
            match decoder.next_frame()? {
                Some(raw) => {
                    let pts = raw.pts_us;
                    self.first_pts.get_or_insert(pts);
                    let frame = Arc::new(redacted(source, raw));
                    self.frames.insert(pts, frame);
                    self.newest = Some(self.newest.map_or(pts, |n| n.max(pts)));
                    self.prune(self.cut(target, span));
                }
                None => self.ended = true,
            }
        }
        Ok(())
    }

    fn pick(&self, target: i64) -> Option<(i64, Arc<Frame>)> {
        self.frames
            .range(..=target)
            .next_back()
            .or_else(|| self.frames.iter().next())
            .map(|(pts, frame)| (*pts, Arc::clone(frame)))
    }

    fn show(
        &mut self,
        read: &Reading<'_>,
        target: i64,
        asker: Option<u32>,
    ) -> Result<Shown, FootageError> {
        self.earliest = self.earliest.min(target);
        self.advance(read.source, target, read.span)?;
        let overshot = self.frames.range(..=target).next().is_none()
            && self.decoder.as_ref().is_some_and(|d| d.start_us > 0);
        if overshot {
            self.start_at(read.tools, read.source, read.starts, 0)?;
            self.earliest = target;
            self.advance(read.source, target, read.span)?;
        }
        let (pts_us, frame) = self.pick(target).ok_or(FootageError::NoFrames)?;
        if let Some(asker) = asker {
            self.asked
                .entry(asker)
                .and_modify(|earliest| *earliest = (*earliest).min(pts_us))
                .or_insert(pts_us);
            self.last_asker = Some(self.last_asker.map_or(asker, |last| last.max(asker)));
        }
        Ok(Shown { pts_us, frame })
    }

    fn written(&mut self, frame: u32) {
        let later = self.asked.split_off(&frame.saturating_add(1));
        let done = std::mem::replace(&mut self.asked, later);
        if let Some((_, earliest)) = done.into_iter().next_back() {
            self.floor = self.floor.max(earliest);
            self.prune(self.floor);
        }
    }

    fn done(&self, written: u32, ahead: usize) -> bool {
        let ahead = u32::try_from(ahead).unwrap_or(u32::MAX);
        !self.pending()
            && self
                .last_asker
                .is_some_and(|last| written > last.saturating_add(ahead))
    }
}

struct Reading<'a> {
    tools: &'a FootageTools,
    source: &'a FootageSource,
    starts: &'a AtomicU32,
    span: i64,
}

#[derive(Debug)]
struct Place {
    span_us: i64,
    ahead: usize,
    readers: Vec<Reader>,
}

#[derive(Debug)]
struct Held {
    source: FootageSource,
    place: Mutex<Place>,
}

fn span_of(source: &FootageSource, window: Window) -> i64 {
    window_span_us(
        window.ahead_frames,
        window.film_fps,
        source.speed,
        source.frame_us(),
    )
}

#[derive(Debug)]
pub struct Footage {
    tools: FootageTools,
    feeds: HashMap<String, Held>,
    terminals: HashMap<String, TerminalState>,
    starts: AtomicU32,
    stalled: AtomicBool,
}

fn locked(place: &Mutex<Place>) -> MutexGuard<'_, Place> {
    place.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Footage {
    pub fn new(
        tools: FootageTools,
        sources: impl IntoIterator<Item = (String, FootageSource)>,
    ) -> Self {
        let window = Window::default();
        Self {
            tools,
            feeds: sources
                .into_iter()
                .map(|(id, source)| {
                    let place = Mutex::new(Place {
                        span_us: span_of(&source, window),
                        ahead: window.ahead_frames,
                        readers: Vec::new(),
                    });
                    (id, Held { source, place })
                })
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
        self.feeds
            .values()
            .map(|held| {
                held.place.try_lock().map_or(0, |place| {
                    place.readers.iter().filter(|r| r.decoder.is_some()).count()
                })
            })
            .sum()
    }

    pub fn kept_frames(&self) -> usize {
        self.feeds
            .values()
            .map(|held| {
                locked(&held.place)
                    .readers
                    .iter()
                    .map(|r| r.frames.len())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn end_idle(&self, idle: Duration) {
        for held in self.feeds.values() {
            if let Ok(mut place) = held.place.try_lock() {
                place.readers.retain(|r| r.last_used.elapsed() < idle);
            }
        }
    }

    pub fn set_window(&self, window: Window) {
        for held in self.feeds.values() {
            let mut place = locked(&held.place);
            place.span_us = span_of(&held.source, window);
            place.ahead = window.ahead_frames;
            for reader in &mut place.readers {
                reader.asked.clear();
                reader.last_asker = None;
                reader.floor = i64::MIN;
                reader.earliest = i64::MAX;
            }
        }
    }

    pub fn window_bytes(&self, window: Window) -> u64 {
        self.feeds
            .values()
            .map(|held| {
                let source = &held.source;
                let frame_us = source.frame_us();
                let span = span_of(source, window);
                let (width, height) = source.size.unwrap_or((0, 0));
                window_frames(span, frame_us)
                    .saturating_mul(u64::from(source.places.max(1)))
                    .saturating_mul(u64::from(width) * u64::from(height))
                    .saturating_mul(PixelLayout::Bgra.bytes())
            })
            .fold(0u64, u64::saturating_add)
    }

    pub fn size(&self, id: &str) -> Option<(u32, u32)> {
        self.feeds.get(id).and_then(|held| held.source.size)
    }

    pub fn snapped(&self, id: &str, time_us: i64) -> i64 {
        self.feeds
            .get(id)
            .map_or(time_us, |held| snapped_time(&held.source, time_us))
    }

    pub fn shown(&self, id: &str, time_us: i64) -> Result<Shown, FootageError> {
        self.shown_for(id, time_us, None)
    }

    pub fn shown_for(
        &self,
        id: &str,
        time_us: i64,
        asker: Option<u32>,
    ) -> Result<Shown, FootageError> {
        self.end_idle(IDLE);
        let held = self
            .feeds
            .get(id)
            .ok_or_else(|| FootageError::Unknown(id.to_owned()))?;
        let source = &held.source;
        let mut place = locked(&held.place);
        let read = Reading {
            tools: &self.tools,
            source,
            starts: &self.starts,
            span: place.span_us,
        };
        let target = file_time(source, time_us) + MATCH_TOLERANCE_US;
        let jump = JUMP_US.max(place.span_us);
        let found = place
            .readers
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.reach(target, jump).map(|cost| (cost, i)))
            .min()
            .map(|(_, i)| i);
        let index = if let Some(index) = found {
            index
        } else {
            let free = place.readers.iter().position(|r| !r.pending());
            let index = free.unwrap_or_else(|| {
                place.readers.push(Reader::new());
                place.readers.len() - 1
            });
            if let Err(error) = place.readers[index].start_at(
                read.tools,
                source,
                read.starts,
                (target - SEEK_MARGIN_US).max(0),
            ) {
                place.readers.swap_remove(index);
                return Err(error);
            }
            index
        };
        let reader = &mut place.readers[index];
        reader.last_used = Instant::now();
        let shown = reader.show(&read, target, asker);
        if shown.is_err() {
            place.readers.swap_remove(index);
        }
        if matches!(shown, Err(FootageError::Stalled)) {
            self.stalled.store(true, Ordering::Relaxed);
        }
        shown
    }

    pub fn written(&self, frame: u32) {
        for held in self.feeds.values() {
            let mut place = locked(&held.place);
            let ahead = place.ahead;
            for reader in &mut place.readers {
                reader.written(frame);
            }
            place.readers.retain(|r| !r.done(frame, ahead));
        }
    }

    pub fn stalled(&self) -> bool {
        self.stalled.load(Ordering::Relaxed)
    }
}
