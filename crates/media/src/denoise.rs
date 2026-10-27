use nnnoiseless::DenoiseState;

pub const DENOISE_RATE: u32 = 48_000;
const FRAME: usize = DenoiseState::FRAME_SIZE;
const DRY_RANGE_DB: f64 = 30.0;

fn dry_share(strength: f64) -> f64 {
    let strength = strength.clamp(0.0, 1.0);
    let at = |s: f64| 10f64.powf(-DRY_RANGE_DB * s / 20.0);
    at(strength) - strength * at(1.0)
}

pub fn denoised(samples: &[i16], strength: f64) -> Vec<i16> {
    let mut state = DenoiseState::new();
    let mut wet = Vec::with_capacity(samples.len() + 2 * FRAME);
    let mut frame_in = [0.0f32; FRAME];
    let mut frame_out = [0.0f32; FRAME];
    let padded = samples.len().div_ceil(FRAME) * FRAME + FRAME;
    for start in (0..padded).step_by(FRAME) {
        for (i, slot) in frame_in.iter_mut().enumerate() {
            *slot = samples.get(start + i).map_or(0.0, |s| f32::from(*s));
        }
        state.process_frame(&mut frame_out, &frame_in);
        wet.extend_from_slice(&frame_out);
    }
    let dry_share = dry_share(strength);
    samples
        .iter()
        .zip(&wet[FRAME..])
        .map(|(dry, wet)| {
            let mixed = f64::from(*dry) * dry_share + f64::from(*wet) * (1.0 - dry_share);
            mixed
                .round()
                .clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice_in_hiss(seconds: f64) -> (Vec<i16>, Vec<i16>) {
        let mut x: u64 = 99;
        let count = (seconds * f64::from(DENOISE_RATE)) as usize;
        let (mut voice, mut noisy) = (Vec::new(), Vec::new());
        for i in 0..count {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let t = i as f64 / f64::from(DENOISE_RATE);
            let syllables = (t * 3.0 * std::f64::consts::TAU).sin().max(0.0);
            let tone = (t * 220.0 * std::f64::consts::TAU).sin()
                + 0.5 * (t * 440.0 * std::f64::consts::TAU).sin();
            let clean = tone * syllables * 0.3;
            let hiss = ((x >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 0.03;
            voice.push((clean * 32767.0) as i16);
            noisy.push(((clean + hiss) * 32767.0) as i16);
        }
        (voice, noisy)
    }

    fn power(samples: &[i16]) -> f64 {
        samples.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / samples.len() as f64
    }

    #[test]
    fn keeps_its_length_and_lines_the_cleaned_sound_up_with_the_original() {
        let (_, noisy) = voice_in_hiss(1.0);
        let out = denoised(&noisy, 1.0);
        assert_eq!(out.len(), noisy.len());
        let lag = (-100i32..=100)
            .max_by(|a, b| {
                let c = |lag: i32| -> f64 {
                    (1000..40_000usize)
                        .map(|i| {
                            let j = (i as i32 + lag) as usize;
                            f64::from(noisy[i]) * f64::from(out[j])
                        })
                        .sum()
                };
                c(*a).total_cmp(&c(*b))
            })
            .unwrap();
        assert!(lag.abs() <= 6, "{lag}");
        assert_eq!(denoised(&noisy, 0.0), noisy);
    }

    #[test]
    fn keeps_less_of_the_original_the_stronger_it_is_in_even_decibel_steps() {
        assert_eq!((dry_share(0.0), dry_share(1.0)), (1.0, 0.0));
        let db = |s: f64| 20.0 * dry_share(s).log10();
        assert!((db(0.5) + 15.8).abs() < 0.1, "{}", db(0.5));
        assert!((db(0.8) + 28.4).abs() < 0.1, "{}", db(0.8));
    }

    #[test]
    fn takes_out_hiss_between_the_words() {
        let (voice, noisy) = voice_in_hiss(3.0);
        let out = denoised(&noisy, 1.0);
        let gaps: Vec<usize> = (24_000..voice.len())
            .filter(|&i| voice[i.saturating_sub(480)..i + 1].iter().all(|s| *s == 0))
            .collect();
        let before: Vec<i16> = gaps.iter().map(|&i| noisy[i]).collect();
        let after: Vec<i16> = gaps.iter().map(|&i| out[i]).collect();
        let drop = 10.0 * (power(&before) / power(&after)).log10();
        assert!(drop > 10.0, "hiss only {drop:.1} dB quieter");
    }
}
