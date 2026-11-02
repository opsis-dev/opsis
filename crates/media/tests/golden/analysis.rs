use opsis_media::analysis::{detect_beats, waveform_levels};
use opsis_media::mix::levels::{Pcm, frame_levels};
use opsis_types::numbers;
use serde_json::{Value, json};

use crate::common::{cases, number, with};

struct Lcg(u32);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(self.0) / 4_294_967_296.0
    }
}

fn click_track(rate: usize, seconds: usize, bpm: f64, seed: u32) -> Vec<f32> {
    let mut next = Lcg(seed);
    let beat = numbers::truncate::<usize>((numbers::float(rate) * 60.0 / bpm).round());
    (0..rate * seconds)
        .map(|i| {
            let since = i % beat;
            let click = if since < 200 {
                if since % 16 < 8 { 0.8 } else { -0.8 }
            } else {
                0.0
            };
            numbers::narrow((next.next() - 0.5).mul_add(0.02, click))
        })
        .collect()
}

fn signal(s: &Value) -> Vec<f32> {
    let (rate, seconds, bpm, seed) = (
        numbers::truncate::<usize>(number(&s["rate"])),
        numbers::truncate::<usize>(number(&s["seconds"])),
        number(&s["bpm"]),
        numbers::truncate::<u32>(number(&s["seed"])),
    );
    if bpm > 0.0 {
        click_track(rate, seconds, bpm, seed)
    } else {
        let mut next = Lcg(seed);
        (0..rate * seconds)
            .map(|_| numbers::narrow(next.next()))
            .collect()
    }
}

const ANALYSIS: &str = "media/analysis.json";

fn stem_levels(s: &Value) -> Value {
    let (sample_rate, channels, fps, frames, gain) = (
        number(&s["sampleRate"]),
        numbers::truncate::<usize>(number(&s["channels"])),
        number(&s["fps"]),
        numbers::truncate::<usize>(number(&s["frames"])),
        number(&s["gain"]),
    );
    let mut next = Lcg(numbers::truncate::<u32>(number(&s["seed"])));
    let count =
        numbers::truncate::<usize>((sample_rate * (numbers::float(frames) - 2.0) / fps).floor());
    let data: Vec<Vec<f32>> = (0..channels)
        .map(|_| {
            (0..count)
                .map(|_| numbers::narrow(next.next().mul_add(2.0, -1.0) * gain))
                .collect()
        })
        .collect();
    let interleaved = (0..count).flat_map(|i| data.iter().map(move |c| c[i]));
    let pcm = Pcm {
        sample_rate,
        channels,
        frames: count,
    };
    serde_json::to_value(frame_levels(&pcm, interleaved, fps, frames)).expect("levels")
}

#[test]
fn waveforms_beats_and_stem_levels_are_as_recorded() {
    let stored = opsis_golden::read(ANALYSIS);
    let signals = cases(&stored["signals"], |case| {
        let samples = signal(&case["signal"]);
        let rate = number(&case["signal"]["rate"]);
        with(
            case,
            &[
                ("levels", json!(waveform_levels(&samples, rate))),
                ("beats", json!(detect_beats(&samples, rate))),
            ],
        )
    });
    let frames = cases(&stored["frames"], |case| {
        with(case, &[("levels", stem_levels(&case["signal"]))])
    });
    opsis_golden::check(ANALYSIS, &json!({ "signals": signals, "frames": frames }));
}
