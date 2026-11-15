use std::collections::HashMap;

use opsis_types::numbers;

use crate::pictures::{
    FullRange, Layout, aac_config, av1_config, avc_config, hevc_config, vp9_config,
};
use crate::read::{Bytes, Short, Source};
use crate::{
    AudioStream, Found, Length, ProbeError, VideoFrames, VideoStream, from_start, ticks_to_us,
};

const FORMAT: &str = "mov,mp4,m4a,3gp,3g2,mj2";
const MOOV_LIMIT: u64 = 512 * 1024 * 1024;
const TOP_LEVEL: [&[u8; 4]; 6] = [b"ftyp", b"moov", b"mdat", b"free", b"skip", b"wide"];

pub fn looks_like(head: &[u8]) -> bool {
    head.get(4..8)
        .is_some_and(|kind| TOP_LEVEL.iter().any(|t| t.as_slice() == kind))
}

fn children(data: &[u8]) -> Vec<([u8; 4], &[u8])> {
    let mut out = Vec::new();
    let mut r = Bytes::new(data);
    while r.left() >= 8 {
        let Ok(size) = r.be32() else { break };
        let Ok(kind) = r.take(4) else { break };
        let kind: [u8; 4] = kind.try_into().unwrap_or_default();
        let (header, size) = match size {
            0 => (8, u64::try_from(r.left() + 8).unwrap_or(0)),
            1 => match r.be64() {
                Ok(large) => (16, large),
                Err(_) => break,
            },
            n => (8, u64::from(n)),
        };
        let Some(body) = usize::try_from(size)
            .ok()
            .and_then(|s| s.checked_sub(header))
        else {
            break;
        };
        let Ok(payload) = r.take(body.min(r.left())) else {
            break;
        };
        out.push((kind, payload));
    }
    out
}

fn child(data: &[u8], kind: [u8; 4]) -> Option<&[u8]> {
    children(data)
        .into_iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, p)| p)
}

fn path<'a>(data: &'a [u8], kinds: &[&[u8; 4]]) -> Option<&'a [u8]> {
    kinds.iter().try_fold(data, |at, kind| child(at, **kind))
}

fn timing(full_box: &[u8]) -> Result<(u32, u64), Short> {
    let mut r = Bytes::new(full_box);
    let version = r.u8()?;
    r.skip(3)?;
    if version == 1 {
        r.skip(16)?;
        Ok((r.be32()?, r.be64()?))
    } else {
        r.skip(8)?;
        Ok((r.be32()?, u64::from(r.be32()?)))
    }
}

#[derive(Default)]
struct Samples {
    count: u64,
    time: u64,
}

struct Track {
    id: u32,
    handler: [u8; 4],
    timescale: u32,
    duration: u64,
    entry: Option<([u8; 4], Vec<u8>)>,
    samples: Samples,
}

fn track(trak: &[u8]) -> Option<Track> {
    let tkhd = child(trak, *b"tkhd")?;
    let mut r = Bytes::new(tkhd);
    let version = r.u8().ok()?;
    r.skip(3).ok()?;
    r.skip(if version == 1 { 16 } else { 8 }).ok()?;
    let id = r.be32().ok()?;
    let mdia = child(trak, *b"mdia")?;
    let (timescale, duration) = timing(child(mdia, *b"mdhd")?).ok()?;
    let handler: [u8; 4] = child(mdia, *b"hdlr")?.get(8..12)?.try_into().ok()?;
    let stbl = path(mdia, &[b"minf", b"stbl"])?;
    let entry = child(stbl, *b"stsd")
        .and_then(|stsd| children(stsd.get(8..)?).into_iter().next())
        .map(|(kind, payload)| (kind, payload.to_vec()));
    let mut samples = Samples::default();
    if let Some(stts) = child(stbl, *b"stts") {
        let mut r = Bytes::new(stts);
        if r.skip(4).is_ok() {
            let count = r.be32().unwrap_or(0);
            for _ in 0..count {
                let (Ok(n), Ok(delta)) = (r.be32(), r.be32()) else {
                    break;
                };
                samples.count += u64::from(n);
                samples.time += u64::from(n) * u64::from(delta);
            }
        }
    }
    Some(Track {
        id,
        handler,
        timescale,
        duration,
        entry,
        samples,
    })
}

fn fragments(moof: &[u8], defaults: &HashMap<u32, u32>, into: &mut HashMap<u32, Samples>) {
    for (kind, traf) in children(moof) {
        if &kind != b"traf" {
            continue;
        }
        let Some(tfhd) = child(traf, *b"tfhd") else {
            continue;
        };
        let mut r = Bytes::new(tfhd);
        let (Ok(flags), Ok(id)) = (r.be32(), r.be32()) else {
            continue;
        };
        let mut duration = defaults.get(&id).copied().unwrap_or(0);
        let skip = |bit: u32, size: usize| if flags & bit != 0 { size } else { 0 };
        if r.skip(skip(0x01, 8) + skip(0x02, 4)).is_err() {
            continue;
        }
        if flags & 0x08 != 0 {
            duration = r.be32().unwrap_or(duration);
        }
        let entry = into.entry(id).or_default();
        for (kind, trun) in children(traf) {
            if &kind != b"trun" {
                continue;
            }
            let mut r = Bytes::new(trun);
            let (Ok(flags), Ok(count)) = (r.be32(), r.be32()) else {
                continue;
            };
            let skip = |bit: u32, size: usize| if flags & bit != 0 { size } else { 0 };
            if r.skip(skip(0x01, 4) + skip(0x04, 4)).is_err() {
                continue;
            }
            for _ in 0..count {
                let own = if flags & 0x100 != 0 {
                    r.be32().ok()
                } else {
                    None
                };
                entry.time += u64::from(own.unwrap_or(duration));
                entry.count += 1;
                if r.skip(skip(0x200, 4) + skip(0x400, 4) + skip(0x800, 4))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

fn esds_config(esds: &[u8]) -> Option<(u8, Vec<u8>)> {
    let mut r = Bytes::new(esds.get(4..)?);
    let mut object = None;
    while r.left() > 2 {
        let tag = r.u8().ok()?;
        let mut size = 0usize;
        for _ in 0..4 {
            let b = r.u8().ok()?;
            size = size << 7 | usize::from(b & 0x7F);
            if b & 0x80 == 0 {
                break;
            }
        }
        match tag {
            3 => {
                r.skip(2).ok()?;
                let flags = r.u8().ok()?;
                if flags & 0x80 != 0 {
                    r.skip(2).ok()?;
                }
                if flags & 0x40 != 0 {
                    let len = r.u8().ok()?;
                    r.skip(usize::from(len)).ok()?;
                }
                if flags & 0x20 != 0 {
                    r.skip(2).ok()?;
                }
            }
            4 => {
                object = Some(r.u8().ok()?);
                r.skip(12).ok()?;
            }
            5 => return Some((object?, r.take(size.min(r.left())).ok()?.to_vec())),
            _ => r.skip(size.min(r.left())).ok()?,
        }
    }
    object.map(|o| (o, Vec::new()))
}

fn video(kind: [u8; 4], entry: &[u8]) -> Option<VideoStream> {
    let mut r = Bytes::new(entry);
    r.skip(24).ok()?;
    let width = u32::from(r.be16().ok()?);
    let height = u32::from(r.be16().ok()?);
    r.skip(46).ok()?;
    let depth = r.be16().ok()?;
    let boxes = entry.get(78..).unwrap_or_default();
    let config = |kind: &[u8; 4]| child(boxes, *kind);
    let plain = || Layout::PLAIN.pix_fmt(FullRange::Never);
    let (codec, pix_fmt) = match &kind {
        b"avc1" | b"avc3" => (
            "h264",
            config(b"avcC")
                .and_then(avc_config)
                .map_or_else(plain, |l| l.pix_fmt(FullRange::Any)),
        ),
        b"hvc1" | b"hev1" => (
            "hevc",
            config(b"hvcC")
                .and_then(hevc_config)
                .map_or_else(plain, |l| l.pix_fmt(FullRange::Only420)),
        ),
        b"vp09" => (
            "vp9",
            config(b"vpcC")
                .and_then(vp9_config)
                .map_or_else(plain, |l| l.pix_fmt(FullRange::Never)),
        ),
        b"vp08" => ("vp8", plain()),
        b"av01" => (
            "av1",
            config(b"av1C")
                .and_then(av1_config)
                .map_or_else(plain, |l| l.pix_fmt(FullRange::Never)),
        ),
        b"apco" | b"apcs" | b"apcn" | b"apch" => ("prores", "yuv422p10le".to_owned()),
        b"ap4h" | b"ap4x" => (
            "prores",
            if depth == 32 {
                "yuva444p12le"
            } else {
                "yuv444p12le"
            }
            .to_owned(),
        ),
        b"mp4v" => (
            match config(b"esds").and_then(esds_config).map(|(o, _)| o) {
                Some(0x6A) => "mpeg1video",
                Some(0x60..=0x65) => "mpeg2video",
                _ => "mpeg4",
            },
            plain(),
        ),
        b"jpeg" | b"mjpa" | b"mjpb" => ("mjpeg", "yuvj420p".to_owned()),
        b"png " => ("png", "rgb24".to_owned()),
        b"s263" | b"h263" => ("h263", plain()),
        _ => return None,
    };
    Some(VideoStream {
        codec: codec.to_owned(),
        width,
        height,
        fps: None,
        pix_fmt,
    })
}

fn audio(kind: [u8; 4], entry: &[u8]) -> Option<AudioStream> {
    let mut r = Bytes::new(entry);
    r.skip(8).ok()?;
    let version = r.be16().ok()?;
    r.skip(6).ok()?;
    let mut channels = u32::from(r.be16().ok()?);
    let bits = r.be16().ok()?;
    r.skip(4).ok()?;
    let mut rate = r.be32().ok()? >> 16;
    let boxes_at = match version {
        1 => 44,
        2 => {
            let mut v2 = Bytes::new(entry.get(32..)?);
            rate = numbers::truncate(v2.bef64().ok()?);
            channels = v2.be32().ok()?;
            64
        }
        _ => 28,
    };
    let boxes = entry.get(boxes_at..).unwrap_or_default();
    let inner = child(boxes, *b"wave").unwrap_or(boxes);
    let config = |kind: &[u8; 4]| child(boxes, *kind).or_else(|| child(inner, *kind));
    let codec = match &kind {
        b"mp4a" => match config(b"esds").and_then(esds_config) {
            Some((0x69 | 0x6B, _)) => "mp3",
            Some((0xA5, _)) => "ac3",
            Some((0xA6, _)) => "eac3",
            Some((_, asc)) => {
                if let Some(aac) = aac_config(&asc) {
                    rate = aac.rate.unwrap_or(rate);
                    if aac.channels > 0 {
                        channels = aac.channels;
                    }
                }
                "aac"
            }
            None => "aac",
        },
        b"Opus" => {
            if let Some(dops) = config(b"dOps") {
                channels = dops.get(1).map_or(channels, |&c| u32::from(c));
            }
            rate = 48_000;
            "opus"
        }
        b"fLaC" => "flac",
        b"alac" => {
            if let Some(alac) = config(b"alac").and_then(|a| a.get(4..)) {
                let mut r = Bytes::new(alac);
                if r.skip(9).is_ok() {
                    channels = r.u8().map_or(channels, u32::from);
                    if r.skip(10).is_ok() {
                        rate = r.be32().unwrap_or(rate);
                    }
                }
            }
            "alac"
        }
        b"ac-3" => "ac3",
        b"ec-3" => "eac3",
        b".mp3" => "mp3",
        b"sowt" => {
            if bits == 8 {
                "pcm_s8"
            } else {
                "pcm_s16le"
            }
        }
        b"twos" => {
            if bits == 8 {
                "pcm_s8"
            } else {
                "pcm_s16be"
            }
        }
        b"in24" => "pcm_s24be",
        b"in32" => "pcm_s32be",
        b"fl32" => "pcm_f32be",
        b"fl64" => "pcm_f64be",
        b"raw " => "pcm_u8",
        b"ulaw" => "pcm_mulaw",
        b"alaw" => "pcm_alaw",
        b"lpcm" | b"ipcm" => "pcm_s16le",
        _ => return None,
    };
    Some(AudioStream {
        codec: codec.to_owned(),
        sample_rate: Some(rate).filter(|&r| r > 0),
        channels: Some(channels).filter(|&c| c > 0),
    })
}

fn rate(count: u64, time: u64, scale: u32) -> Option<f64> {
    (count > 0 && time > 0)
        .then(|| numbers::float_wide(count) * f64::from(scale) / numbers::float_wide(time))
}

fn top_boxes(source: &mut Source) -> Result<(Vec<u8>, Vec<Vec<u8>>), ProbeError> {
    let mut moov = None;
    let mut moofs = Vec::new();
    let mut at = 0u64;
    while at + 8 <= source.len() {
        let head = source.bytes(at, 16)?;
        let mut r = Bytes::new(&head);
        let size = r.be32()?;
        let kind: [u8; 4] = r.take(4)?.try_into().map_err(|_| ProbeError::Damaged)?;
        let (header, size) = match size {
            0 => (8, source.len() - at),
            1 => (16, r.be64()?),
            n => (8, u64::from(n)),
        };
        if size < header {
            break;
        }
        if size <= MOOV_LIMIT && (&kind == b"moov" || &kind == b"moof") {
            let body = source.bytes(at + header, usize::try_from(size - header).unwrap_or(0))?;
            if &kind == b"moov" {
                moov = Some(body);
            } else {
                moofs.push(body);
            }
        }
        at += size;
    }
    Ok((moov.ok_or(ProbeError::Unknown)?, moofs))
}

pub fn probe(source: &mut Source) -> Result<Found, ProbeError> {
    let (moov, moofs) = top_boxes(source)?;
    let (scale, length) = child(&moov, *b"mvhd").map_or(Ok((0, 0)), timing)?;
    let mut defaults = HashMap::new();
    let mut whole = Length::ratio(length, u64::from(scale));
    if let Some(mvex) = child(&moov, *b"mvex") {
        for (kind, payload) in children(mvex) {
            let mut r = Bytes::new(payload);
            if &kind == b"trex"
                && r.skip(4).is_ok()
                && let (Ok(id), Ok(_), Ok(duration)) = (r.be32(), r.be32(), r.be32())
            {
                defaults.insert(id, duration);
            }
        }
    }
    let mut fragmented = HashMap::new();
    for moof in &moofs {
        fragments(moof, &defaults, &mut fragmented);
    }
    let tracks: Vec<Track> = children(&moov)
        .into_iter()
        .filter(|(k, _)| k == b"trak")
        .filter_map(|(_, trak)| track(trak))
        .map(|mut t| {
            if let Some(more) = fragmented.remove(&t.id) {
                t.samples.count += more.count;
                t.samples.time += more.time;
            }
            t
        })
        .collect();
    if whole.is_none() {
        whole = Length::longest(
            tracks
                .iter()
                .map(|t| Length::ratio(t.duration.max(t.samples.time), u64::from(t.timescale))),
        );
    }
    let video = tracks.iter().find_map(|t| {
        let (kind, entry) = t.entry.as_ref()?;
        (&t.handler == b"vide")
            .then(|| video(*kind, entry))
            .flatten()
            .map(|v| VideoStream {
                fps: rate(t.samples.count, t.samples.time, t.timescale),
                ..v
            })
    });
    let audio = tracks.iter().find_map(|t| {
        let (kind, entry) = t.entry.as_ref()?;
        (&t.handler == b"soun")
            .then(|| audio(*kind, entry))
            .flatten()
    });
    Ok(Found {
        format: FORMAT,
        length: whole,
        video,
        audio,
    })
}

#[derive(Debug, Default, Clone, Copy)]
struct Edit {
    delay: u64,
    media_time: i64,
}

fn edit_of(trak: &[u8]) -> Option<Edit> {
    let Some(elst) = path(trak, &[b"edts", b"elst"]) else {
        return Some(Edit::default());
    };
    let mut r = Bytes::new(elst);
    let version = r.u8().ok()?;
    r.skip(3).ok()?;
    let count = r.be32().ok()?;
    let mut edit = Edit::default();
    let mut playing = false;
    for _ in 0..count {
        let (duration, media_time) = if version == 1 {
            (r.be64().ok()?, r.bei64().ok()?)
        } else {
            (u64::from(r.be32().ok()?), i64::from(r.bei32().ok()?))
        };
        r.skip(4).ok()?;
        if playing {
            return None;
        }
        if media_time == -1 {
            edit.delay += duration;
        } else {
            edit.media_time = media_time;
            playing = true;
        }
    }
    Some(edit)
}

fn turned(tkhd: &[u8]) -> bool {
    let at = if tkhd.first() == Some(&1) { 52 } else { 40 };
    let entry = |i: usize| {
        tkhd.get(at + i * 4..at + i * 4 + 4)
            .and_then(|b| <[u8; 4]>::try_from(b).ok())
            .map(i32::from_be_bytes)
    };
    matches!((entry(0), entry(4)), (Some(0), Some(0)))
}

fn sample_times(stbl: &[u8]) -> Vec<i64> {
    let mut decode = Vec::new();
    if let Some(stts) = child(stbl, *b"stts") {
        let mut r = Bytes::new(stts);
        let mut time = 0i64;
        if r.skip(4).is_ok() {
            let count = r.be32().unwrap_or(0);
            for _ in 0..count {
                let (Ok(n), Ok(delta)) = (r.be32(), r.be32()) else {
                    break;
                };
                for _ in 0..n {
                    decode.push(time);
                    time += i64::from(delta);
                }
            }
        }
    }
    if let Some(ctts) = child(stbl, *b"ctts") {
        let mut r = Bytes::new(ctts);
        let mut at = 0usize;
        if r.skip(4).is_ok() {
            let count = r.be32().unwrap_or(0);
            for _ in 0..count {
                let (Ok(n), Ok(offset)) = (r.be32(), r.bei32()) else {
                    break;
                };
                for _ in 0..n {
                    if let Some(time) = decode.get_mut(at) {
                        *time += i64::from(offset);
                    }
                    at += 1;
                }
            }
        }
    }
    decode
}

struct Run {
    duration: u32,
    next: i64,
}

fn fragment_times(moof: &[u8], id: u32, run: &mut Run, into: &mut Vec<i64>) {
    for (kind, traf) in children(moof) {
        if &kind != b"traf" {
            continue;
        }
        let Some(tfhd) = child(traf, *b"tfhd") else {
            continue;
        };
        let mut r = Bytes::new(tfhd);
        let (Ok(flags), Ok(track)) = (r.be32(), r.be32()) else {
            continue;
        };
        if track != id {
            continue;
        }
        let skip = |bit: u32, size: usize| if flags & bit != 0 { size } else { 0 };
        let mut duration = run.duration;
        if r.skip(skip(0x01, 8) + skip(0x02, 4)).is_ok() && flags & 0x08 != 0 {
            duration = r.be32().unwrap_or(duration);
        }
        if let Some(tfdt) = child(traf, *b"tfdt") {
            let mut r = Bytes::new(tfdt);
            let version = r.u8().unwrap_or(0);
            if r.skip(3).is_ok() {
                let base = if version == 1 {
                    r.be64().ok()
                } else {
                    r.be32().ok().map(u64::from)
                };
                if let Some(base) = base.and_then(|b| i64::try_from(b).ok()) {
                    run.next = base;
                }
            }
        }
        for (kind, trun) in children(traf) {
            if &kind != b"trun" {
                continue;
            }
            let mut r = Bytes::new(trun);
            let (Ok(flags), Ok(count)) = (r.be32(), r.be32()) else {
                continue;
            };
            let skip = |bit: u32, size: usize| if flags & bit != 0 { size } else { 0 };
            if r.skip(skip(0x01, 4) + skip(0x04, 4)).is_err() {
                continue;
            }
            for _ in 0..count {
                let own = if flags & 0x100 != 0 {
                    r.be32().ok()
                } else {
                    None
                };
                if r.skip(skip(0x200, 4) + skip(0x400, 4)).is_err() {
                    break;
                }
                let offset = if flags & 0x800 != 0 {
                    r.bei32().unwrap_or(0)
                } else {
                    0
                };
                into.push(run.next + i64::from(offset));
                run.next += i64::from(own.unwrap_or(duration));
            }
        }
    }
}

fn delay_us(edit: Edit, movie_scale: u32) -> Option<i64> {
    ticks_to_us(i128::from(edit.delay), i128::from(movie_scale.max(1)))
}

pub fn frames(source: &mut Source) -> Result<Option<VideoFrames>, ProbeError> {
    let (moov, moofs) = top_boxes(source)?;
    let (movie_scale, _) = child(&moov, *b"mvhd").map_or(Ok((0, 0)), timing)?;
    let mut defaults = HashMap::new();
    if let Some(mvex) = child(&moov, *b"mvex") {
        for (kind, payload) in children(mvex) {
            let mut r = Bytes::new(payload);
            if &kind == b"trex"
                && r.skip(4).is_ok()
                && let (Ok(id), Ok(_), Ok(duration)) = (r.be32(), r.be32(), r.be32())
            {
                defaults.insert(id, duration);
            }
        }
    }
    let traks: Vec<&[u8]> = children(&moov)
        .into_iter()
        .filter(|(k, _)| k == b"trak")
        .map(|(_, trak)| trak)
        .collect();
    let picture = traks.iter().find_map(|trak| {
        let t = track(trak)?;
        let (kind, entry) = t.entry.as_ref()?;
        (&t.handler == b"vide")
            .then(|| video(*kind, entry))
            .flatten()
            .map(|v| (*trak, t.id, t.timescale, v))
    });
    let Some((trak, id, scale, stream)) = picture else {
        return Ok(None);
    };
    let Some(edit) = edit_of(trak) else {
        return Ok(None);
    };
    let Some(stbl) = path(trak, &[b"mdia", b"minf", b"stbl"]) else {
        return Ok(None);
    };
    let mut times = sample_times(stbl);
    let mut run = Run {
        duration: defaults.get(&id).copied().unwrap_or(0),
        next: sample_end(stbl),
    };
    for moof in &moofs {
        fragment_times(moof, id, &mut run, &mut times);
    }
    let Some(delay) = delay_us(edit, movie_scale) else {
        return Ok(None);
    };
    let presented: Vec<i64> = times
        .into_iter()
        .filter(|&t| t >= edit.media_time)
        .filter_map(|t| {
            ticks_to_us(i128::from(t - edit.media_time), i128::from(scale)).map(|us| us + delay)
        })
        .collect();
    let Some(&first) = presented.iter().min() else {
        return Ok(None);
    };
    let sound_start = traks
        .iter()
        .filter(|trak| track(trak).is_some_and(|t| &t.handler == b"soun"))
        .filter_map(|trak| edit_of(trak).and_then(|e| delay_us(e, movie_scale)))
        .min();
    let start = sound_start.map_or(first, |s| s.min(first));
    let (width, height) = child(trak, *b"tkhd")
        .filter(|tkhd| turned(tkhd))
        .map_or((stream.width, stream.height), |_| {
            (stream.height, stream.width)
        });
    Ok(Some(VideoFrames {
        times_us: from_start(presented, start),
        width,
        height,
    }))
}

fn sample_end(stbl: &[u8]) -> i64 {
    let Some(stts) = child(stbl, *b"stts") else {
        return 0;
    };
    let mut r = Bytes::new(stts);
    let mut time = 0i64;
    if r.skip(4).is_ok() {
        let count = r.be32().unwrap_or(0);
        for _ in 0..count {
            let (Ok(n), Ok(delta)) = (r.be32(), r.be32()) else {
                break;
            };
            time += i64::from(n) * i64::from(delta);
        }
    }
    time
}
