use std::path::Path;

use opsis_compile::text::round_to;
use opsis_types::mix::{SILENCE_DB, StemLevels};
use opsis_types::numbers;

const EPSILON: f64 = 1e-6;

fn to_db(value: f64) -> f64 {
    if value <= EPSILON {
        SILENCE_DB
    } else {
        round_to(20.0 * value.log10(), 1)
    }
}

#[derive(Debug)]
pub struct Pcm {
    pub sample_rate: f64,
    pub channels: usize,
    pub frames: usize,
}

pub fn frame_levels(
    pcm: &Pcm,
    samples: impl IntoIterator<Item = f32>,
    fps: f64,
    frames: usize,
) -> StemLevels {
    let per = pcm.sample_rate / fps;
    let mut samples = samples.into_iter();
    let mut at = 0usize;
    let mut levels = StemLevels {
        fps,
        peak: Vec::with_capacity(frames),
        rms: Vec::with_capacity(frames),
    };
    for i in 0..frames {
        let start = numbers::truncate::<usize>((numbers::float(i) * per).floor());
        let end = numbers::truncate::<usize>(
            (numbers::float(i + 1) * per)
                .floor()
                .min(numbers::float(pcm.frames)),
        );
        while at < start {
            samples.by_ref().take(pcm.channels).for_each(drop);
            at += 1;
        }
        let (mut peak, mut squares, mut count) = (0f64, 0f64, 0usize);
        while at < end {
            for v in samples.by_ref().take(pcm.channels) {
                let v = f64::from(v);
                peak = peak.max(v.abs());
                squares += v * v;
                count += 1;
            }
            at += 1;
        }
        levels.peak.push(to_db(peak));
        levels.rms.push(to_db(if count > 0 {
            (squares / numbers::float(count)).sqrt()
        } else {
            0.0
        }));
    }
    levels
}

#[derive(Debug, thiserror::Error)]
pub enum WavError {
    #[error("Not a WAV file: {0}")]
    Read(#[from] hound::Error),
}

pub fn wav_levels(file: &Path, fps: f64, frames: usize) -> Result<StemLevels, WavError> {
    let mut reader = hound::WavReader::open(file)?;
    let spec = reader.spec();
    let pcm = Pcm {
        sample_rate: f64::from(spec.sample_rate),
        channels: usize::from(spec.channels),
        frames: reader.duration() as usize,
    };
    Ok(match spec.sample_format {
        hound::SampleFormat::Float => frame_levels(
            &pcm,
            reader.samples::<f32>().map_while(Result::ok),
            fps,
            frames,
        ),
        hound::SampleFormat::Int => {
            let scale = 2f64.powi(i32::from(spec.bits_per_sample) - 1);
            let samples = reader
                .samples::<i32>()
                .map_while(Result::ok)
                .map(move |s| numbers::narrow(f64::from(s) / scale));
            frame_levels(&pcm, samples, fps, frames)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(file: &Path, spec: hound::WavSpec, samples: &[f32]) {
        let mut writer = hound::WavWriter::create(file, spec).unwrap();
        for s in samples {
            match spec.sample_format {
                hound::SampleFormat::Float => writer.write_sample(*s).unwrap(),
                hound::SampleFormat::Int => writer
                    .write_sample(numbers::truncate::<i16>(f64::from(*s * 32768.0)))
                    .unwrap(),
            }
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn measures_peak_and_loudness_per_video_frame() {
        let pcm = Pcm {
            sample_rate: 4.0,
            channels: 1,
            frames: 4,
        };
        let levels = frame_levels(&pcm, [0.5, -1.0, 0.0, 0.0], 2.0, 2);
        assert_eq!(
            (levels.peak, levels.rms),
            (vec![0.0, -120.0], vec![-2.0, -120.0])
        );
    }

    #[test]
    fn reads_16_bit_and_float_stems_from_disk_without_loading_them_whole() {
        let dir = tempfile::tempdir().unwrap();
        let int = dir.path().join("int.wav");
        write_wav(
            &int,
            hound::WavSpec {
                channels: 2,
                sample_rate: 4,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
            &[0.5, -1.0, 0.0, 0.25],
        );
        let levels = wav_levels(&int, 4.0, 2).unwrap();
        assert_eq!(levels.peak, [0.0, -12.0]);
        let float = dir.path().join("float.wav");
        write_wav(
            &float,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
            &[0.75, 0.0],
        );
        assert_eq!(
            wav_levels(&float, 4.0, 2).unwrap().peak,
            [round_to(20.0 * 0.75f64.log10(), 1), -120.0]
        );
        assert!(wav_levels(&dir.path().join("missing.wav"), 30.0, 1).is_err());
    }
}
