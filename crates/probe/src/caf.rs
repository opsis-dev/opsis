use opsis_types::numbers;

use crate::read::{Bytes, Source};
use crate::{AudioStream, Found, Length, ProbeError};

struct Description {
    rate: f64,
    format: [u8; 4],
    flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    channels: u32,
    bits: u32,
}

fn codec(d: &Description) -> String {
    let float = d.flags & 1 != 0;
    let little = d.flags & 2 != 0;
    let order = if little { "le" } else { "be" };
    match &d.format {
        b"lpcm" if float => format!("pcm_f{}{order}", d.bits),
        b"lpcm" if d.bits == 8 => "pcm_s8".to_owned(),
        b"lpcm" => format!("pcm_s{}{order}", d.bits),
        b"aac " => "aac".to_owned(),
        b"alac" => "alac".to_owned(),
        b"ulaw" => "pcm_mulaw".to_owned(),
        b"alaw" => "pcm_alaw".to_owned(),
        b"opus" => "opus".to_owned(),
        b"flac" => "flac".to_owned(),
        b"ima4" => "adpcm_ima_qt".to_owned(),
        b".mp3" => "mp3".to_owned(),
        other => String::from_utf8_lossy(other).trim().to_lowercase(),
    }
}

pub fn probe(source: &mut Source) -> Result<Found, ProbeError> {
    let mut at = 8u64;
    let mut description = None;
    let mut data = None;
    let mut valid_frames = None;
    while at + 12 <= source.len() {
        let head = source.bytes(at, 12)?;
        let mut r = Bytes::new(&head);
        let kind: [u8; 4] = r.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
        let size = r.bei64()?;
        let body_at = at + 12;
        let room = source.len() - body_at;
        let size = u64::try_from(size).map_or(room, |s| s.min(room));
        match &kind {
            b"desc" => {
                let body = source.bytes(body_at, 32)?;
                let mut d = Bytes::new(&body);
                description = Some(Description {
                    rate: d.bef64()?,
                    format: d.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?,
                    flags: d.be32()?,
                    bytes_per_packet: d.be32()?,
                    frames_per_packet: d.be32()?,
                    channels: d.be32()?,
                    bits: d.be32()?,
                });
            }
            b"data" => data = Some(size.saturating_sub(4)),
            b"pakt" => {
                let body = source.bytes(body_at, 16)?;
                let mut p = Bytes::new(&body);
                p.skip(8)?;
                valid_frames = Some(p.bei64()?);
            }
            _ => {}
        }
        at = body_at + size;
    }
    let d = description.ok_or(ProbeError::Damaged)?;
    let frames = valid_frames
        .and_then(|f| u64::try_from(f).ok())
        .or_else(|| {
            let packets = data? / u64::from(d.bytes_per_packet).max(1);
            (d.bytes_per_packet > 0).then(|| packets * u64::from(d.frames_per_packet))
        });
    Ok(Found {
        format: "caf",
        length: frames.and_then(|f| Length::frames_at(f, d.rate)),
        video: None,
        audio: Some(AudioStream {
            codec: codec(&d),
            sample_rate: Some(numbers::truncate::<u32>(d.rate)).filter(|&r| r > 0),
            channels: Some(d.channels).filter(|&c| c > 0),
        }),
    })
}
