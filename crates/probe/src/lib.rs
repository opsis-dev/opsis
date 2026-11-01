mod asf;
mod audio;
mod caf;
mod image;
mod iso;
mod matroska;
mod mpeg;
mod ogg;
mod pictures;
mod read;
mod riff;

use std::path::Path;

use opsis_types::numbers;
use serde::{Deserialize, Serialize};

use crate::read::Source;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStream {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: Option<f64>,
    pub pix_fmt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStream {
    pub codec: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    pub duration_ms: Option<u32>,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioStream>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("couldn't read the file: {0}")]
    Read(#[from] std::io::Error),
    #[error("not a video, image or sound file")]
    Unknown,
    #[error("the file is damaged")]
    Damaged,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Length {
    Ratio { count: u64, per: u64 },
    Seconds(f64),
}

impl Length {
    pub fn ratio(count: u64, per: u64) -> Option<Self> {
        (count > 0 && per > 0).then_some(Self::Ratio { count, per })
    }

    pub fn seconds(seconds: f64) -> Option<Self> {
        (seconds.is_finite() && seconds > 0.0).then_some(Self::Seconds(seconds))
    }

    pub fn frames_at(frames: u64, rate: f64) -> Option<Self> {
        if rate.fract() == 0.0 && rate > 0.0 && rate <= f64::from(u32::MAX) {
            return Self::ratio(frames, u64::from(numbers::truncate::<u32>(rate)));
        }
        Self::seconds(numbers::float_wide(frames) / rate)
    }

    fn as_seconds(self) -> f64 {
        match self {
            Self::Ratio { count, per } => numbers::float_wide(count) / numbers::float_wide(per),
            Self::Seconds(seconds) => seconds,
        }
    }

    fn longer_than(self, other: Self) -> bool {
        match (self, other) {
            (Self::Ratio { count: a, per: p }, Self::Ratio { count: b, per: q }) => {
                u128::from(a) * u128::from(q) > u128::from(b) * u128::from(p)
            }
            _ => self.as_seconds() > other.as_seconds(),
        }
    }

    pub fn longest(lengths: impl IntoIterator<Item = Option<Self>>) -> Option<Self> {
        lengths
            .into_iter()
            .flatten()
            .reduce(|a, b| if b.longer_than(a) { b } else { a })
    }

    fn milliseconds(self) -> Option<u32> {
        match self {
            Self::Ratio { count, per } => {
                let per = u128::from(per);
                let ms = (u128::from(count) * 2000 + per) / (2 * per);
                u32::try_from(ms).ok().filter(|&ms| ms > 0)
            }
            Self::Seconds(seconds) => {
                let ms = (seconds * 1000.0).round();
                (ms.is_finite() && ms > 0.0 && ms <= f64::from(u32::MAX))
                    .then_some(numbers::truncate(ms))
            }
        }
    }
}

pub(crate) struct Found {
    pub format: &'static str,
    pub length: Option<Length>,
    pub video: Option<VideoStream>,
    pub audio: Option<AudioStream>,
}

pub(crate) const STILL_FPS: f64 = 25.0;

const SNIFF: usize = 4096;

pub fn probe(path: &Path) -> Result<Probe, ProbeError> {
    let mut source = Source::open(path)?;
    let head = source.bytes(0, SNIFF)?;
    let found = sniffed(&mut source, &head)?;
    Ok(Probe {
        duration_ms: found.length.and_then(Length::milliseconds),
        format: found.format.to_owned(),
        video: found.video.map(|v| VideoStream {
            fps: v.fps.map(rational_rate),
            ..v
        }),
        audio: found.audio,
    })
}

const RATE_SNAP: f64 = 1e-6;

fn rational_rate(fps: f64) -> f64 {
    let whole = fps.round();
    let ntsc = (fps * 1.001).round() / 1.001;
    [whole, ntsc]
        .into_iter()
        .find(|r| *r > 0.0 && ((fps - r) / r).abs() < RATE_SNAP)
        .unwrap_or(fps)
}

fn sniffed(source: &mut Source, head: &[u8]) -> Result<Found, ProbeError> {
    let starts = |magic: &[u8]| head.starts_with(magic);
    let at = |offset: usize, magic: &[u8]| head.get(offset..offset + magic.len()) == Some(magic);
    if iso::looks_like(head) {
        return iso::probe(source);
    }
    if starts(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return matroska::probe(source);
    }
    if starts(b"RIFF") && at(8, b"WAVE") || starts(b"RF64") {
        return riff::wav(source);
    }
    if starts(b"RIFF") && at(8, b"AVI ") {
        return riff::avi(source);
    }
    if starts(b"RIFF") && at(8, b"WEBP") {
        return image::webp(head);
    }
    if starts(b"FORM") && (at(8, b"AIFF") || at(8, b"AIFC")) {
        return audio::aiff(source);
    }
    if starts(b"caff") {
        return caf::probe(source);
    }
    if starts(b"OggS") {
        return ogg::probe(source);
    }
    if starts(&asf::HEADER) {
        return asf::probe(source);
    }
    if starts(&[0x89, b'P', b'N', b'G']) {
        return image::png(head);
    }
    if starts(&[0xFF, 0xD8, 0xFF]) {
        return image::jpeg(source);
    }
    if starts(b"GIF87a") || starts(b"GIF89a") {
        return image::gif(source);
    }
    if image::is_svg(head) {
        return image::svg(source);
    }
    mpeg::probe(source, head)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::Length;

    proptest! {
        #[test]
        fn rounds_a_length_to_the_nearest_millisecond_with_halves_up(count in 1u64..10_000_000_000, per in 1u64..10_000_000) {
            let Some(ms) = Length::ratio(count, per).and_then(Length::milliseconds) else {
                return Ok(());
            };
            let gap = i128::from(ms) * i128::from(per) - i128::from(count) * 1000;
            prop_assert!(2 * gap.abs() <= i128::from(per));
            prop_assert!(2 * gap != -i128::from(per));
        }
    }

    #[test]
    fn rounds_an_exact_half_millisecond_up() {
        assert_eq!(
            Length::ratio(4044, 8000).and_then(Length::milliseconds),
            Some(506)
        );
    }
}
