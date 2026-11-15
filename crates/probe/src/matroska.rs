use opsis_types::numbers;

use crate::ffv1::{Parameters, ffv1_parameters};
use crate::pictures::{
    FullRange, Layout, aac_config, av1_config, avc_config, hevc_config, vp9_frame,
};
use crate::read::{Bytes, Source};
use crate::{
    AudioStream, Found, Length, ProbeError, VideoFrames, VideoStream, from_start, ticks_to_us,
};

const FORMAT: &str = "matroska,webm";

const SEGMENT: u32 = 0x1853_8067;
const INFO: u32 = 0x1549_A966;
const TRACKS: u32 = 0x1654_AE6B;
const CLUSTER: u32 = 0x1F43_B675;
const TOP_LEVEL: [u32; 8] = [
    0x114D_9B74,
    INFO,
    TRACKS,
    CLUSTER,
    0x1C53_BB6B,
    0x1254_C367,
    0x1941_A469,
    0x1043_A770,
];
const TRACK_ENTRY: u32 = 0xAE;
const TIMESTAMP: u32 = 0xE7;
const SIMPLE_BLOCK: u32 = 0xA3;
const BLOCK_GROUP: u32 = 0xA0;
const BLOCK: u32 = 0xA1;
const FRAME_SAMPLE: usize = 24;
const FIRST_FRAME_SAMPLE: usize = 4096;
const BLOCK_TIME_AND_FLAGS: usize = 3;
const RATE_SAMPLES: usize = 30;
const ELEMENT_LIMIT: u64 = 64 * 1024 * 1024;

struct Element {
    id: u32,
    data: u64,
    size: Option<u64>,
}

fn vint(bytes: &[u8], keep_marker: bool) -> Option<(u64, usize)> {
    let first = *bytes.first()?;
    if first == 0 {
        return None;
    }
    let len = usize::try_from(first.leading_zeros()).ok()? + 1;
    let mut value = if keep_marker {
        u64::from(first)
    } else {
        u64::from(first) & (0xFF >> len)
    };
    for &b in bytes.get(1..len)? {
        value = value << 8 | u64::from(b);
    }
    Some((value, len))
}

const fn unknown(value: u64, len: usize) -> bool {
    value == (1u64 << (7 * len)) - 1
}

fn element(source: &mut Source, at: u64) -> Result<Option<Element>, ProbeError> {
    let head = source.bytes(at, 12)?;
    let Some((id, id_len)) = vint(&head, true) else {
        return Ok(None);
    };
    let Some((size, size_len)) = head.get(id_len..).and_then(|rest| vint(rest, false)) else {
        return Ok(None);
    };
    Ok(Some(Element {
        id: u32::try_from(id).map_err(|_| ProbeError::Damaged)?,
        data: at + u64::try_from(id_len + size_len).unwrap_or(0),
        size: (!unknown(size, size_len)).then_some(size),
    }))
}

fn items(data: &[u8]) -> Vec<(u32, &[u8])> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < data.len() {
        let Some((id, id_len)) = vint(&data[at..], true) else {
            break;
        };
        let Some((size, size_len)) = data.get(at + id_len..).and_then(|r| vint(r, false)) else {
            break;
        };
        let start = at + id_len + size_len;
        let end = usize::try_from(size)
            .ok()
            .and_then(|s| start.checked_add(s))
            .unwrap_or(data.len());
        let Some(body) = data.get(start..end.min(data.len())) else {
            break;
        };
        out.push((u32::try_from(id).unwrap_or(0), body));
        at = end;
    }
    out
}

fn uint(data: &[u8]) -> u64 {
    data.iter().fold(0, |v, &b| v << 8 | u64::from(b))
}

fn float(data: &[u8]) -> Option<f64> {
    let mut r = Bytes::new(data);
    match data.len() {
        4 => r.bef32().ok().map(f64::from),
        8 => r.bef64().ok(),
        _ => None,
    }
}

fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data)
        .trim_end_matches('\0')
        .to_owned()
}

#[derive(Default)]
struct TrackEntry {
    number: u64,
    kind: u64,
    codec: String,
    private: Vec<u8>,
    default_duration: Option<u64>,
    width: u32,
    height: u32,
    rate: Option<f64>,
    channels: Option<u64>,
    depth: Option<u64>,
    codec_delay: u64,
}

fn track(data: &[u8]) -> TrackEntry {
    let mut t = TrackEntry::default();
    for (id, body) in items(data) {
        match id {
            0xD7 => t.number = uint(body),
            0x83 => t.kind = uint(body),
            0x86 => t.codec = text(body),
            0x63A2 => t.private = body.to_vec(),
            0x0023_E383 => t.default_duration = Some(uint(body)),
            0x56AA => t.codec_delay = uint(body),
            0xE0 => {
                for (id, body) in items(body) {
                    match id {
                        0xB0 => t.width = u32::try_from(uint(body)).unwrap_or(0),
                        0xBA => t.height = u32::try_from(uint(body)).unwrap_or(0),
                        _ => {}
                    }
                }
            }
            0xE1 => {
                for (id, body) in items(body) {
                    match id {
                        0xB5 => t.rate = float(body),
                        0x9F => t.channels = Some(uint(body)),
                        0x6264 => t.depth = Some(uint(body)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    t
}

struct Blocks {
    video: Option<u64>,
    first_frame: Option<Vec<u8>>,
    times: Vec<i64>,
    first: Option<i64>,
    last: Option<i64>,
    every: bool,
    starts: Vec<(u64, i64)>,
}

fn block(
    source: &mut Source,
    at: u64,
    cluster_time: i64,
    blocks: &mut Blocks,
) -> Result<(), ProbeError> {
    let head = source.bytes(at, 12 + FRAME_SAMPLE)?;
    let Some((track, len)) = vint(&head, false) else {
        return Ok(());
    };
    let mut r = Bytes::new(head.get(len..).ok_or(ProbeError::Damaged)?);
    let time = cluster_time + i64::from(r.bei16()?);
    r.u8()?;
    blocks.first = Some(blocks.first.map_or(time, |f| f.min(time)));
    if blocks.every {
        match blocks.starts.iter_mut().find(|(t, _)| *t == track) {
            Some((_, start)) => *start = (*start).min(time),
            None => blocks.starts.push((track, time)),
        }
    }
    blocks.last = Some(blocks.last.map_or(time, |l| l.max(time)));
    if Some(track) == blocks.video {
        if blocks.every || blocks.times.len() < RATE_SAMPLES {
            blocks.times.push(time);
        }
        if blocks.first_frame.is_none() && !blocks.every {
            let frame_at = at + u64::try_from(len + BLOCK_TIME_AND_FLAGS).unwrap_or(0);
            blocks.first_frame = Some(source.bytes(frame_at, FIRST_FRAME_SAMPLE)?);
        }
    }
    Ok(())
}

fn cluster(source: &mut Source, el: &Element, blocks: &mut Blocks) -> Result<u64, ProbeError> {
    let end = el.size.map_or(source.len(), |s| el.data + s);
    let mut at = el.data;
    let mut time = 0i64;
    while at < end {
        let Some(child) = element(source, at)? else {
            break;
        };
        if el.size.is_none() && TOP_LEVEL.contains(&child.id) {
            return Ok(at);
        }
        let size = child.size.unwrap_or(0);
        match child.id {
            TIMESTAMP => {
                let bytes = source.bytes(child.data, usize::try_from(size).unwrap_or(0))?;
                time = i64::try_from(uint(&bytes)).unwrap_or(0);
            }
            SIMPLE_BLOCK => block(source, child.data, time, blocks)?,
            BLOCK_GROUP => {
                let mut inner = child.data;
                while inner < child.data + size {
                    let Some(part) = element(source, inner)? else {
                        break;
                    };
                    if part.id == BLOCK {
                        block(source, part.data, time, blocks)?;
                    }
                    inner = part.data + part.size.unwrap_or(0);
                }
            }
            _ => {}
        }
        at = child.data + size;
    }
    Ok(end)
}

const BITMAP_HEADER: usize = 40;
const FOURCC_AT: usize = 16;

fn ffv1_record(t: &TrackEntry) -> Option<&[u8]> {
    match t.codec.as_str() {
        "V_FFV1" => Some(&t.private),
        "V_MS/VFW/FOURCC" if t.private.get(FOURCC_AT..FOURCC_AT + 4) == Some(b"FFV1") => {
            t.private.get(BITMAP_HEADER..)
        }
        _ => None,
    }
}

fn video_stream(t: &TrackEntry, first_frame: Option<&[u8]>) -> VideoStream {
    if let Some(record) = ffv1_record(t) {
        let pix_fmt = ffv1_parameters(record, first_frame)
            .and_then(Parameters::pix_fmt)
            .unwrap_or_else(|| Layout::PLAIN.pix_fmt(FullRange::Never));
        return VideoStream {
            codec: "ffv1".to_owned(),
            width: t.width,
            height: t.height,
            fps: None,
            pix_fmt,
        };
    }
    let plain = || Layout::PLAIN.pix_fmt(FullRange::Never);
    let (codec, pix_fmt) = match t.codec.as_str() {
        "V_VP9" => (
            "vp9",
            first_frame
                .and_then(vp9_frame)
                .map_or_else(plain, |l| l.pix_fmt(FullRange::Never)),
        ),
        "V_VP8" => ("vp8", plain()),
        "V_AV1" => (
            "av1",
            av1_config(&t.private).map_or_else(plain, |l| l.pix_fmt(FullRange::Never)),
        ),
        "V_MPEG4/ISO/AVC" => (
            "h264",
            avc_config(&t.private).map_or_else(plain, |l| l.pix_fmt(FullRange::Any)),
        ),
        "V_MPEGH/ISO/HEVC" => (
            "hevc",
            hevc_config(&t.private).map_or_else(plain, |l| l.pix_fmt(FullRange::Only420)),
        ),
        "V_MJPEG" => ("mjpeg", "yuvj420p".to_owned()),
        "V_PRORES" => ("prores", "yuv422p10le".to_owned()),
        "V_THEORA" => ("theora", plain()),
        "V_MPEG1" => ("mpeg1video", plain()),
        "V_MPEG2" => ("mpeg2video", plain()),
        _ => ("mpeg4", plain()),
    };
    VideoStream {
        codec: codec.to_owned(),
        width: t.width,
        height: t.height,
        fps: None,
        pix_fmt,
    }
}

fn audio_stream(t: &TrackEntry) -> AudioStream {
    let bits = t.depth.unwrap_or(16);
    let mut rate = t.rate.map(numbers::truncate::<u32>);
    let mut channels = t.channels.and_then(|c| u32::try_from(c).ok());
    let codec = match t.codec.as_str() {
        "A_OPUS" => {
            rate = Some(48_000);
            "opus".to_owned()
        }
        "A_VORBIS" => "vorbis".to_owned(),
        c if c.starts_with("A_AAC") => {
            if let Some(aac) = aac_config(&t.private) {
                rate = aac.rate.or(rate);
                if aac.channels > 0 {
                    channels = Some(aac.channels);
                }
            }
            "aac".to_owned()
        }
        "A_MPEG/L3" => "mp3".to_owned(),
        "A_MPEG/L2" => "mp2".to_owned(),
        "A_FLAC" => "flac".to_owned(),
        "A_PCM/INT/LIT" if bits == 8 => "pcm_u8".to_owned(),
        "A_PCM/INT/LIT" => format!("pcm_s{bits}le"),
        "A_PCM/INT/BIG" => format!("pcm_s{bits}be"),
        "A_PCM/FLOAT/IEEE" => format!("pcm_f{bits}le"),
        "A_AC3" => "ac3".to_owned(),
        "A_EAC3" => "eac3".to_owned(),
        "A_DTS" => "dts".to_owned(),
        "A_TRUEHD" => "truehd".to_owned(),
        "A_ALAC" => "alac".to_owned(),
        other => other.trim_start_matches("A_").to_lowercase(),
    };
    AudioStream {
        codec,
        sample_rate: rate.filter(|&r| r > 0),
        channels: channels.filter(|&c| c > 0),
    }
}

fn median_step(times: &[i64]) -> Option<i64> {
    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    let mut steps: Vec<i64> = sorted
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|&d| d > 0)
        .collect();
    steps.sort_unstable();
    steps.get(steps.len() / 2).copied()
}

fn segment_of(source: &mut Source) -> Result<Element, ProbeError> {
    let Some(header) = element(source, 0)? else {
        return Err(ProbeError::Unknown);
    };
    let mut at = header.data + header.size.ok_or(ProbeError::Damaged)?;
    loop {
        let Some(el) = element(source, at)? else {
            return Err(ProbeError::Unknown);
        };
        if el.id == SEGMENT {
            return Ok(el);
        }
        at = el.data + el.size.ok_or(ProbeError::Damaged)?;
    }
}

const fn no_blocks(every: bool) -> Blocks {
    Blocks {
        video: None,
        first_frame: None,
        times: Vec::new(),
        first: None,
        last: None,
        every,
        starts: Vec::new(),
    }
}

pub fn frames(source: &mut Source) -> Result<Option<VideoFrames>, ProbeError> {
    let segment = segment_of(source)?;
    let end = segment.size.map_or(source.len(), |s| segment.data + s);
    let mut scale = 1_000_000u64;
    let mut tracks: Vec<TrackEntry> = Vec::new();
    let mut found = no_blocks(true);
    let mut at = segment.data;
    while at < end {
        let Some(el) = element(source, at)? else {
            break;
        };
        if el.id == CLUSTER {
            found.video = tracks.iter().find(|t| t.kind == 1).map(|t| t.number);
            at = cluster(source, &el, &mut found)?;
            continue;
        }
        let Some(size) = el.size else {
            break;
        };
        if (el.id == INFO || el.id == TRACKS) && size <= ELEMENT_LIMIT {
            let body = source.bytes(el.data, usize::try_from(size).unwrap_or(0))?;
            for (id, data) in items(&body) {
                match id {
                    0x002A_D7B1 => scale = uint(data),
                    TRACK_ENTRY => tracks.push(track(data)),
                    _ => {}
                }
            }
        }
        at = el.data + size;
    }
    let Some(video) = tracks.iter().find(|t| t.kind == 1) else {
        return Ok(None);
    };
    let per_tick = i128::from(scale);
    let to_us = |ticks: i64| ticks_to_us(i128::from(ticks) * per_tick, 1_000_000_000);
    let delay_of = |number: u64| {
        tracks
            .iter()
            .find(|t| t.number == number)
            .map_or(0, |t| i128::from(t.codec_delay))
    };
    let start_ticks = found
        .starts
        .iter()
        .filter_map(|&(number, first)| {
            let delay = (delay_of(number) + per_tick / 2) / per_tick.max(1);
            i64::try_from(i128::from(first) - delay).ok()
        })
        .min();
    let Some(start) = start_ticks.and_then(to_us) else {
        return Ok(None);
    };
    let times: Option<Vec<i64>> = found.times.iter().map(|&t| to_us(t)).collect();
    Ok(times.map(|times| VideoFrames {
        times_us: from_start(times, start),
        width: video.width,
        height: video.height,
    }))
}

pub fn probe(source: &mut Source) -> Result<Found, ProbeError> {
    let segment = segment_of(source)?;
    let end = segment.size.map_or(source.len(), |s| segment.data + s);
    let mut scale = 1_000_000u64;
    let mut duration = None;
    let mut tracks: Vec<TrackEntry> = Vec::new();
    let mut blocks = no_blocks(false);
    let mut at = segment.data;
    while at < end {
        let Some(el) = element(source, at)? else {
            break;
        };
        if el.id == CLUSTER {
            blocks.video = tracks.iter().find(|t| t.kind == 1).map(|t| t.number);
            let video = tracks.iter().find(|t| t.kind == 1);
            let need_frame = video.is_some_and(|t| {
                t.codec == "V_VP9" || ffv1_record(t).is_some_and(<[u8]>::is_empty)
            }) && blocks.first_frame.is_none();
            let need_rate = video.is_some_and(|t| t.default_duration.is_none())
                && blocks.times.len() < RATE_SAMPLES;
            if !need_frame && !need_rate && duration.is_some() {
                break;
            }
            at = cluster(source, &el, &mut blocks)?;
            continue;
        }
        let size = el.size.ok_or(ProbeError::Damaged)?;
        if (el.id == INFO || el.id == TRACKS) && size <= ELEMENT_LIMIT {
            let body = source.bytes(el.data, usize::try_from(size).unwrap_or(0))?;
            for (id, data) in items(&body) {
                match id {
                    0x002A_D7B1 => scale = uint(data),
                    0x4489 => duration = float(data),
                    TRACK_ENTRY => tracks.push(track(data)),
                    _ => {}
                }
            }
        }
        at = el.data + size;
    }
    let to_seconds = |ticks: f64| ticks * numbers::float_wide(scale) / 1e9;
    let step = median_step(&blocks.times);
    let video = tracks.iter().find(|t| t.kind == 1).map(|t| {
        let fps = t
            .default_duration
            .map(|ns| 1e9 / numbers::float_wide(ns))
            .or_else(|| step.map(|s| 1.0 / to_seconds(numbers::float_wide(s.unsigned_abs()))));
        VideoStream {
            fps: fps.filter(|f| f.is_finite() && *f > 0.0),
            ..video_stream(t, blocks.first_frame.as_deref())
        }
    });
    let audio = tracks.iter().find(|t| t.kind == 2).map(audio_stream);
    let length = duration
        .and_then(|ticks| Length::seconds(to_seconds(ticks)))
        .or_else(|| {
            let ticks = (blocks.last? - blocks.first? + step.unwrap_or(0)).unsigned_abs();
            Length::ratio(ticks.checked_mul(scale)?, 1_000_000_000)
        });
    Ok(Found {
        format: FORMAT,
        length,
        video,
        audio,
    })
}
