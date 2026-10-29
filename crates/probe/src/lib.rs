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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub(crate) struct Found {
    pub format: &'static str,
    pub seconds: Option<f64>,
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
        duration_ms: found.seconds.and_then(milliseconds),
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

fn milliseconds(seconds: f64) -> Option<u32> {
    let ms = (seconds * 1000.0).round();
    (ms.is_finite() && ms > 0.0 && ms <= f64::from(u32::MAX)).then_some(numbers::truncate(ms))
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
