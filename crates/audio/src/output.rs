use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opsis_compile::mix::eq::{Biquad, coefficients, highpass};
use opsis_types::mix::{EqBand, EqType};

use crate::STEM_RATE;
use crate::dsp::{FilterState, Smoother, db_to_gain, pan_stereo};
use crate::render::{Pace, fill, frames_in};

const SMOOTHING_S: f64 = 0.02;
const MAX_FILTERS: usize = 6;
const BLIP_S: f64 = 0.09;
const BLIP_GAIN: f32 = 0.9;
const SCRUB_EVERY: Duration = Duration::from_millis(45);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
    Band(EqType),
    Highpass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewFilter {
    pub kind: FilterKind,
    pub freq: f64,
    pub gain: f64,
    pub q: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Preview {
    pub gain_db: f64,
    pub pan: f64,
    pub filters: Vec<PreviewFilter>,
}

#[derive(Debug, Clone, Copy)]
struct Blip {
    position: f64,
    left: usize,
    total: usize,
}

#[derive(Debug, Default)]
struct Shared {
    samples: Option<Arc<Vec<f32>>>,
    position: f64,
    playing: bool,
    rate: f64,
    preview: Preview,
    blip: Option<Blip>,
}

fn biquad_of(filter: &PreviewFilter, rate: f64) -> Biquad {
    match filter.kind {
        FilterKind::Highpass => highpass(filter.freq, filter.q, rate),
        FilterKind::Band(kind) => coefficients(
            &EqBand {
                kind,
                freq: filter.freq,
                gain: filter.gain,
                q: filter.q,
            },
            rate,
        ),
    }
}

struct Chain {
    rate: f64,
    gain: Smoother,
    pan: Smoother,
    wanted: Vec<PreviewFilter>,
    biquads: Vec<Biquad>,
    states: Vec<[FilterState; 2]>,
}

impl Chain {
    fn new(rate: f64) -> Self {
        Self {
            rate,
            gain: Smoother::new(1.0, SMOOTHING_S, rate),
            pan: Smoother::new(0.0, SMOOTHING_S, rate),
            wanted: Vec::new(),
            biquads: Vec::new(),
            states: Vec::new(),
        }
    }

    fn follow(&mut self, preview: &Preview) {
        let wanted: Vec<PreviewFilter> =
            preview.filters.iter().take(MAX_FILTERS).copied().collect();
        if wanted != self.wanted {
            self.biquads = wanted.iter().map(|f| biquad_of(f, self.rate)).collect();
            self.states
                .resize(wanted.len(), [FilterState::default(); 2]);
            self.wanted = wanted;
        }
    }

    fn process(&mut self, out: &mut [f32], channels: usize, preview: &Preview) {
        let neutral = self.wanted.is_empty()
            && (self.gain.value() - 1.0).abs() < 1e-6
            && self.pan.value().abs() < 1e-6
            && preview.gain_db.abs() < 1e-9
            && preview.pan.abs() < 1e-9;
        if neutral || channels < 2 {
            return;
        }
        let target_gain = db_to_gain(preview.gain_db);
        for frame in out.chunks_mut(channels) {
            let mut left = f64::from(frame[0]);
            let mut right = f64::from(frame[1]);
            for (biquad, state) in self.biquads.iter().zip(self.states.iter_mut()) {
                left = state[0].process(biquad, left);
                right = state[1].process(biquad, right);
            }
            let (l, r) = pan_stereo(left, right, self.pan.next(preview.pan));
            let gain = self.gain.next(target_gain);
            frame[0] = opsis_types::numbers::narrow(l * gain);
            frame[1] = opsis_types::numbers::narrow(r * gain);
        }
    }
}

fn render(shared: &mut Shared, out: &mut [f32], channels: usize, base_step: f64) {
    out.fill(0.0);
    let Some(samples) = shared.samples.clone() else {
        return;
    };
    if shared.playing {
        let pace = Pace {
            step: base_step * shared.rate,
            volume: 1.0,
        };
        shared.position = fill(out, channels, &samples, shared.position, pace);
        return;
    }
    let Some(mut blip) = shared.blip else {
        return;
    };
    let width = channels.max(1);
    let frames = (out.len() / width).min(blip.left);
    let part = &mut out[..frames * width];
    let done = blip.total - blip.left;
    blip.position = fill(
        part,
        channels,
        &samples,
        blip.position,
        Pace {
            step: base_step,
            volume: 1.0,
        },
    );
    for (i, frame) in part.chunks_mut(width).enumerate() {
        let fraction = opsis_types::numbers::narrow(
            opsis_types::numbers::float(done + i) / opsis_types::numbers::float(blip.total.max(1)),
        );
        let gain = BLIP_GAIN * (1.0 - fraction);
        for value in frame {
            *value *= gain;
        }
    }
    blip.left -= frames;
    shared.blip = (blip.left > 0).then_some(blip);
}

#[derive(Debug)]
pub struct Output {
    last_scrub: Option<Instant>,
    shared: Arc<Mutex<Shared>>,
    device_rate: Arc<Mutex<Option<u32>>>,
    stop: Option<(mpsc::Sender<()>, std::thread::JoinHandle<()>)>,
}

#[cfg(debug_assertions)]
const DEVICE_ENV: &str = "OPSIS_AUDIO_DEVICE";

fn output_device(host: &cpal::Host) -> Option<cpal::Device> {
    #[cfg(debug_assertions)]
    if let Ok(given) = std::env::var(DEVICE_ENV) {
        let device = given
            .parse::<cpal::DeviceId>()
            .ok()
            .and_then(|id| host.device_by_id(&id));
        if device.is_none() {
            log::warn!("no sound output called {given}");
        }
        return device;
    }
    host.default_output_device()
}

fn open_stream(
    shared: &Arc<Mutex<Shared>>,
    device_rate: &Arc<Mutex<Option<u32>>>,
) -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = output_device(&host)?;
    let config = device.default_output_config().ok()?;
    let rate = config.sample_rate();
    let channels = usize::from(config.channels());
    *device_rate.lock().expect("device rate") = Some(rate);
    let state = Arc::clone(shared);
    let base_step = f64::from(STEM_RATE) / f64::from(rate);
    let mut chain = Chain::new(f64::from(rate));
    let stream = device
        .build_output_stream(
            config.into(),
            move |out: &mut [f32], _| {
                let Ok(mut shared) = state.try_lock() else {
                    out.fill(0.0);
                    return;
                };
                render(&mut shared, out, channels, base_step);
                chain.follow(&shared.preview);
                chain.process(out, channels, &shared.preview);
            },
            |error| log::warn!("the sound output stopped: {error}"),
            None,
        )
        .ok()?;
    stream.play().ok()?;
    Some(stream)
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

impl Output {
    pub fn new() -> Self {
        Self {
            last_scrub: None,
            shared: Arc::new(Mutex::new(Shared {
                rate: 1.0,
                ..Shared::default()
            })),
            device_rate: Arc::new(Mutex::new(None)),
            stop: None,
        }
    }

    fn ensure_stream(&mut self) {
        if self.stop.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let shared = Arc::clone(&self.shared);
        let device_rate = Arc::clone(&self.device_rate);
        let started = std::thread::Builder::new()
            .name("sound output".to_owned())
            .spawn(move || {
                let stream = open_stream(&shared, &device_rate);
                if stream.is_none() {
                    log::info!("no sound output device; playback stays silent");
                }
                let _ = receiver.recv();
                drop(stream);
            });
        match started {
            Ok(thread) => self.stop = Some((sender, thread)),
            Err(error) => log::warn!("starting the sound output: {error}"),
        }
    }

    pub fn close(&mut self) {
        if let Some((sender, thread)) = self.stop.take() {
            let _ = sender.send(());
            if thread.join().is_err() {
                log::warn!("the sound output thread panicked");
            }
        }
    }

    pub fn set_samples(&mut self, samples: Option<Arc<Vec<f32>>>) {
        self.ensure_stream();
        let mut shared = self.shared.lock().expect("sound state");
        shared.samples = samples;
    }

    pub fn has_samples(&self) -> bool {
        self.shared.lock().expect("sound state").samples.is_some()
    }

    pub fn play(&mut self, seconds: f64, rate: f64) {
        self.ensure_stream();
        let mut shared = self.shared.lock().expect("sound state");
        shared.position = seconds * f64::from(STEM_RATE);
        shared.rate = rate;
        shared.playing = rate > 0.0;
    }

    pub fn pause(&self) {
        self.shared.lock().expect("sound state").playing = false;
    }

    pub fn seek(&self, seconds: f64) {
        self.shared.lock().expect("sound state").position = seconds * f64::from(STEM_RATE);
    }

    pub fn set_preview(&self, preview: Preview) {
        self.shared.lock().expect("sound state").preview = preview;
    }

    pub fn scrub(&mut self, seconds: f64) {
        let now = Instant::now();
        if self.last_scrub.is_some_and(|last| now - last < SCRUB_EVERY) {
            return;
        }
        self.last_scrub = Some(now);
        self.ensure_stream();
        let mut shared = self.shared.lock().expect("sound state");
        if shared.playing || shared.samples.is_none() {
            return;
        }
        let total =
            opsis_types::numbers::truncate::<usize>((BLIP_S * f64::from(STEM_RATE)).round());
        shared.blip = Some(Blip {
            position: seconds.max(0.0) * f64::from(STEM_RATE),
            left: total,
            total,
        });
    }

    pub fn clock(&self) -> Option<f64> {
        let audible = self.device_rate.lock().expect("device rate").is_some();
        let shared = self.shared.lock().expect("sound state");
        let samples = shared.samples.as_ref()?;
        let end = opsis_types::numbers::float(frames_in(samples));
        (audible && shared.playing && shared.position < end)
            .then(|| shared.position / f64::from(STEM_RATE))
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        self.close();
    }
}
