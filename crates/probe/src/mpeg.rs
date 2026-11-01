use crate::pictures::aac_rate_of;
use crate::read::{Bits, Bytes, Source};
use crate::{AudioStream, Found, Length, ProbeError};

const SEARCH: usize = 64 * 1024;

pub fn stream_info_with_length(data: &[u8]) -> Option<(AudioStream, Option<Length>)> {
    let mut bits = Bits::new(data.get(10..)?);
    let rate = bits.bits(20).ok()?;
    let channels = bits.bits(3).ok()? + 1;
    bits.skip(5).ok()?;
    let total = u64::from(bits.bits(4).ok()?) << 32 | u64::from(bits.bits(32).ok()?);
    let stream = AudioStream {
        codec: "flac".to_owned(),
        sample_rate: Some(rate).filter(|&r| r > 0),
        channels: Some(channels),
    };
    Some((stream, Length::ratio(total, u64::from(rate))))
}

pub fn stream_info(data: &[u8]) -> Option<AudioStream> {
    stream_info_with_length(data).map(|(s, _)| s)
}

fn id3_end(head: &[u8]) -> u64 {
    if !head.starts_with(b"ID3") || head.len() < 10 {
        return 0;
    }
    let size = head[6..10]
        .iter()
        .fold(0u64, |v, &b| v << 7 | u64::from(b & 0x7F));
    let footer = if head[5] & 0x10 != 0 { 10 } else { 0 };
    10 + size + footer
}

fn flac(source: &mut Source, at: u64) -> Result<Found, ProbeError> {
    let mut block = at + 4;
    loop {
        let head = source.bytes(block, 4)?;
        let mut r = Bytes::new(&head);
        let kind = r.u8()?;
        let size = r.be24()?;
        if kind & 0x7F == 0 {
            let body = source.bytes(block + 4, usize::try_from(size).unwrap_or(0))?;
            let (audio, length) = stream_info_with_length(&body).ok_or(ProbeError::Damaged)?;
            return Ok(Found {
                format: "flac",
                length,
                video: None,
                audio: Some(audio),
            });
        }
        if kind & 0x80 != 0 {
            return Err(ProbeError::Damaged);
        }
        block += 4 + u64::from(size);
    }
}

struct Frame {
    layer: u32,
    rate: u32,
    bitrate: u32,
    channels: u32,
    samples: u32,
    length: u32,
    mpeg1: bool,
}

const BITRATES: [[u32; 15]; 5] = [
    [
        0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448,
    ],
    [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384,
    ],
    [
        0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ],
    [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256,
    ],
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
];

fn frame(bytes: &[u8]) -> Option<Frame> {
    let mut bits = Bits::new(bytes);
    if bits.bits(11).ok()? != 0x7FF {
        return None;
    }
    let version = bits.bits(2).ok()?;
    let layer_bits = bits.bits(2).ok()?;
    if version == 1 || layer_bits == 0 {
        return None;
    }
    bits.skip(1).ok()?;
    let rate_index = bits.bits(4).ok()?;
    let sampling = bits.bits(2).ok()?;
    let padding = bits.bits(1).ok()?;
    bits.skip(1).ok()?;
    let mode = bits.bits(2).ok()?;
    if rate_index == 0 || rate_index == 15 || sampling == 3 {
        return None;
    }
    let layer = 4 - layer_bits;
    let mpeg1 = version == 3;
    let table = match (mpeg1, layer) {
        (true, 1) => 0,
        (true, 2) => 1,
        (true, _) => 2,
        (false, 1) => 3,
        (false, _) => 4,
    };
    let bitrate = BITRATES[table][usize::try_from(rate_index).ok()?] * 1000;
    let base = [44_100, 48_000, 32_000][usize::try_from(sampling).ok()?];
    let rate = match version {
        3 => base,
        2 => base / 2,
        _ => base / 4,
    };
    let samples = match (layer, mpeg1) {
        (1, _) => 384,
        (2, _) | (3, true) => 1152,
        _ => 576,
    };
    let length = if layer == 1 {
        (12 * bitrate / rate + padding) * 4
    } else {
        samples / 8 * bitrate / rate + padding
    };
    Some(Frame {
        layer,
        rate,
        bitrate,
        channels: if mode == 3 { 1 } else { 2 },
        samples,
        length,
        mpeg1,
    })
}

fn vbr_frames(bytes: &[u8], f: &Frame) -> Option<u32> {
    let side = match (f.mpeg1, f.channels) {
        (true, 1) => 17,
        (true, _) => 32,
        (false, 1) => 9,
        (false, _) => 17,
    };
    let at = 4 + side;
    let tag = bytes.get(at..at + 4)?;
    if tag == b"Xing" || tag == b"Info" {
        let mut r = Bytes::new(bytes.get(at + 4..)?);
        let flags = r.be32().ok()?;
        return (flags & 1 != 0).then(|| r.be32().ok()).flatten();
    }
    if bytes.get(36..40)? == b"VBRI" {
        let mut r = Bytes::new(bytes.get(40..)?);
        r.skip(10).ok()?;
        return r.be32().ok();
    }
    None
}

fn mp3(source: &mut Source, at: u64, bytes: &[u8], f: &Frame) -> Found {
    let length = vbr_frames(bytes, f).map_or_else(
        || {
            let bits = source.len().saturating_sub(at).checked_mul(8)?;
            Length::ratio(bits, u64::from(f.bitrate))
        },
        |frames| Length::ratio(u64::from(frames) * u64::from(f.samples), u64::from(f.rate)),
    );
    Found {
        format: "mp3",
        length,
        video: None,
        audio: Some(AudioStream {
            codec: match f.layer {
                1 => "mp1",
                2 => "mp2",
                _ => "mp3",
            }
            .to_owned(),
            sample_rate: Some(f.rate),
            channels: Some(f.channels),
        }),
    }
}

fn adts(source: &mut Source, start: u64) -> Result<Found, ProbeError> {
    let mut at = start;
    let mut frames = 0u64;
    let mut stream = None;
    while at + 7 <= source.len() {
        let head = source.bytes(at, 7)?;
        let mut bits = Bits::new(&head);
        if bits.bits(12)? != 0xFFF {
            break;
        }
        bits.skip(4)?;
        bits.skip(2)?;
        let rate_index = bits.bits(4)?;
        bits.skip(1)?;
        let channels = bits.bits(3)?;
        bits.skip(4)?;
        let length = bits.bits(13)?;
        bits.skip(11)?;
        let blocks = bits.bits(2)? + 1;
        if length < 7 {
            break;
        }
        if stream.is_none() {
            stream = Some((aac_rate_of(rate_index), channels));
        }
        frames += u64::from(blocks);
        at += u64::from(length);
    }
    let (rate, channels) = stream.ok_or(ProbeError::Damaged)?;
    Ok(Found {
        format: "aac",
        length: rate.and_then(|r| Length::ratio(frames * 1024, u64::from(r))),
        video: None,
        audio: Some(AudioStream {
            codec: "aac".to_owned(),
            sample_rate: rate,
            channels: Some(channels).filter(|&c| c > 0),
        }),
    })
}

pub fn probe(source: &mut Source, head: &[u8]) -> Result<Found, ProbeError> {
    let start = id3_end(head);
    let window = source.bytes(start, SEARCH)?;
    if window.starts_with(b"fLaC") {
        return flac(source, start);
    }
    if window.len() >= 2 && window[0] == 0xFF && window[1] & 0xF6 == 0xF0 {
        return adts(source, start);
    }
    for i in 0..window.len().saturating_sub(4) {
        if window[i] != 0xFF || window[i + 1] & 0xE0 != 0xE0 {
            continue;
        }
        let Some(f) = frame(&window[i..]) else {
            continue;
        };
        let next = i + usize::try_from(f.length).unwrap_or(usize::MAX);
        let follows = window
            .get(next..)
            .is_none_or(|rest| rest.len() < 4 || frame(rest).is_some());
        if f.length > 0 && follows {
            let at = start + u64::try_from(i).unwrap_or(0);
            return Ok(mp3(source, at, &window[i..], &f));
        }
    }
    Err(ProbeError::Unknown)
}
