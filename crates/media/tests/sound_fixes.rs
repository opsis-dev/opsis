use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_types::numbers;

use opsis_compile::nodes::media::{
    ChangePitch, CleanSpeech, NoiseMethod, SoundPicture, SoundPictureKind, VoiceEffect,
    VoiceEffects,
};
use opsis_compile::nodes::media::{FitMusic, FitWay, fit_plan};
use opsis_compile::nodes::media::{Pause, kept_parts};
use opsis_media::denoise::denoised;
use opsis_media::filters::{fit_graph, fit_inputs};
use opsis_media::filters::{
    hum_filter, hum_listen_filter, joined_parts_graph, noise_level, noise_level_listen_filter,
    pitch_filter, repair_filter, rms_level, silence_listen_filter, silences, sound_picture_graph,
    spectral_floor, speech_filter, voice_filter,
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
            .write_sample(numbers::truncate::<i16>(f64::from(
                s.clamp(-1.0, 1.0) * f32::from(i16::MAX),
            )))
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
    (0..numbers::truncate::<usize>(f64::from(numbers::narrow(f64::from(RATE)) * seconds)))
        .map(|i| {
            0.2 * (numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE))
                * 440.0
                * std::f32::consts::TAU)
                .sin()
        })
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
        let phase = numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE))
            * hertz
            * std::f32::consts::TAU;
        re += s * phase.cos();
        im += s * phase.sin();
    }
    (re * re + im * im).sqrt() / numbers::narrow(numbers::float(samples.len()))
}

#[test]
#[ignore = "needs the dev engine"]
fn takes_mains_hum_out_and_leaves_the_voice() {
    let dir = tempfile::tempdir().unwrap();
    let samples: Vec<f32> = tone(2.0)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let t = numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE));
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
        .map(|(i, s)| {
            s + 0.1
                * (numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE))
                    * 60.0
                    * std::f32::consts::TAU)
                    .sin()
        })
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
    let seconds = numbers::float(read(&dir.path().join("out.wav")).len()) / f64::from(RATE);
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
            let t = numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE));
            let syllables = (t * 3.0 * std::f32::consts::TAU).sin().max(0.0);
            let voice = 0.3
                * syllables
                * ((t * 180.0 * std::f32::consts::TAU).sin()
                    + 0.5 * (t * 360.0 * std::f32::consts::TAU).sin());
            let hiss = (numbers::narrow(numbers::float_wide(x >> 40))
                / numbers::narrow(numbers::float_wide(1u64 << 24))
                - 0.5)
                * 0.03;
            (voice, voice + hiss)
        })
        .unzip()
}

fn gap_power(voice: &[f32], sound: &[f32]) -> f32 {
    let gaps: Vec<f32> = (RATE as usize / 2..voice.len().min(sound.len()))
        .filter(|&i| voice[i - 2400..=i].iter().all(|s| *s == 0.0))
        .map(|i| sound[i] * sound[i])
        .collect();
    gaps.iter().sum::<f32>() / numbers::narrow(numbers::float(gaps.len()))
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
        .map(|s| numbers::truncate::<i16>(f64::from(s * f32::from(i16::MAX))))
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
            .filter(|&i| {
                (numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE))
                    * 3.0
                    * std::f32::consts::TAU)
                    .sin()
                    > 0.5
            })
            .map(|i| sound[i] * sound[i])
            .collect();
        10.0 * (loud.iter().sum::<f32>() / numbers::narrow(numbers::float(loud.len()))).log10()
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

fn crossings_per_second(samples: &[f32]) -> f32 {
    let middle = &samples[samples.len() / 4..samples.len() * 3 / 4];
    let rising = middle
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    numbers::narrow(numbers::float(rising))
        / (numbers::narrow(numbers::float(middle.len())) / numbers::narrow(f64::from(RATE)))
}

#[test]
#[ignore = "needs the dev engine"]
fn raises_a_tone_an_octave_and_keeps_its_length() {
    let dir = tempfile::tempdir().unwrap();
    let source = tone(2.0);
    write(&dir.path().join("in.wav"), &source);
    let up = ChangePitch {
        semitones: 12.0,
        keep_character: false,
    };
    filtered(
        &dir.path().join("in.wav"),
        &pitch_filter(&up, Some(2000)),
        &dir.path().join("out.wav"),
    );
    let out = read(&dir.path().join("out.wav"));
    let frame = RATE as usize / 30;
    assert!(
        out.len().abs_diff(source.len()) <= frame,
        "{} samples against {}",
        out.len(),
        source.len()
    );
    let level = |part: &[f32]| {
        (part.iter().map(|s| s * s).sum::<f32>() / numbers::narrow(numbers::float(part.len())))
            .sqrt()
    };
    let opening = level(&out[RATE as usize / 20..RATE as usize / 5]);
    let steady = level(&out[RATE as usize..RATE as usize * 3 / 2]);
    assert!(
        opening > steady * 0.8,
        "the start is as loud as the rest: {opening} against {steady}"
    );
    let pitch = crossings_per_second(&out);
    assert!((pitch - 880.0).abs() < 10.0, "{pitch} Hz");
    assert!((crossings_per_second(&source) - 440.0).abs() < 5.0);
}

fn click_at(seconds: f32, at: f32) -> Vec<f32> {
    let mut samples =
        vec![
            0.0f32;
            numbers::truncate::<usize>(f64::from(numbers::narrow(f64::from(RATE)) * seconds))
        ];
    let start = numbers::truncate::<usize>(f64::from(numbers::narrow(f64::from(RATE)) * at));
    for (i, s) in samples[start..start + 480].iter_mut().enumerate() {
        *s = 0.8 * (numbers::narrow(numbers::float(i)) / 480.0 * std::f32::consts::TAU * 5.0).sin();
    }
    samples
}

fn rms_between(samples: &[f32], from: f32, to: f32) -> f32 {
    let part =
        &samples[numbers::truncate::<usize>(f64::from(numbers::narrow(f64::from(RATE)) * from))
            ..numbers::truncate::<usize>(f64::from(numbers::narrow(f64::from(RATE)) * to))];
    (part.iter().map(|s| s * s).sum::<f32>() / numbers::narrow(numbers::float(part.len()))).sqrt()
}

fn voiced(dir: &Path, samples: &[f32], p: &VoiceEffects) -> Vec<f32> {
    let input = dir.join("in.wav");
    write(&input, samples);
    let room = opsis_media::generate::room_response(p.effect, 3).map(|channels| {
        let file = dir.join("room.wav");
        opsis_media::generate::write_stereo_wav(&file, &channels).unwrap();
        opsis_media::grade::filter_path(&file.to_string_lossy())
    });
    let output = dir.join("out.wav");
    filtered(&input, &voice_filter(p, room.as_deref()), &output);
    read(&output)
}

#[test]
#[ignore = "needs the dev engine"]
fn makes_each_voice_effect_at_the_same_length() {
    let dir = tempfile::tempdir().unwrap();
    let low_and_high: Vec<f32> = (0..RATE as usize * 2)
        .map(|i| {
            let t = numbers::narrow(numbers::float(i)) / numbers::narrow(f64::from(RATE));
            0.3 * (t * 100.0 * std::f32::consts::TAU).sin()
                + 0.3 * (t * 1000.0 * std::f32::consts::TAU).sin()
        })
        .collect();
    let effect = |effect: VoiceEffect| VoiceEffects {
        effect,
        ..VoiceEffects::default()
    };
    let phone = voiced(dir.path(), &low_and_high, &effect(VoiceEffect::Telephone));
    assert_eq!(phone.len(), low_and_high.len());
    assert!(
        energy_at(&phone[RATE as usize / 2..], 100.0)
            < energy_at(&low_and_high[RATE as usize / 2..], 100.0) / 30.0,
        "the telephone cuts the lows"
    );
    assert!(
        energy_at(&phone[RATE as usize / 2..], 1000.0)
            > energy_at(&low_and_high[RATE as usize / 2..], 1000.0) / 4.0
    );
    let radio = voiced(dir.path(), &low_and_high, &effect(VoiceEffect::Radio));
    assert_eq!(radio.len(), low_and_high.len());

    let click = click_at(3.0, 0.5);
    for room in [VoiceEffect::SmallRoom, VoiceEffect::Hall] {
        let wet = voiced(dir.path(), &click, &effect(room));
        assert_eq!(wet.len(), click.len());
        let tail = rms_between(&wet, 0.6, 0.7);
        assert!(tail > 1e-3, "{room:?} rings after the click: {tail}");
        assert_eq!(rms_between(&click, 0.6, 0.7), 0.0);
    }
    for room in [VoiceEffect::SmallRoom, VoiceEffect::Hall] {
        let wet = voiced(dir.path(), &low_and_high, &effect(room));
        let change =
            20.0 * (rms_between(&wet, 1.0, 1.9) / rms_between(&low_and_high, 1.0, 1.9)).log10();
        assert!(
            change.abs() < 6.0,
            "{room:?} changes a steady sound by {change:.1} dB"
        );
    }
    let small = voiced(dir.path(), &click, &effect(VoiceEffect::SmallRoom));
    let hall = voiced(dir.path(), &click, &effect(VoiceEffect::Hall));
    assert!(
        rms_between(&hall, 1.5, 1.6) > rms_between(&small, 1.5, 1.6) * 10.0,
        "a hall rings longer"
    );

    let echo = voiced(
        dir.path(),
        &click,
        &VoiceEffects {
            effect: VoiceEffect::Echo,
            delay_ms: 400,
            decay: 0.5,
            ..VoiceEffects::default()
        },
    );
    let first = rms_between(&echo, 0.5, 0.51);
    let repeat = rms_between(&echo, 0.9, 0.91);
    assert!(
        (repeat / first - 0.5).abs() < 0.05,
        "the echo comes back at half: {}",
        repeat / first
    );
    assert!(rms_between(&echo, 0.6, 0.85) < 1e-4);
}

#[test]
#[ignore = "needs the dev engine"]
fn fits_a_song_to_within_a_frame_of_the_film_each_way() {
    let dir = tempfile::tempdir().unwrap();
    let song = tone(20.0);
    let input = dir.path().join("song.wav");
    write(&input, &song);
    let frame = RATE as usize / 30;
    for (way, target_ms) in [
        (FitWay::Both, 21_000.0),
        (FitWay::Both, 50_000.0),
        (FitWay::Both, 10_000.0),
        (FitWay::Stretch, 15_000.0),
        (FitWay::Loop, 33_000.0),
    ] {
        let p = FitMusic {
            way,
            loop_from_ms: 5_000,
            loop_to_ms: 15_000,
            ..FitMusic::default()
        };
        let plan = fit_plan(20_000.0, target_ms, &p);
        let output = dir.path().join("out.wav");
        let mut command = Command::new(ffmpeg());
        command.args(["-v", "error", "-y"]);
        for _ in 0..fit_inputs(plan) {
            command.arg("-i").arg(&input);
        }
        let status = command
            .args([
                "-filter_complex",
                &fit_graph(plan, target_ms, &p),
                "-map",
                "[out]",
            ])
            .arg(&output)
            .status()
            .unwrap();
        assert!(status.success(), "{plan:?}");
        let out = read(&output);
        let want = numbers::truncate::<usize>(target_ms / 1000.0 * f64::from(RATE));
        assert!(
            out.len().abs_diff(want) <= frame,
            "{plan:?}: {} samples against {want}",
            out.len()
        );
        let tail = &out[out.len() - 480..];
        assert!(
            tail.iter().all(|s| s.abs() < 0.01),
            "{plan:?} fades out at the end"
        );
    }
}
