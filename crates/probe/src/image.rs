use crate::read::{Bytes, Source};
use crate::{Found, Length, ProbeError, STILL_FPS, VideoStream};

const JPEG_FRAMES_PER_SECOND: u64 = 25;
const GIF_DEFAULT_DELAY: u32 = 10;
const GIF_MIN_DELAY: u32 = 2;
const SVG_HEAD: usize = 4096;

fn still(format: &'static str, codec: &str, width: u32, height: u32, pix_fmt: &str) -> Found {
    Found {
        format,
        length: None,
        video: Some(VideoStream {
            codec: codec.to_owned(),
            width,
            height,
            fps: Some(STILL_FPS),
            pix_fmt: pix_fmt.to_owned(),
        }),
        audio: None,
    }
}

pub fn png(head: &[u8]) -> Result<Found, ProbeError> {
    let mut r = Bytes::new(head.get(16..).ok_or(ProbeError::Damaged)?);
    let width = r.be32()?;
    let height = r.be32()?;
    let depth = r.u8()?;
    let color = r.u8()?;
    let pix = match (color, depth) {
        (0, 1) => "monob",
        (0, 16) => "gray16be",
        (0, _) => "gray",
        (2, 16) => "rgb48be",
        (2, _) => "rgb24",
        (3, _) => "pal8",
        (4, 16) => "ya16be",
        (4, _) => "ya8",
        (6, 16) => "rgba64be",
        _ => "rgba",
    };
    Ok(still("png_pipe", "png", width, height, pix))
}

fn jpeg_pix(components: &[(u8, u8)]) -> &'static str {
    match components {
        [_] => "gray",
        [(yh, yv), (ch, cv), ..] if components.len() == 3 => {
            match (yh / (*ch).max(1), yv / (*cv).max(1)) {
                (2, 2) => "yuvj420p",
                (2, 1) => "yuvj422p",
                (4, 1) => "yuvj411p",
                (1, 2) => "yuvj440p",
                _ => "yuvj444p",
            }
        }
        _ => "cmyk",
    }
}

pub fn jpeg(source: &mut Source) -> Result<Found, ProbeError> {
    let mut at = 2u64;
    while at + 4 <= source.len() {
        let head = source.bytes(at, 4)?;
        if head[0] != 0xFF {
            return Err(ProbeError::Damaged);
        }
        let marker = head[1];
        if marker == 0xFF {
            at += 1;
            continue;
        }
        let length = u64::from(u16::from_be_bytes([head[2], head[3]]));
        let is_frame = (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker);
        if is_frame {
            let body = source.bytes(at + 4, usize::try_from(length).unwrap_or(0))?;
            let mut r = Bytes::new(&body);
            r.u8()?;
            let height = u32::from(r.be16()?);
            let width = u32::from(r.be16()?);
            let count = usize::from(r.u8()?);
            let mut components = Vec::with_capacity(count);
            for _ in 0..count {
                r.u8()?;
                let sampling = r.u8()?;
                r.u8()?;
                components.push((sampling >> 4, sampling & 0x0F));
            }
            let mut found = still("image2", "mjpeg", width, height, jpeg_pix(&components));
            found.length = Length::ratio(1, JPEG_FRAMES_PER_SECOND);
            return Ok(found);
        }
        at += 2 + length;
    }
    Err(ProbeError::Damaged)
}

fn skip_blocks(source: &mut Source, mut at: u64) -> Result<u64, ProbeError> {
    loop {
        let size = *source.bytes(at, 1)?.first().ok_or(ProbeError::Damaged)?;
        at += 1 + u64::from(size);
        if size == 0 {
            return Ok(at);
        }
    }
}

pub fn gif(source: &mut Source) -> Result<Found, ProbeError> {
    let head = source.bytes(6, 7)?;
    let mut r = Bytes::new(&head);
    let width = u32::from(r.le16()?);
    let height = u32::from(r.le16()?);
    let packed = r.u8()?;
    let mut at = 13u64;
    if packed & 0x80 != 0 {
        at += 3 << ((packed & 7) + 1);
    }
    let mut frames = 0u32;
    let mut centiseconds = 0u32;
    let mut delay = None;
    while at < source.len() {
        let kind = *source.bytes(at, 1)?.first().ok_or(ProbeError::Damaged)?;
        match kind {
            0x21 => {
                let ext = source.bytes(at + 1, 6)?;
                if ext.first() == Some(&0xF9) && ext.len() >= 6 {
                    delay = Some(u32::from(u16::from_le_bytes([ext[3], ext[4]])));
                }
                at = skip_blocks(source, at + 2)?;
            }
            0x2C => {
                let desc = source.bytes(at + 1, 9)?;
                let local = desc.get(8).copied().unwrap_or(0);
                at += 10;
                if local & 0x80 != 0 {
                    at += 3 << ((local & 7) + 1);
                }
                at = skip_blocks(source, at + 1)?;
                let d = delay.take().unwrap_or(0);
                centiseconds += if d < GIF_MIN_DELAY {
                    GIF_DEFAULT_DELAY
                } else {
                    d
                };
                frames += 1;
            }
            _ => break,
        }
    }
    Ok(Found {
        format: "gif",
        length: Length::ratio(u64::from(centiseconds), 100),
        video: Some(VideoStream {
            codec: "gif".to_owned(),
            width,
            height,
            fps: (centiseconds > 0).then(|| f64::from(frames) * 100.0 / f64::from(centiseconds)),
            pix_fmt: "bgra".to_owned(),
        }),
        audio: None,
    })
}

pub fn webp(head: &[u8]) -> Result<Found, ProbeError> {
    let mut at = 12;
    let mut canvas = None;
    while let Some(chunk) = head.get(at..at + 8) {
        let id = &chunk[..4];
        let size = usize::try_from(u32::from_le_bytes(
            chunk[4..8].try_into().map_err(|_| ProbeError::Damaged)?,
        ))
        .map_err(|_| ProbeError::Damaged)?;
        let body = head.get(at + 8..).ok_or(ProbeError::Damaged)?;
        let mut r = Bytes::new(body);
        match id {
            b"VP8X" => {
                r.skip(4)?;
                let w = r.be24().map(u32::swap_bytes)? >> 8;
                let h = r.be24().map(u32::swap_bytes)? >> 8;
                canvas = Some((w + 1, h + 1));
            }
            b"VP8 " => {
                r.skip(6)?;
                let (w, h) = (u32::from(r.le16()? & 0x3FFF), u32::from(r.le16()? & 0x3FFF));
                let (w, h) = canvas.unwrap_or((w, h));
                return Ok(still("webp_pipe", "webp", w, h, "yuv420p"));
            }
            b"VP8L" => {
                r.skip(1)?;
                let packed = r.le32()?;
                let (w, h) = ((packed & 0x3FFF) + 1, (packed >> 14 & 0x3FFF) + 1);
                let (w, h) = canvas.unwrap_or((w, h));
                return Ok(still("webp_pipe", "webp", w, h, "argb"));
            }
            _ => {}
        }
        at += 8 + size + size % 2;
    }
    let (w, h) = canvas.ok_or(ProbeError::Damaged)?;
    Ok(still("webp_pipe", "webp", w, h, "yuv420p"))
}

pub fn is_svg(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(head);
    let start = text.trim_start_matches('\u{feff}').trim_start();
    start.starts_with('<') && text.contains("<svg")
}

fn length(value: &str) -> Option<f64> {
    let number: String = value
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let unit = value.trim()[number.len()..].trim();
    let n: f64 = number.parse().ok()?;
    (unit.is_empty() || unit == "px").then_some(n)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.match_indices(name).find(|(i, _)| {
        tag[..*i].ends_with(char::is_whitespace)
            && tag[i + name.len()..].trim_start().starts_with('=')
    })?;
    let rest = tag[at.0 + name.len()..]
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let body = &rest[1..];
    Some(&body[..body.find(quote)?])
}

pub fn svg(source: &mut Source) -> Result<Found, ProbeError> {
    let head = source.bytes(0, SVG_HEAD)?;
    let text = String::from_utf8_lossy(&head);
    let tag_at = text.find("<svg").ok_or(ProbeError::Unknown)?;
    let tag = &text[tag_at..text[tag_at..].find('>').map_or(text.len(), |e| tag_at + e)];
    let view: Vec<f64> = attribute(tag, "viewBox").map_or_default(|v| {
        v.split([' ', ','])
            .filter_map(|n| n.trim().parse().ok())
            .collect()
    });
    let width = attribute(tag, "width")
        .and_then(length)
        .or_else(|| view.get(2).copied());
    let height = attribute(tag, "height")
        .and_then(length)
        .or_else(|| view.get(3).copied());
    let size = |v: Option<f64>| v.map_or(0, opsis_types::numbers::whole);
    Ok(still(
        "svg_pipe",
        "svg",
        size(width),
        size(height),
        "unknown",
    ))
}
