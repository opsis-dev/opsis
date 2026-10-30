use crate::read::{Bytes, Source};
use crate::riff::{wave_audio, wave_format};
use crate::{Found, Length, ProbeError, VideoStream};

pub const HEADER: [u8; 16] = [
    0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C,
];
const FILE_PROPERTIES: [u8; 16] = [
    0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65,
];
const STREAM_PROPERTIES: [u8; 16] = [
    0x91, 0x07, 0xDC, 0xB7, 0xB7, 0xA9, 0xCF, 0x11, 0x8E, 0xE6, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65,
];
const AUDIO: [u8; 16] = [
    0x40, 0x9E, 0x69, 0xF8, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B,
];
const VIDEO: [u8; 16] = [
    0xC0, 0xEF, 0x19, 0xBC, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B,
];
const HEADER_LIMIT: u64 = 16 * 1024 * 1024;

fn video(data: &[u8]) -> Option<VideoStream> {
    let mut r = Bytes::new(data);
    let width = r.le32().ok()?;
    let height = r.le32().ok()?;
    r.skip(3 + 16).ok()?;
    let fourcc = String::from_utf8_lossy(r.take(4).ok()?).to_uppercase();
    let codec = match fourcc.as_str() {
        "WMV1" => "wmv1",
        "WMV2" => "wmv2",
        "WMV3" => "wmv3",
        "WVC1" | "WMVA" => "vc1",
        "MP4S" | "M4S2" => "mpeg4",
        "H264" => "h264",
        _ => return None,
    };
    Some(VideoStream {
        codec: codec.to_owned(),
        width,
        height,
        fps: None,
        pix_fmt: "yuv420p".to_owned(),
    })
}

pub fn probe(source: &mut Source) -> Result<Found, ProbeError> {
    let head = source.bytes(16, 8)?;
    let size = Bytes::new(&head).le64()?;
    if size > HEADER_LIMIT {
        return Err(ProbeError::Damaged);
    }
    let header = source.bytes(0, usize::try_from(size).unwrap_or(0))?;
    let mut r = Bytes::new(header.get(30..).ok_or(ProbeError::Damaged)?);
    let mut found = Found {
        format: "asf",
        length: None,
        video: None,
        audio: None,
    };
    while r.left() >= 24 {
        let guid = r.take(16)?;
        let object = usize::try_from(r.le64()?).map_err(|_| ProbeError::Damaged)?;
        let body = r.take(object.saturating_sub(24).min(r.left()))?;
        let mut b = Bytes::new(body);
        if guid == FILE_PROPERTIES {
            b.skip(40)?;
            let play = b.le64()?;
            b.skip(8)?;
            let preroll = b.le64()?;
            found.length = preroll
                .checked_mul(10_000)
                .and_then(|p| play.checked_sub(p))
                .and_then(|ticks| Length::ratio(ticks, 10_000_000));
        } else if guid == STREAM_PROPERTIES {
            let kind = b.take(16)?;
            b.skip(16 + 8)?;
            let length = usize::try_from(b.le32()?).map_err(|_| ProbeError::Damaged)?;
            b.skip(4 + 2 + 4)?;
            let data = b.take(length.min(b.left()))?;
            if kind == AUDIO && found.audio.is_none() {
                found.audio = wave_format(data).map(|f| wave_audio(&f));
            } else if kind == VIDEO && found.video.is_none() {
                found.video = video(data);
            }
        }
    }
    Ok(found)
}
