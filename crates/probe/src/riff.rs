use crate::read::{Bytes, Source};
use crate::{AudioStream, Found, Length, ProbeError, VideoStream};

const LIST_LIMIT: u64 = 16 * 1024 * 1024;

struct Chunk {
    id: [u8; 4],
    data: u64,
    size: u64,
}

fn chunk(source: &mut Source, at: u64) -> Result<Option<Chunk>, ProbeError> {
    let head = source.bytes(at, 8)?;
    if head.len() < 8 {
        return Ok(None);
    }
    let mut r = Bytes::new(&head);
    let id: [u8; 4] = r.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
    Ok(Some(Chunk {
        id,
        data: at + 8,
        size: u64::from(r.le32()?),
    }))
}

fn next(c: &Chunk) -> u64 {
    c.data + c.size + c.size % 2
}

pub struct WaveFormat {
    pub tag: u16,
    pub channels: u16,
    pub rate: u32,
    pub byte_rate: u32,
    pub bits: u16,
}

pub fn wave_format(data: &[u8]) -> Option<WaveFormat> {
    let mut r = Bytes::new(data);
    let mut tag = r.le16().ok()?;
    let channels = r.le16().ok()?;
    let rate = r.le32().ok()?;
    let byte_rate = r.le32().ok()?;
    r.skip(2).ok()?;
    let bits = r.le16().unwrap_or(0);
    if tag == 0xFFFE && r.skip(8).is_ok() {
        tag = r.le16().unwrap_or(tag);
    }
    Some(WaveFormat {
        tag,
        channels,
        rate,
        byte_rate,
        bits,
    })
}

pub fn wave_codec(format: &WaveFormat) -> String {
    match (format.tag, format.bits) {
        (1, 8) => "pcm_u8".to_owned(),
        (1, bits) => format!("pcm_s{bits}le"),
        (3, bits) => format!("pcm_f{bits}le"),
        (6, _) => "pcm_alaw".to_owned(),
        (7, _) => "pcm_mulaw".to_owned(),
        (2, _) => "adpcm_ms".to_owned(),
        (0x11, _) => "adpcm_ima_wav".to_owned(),
        (0x50, _) => "mp2".to_owned(),
        (0x55, _) => "mp3".to_owned(),
        (0x160, _) => "wmav1".to_owned(),
        (0x161, _) => "wmav2".to_owned(),
        (0x162, _) => "wmapro".to_owned(),
        (0x163, _) => "wmalossless".to_owned(),
        (0xFF | 0x1610, _) => "aac".to_owned(),
        (tag, _) => format!("0x{tag:04x}"),
    }
}

pub fn wave_audio(format: &WaveFormat) -> AudioStream {
    AudioStream {
        codec: wave_codec(format),
        sample_rate: Some(format.rate).filter(|&r| r > 0),
        channels: Some(u32::from(format.channels)).filter(|&c| c > 0),
    }
}

pub fn wav(source: &mut Source) -> Result<Found, ProbeError> {
    let mut at = 12;
    let mut format = None;
    let mut data_size = None;
    let mut wide_size = None;
    while let Some(c) = chunk(source, at)? {
        match &c.id {
            b"ds64" => {
                let body = source.bytes(c.data, 16)?;
                let mut r = Bytes::new(&body);
                r.skip(8)?;
                wide_size = Some(r.le64()?);
            }
            b"fmt " => {
                let body = source.bytes(c.data, usize::try_from(c.size.min(64)).unwrap_or(0))?;
                format = wave_format(&body);
            }
            b"data" => {
                let room = source.len().saturating_sub(c.data);
                let size = if c.size == u64::from(u32::MAX) {
                    wide_size.unwrap_or(room)
                } else {
                    c.size
                };
                data_size = Some(size.min(room));
                break;
            }
            _ => {}
        }
        at = next(&c);
    }
    let format = format.ok_or(ProbeError::Damaged)?;
    Ok(Found {
        format: "wav",
        length: data_size.and_then(|size| Length::ratio(size, u64::from(format.byte_rate))),
        video: None,
        audio: Some(wave_audio(&format)),
    })
}

fn avi_video(handler: [u8; 4], strf: &[u8]) -> Option<(String, String, u32, u32)> {
    let mut r = Bytes::new(strf);
    r.skip(4).ok()?;
    let width = r.le32().ok()?;
    let height = r.le32().ok()?.cast_signed().unsigned_abs();
    r.skip(4).ok()?;
    let compression: [u8; 4] = r.take(4).ok()?.try_into().ok()?;
    let fourcc = if compression == [0; 4] {
        handler
    } else {
        compression
    };
    let name = String::from_utf8_lossy(&fourcc).to_uppercase();
    let (codec, pix) = match name.trim() {
        "MJPG" | "AVRN" | "LJPG" | "JPGL" => ("mjpeg", "yuvj420p"),
        "H264" | "X264" | "AVC1" | "DAVC" => ("h264", "yuv420p"),
        "HEVC" | "H265" | "HVC1" => ("hevc", "yuv420p"),
        "XVID" | "DIVX" | "DX50" | "FMP4" | "MP4V" => ("mpeg4", "yuv420p"),
        "VP80" => ("vp8", "yuv420p"),
        "VP90" => ("vp9", "yuv420p"),
        "" => ("rawvideo", "bgr24"),
        other => return Some((other.to_lowercase(), "yuv420p".to_owned(), width, height)),
    };
    Some((codec.to_owned(), pix.to_owned(), width, height))
}

pub fn avi(source: &mut Source) -> Result<Found, ProbeError> {
    let mut at = 12;
    let mut hdrl = None;
    while let Some(c) = chunk(source, at)? {
        if &c.id == b"LIST" && c.size <= LIST_LIMIT {
            let body = source.bytes(c.data, usize::try_from(c.size).unwrap_or(0))?;
            if body.starts_with(b"hdrl") {
                hdrl = Some(body);
                break;
            }
        }
        at = next(&c);
    }
    let hdrl = hdrl.ok_or(ProbeError::Damaged)?;
    let mut video = None;
    let mut audio = None;
    let mut longest = None;
    let mut r = Bytes::new(&hdrl[4..]);
    while r.left() >= 8 {
        let id: [u8; 4] = r.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
        let size = usize::try_from(r.le32()?).map_err(|_| ProbeError::Damaged)?;
        let body = r.take(size.min(r.left()))?;
        if size % 2 == 1 && r.left() > 0 {
            r.skip(1)?;
        }
        if &id != b"LIST" || !body.starts_with(b"strl") {
            continue;
        }
        let mut strh = None;
        let mut strf = None;
        let mut inner = Bytes::new(&body[4..]);
        while inner.left() >= 8 {
            let id: [u8; 4] = inner.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
            let size = usize::try_from(inner.le32()?).map_err(|_| ProbeError::Damaged)?;
            let part = inner.take(size.min(inner.left()))?;
            if size % 2 == 1 && inner.left() > 0 {
                inner.skip(1)?;
            }
            match &id {
                b"strh" => strh = Some(part),
                b"strf" => strf = Some(part),
                _ => {}
            }
        }
        let (Some(strh), Some(strf)) = (strh, strf) else {
            continue;
        };
        let mut h = Bytes::new(strh);
        let kind: [u8; 4] = h.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
        let handler: [u8; 4] = h.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
        h.skip(12)?;
        let scale = h.le32()?;
        let rate = h.le32()?;
        h.skip(4)?;
        let length = h.le32()?;
        let fps = (scale > 0 && rate > 0).then(|| f64::from(rate) / f64::from(scale));
        longest = Length::longest([
            longest,
            Length::ratio(u64::from(length) * u64::from(scale), u64::from(rate)),
        ]);
        match &kind {
            b"vids" if video.is_none() => {
                if let Some((codec, pix_fmt, width, height)) = avi_video(handler, strf) {
                    video = Some(VideoStream {
                        codec,
                        width,
                        height,
                        fps,
                        pix_fmt,
                    });
                }
            }
            b"auds" if audio.is_none() => {
                audio = wave_format(strf).map(|f| wave_audio(&f));
            }
            _ => {}
        }
    }
    Ok(Found {
        format: "avi",
        length: longest,
        video,
        audio,
    })
}
