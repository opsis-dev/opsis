use std::path::Path;

use opsis_compile::nodes::audio::NoiseColour;
use opsis_types::numbers::whole;

pub const RATE: u32 = 48_000;
const LOOP_SECONDS: f64 = 10.0;
const SEAM_SECONDS: f64 = 0.1;

struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

fn samples_for(seconds: f64) -> usize {
    (seconds * f64::from(RATE)).round() as usize
}

pub fn decibels(rms: f64) -> f64 {
    20.0 * rms.max(1e-12).log10()
}

pub fn rms(samples: &[f32]) -> f64 {
    (samples.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / samples.len().max(1) as f64)
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
        let k = i as f64 / seam as f64;
        looped[i] = looped[i] * k + raw[length + i] * (1.0 - k);
    }
    let current = (looped.iter().map(|s| s * s).sum::<f64>() / length as f64).sqrt();
    let gain = 10f64.powf(level_db / 20.0) / current.max(1e-12);
    looped.iter().map(|s| (s * gain) as f32).collect()
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
        let value = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        writer.write_sample(value)?;
        writer.write_sample(value)?;
    }
    writer.finalize()
}

pub fn length_ms(samples: &[f32]) -> u32 {
    whole(samples.len() as f64 * 1000.0 / f64::from(RATE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band(samples: &[f32], low_hz: f64, high_hz: f64) -> f64 {
        let n = 4096;
        let mut total = 0.0;
        for k in (low_hz * n as f64 / f64::from(RATE)) as usize
            ..(high_hz * n as f64 / f64::from(RATE)) as usize
        {
            let (mut re, mut im) = (0.0, 0.0);
            for (i, s) in samples[..n].iter().enumerate() {
                let phase = std::f64::consts::TAU * k as f64 * i as f64 / n as f64;
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
            let typical = rms(&tone) as f32 * 4.0;
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
}
