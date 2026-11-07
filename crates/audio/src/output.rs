use std::sync::{Arc, Mutex, mpsc};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::STEM_RATE;
use crate::render::{Pace, fill, frames_in};

#[derive(Debug, Default)]
struct Shared {
    samples: Option<Arc<Vec<f32>>>,
    position: f64,
    playing: bool,
    rate: f64,
}

#[derive(Debug)]
pub struct Output {
    shared: Arc<Mutex<Shared>>,
    device_rate: Arc<Mutex<Option<u32>>>,
    stop: Option<mpsc::Sender<()>>,
}

fn open_stream(
    shared: &Arc<Mutex<Shared>>,
    device_rate: &Arc<Mutex<Option<u32>>>,
) -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    let rate = config.sample_rate();
    let channels = usize::from(config.channels());
    *device_rate.lock().expect("device rate") = Some(rate);
    let state = Arc::clone(shared);
    let base_step = f64::from(STEM_RATE) / f64::from(rate);
    let stream = device
        .build_output_stream(
            config.into(),
            move |out: &mut [f32], _| {
                let Ok(mut shared) = state.try_lock() else {
                    out.fill(0.0);
                    return;
                };
                match (&shared.samples, shared.playing) {
                    (Some(samples), true) => {
                        let samples = Arc::clone(samples);
                        let pace = Pace {
                            step: base_step * shared.rate,
                            volume: 1.0,
                        };
                        shared.position = fill(out, channels, &samples, shared.position, pace);
                    }
                    _ => out.fill(0.0),
                }
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
            Ok(_) => self.stop = Some(sender),
            Err(error) => log::warn!("starting the sound output: {error}"),
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
        if let Some(sender) = self.stop.take() {
            let _ = sender.send(());
        }
    }
}
