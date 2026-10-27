use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_compile::nodes::media::{CleanSpeech, NoiseMethod, SoundPicture, SoundPictureKind};
use opsis_compile::nodes::media::{Pause, kept_parts};
use opsis_media::denoise::denoised;
use opsis_media::filters::{
    hum_filter, hum_listen_filter, joined_parts_graph, noise_level, noise_level_listen_filter,
    repair_filter, rms_level, silence_listen_filter, silences, sound_picture_graph, spectral_floor,
    speech_filter,
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

#[test]
fn keeps_the_middle_of_each_long_pause_and_leaves_short_ones() {
    let pauses = [
        Pause {
            start_ms: 1000.0,
            end_ms: 3000.0,
        },
        Pause {
            start_ms: 4000.0,
            end_ms: 4300.0,
        },
        Pause {
            start_ms: 5000.0,
            end_ms: 6000.0,
        },
    ];
    assert_eq!(
        kept_parts(8000.0, &pauses, 400.0),
        [(0.0, 1200.0), (2800.0, 5200.0), (5800.0, 8000.0)]
    );
    assert_eq!(kept_parts(2000.0, &[], 400.0), [(0.0, 2000.0)]);
    let lines: Vec<String> = [
        "[silencedetect @ 0x1] silence_start: 1.0005",
        "[silencedetect @ 0x1] silence_end: 3.001 | silence_duration: 2.0005",
        "[silencedetect @ 0x1] silence_start: -0.002",
        "[silencedetect @ 0x1] silence_end: 0.9 | silence_duration: 0.9",
    ]
    .iter()
    .map(|l| (*l).to_owned())
    .collect();
    let found = silences(&lines);
    assert_eq!(found.len(), 2);
    assert!((found[0].start_ms - 1000.5).abs() < 1e-6 && (found[0].end_ms - 3001.0).abs() < 1e-6);
    assert_eq!(found[1].start_ms, 0.0);
}

#[test]
#[ignore = "needs the dev engine"]
fn shortens_known_gaps_to_the_expected_length() {
    let dir = tempfile::tempdir().unwrap();
    let speech = tone(1.0);
    let gap = vec![0.0f32; (RATE * 2) as usize];
    let samples: Vec<f32> = [&speech[..], &gap, &speech, &gap, &speech].concat();
    write(&dir.path().join("in.wav"), &samples);
    let input = dir.path().join("in.wav");
    let out = Command::new(ffmpeg())
        .args(["-v", "info", "-i"])
        .arg(&input)
        .args(["-af", &silence_listen_filter(-40.0, 700), "-f", "null", "-"])
        .output()
        .unwrap();
    let lines: Vec<String> = String::from_utf8_lossy(&out.stderr)
        .lines()
        .map(str::to_owned)
        .collect();
    let pauses = silences(&lines);
    assert_eq!(pauses.len(), 2, "{lines:#?}");
    let parts = kept_parts(7000.0, &pauses, 400.0);
    let status = Command::new(ffmpeg())
        .args(["-v", "error", "-y", "-i"])
        .arg(&input)
        .args([
            "-filter_complex",
            &joined_parts_graph(&parts),
            "-map",
            "[out]",
        ])
        .arg(dir.path().join("out.wav"))
        .status()
        .unwrap();
    assert!(status.success());
    let seconds = read(&dir.path().join("out.wav")).len() as f64 / f64::from(RATE);
    assert!(
        (seconds - (3.0 + 2.0 * 0.4 - 2.0 * 0.01)).abs() < 0.05,
        "{seconds} s"
    );
}

fn voice_in_hiss() -> (Vec<f32>, Vec<f32>) {
    let mut x: u64 = 7;
    (0..RATE as usize * 3)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let t = i as f32 / RATE as f32;
            let syllables = (t * 3.0 * std::f32::consts::TAU).sin().max(0.0);
            let voice = 0.3
                * syllables
                * ((t * 180.0 * std::f32::consts::TAU).sin()
                    + 0.5 * (t * 360.0 * std::f32::consts::TAU).sin());
            let hiss = ((x >> 40) as f32 / (1u64 << 24) as f32 - 0.5) * 0.03;
            (voice, voice + hiss)
        })
        .unzip()
}

fn gap_power(voice: &[f32], sound: &[f32]) -> f32 {
    let gaps: Vec<f32> = (RATE as usize / 2..voice.len().min(sound.len()))
        .filter(|&i| voice[i - 2400..=i].iter().all(|s| *s == 0.0))
        .map(|i| sound[i] * sound[i])
        .collect();
    gaps.iter().sum::<f32>() / gaps.len() as f32
}

fn noise_of(input: &Path) -> f64 {
    let out = Command::new(ffmpeg())
        .args(["-v", "info", "-i"])
        .arg(input)
        .args(["-af", noise_level_listen_filter(), "-f", "null", "-"])
        .output()
        .unwrap();
    let lines: Vec<String> = String::from_utf8_lossy(&out.stderr)
        .lines()
        .map(str::to_owned)
        .collect();
    noise_level(&lines).unwrap()
}

#[test]
#[ignore = "needs the dev engine"]
fn cleans_hiss_out_of_speech_both_ways() {
    let dir = tempfile::tempdir().unwrap();
    let (voice, noisy) = voice_in_hiss();
    let noisy_path = dir.path().join("noisy.wav");
    write(&noisy_path, &noisy);
    let before = gap_power(&voice, &noisy);
    let level = noise_of(&noisy_path);
    assert!((level + 41.4).abs() < 1.0, "{level}");

    let p = CleanSpeech::default();
    assert_eq!(p.method, NoiseMethod::Learned);
    let whole: Vec<i16> = noisy
        .iter()
        .map(|s| (s * f32::from(i16::MAX)) as i16)
        .collect();
    let cleaned: Vec<f32> = denoised(&whole, p.noise)
        .iter()
        .map(|s| f32::from(*s) / f32::from(i16::MAX))
        .collect();
    write(&dir.path().join("voice.wav"), &cleaned);
    filtered(
        &dir.path().join("voice.wav"),
        &speech_filter(&p, None),
        &dir.path().join("learned.wav"),
    );
    let learned = read(&dir.path().join("learned.wav"));
    let learned_drop = 10.0 * (before / gap_power(&voice, &learned)).log10();
    assert!(
        learned_drop > 15.0,
        "learned took {learned_drop:.1} dB of hiss out"
    );

    let spectral = CleanSpeech {
        method: NoiseMethod::Spectral,
        ..CleanSpeech::default()
    };
    filtered(
        &noisy_path,
        &speech_filter(&spectral, Some(spectral_floor(Some(level)))),
        &dir.path().join("spectral.wav"),
    );
    let spectral_out = read(&dir.path().join("spectral.wav"));
    let spectral_drop = 10.0 * (before / gap_power(&voice, &spectral_out)).log10();
    assert!(
        spectral_drop > 15.0,
        "spectral took {spectral_drop:.1} dB of hiss out"
    );

    let levels = (
        noise_of(&dir.path().join("learned.wav")),
        noise_of(&dir.path().join("spectral.wav")),
    );
    assert!(
        levels.0 < level - 15.0 && levels.1 < level - 15.0,
        "{level} {levels:?}"
    );
    let speech = |sound: &[f32]| {
        let loud: Vec<f32> = (0..voice.len().min(sound.len()))
            .filter(|&i| (i as f32 / RATE as f32 * 3.0 * std::f32::consts::TAU).sin() > 0.5)
            .map(|i| sound[i] * sound[i])
            .collect();
        10.0 * (loud.iter().sum::<f32>() / loud.len() as f32).log10()
    };
    let kept = (
        speech(&learned) - speech(&noisy),
        speech(&spectral_out) - speech(&noisy),
    );
    assert!(
        kept.0.abs() < 6.0 && kept.1.abs() < 6.0,
        "the voice moved {kept:?} dB"
    );
}

#[test]
fn reads_the_noise_level_from_the_quietest_tenth_of_short_windows() {
    let mut lines: Vec<String> = (0..20)
        .map(|i| {
            format!(
                "[Parsed_ametadata_4 @ 0x1] lavfi.astats.Overall.RMS_level={}",
                -20 - i
            )
        })
        .collect();
    lines.push("lavfi.astats.Overall.RMS_level=-inf".to_owned());
    lines.push("Stream mapping:".to_owned());
    assert_eq!(noise_level(&lines), Some(-37.0));
    assert_eq!(noise_level(&[]), None);
    assert_eq!(
        (
            spectral_floor(None),
            spectral_floor(Some(-41.42)),
            spectral_floor(Some(-15.0)),
            spectral_floor(Some(-95.0))
        ),
        (-50.0, -38.4, -20.0, -80.0)
    );
}
