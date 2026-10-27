use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_compile::nodes::media::{SoundPicture, SoundPictureKind};
use opsis_media::filters::{
    hum_filter, hum_listen_filter, repair_filter, rms_level, sound_picture_graph,
};

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

fn energy_at(samples: &[f32], hertz: f32) -> f32 {
    let (mut re, mut im) = (0.0f32, 0.0f32);
    for (i, s) in samples.iter().enumerate() {
        let phase = i as f32 / RATE as f32 * hertz * std::f32::consts::TAU;
        re += s * phase.cos();
        im += s * phase.sin();
    }
    (re * re + im * im).sqrt() / samples.len() as f32
}

#[test]
#[ignore = "needs the dev engine"]
fn takes_mains_hum_out_and_leaves_the_voice() {
    let dir = tempfile::tempdir().unwrap();
    let samples: Vec<f32> = tone(2.0)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let t = i as f32 / RATE as f32;
            s + 0.1 * (t * 50.0 * std::f32::consts::TAU).sin()
                + 0.05 * (t * 100.0 * std::f32::consts::TAU).sin()
        })
        .collect();
    write(&dir.path().join("in.wav"), &samples);
    filtered(
        &dir.path().join("in.wav"),
        &hum_filter(50.0, 4, 2.0),
        &dir.path().join("out.wav"),
    );
    let out = read(&dir.path().join("out.wav"));
    let (before, after) = (&samples[RATE as usize / 2..], &out[RATE as usize / 2..]);
    assert!(
        energy_at(after, 50.0) < energy_at(before, 50.0) / 10.0,
        "50 Hz is gone"
    );
    assert!(
        energy_at(after, 100.0) < energy_at(before, 100.0) / 10.0,
        "100 Hz is gone"
    );
    assert!(
        energy_at(after, 440.0) > energy_at(before, 440.0) * 0.9,
        "the tone stays"
    );
}

fn listened(input: &Path, hertz: f64) -> Option<f64> {
    let out = Command::new(ffmpeg())
        .args(["-v", "info", "-i"])
        .arg(input)
        .args(["-af", &hum_listen_filter(hertz), "-f", "null", "-"])
        .output()
        .unwrap();
    let lines: Vec<String> = String::from_utf8_lossy(&out.stderr)
        .lines()
        .map(str::to_owned)
        .collect();
    rms_level(&lines)
}

#[test]
#[ignore = "needs the dev engine"]
fn hears_which_mains_hum_a_recording_has() {
    let dir = tempfile::tempdir().unwrap();
    let samples: Vec<f32> = tone(2.0)
        .iter()
        .enumerate()
        .map(|(i, s)| s + 0.1 * (i as f32 / RATE as f32 * 60.0 * std::f32::consts::TAU).sin())
        .collect();
    write(&dir.path().join("in.wav"), &samples);
    let fifty = listened(&dir.path().join("in.wav"), 50.0).expect("a level at 50 Hz");
    let sixty = listened(&dir.path().join("in.wav"), 60.0).expect("a level at 60 Hz");
    assert!(
        sixty > fifty + 3.0,
        "60 Hz is louder: {sixty} against {fifty}"
    );
}

fn png_size(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap();
    let at = |i: usize| u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
    (at(16), at(20))
}

#[test]
#[ignore = "needs the dev engine"]
fn draws_sound_pictures_at_the_asked_size() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("in.wav"), &tone(1.0));
    for kind in [SoundPictureKind::Waveform, SoundPictureKind::Spectrum] {
        let p = SoundPicture {
            kind,
            width: 640,
            height: 160,
            ..SoundPicture::default()
        };
        let out = dir.path().join("out.png");
        let status = Command::new(ffmpeg())
            .args(["-v", "error", "-y", "-i"])
            .arg(dir.path().join("in.wav"))
            .args([
                "-filter_complex",
                &sound_picture_graph(&p),
                "-map",
                "[out]",
                "-frames:v",
                "1",
            ])
            .arg(&out)
            .status()
            .unwrap();
        assert!(status.success(), "{kind:?}");
        assert_eq!(png_size(&out), (640, 160), "{kind:?}");
    }
}
