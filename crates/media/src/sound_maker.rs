use std::f64::consts::PI;

use fundsp::prelude32::{
    AudioUnit, bandpass, bandpass_hz, dc, envelope, highpass_hz, lowpass, lowpass_hz, noise, saw,
    saw_hz, sine, sine_hz,
};

use crate::generate::RATE;

use opsis_compile::nodes::audio::{SoundPreset, SoundShape};
use opsis_types::numbers;

const PEAK: f32 = 0.7;
const EDGE_SECONDS: f64 = 0.005;

fn count(seconds: f64) -> usize {
    numbers::truncate::<usize>((seconds * f64::from(RATE)).round())
}

fn pulled(mut node: impl AudioUnit, seconds: f64) -> Vec<f32> {
    node.set_sample_rate(f64::from(RATE));
    node.reset();
    (0..count(seconds)).map(|_| node.get_mono()).collect()
}

fn hump(x: f64, peak: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    if x < peak {
        (x / peak * PI / 2.0).sin().powi(2)
    } else {
        ((1.0 - x) / (1.0 - peak) * PI / 2.0).sin().powi(2)
    }
}

fn whoosh(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let factor = 2f64.powf(s.pitch / 12.0);
    let low = 300.0 * factor;
    let high = (1500.0 + 5000.0 * s.brightness) * factor;
    let centre =
        envelope(move |t: f32| numbers::narrow(low + (high - low) * hump(f64::from(t) / d, 0.55)));
    let loudness = envelope(move |t: f32| numbers::narrow(hump(f64::from(t) / d, 0.6)));
    let graph = ((noise().seed(s.seed) | centre | dc(1.2)) >> bandpass()) * loudness;
    pulled(graph, d)
}

fn riser(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let factor = 2f64.powf(s.pitch / 12.0);
    let top = (2000.0 + 6000.0 * s.brightness) * factor;
    let sweep = envelope(move |t: f32| {
        numbers::narrow(200.0 * factor * (top / (200.0 * factor)).powf(f64::from(t) / d))
    });
    let tone =
        envelope(move |t: f32| numbers::narrow(110.0 * factor * 4f64.powf(f64::from(t) / d)));
    let loudness = envelope(move |t: f32| numbers::narrow((f64::from(t) / d).powi(2)));
    let air = (noise().seed(s.seed) | sweep | dc(2.0)) >> bandpass();
    let graph = (air * 0.8 + (tone >> saw() >> lowpass_hz(1800.0, 0.7)) * 0.25) * loudness;
    pulled(graph, d)
}

fn pop(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let factor = numbers::narrow(2f64.powf(s.pitch / 12.0));
    let fall = envelope(move |t: f32| factor * (160.0 + 800.0 * (-t / 0.015).exp()));
    let decay = envelope(move |t: f32| (-t / (0.05 * numbers::narrow(d) / 0.15)).exp());
    let snap = numbers::narrow(s.brightness * 0.4);
    let crack = (noise().seed(s.seed) >> highpass_hz(3000.0, 0.7))
        * envelope(move |t: f32| snap * (-t / 0.003).exp());
    let graph = (fall >> sine()) * decay + crack;
    pulled(graph, d)
}

const BELL_RATIOS: [f32; 5] = [1.0, 2.0, 2.76, 5.4, 8.93];
const BELL_LEVELS: [f32; 5] = [1.0, 0.5, 0.4, 0.25, 0.15];
const BELL_DECAYS: [f32; 5] = [1.2, 0.9, 0.6, 0.35, 0.2];

fn ding(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let base = numbers::narrow(880.0 * 2f64.powf(s.pitch / 12.0));
    let stretch = numbers::narrow(d / 1.5);
    let bright = numbers::narrow(s.brightness);
    let mut out = vec![0.0f32; count(d)];
    for (i, ratio) in BELL_RATIOS.iter().enumerate() {
        let level = BELL_LEVELS[i] * if i >= 3 { 0.3 + 1.4 * bright } else { 1.0 };
        let decay = BELL_DECAYS[i] * stretch;
        let partial = sine_hz(base * ratio)
            * envelope(move |t: f32| level * (t / 0.002).min(1.0) * (-t / decay).exp());
        for (slot, v) in out.iter_mut().zip(pulled(partial, d)) {
            *slot += v;
        }
    }
    out
}

fn tap(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let factor = numbers::narrow(2f64.powf(s.pitch / 12.0));
    let centre = (2000.0 + 4000.0 * numbers::narrow(s.brightness)) * factor;
    let tap = (noise().seed(s.seed) >> bandpass_hz(centre, 2.0))
        * envelope(move |t: f32| (-t / 0.004).exp());
    let tick = sine_hz(1500.0 * factor) * envelope(move |t: f32| 0.5 * (-t / 0.01).exp());
    pulled(tap + tick, d)
}

const PRESS_AT: f32 = 0.001;
const BURST_SECONDS: f32 = 0.0008;
const RELEASE_LEVEL: f32 = 0.6;
const CLICK_EDGE_SECONDS: f64 = 0.0005;

fn burst(t: f32, at: f32) -> f32 {
    if t < at {
        0.0
    } else {
        (-(t - at) / BURST_SECONDS).exp()
    }
}

fn click(s: &SoundShape) -> Vec<f32> {
    let factor = numbers::narrow(2f64.powf(s.pitch / 12.0));
    let bright = numbers::narrow(s.brightness);
    let gap = 0.03 + 0.03 * numbers::narrow(numbers::float_wide(s.seed % 5)) / 4.0;
    let ring = (3500.0 + 3500.0 * bright) * factor;
    let mut source = noise().seed(s.seed);
    let mut snap = highpass_hz((1500.0 + 2500.0 * bright) * factor, 0.7);
    let mut press = bandpass_hz(ring, 5.0);
    let mut release = bandpass_hz(ring * 1.2, 5.0);
    for node in [
        &mut source as &mut dyn AudioUnit,
        &mut snap,
        &mut press,
        &mut release,
    ] {
        node.set_sample_rate(f64::from(RATE));
        node.reset();
    }
    (0..count(s.seconds))
        .map(|i| {
            let t = numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE));
            let n = source.get_mono();
            let first = burst(t, PRESS_AT);
            let second = burst(t, PRESS_AT + gap) * RELEASE_LEVEL;
            snap.filter_mono(n * (first + second))
                + press.filter_mono(n * first) * 2.0
                + release.filter_mono(n * second) * 2.0
        })
        .collect()
}

fn swell(s: &SoundShape) -> Vec<f32> {
    let d = s.seconds;
    let factor = numbers::narrow(2f64.powf(s.pitch / 12.0));
    let drift = 1.0 + numbers::narrow(numbers::float_wide(s.seed % 7)) * 0.001;
    let open = numbers::narrow(1000.0 + 4000.0 * s.brightness);
    let cutoff = envelope(move |t: f32| {
        300.0 + (open - 300.0) * numbers::narrow(hump(f64::from(t) / d, 0.7))
    });
    let loudness = envelope(move |t: f32| numbers::narrow(hump(f64::from(t) / d, 0.75)));
    let voices = saw_hz(220.0 * factor)
        + saw_hz(220.0 * factor * 1.005 * drift)
        + saw_hz(220.0 * factor * 0.995)
        + saw_hz(330.0 * factor) * 0.6;
    let graph = ((voices | cutoff | dc(0.6)) >> lowpass()) * loudness;
    pulled(graph, d)
}

fn finished(mut samples: Vec<f32>, edge_seconds: f64) -> Vec<f32> {
    let edge = count(edge_seconds).min(samples.len() / 2).max(1);
    let len = samples.len();
    for i in 0..edge {
        let gain = numbers::narrow(numbers::float(i)) / numbers::narrow(numbers::float(edge));
        samples[i] *= gain;
        samples[len - 1 - i] *= gain;
    }
    let peak = samples.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if peak > 0.0 {
        for v in &mut samples {
            *v *= PEAK / peak;
        }
    }
    samples
}

pub fn make(shape: &SoundShape) -> [Vec<f32>; 2] {
    let edge = match shape.preset {
        SoundPreset::Click => CLICK_EDGE_SECONDS,
        _ => EDGE_SECONDS,
    };
    let mono = finished(
        match shape.preset {
            SoundPreset::Whoosh => whoosh(shape),
            SoundPreset::Riser => riser(shape),
            SoundPreset::Pop => pop(shape),
            SoundPreset::Ding => ding(shape),
            SoundPreset::Tap => tap(shape),
            SoundPreset::Swell => swell(shape),
            SoundPreset::Click => click(shape),
        },
        edge,
    );
    if shape.preset != SoundPreset::Whoosh {
        return [mono.clone(), mono];
    }
    let len = numbers::float(mono.len().max(1));
    let (mut left, mut right) = (
        Vec::with_capacity(mono.len()),
        Vec::with_capacity(mono.len()),
    );
    for (i, v) in mono.iter().enumerate() {
        let angle = (0.15 + 0.7 * numbers::float(i) / len) * PI / 2.0;
        left.push(v * numbers::narrow(angle.cos()) * 1.4);
        right.push(v * numbers::narrow(angle.sin()) * 1.4);
    }
    [left, right]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(preset: SoundPreset) -> SoundShape {
        SoundShape {
            preset,
            seconds: preset.usual_seconds(),
            pitch: 0.0,
            brightness: 0.5,
            seed: 3,
        }
    }

    #[test]
    fn makes_each_preset_at_its_length_and_peak_and_the_same_from_the_same_seed() {
        for preset in SoundPreset::ALL {
            let [left, right] = make(&shape(preset));
            assert_eq!(left.len(), count(preset.usual_seconds()), "{preset:?}");
            assert_eq!(left.len(), right.len());
            let peak = left
                .iter()
                .chain(&right)
                .fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(peak > 0.3 && peak <= 1.0, "{preset:?} peaks at {peak}");
            assert!(left.iter().all(|v| v.is_finite()));
            assert_eq!(make(&shape(preset))[0], left, "{preset:?}");
        }
        let other = SoundShape {
            seed: 4,
            ..shape(SoundPreset::Whoosh)
        };
        assert_ne!(make(&other)[0], make(&shape(SoundPreset::Whoosh))[0]);
    }
}

#[cfg(test)]
mod clicks {
    use super::*;

    #[test]
    fn clicks_with_a_press_then_a_quieter_release_a_few_hundredths_of_a_second_later() {
        let shape = SoundShape {
            preset: SoundPreset::Click,
            seconds: 0.1,
            pitch: 0.0,
            brightness: 0.5,
            seed: 1,
        };
        let [samples, _] = make(&shape);
        let loudest = |from: f32, to: f32| {
            samples[numbers::truncate::<usize>(f64::from(from * numbers::narrow(f64::from(RATE))))
                ..numbers::truncate::<usize>(f64::from(to * numbers::narrow(f64::from(RATE))))]
                .iter()
                .fold(0.0f32, |m, v| m.max(v.abs()))
        };
        let gap = 0.03 + 0.03 * 1.0 / 4.0;
        let (press, between, release) = (
            loudest(0.0, 0.004),
            loudest(0.012, PRESS_AT + gap - 0.002),
            loudest(PRESS_AT + gap, PRESS_AT + gap + 0.004),
        );
        assert!(press > 0.5, "the press: {press}");
        assert!(between < press / 20.0, "silence between: {between}");
        assert!(
            release > press * 0.3 && release < press,
            "a quieter release: {release}"
        );
    }
}
