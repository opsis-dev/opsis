use std::path::Path;

use ebur128::{EbuR128, Mode};
use opsis_compile::text::round_to;
use opsis_types::mix::{Loudness, SILENCE_DB};

const LOUDNESS_FLOOR_DB: f64 = -70.0;
const BLOCK_FRAMES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum LoudnessError {
    #[error(transparent)]
    Wav(#[from] hound::Error),
    #[error("Couldn't measure the loudness: {0}")]
    Meter(#[from] ebur128::Error),
}

fn reading(value: Result<f64, ebur128::Error>) -> f64 {
    match value {
        Ok(v) if v.is_finite() => round_to(v.max(SILENCE_DB), 1),
        _ => SILENCE_DB,
    }
}

pub struct Meter {
    ebu: EbuR128,
    channels: usize,
    per_frame: f64,
    fed: usize,
    loudness: Loudness,
    frames: usize,
}

impl Meter {
    pub fn new(
        channels: usize,
        sample_rate: u32,
        fps: f64,
        frames: usize,
    ) -> Result<Self, LoudnessError> {
        let ebu = EbuR128::new(
            u32::try_from(channels).unwrap_or(2),
            sample_rate,
            Mode::I | Mode::LRA | Mode::TRUE_PEAK,
        )?;
        let loudness = Loudness {
            fps,
            momentary: Vec::with_capacity(frames),
            short_term: Vec::with_capacity(frames),
            integrated: None,
            range: None,
            true_peak: None,
        };
        Ok(Self {
            ebu,
            channels,
            per_frame: f64::from(sample_rate) / fps,
            fed: 0,
            loudness,
            frames,
        })
    }

    fn frame_end(&self, frame: usize) -> usize {
        ((frame + 1) as f64 * self.per_frame).round() as usize
    }

    pub fn feed(&mut self, interleaved: &[f32]) -> Result<(), LoudnessError> {
        let mut rest = interleaved;
        while !rest.is_empty() {
            let frame = self.loudness.momentary.len();
            let wanted = if frame < self.frames {
                self.frame_end(frame).saturating_sub(self.fed)
            } else {
                usize::MAX
            };
            let take = (wanted.saturating_mul(self.channels)).min(rest.len());
            let (now, later) = rest.split_at(take);
            self.ebu.add_frames_f32(now)?;
            self.fed += now.len() / self.channels;
            rest = later;
            if frame < self.frames && self.fed >= self.frame_end(frame) {
                self.close_frame();
            }
        }
        Ok(())
    }

    fn close_frame(&mut self) {
        self.loudness
            .momentary
            .push(reading(self.ebu.loudness_momentary()));
        self.loudness
            .short_term
            .push(reading(self.ebu.loudness_shortterm()));
    }

    pub fn finish(mut self) -> Result<Loudness, LoudnessError> {
        while self.loudness.momentary.len() < self.frames {
            let last = (
                self.loudness.momentary.last().copied(),
                self.loudness.short_term.last().copied(),
            );
            self.loudness.momentary.push(last.0.unwrap_or(SILENCE_DB));
            self.loudness.short_term.push(last.1.unwrap_or(SILENCE_DB));
        }
        let integrated = self.ebu.loudness_global()?;
        self.loudness.integrated = (integrated.is_finite() && integrated > LOUDNESS_FLOOR_DB)
            .then(|| round_to(integrated, 1));
        let range = self.ebu.loudness_range()?;
        self.loudness.range = range.is_finite().then(|| round_to(range, 1));
        let mut peak = 0f64;
        for channel in 0..self.channels {
            peak = peak.max(self.ebu.true_peak(u32::try_from(channel).unwrap_or(0))?);
        }
        let db = 20.0 * peak.log10();
        self.loudness.true_peak = (db.is_finite() && db > SILENCE_DB).then(|| round_to(db, 1));
        Ok(self.loudness)
    }
}

pub fn wav_loudness(file: &Path, fps: f64, frames: usize) -> Result<Loudness, LoudnessError> {
    let mut reader = hound::WavReader::open(file)?;
    let spec = reader.spec();
    let channels = usize::from(spec.channels);
    let mut meter = Meter::new(channels, spec.sample_rate, fps, frames)?;
    let mut block = Vec::with_capacity(BLOCK_FRAMES * channels);
    let scale = 2f32.powi(i32::from(spec.bits_per_sample) - 1);
    let mut push = |sample: f32, meter: &mut Meter| -> Result<(), LoudnessError> {
        block.push(sample);
        if block.len() == block.capacity() {
            meter.feed(&block)?;
            block.clear();
        }
        Ok(())
    };
    match spec.sample_format {
        hound::SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                push(sample?, &mut meter)?;
            }
        }
        hound::SampleFormat::Int => {
            for sample in reader.samples::<i32>() {
                push(sample? as f32 / scale, &mut meter)?;
            }
        }
    }
    meter.feed(&block)?;
    meter.finish()
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::*;

    fn stereo_sine(file: &Path, seconds: f64, db: f64) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(file, spec).unwrap();
        let amplitude = 10f64.powf(db / 20.0);
        for i in 0..(48_000.0 * seconds) as usize {
            let v = (amplitude * (TAU * 1000.0 * i as f64 / 48_000.0).sin()) as f32;
            writer.write_sample(v).unwrap();
            writer.write_sample(v).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn reads_the_reference_tone_at_minus_23_lufs_with_one_reading_per_frame() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tone.wav");
        stereo_sine(&file, 10.0, -23.0);
        let loudness = wav_loudness(&file, 30.0, 300).unwrap();
        assert_eq!(
            (loudness.momentary.len(), loudness.short_term.len()),
            (300, 300)
        );
        assert!(
            (loudness.integrated.unwrap() + 23.0).abs() <= 0.1,
            "{:?}",
            loudness.integrated
        );
        assert!((loudness.momentary[299] + 23.0).abs() <= 0.1);
        assert!(
            (loudness.true_peak.unwrap() + 23.0).abs() <= 0.2,
            "{:?}",
            loudness.true_peak
        );
        assert!(loudness.range.unwrap() < 1.0);
    }

    #[test]
    fn reports_silence_as_no_reading() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("quiet.wav");
        stereo_sine(&file, 2.0, -200.0);
        let loudness = wav_loudness(&file, 30.0, 60).unwrap();
        assert_eq!((loudness.integrated, loudness.true_peak), (None, None));
        assert!(loudness.momentary.iter().all(|m| *m == SILENCE_DB));
    }
}
