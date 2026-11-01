use std::path::Path;

use opsis_compile::nodes::audio::{NoiseColour, ToneKind, ToneWave};
use opsis_compile::nodes::media::VoiceEffect;
use opsis_types::numbers;
use opsis_types::numbers::whole;

pub const RATE: u32 = 48_000;
const LOOP_SECONDS: f64 = 10.0;
const SEAM_SECONDS: f64 = 0.1;
const EDGE_SECONDS: f64 = 0.005;

struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        numbers::float_wide(self.0 >> 11) / numbers::float_wide(1u64 << 53) * 2.0 - 1.0
    }
}

fn samples_for(seconds: f64) -> usize {
    numbers::truncate::<usize>((seconds * f64::from(RATE)).round())
}

pub fn decibels(rms: f64) -> f64 {
    20.0 * rms.max(1e-12).log10()
}

pub fn rms(samples: &[f32]) -> f64 {
    (samples.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>()
        / numbers::float(samples.len().max(1)))
    .sqrt()
}

fn coloured(colour: NoiseColour, count: usize, seed: u64) -> Vec<f64> {
    let mut random = Seeded(seed | 1);
    let mut pink = [0.0f64; 7];
    let mut brown = 0.0f64;
    (0..count)
        .map(|_| {
            let white = random.next();
            match colour {
                NoiseColour::White => white,
                NoiseColour::Pink => {
                    pink[0] = 0.99886 * pink[0] + white * 0.0555179;
                    pink[1] = 0.99332 * pink[1] + white * 0.0750759;
                    pink[2] = 0.96900 * pink[2] + white * 0.1538520;
                    pink[3] = 0.86650 * pink[3] + white * 0.3104856;
                    pink[4] = 0.55000 * pink[4] + white * 0.5329522;
                    pink[5] = -0.7616 * pink[5] - white * 0.0168980;
                    let out = pink.iter().sum::<f64>() + white * 0.5362;
                    pink[6] = white * 0.115926;
                    out
                }
                NoiseColour::Brown => {
                    brown = (brown + 0.02 * white) / 1.02;
                    brown
                }
            }
        })
        .collect()
}

pub fn room_tone(colour: NoiseColour, level_db: f64, seed: u64) -> Vec<f32> {
    let length = samples_for(LOOP_SECONDS);
    let seam = samples_for(SEAM_SECONDS);
    let raw = coloured(colour, length + seam, seed);
    let mut looped: Vec<f64> = raw[..length].to_vec();
    for i in 0..seam {
        let k = numbers::float(i) / numbers::float(seam);
        looped[i] = looped[i] * k + raw[length + i] * (1.0 - k);
    }
    let current = (looped.iter().map(|s| s * s).sum::<f64>() / numbers::float(length)).sqrt();
    let gain = 10f64.powf(level_db / 20.0) / current.max(1e-12);
    looped.iter().map(|s| numbers::narrow(s * gain)).collect()
}

fn wave_at(wave: ToneWave, cycles: f64) -> f64 {
    let turn = cycles.fract();
    match wave {
        ToneWave::Sine => (cycles * std::f64::consts::TAU).sin(),
        ToneWave::Triangle => 1.0 - 4.0 * (turn - 0.5).abs(),
        ToneWave::Square => {
            if turn < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        ToneWave::Saw => 2.0 * turn - 1.0,
    }
}

fn wave_level(wave: ToneWave) -> f64 {
    match wave {
        ToneWave::Sine | ToneWave::Triangle => 0.5,
        ToneWave::Square => 0.3,
        ToneWave::Saw => 0.35,
    }
}

fn beep(pitch: f64, seconds: f64, wave: ToneWave) -> Vec<f32> {
    let count = samples_for(seconds);
    let edge = samples_for(EDGE_SECONDS).min(count / 2).max(1);
    let level = wave_level(wave);
    (0..count)
        .map(|i| {
            let shape = (numbers::float(i.min(count - 1 - i)) / numbers::float(edge)).min(1.0);
            let cycles = numbers::float(i) / f64::from(RATE) * pitch;
            numbers::narrow(wave_at(wave, cycles) * shape * level)
        })
        .collect()
}

#[derive(Debug)]
pub struct Beeps {
    pub kind: ToneKind,
    pub pitch_hz: f64,
    pub length_ms: f64,
    pub count: u32,
    pub spacing_ms: f64,
    pub wave: ToneWave,
    pub final_length: f64,
    pub final_pitch: f64,
}

pub fn beeps(p: &Beeps) -> Vec<f32> {
    let one = beep(p.pitch_hz, p.length_ms / 1000.0, p.wave);
    if p.kind != ToneKind::Countdown {
        return one;
    }
    let step = samples_for(p.spacing_ms / 1000.0);
    let last = beep(
        p.pitch_hz * 2f64.powf(p.final_pitch / 12.0),
        p.length_ms / 1000.0 * p.final_length,
        p.wave,
    );
    let starts: Vec<usize> = (0..=p.count as usize).map(|n| n * step).collect();
    let total = starts.last().copied().unwrap_or(0) + last.len();
    let mut out = vec![0.0f32; total.max(one.len())];
    for (n, &start) in starts.iter().enumerate() {
        let sound = if n == starts.len() - 1 { &last } else { &one };
        for (i, s) in sound.iter().enumerate() {
            if let Some(slot) = out.get_mut(start + i) {
                *slot += s;
            }
        }
    }
    out
}

struct Room {
    seconds: f64,
    reverb_seconds: f64,
    reflections: &'static [(f64, f32)],
}

const SMALL_ROOM: Room = Room {
    seconds: 0.6,
    reverb_seconds: 0.35,
    reflections: &[(0.0053, 0.6), (0.0089, 0.5), (0.0131, 0.4), (0.0177, 0.35)],
};

const HALL: Room = Room {
    seconds: 3.0,
    reverb_seconds: 2.2,
    reflections: &[
        (0.019, 0.5),
        (0.027, 0.45),
        (0.037, 0.4),
        (0.053, 0.3),
        (0.071, 0.25),
    ],
};

const SIXTY_DB: f64 = 6.907_755;

fn room_of(effect: VoiceEffect) -> Option<&'static Room> {
    match effect {
        VoiceEffect::SmallRoom => Some(&SMALL_ROOM),
        VoiceEffect::Hall => Some(&HALL),
        VoiceEffect::Telephone | VoiceEffect::Radio | VoiceEffect::Echo => None,
    }
}

fn room_channel(room: &Room, seed: u64) -> Vec<f32> {
    let count = samples_for(room.seconds);
    let mut random = Seeded(seed | 1);
    let mut out = vec![0.0f64; count];
    let mut damped = 0.0f64;
    for (i, slot) in out.iter_mut().enumerate() {
        let t = numbers::float(i) / f64::from(RATE);
        let decay = (-SIXTY_DB * t / room.reverb_seconds).exp();
        let darker = (0.2 + 0.75 * t / room.reverb_seconds).min(0.95);
        damped = damped * darker + random.next() * (1.0 - darker);
        let level = ((1.0 + darker) / (1.0 - darker)).sqrt();
        *slot = damped * decay * level;
    }
    out[0] = 1.0;
    for &(at, gain) in room.reflections {
        let jitter = 1.0 + 0.04 * random.next();
        if let Some(slot) = out.get_mut(samples_for(at * jitter)) {
            *slot += f64::from(gain);
        }
    }
    let energy = out.iter().map(|s| s * s).sum::<f64>().sqrt().max(1e-12);
    out.iter().map(|s| numbers::narrow(s / energy)).collect()
}

pub fn room_response(
    effect: VoiceEffect,
    seed: u64,
    reverb_ms: Option<u32>,
) -> Option<[Vec<f32>; 2]> {
    let own = room_of(effect)?;
    let reverb_seconds = reverb_ms.map_or(own.reverb_seconds, |ms| f64::from(ms) / 1000.0);
    let room = &Room {
        seconds: own
            .seconds
            .max(reverb_seconds * own.seconds / own.reverb_seconds),
        reverb_seconds,
        reflections: own.reflections,
    };
    Some([
        room_channel(room, seed),
        room_channel(room, seed.rotate_left(17) ^ 0x9e37),
    ])
}

pub fn write_stereo_wav(path: &Path, channels: &[Vec<f32>; 2]) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for (left, right) in channels[0].iter().zip(&channels[1]) {
        writer.write_sample(*left)?;
        writer.write_sample(*right)?;
    }
    writer.finalize()
}

pub fn write_stereo_pcm(path: &Path, channels: &[Vec<f32>; 2]) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    let pcm =
        |s: f32| numbers::truncate::<i16>(f64::from(s.clamp(-1.0, 1.0) * f32::from(i16::MAX)));
    for (left, right) in channels[0].iter().zip(&channels[1]) {
        writer.write_sample(pcm(*left))?;
        writer.write_sample(pcm(*right))?;
    }
    writer.finalize()
}

pub fn write_wav(path: &Path, samples: &[f32]) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for s in samples {
        let value = numbers::truncate::<i16>(f64::from(s.clamp(-1.0, 1.0) * f32::from(i16::MAX)));
        writer.write_sample(value)?;
        writer.write_sample(value)?;
    }
    writer.finalize()
}

pub fn length_ms(samples: &[f32]) -> u32 {
    whole(numbers::float(samples.len()) * 1000.0 / f64::from(RATE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band(samples: &[f32], low_hz: f64, high_hz: f64) -> f64 {
        let n = 4096;
        let mut total = 0.0;
        for k in numbers::truncate::<usize>(low_hz * numbers::float(n) / f64::from(RATE))
            ..numbers::truncate::<usize>(high_hz * numbers::float(n) / f64::from(RATE))
        {
            let (mut re, mut im) = (0.0, 0.0);
            for (i, s) in samples[..n].iter().enumerate() {
                let phase = std::f64::consts::TAU * numbers::float(k) * numbers::float(i)
                    / numbers::float(n);
                re += f64::from(*s) * phase.cos();
                im += f64::from(*s) * phase.sin();
            }
            total += re * re + im * im;
        }
        total
    }

    #[test]
    fn makes_room_tone_at_the_asked_level_with_a_seamless_loop() {
        for colour in [NoiseColour::White, NoiseColour::Pink, NoiseColour::Brown] {
            let tone = room_tone(colour, -45.0, 7);
            assert_eq!(tone.len(), samples_for(LOOP_SECONDS));
            assert!((decibels(rms(&tone)) + 45.0).abs() < 0.01, "{colour:?}");
            let jump = (tone[0] - tone[tone.len() - 1]).abs();
            let typical = numbers::narrow(rms(&tone)) * 4.0;
            assert!(jump < typical, "{colour:?} jumps {jump} at the loop");
        }
        assert_eq!(
            room_tone(NoiseColour::Pink, -45.0, 7),
            room_tone(NoiseColour::Pink, -45.0, 7)
        );
        assert_ne!(
            room_tone(NoiseColour::Pink, -45.0, 7),
            room_tone(NoiseColour::Pink, -45.0, 8)
        );
    }

    #[test]
    fn colours_its_noise_darker_from_white_to_pink_to_brown() {
        let tilt = |colour| {
            let tone = room_tone(colour, -30.0, 3);
            band(&tone, 100.0, 400.0) / band(&tone, 4000.0, 16000.0)
        };
        let (white, pink, brown) = (
            tilt(NoiseColour::White),
            tilt(NoiseColour::Pink),
            tilt(NoiseColour::Brown),
        );
        assert!(white < pink && pink < brown, "{white} {pink} {brown}");
    }

    fn onsets(samples: &[f32]) -> Vec<usize> {
        let mut found = Vec::new();
        let mut quiet = true;
        for (i, s) in samples.iter().enumerate() {
            if quiet && s.abs() > 0.05 {
                found.push(i);
                quiet = false;
            } else if !quiet
                && samples[i.saturating_sub(480)..=i]
                    .iter()
                    .all(|v| v.abs() < 0.01)
            {
                quiet = true;
            }
        }
        found
    }

    #[test]
    fn counts_down_with_short_beeps_a_second_apart_then_a_long_one() {
        let p = Beeps {
            kind: ToneKind::Countdown,
            pitch_hz: 880.0,
            length_ms: 150.0,
            count: 3,
            spacing_ms: 1000.0,
            wave: ToneWave::Sine,
            final_length: 3.0,
            final_pitch: 0.0,
        };
        let sound = beeps(&p);
        let starts = onsets(&sound);
        assert_eq!(starts.len(), 4, "{starts:?}");
        for pair in starts.windows(2) {
            let gap = numbers::float(pair[1] - pair[0]) / f64::from(RATE);
            assert!((gap - 1.0).abs() < 0.005, "{gap}");
        }
        assert_eq!(length_ms(&sound), 3000 + 450);
        let single = beeps(&Beeps {
            kind: ToneKind::Beep,
            ..p
        });
        assert_eq!(length_ms(&single), 150);
        assert!(single.iter().all(|s| s.abs() <= 0.5));
    }

    #[test]
    fn makes_rooms_that_die_away_sixty_decibels_over_their_reverb_time() {
        for (effect, room) in [
            (VoiceEffect::SmallRoom, &SMALL_ROOM),
            (VoiceEffect::Hall, &HALL),
        ] {
            let [left, right] = room_response(effect, 5, None).unwrap();
            assert_eq!(left.len(), samples_for(room.seconds));
            let energy: f64 = left.iter().map(|s| f64::from(*s).powi(2)).sum();
            assert!((energy - 1.0).abs() < 1e-3, "{energy}");
            let window = |at: f64| {
                let start = samples_for(at);
                rms(&left[start..start + samples_for(0.02)])
            };
            let early = window(room.reverb_seconds * 0.1);
            let late = window(room.reverb_seconds * 0.9);
            let fall = decibels(early) - decibels(late);
            assert!(
                (fall - 48.0).abs() < 8.0,
                "{effect:?} falls {fall:.1} dB over 80% of its reverb time"
            );
            assert_ne!(left, right);
            assert_eq!(room_response(effect, 5, None).unwrap()[0], left);
        }
        assert!(room_response(VoiceEffect::Echo, 5, None).is_none());
    }
}
