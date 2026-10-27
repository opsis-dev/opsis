use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_media::filters::repair_filter;

const RATE: u32 = 48_000;

fn ffmpeg() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine/ffmpeg/ffmpeg")
}

fn write(path: &Path, samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for s in samples {
        writer
            .write_sample((s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();
}

fn read(path: &Path) -> Vec<f32> {
    hound::WavReader::open(path)
        .unwrap()
        .samples::<i16>()
        .map(|s| f32::from(s.unwrap()) / f32::from(i16::MAX))
        .collect()
}

fn filtered(input: &Path, filter: &str, output: &Path) {
    let status = Command::new(ffmpeg())
        .args(["-v", "error", "-y", "-i"])
        .arg(input)
        .args(["-af", filter])
        .arg(output)
        .status()
        .unwrap();
    assert!(status.success());
}

fn tone(seconds: f32) -> Vec<f32> {
    (0..(RATE as f32 * seconds) as usize)
        .map(|i| 0.2 * (i as f32 / RATE as f32 * 440.0 * std::f32::consts::TAU).sin())
        .collect()
}

#[test]
#[ignore = "needs the dev engine"]
fn takes_a_click_out_of_a_tone() {
    let dir = tempfile::tempdir().unwrap();
    let mut samples = tone(1.0);
    let click = RATE as usize / 2;
    for s in &mut samples[click..click + 4] {
        *s = 0.95;
    }
    write(&dir.path().join("in.wav"), &samples);
    filtered(
        &dir.path().join("in.wav"),
        &repair_filter(true, false, 0.5).unwrap(),
        &dir.path().join("out.wav"),
    );
    let out = read(&dir.path().join("out.wav"));
    let peak = out[click - 20..click + 24]
        .iter()
        .fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(peak < 0.5, "the click is gone: {peak}");
}
