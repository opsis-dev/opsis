use std::collections::HashMap;

use crate::mpeg::stream_info;
use crate::read::{Bytes, Source};
use crate::{AudioStream, Found, Length, ProbeError, VideoStream};

const TAIL: usize = 128 * 1024;
const HEADER_PAGES: usize = 16;

struct Page {
    kind: u8,
    granule: i64,
    serial: u32,
    body: Vec<u8>,
    end: u64,
}

fn page(source: &mut Source, at: u64) -> Result<Option<Page>, ProbeError> {
    let head = source.bytes(at, 27 + 255)?;
    if !head.starts_with(b"OggS") || head.len() < 27 {
        return Ok(None);
    }
    let mut r = Bytes::new(&head[4..]);
    r.u8()?;
    let kind = r.u8()?;
    let granule = r.lei64()?;
    let serial = r.le32()?;
    r.skip(8)?;
    let segments = usize::from(r.u8()?);
    let lacing = r.take(segments)?;
    let size: usize = lacing.iter().map(|&b| usize::from(b)).sum();
    let start = at + 27 + u64::try_from(segments).unwrap_or(0);
    let body = source.bytes(start, size)?;
    Ok(Some(Page {
        kind,
        granule,
        serial,
        end: start + u64::try_from(size).unwrap_or(0),
        body,
    }))
}

enum Codec {
    Opus,
    Vorbis(u32),
    Flac(u32),
    Speex(u32),
    Theora,
}

fn identify(packet: &[u8]) -> Option<(Codec, Option<AudioStream>, Option<VideoStream>)> {
    let mut r = Bytes::new(packet);
    if packet.starts_with(b"OpusHead") {
        r.skip(9).ok()?;
        let channels = u32::from(r.u8().ok()?);
        return Some((Codec::Opus, Some(stream("opus", 48_000, channels)), None));
    }
    if packet.starts_with(b"\x01vorbis") {
        r.skip(11).ok()?;
        let channels = u32::from(r.u8().ok()?);
        let rate = r.le32().ok()?;
        return Some((
            Codec::Vorbis(rate),
            Some(stream("vorbis", rate, channels)),
            None,
        ));
    }
    if packet.starts_with(b"\x7FFLAC") {
        let info = stream_info(packet.get(17..)?)?;
        let rate = info.sample_rate.unwrap_or(0);
        return Some((Codec::Flac(rate), Some(info), None));
    }
    if packet.starts_with(b"Speex   ") {
        r.skip(36).ok()?;
        let rate = r.le32().ok()?;
        r.skip(8).ok()?;
        let channels = r.le32().ok()?;
        return Some((
            Codec::Speex(rate),
            Some(stream("speex", rate, channels)),
            None,
        ));
    }
    if packet.starts_with(b"\x80theora") {
        r.skip(14).ok()?;
        let width = r.be24().ok()?;
        let height = r.be24().ok()?;
        r.skip(2).ok()?;
        let (num, den) = (r.be32().ok()?, r.be32().ok()?);
        let fps = (num > 0 && den > 0).then(|| f64::from(num) / f64::from(den));
        let video = VideoStream {
            codec: "theora".to_owned(),
            width,
            height,
            fps,
            pix_fmt: "yuv420p".to_owned(),
        };
        return Some((Codec::Theora, None, Some(video)));
    }
    None
}

fn stream(codec: &str, rate: u32, channels: u32) -> AudioStream {
    AudioStream {
        codec: codec.to_owned(),
        sample_rate: Some(rate).filter(|&r| r > 0),
        channels: Some(channels).filter(|&c| c > 0),
    }
}

fn length(codec: &Codec, granule: i64) -> Option<Length> {
    let per = match codec {
        Codec::Opus => 48_000,
        Codec::Vorbis(rate) | Codec::Flac(rate) | Codec::Speex(rate) => *rate,
        Codec::Theora => return None,
    };
    Length::ratio(u64::try_from(granule).ok()?, u64::from(per))
}

pub fn probe(source: &mut Source) -> Result<Found, ProbeError> {
    let mut codecs: HashMap<u32, Codec> = HashMap::new();
    let mut audio = None;
    let mut video = None;
    let mut at = 0u64;
    for _ in 0..HEADER_PAGES {
        let Some(p) = page(source, at)? else { break };
        if p.kind & 0x02 == 0 {
            break;
        }
        if let Some((codec, a, v)) = identify(&p.body) {
            codecs.insert(p.serial, codec);
            audio = audio.or(a);
            video = video.or(v);
        }
        at = p.end;
    }
    if codecs.is_empty() {
        return Err(ProbeError::Unknown);
    }
    let from = source
        .len()
        .saturating_sub(u64::try_from(TAIL).unwrap_or(0));
    let tail = source.bytes(from, TAIL)?;
    let mut last: HashMap<u32, i64> = HashMap::new();
    let mut i = 0;
    while let Some(found) = tail[i..].windows(4).position(|w| w == b"OggS") {
        let start = from + u64::try_from(i + found).unwrap_or(0);
        if let Some(p) = page(source, start)?
            && p.granule >= 0
        {
            last.insert(p.serial, p.granule);
        }
        i += found + 4;
    }
    let longest = Length::longest(
        codecs
            .iter()
            .map(|(serial, codec)| length(codec, *last.get(serial)?)),
    );
    Ok(Found {
        format: "ogg",
        length: longest,
        video,
        audio,
    })
}
