use std::borrow::Cow;

use serde_json::{Map, Value, json};
use skrifa::instance::{Location, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::raw::types::Tag;
use skrifa::string::StringId;
use skrifa::{FontRef, GlyphId, MetadataProvider};

use crate::PrepareError;

const LATIN_RANGES: [(u32, u32); 6] = [
    (0x20, 0x7e),
    (0xa0, 0x24f),
    (0x2010, 0x2027),
    (0x2030, 0x203a),
    (0x20ac, 0x20ac),
    (0x2122, 0x2122),
];
const WEIGHT_AXIS: Tag = Tag::new(b"wght");
const DECIMALS: f64 = 100.0;
const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";
const WOFF_SIGNATURE: &[u8; 4] = b"wOFF";

fn rounded(value: f64) -> f64 {
    let n = (value * DECIMALS).round() / DECIMALS;
    if n == 0.0 { 0.0 } else { n }
}

fn js_number(value: f64) -> Value {
    let n = rounded(value);
    if n.fract() == 0.0 && n.abs() < 1e15 {
        Value::from(n as i64)
    } else {
        Value::from(n)
    }
}

fn number(value: f64) -> String {
    let n = rounded(value);
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Default)]
struct Bounds {
    min_x: f64,
    max_x: f64,
    any: bool,
}

impl Bounds {
    fn add(&mut self, x: f64) {
        if self.any {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
        } else {
            (self.min_x, self.max_x, self.any) = (x, x, true);
        }
    }

    fn quad(&mut self, p0: f64, p1: f64, p2: f64) {
        let denominator = p0 - 2.0 * p1 + p2;
        if denominator != 0.0 {
            let t = (p0 - p1) / denominator;
            if t > 0.0 && t < 1.0 {
                let u = 1.0 - t;
                self.add(u * u * p0 + 2.0 * u * t * p1 + t * t * p2);
            }
        }
        self.add(p2);
    }

    fn cubic(&mut self, p0: f64, p1: f64, p2: f64, p3: f64) {
        let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
        let b = 2.0 * (p0 - 2.0 * p1 + p2);
        let c = p1 - p0;
        let at = |t: f64| {
            let u = 1.0 - t;
            u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
        };
        let mut roots = Vec::new();
        if a.abs() < 1e-12 {
            if b.abs() > 1e-12 {
                roots.push(-c / b);
            }
        } else {
            let disc = b * b - 4.0 * a * c;
            if disc >= 0.0 {
                let s = disc.sqrt();
                roots.extend([(-b + s) / (2.0 * a), (-b - s) / (2.0 * a)]);
            }
        }
        for t in roots.into_iter().filter(|t| *t > 0.0 && *t < 1.0) {
            self.add(at(t));
        }
        self.add(p3);
    }
}

#[derive(Default)]
struct Pen {
    parts: Vec<String>,
    start: Option<Point>,
    last: Option<Point>,
    pending_line: Option<Point>,
    bounds: Bounds,
}

impl Pen {
    fn flush_line(&mut self) {
        if let Some(p) = self.pending_line.take() {
            self.parts
                .push(format!("l {} {}", number(p.x), number(p.y)));
        }
    }
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush_line();
        let p = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        self.parts
            .push(format!("m {} {}", number(p.x), number(p.y)));
        self.bounds.add(p.x);
        (self.start, self.last) = (Some(p), Some(p));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.flush_line();
        let p = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        self.bounds.add(p.x);
        self.pending_line = Some(p);
        self.last = Some(p);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.flush_line();
        let from = self.last.unwrap_or(Point { x: 0.0, y: 0.0 });
        let (cx, cy, x, y) = (f64::from(cx0), f64::from(cy0), f64::from(x), f64::from(y));
        self.bounds.quad(from.x, cx, x);
        self.parts.push(format!(
            "q {} {} {} {}",
            number(x),
            number(y),
            number(cx),
            number(cy)
        ));
        self.last = Some(Point { x, y });
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.flush_line();
        let from = self.last.unwrap_or(Point { x: 0.0, y: 0.0 });
        let [c0x, c0y, c1x, c1y, x, y] = [cx0, cy0, cx1, cy1, x, y].map(f64::from);
        self.bounds.cubic(from.x, c0x, c1x, x);
        self.parts.push(format!(
            "b {} {} {} {} {} {}",
            number(x),
            number(y),
            number(c0x),
            number(c0y),
            number(c1x),
            number(c1y)
        ));
        self.last = Some(Point { x, y });
    }

    fn close(&mut self) {
        if self.pending_line.is_some() && self.pending_line != self.start {
            self.flush_line();
        }
        self.pending_line = None;
        self.last = self.start;
    }
}

fn sfnt(bytes: &[u8]) -> Result<Cow<'_, [u8]>, PrepareError> {
    let signature = bytes.get(..4);
    if signature == Some(WOFF2_SIGNATURE) {
        return wuff::decompress_woff2(bytes)
            .map(Cow::Owned)
            .map_err(|_| PrepareError::Unreadable);
    }
    if signature == Some(WOFF_SIGNATURE) {
        return wuff::decompress_woff1(bytes)
            .map(Cow::Owned)
            .map_err(|_| PrepareError::Unreadable);
    }
    Ok(Cow::Borrowed(bytes))
}

fn first_font(data: &[u8]) -> Result<FontRef<'_>, PrepareError> {
    FontRef::new(data)
        .or_else(|_| FontRef::from_index(data, 0))
        .map_err(|_| PrepareError::Unreadable)
}

fn family_name(font: &FontRef<'_>) -> String {
    font.localized_strings(StringId::FAMILY_NAME)
        .english_or_first()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn location_at(font: &FontRef<'_>, weight: f64) -> Location {
    let has_weight = font.axes().iter().any(|axis| axis.tag() == WEIGHT_AXIS);
    if has_weight {
        font.axes().location([("wght", weight as f32)])
    } else {
        font.axes().location(std::iter::empty::<(&str, f32)>())
    }
}

fn exact_advance(font: &FontRef<'_>, gid: GlyphId, location: &Location) -> Option<f64> {
    let base = f64::from(font.hmtx().ok()?.advance(gid)?);
    let coords = location.coords();
    if coords.iter().all(|c| c.to_bits() == 0) {
        return Some(base);
    }
    if let Ok(hvar) = font.hvar() {
        return Some(base + hvar.advance_delta(gid, coords)?.to_f64());
    }
    let (glyf, loca) = (font.glyf().ok()?, font.loca(None).ok()?);
    let deltas = font
        .gvar()
        .ok()?
        .phantom_point_deltas(&glyf, &loca, coords, gid)?;
    Some(base + (deltas[1].x - deltas[0].x).to_f64())
}

pub fn glyphs(font: &FontRef<'_>, weight: f64) -> Map<String, Value> {
    let location = location_at(font, weight);
    let metrics = font.glyph_metrics(Size::unscaled(), &location);
    let outlines = font.outline_glyphs();
    let charmap = font.charmap();
    let mut glyphs = Map::new();
    for (from, to) in LATIN_RANGES {
        for code in from..=to {
            let Some(gid) = charmap.map(code).filter(|g| *g != GlyphId::NOTDEF) else {
                continue;
            };
            let Some(ch) = char::from_u32(code) else {
                continue;
            };
            let mut pen = Pen::default();
            let advance = exact_advance(font, gid, &location)
                .or_else(|| metrics.advance_width(gid).map(f64::from));
            if let Some(outline) = outlines.get(gid) {
                let settings = DrawSettings::unhinted(Size::unscaled(), &location);
                match outline.draw(settings, &mut pen) {
                    Ok(_) => {}
                    Err(_) => pen = Pen::default(),
                }
            }
            pen.flush_line();
            let bound = |x: f64| {
                if pen.bounds.any {
                    js_number(x)
                } else {
                    Value::Null
                }
            };
            glyphs.insert(
                ch.to_string(),
                json!({
                    "ha": js_number(advance.unwrap_or_default()),
                    "x_min": bound(pen.bounds.min_x),
                    "x_max": bound(pen.bounds.max_x),
                    "o": pen.parts.join(" "),
                }),
            );
        }
    }
    glyphs
}

pub fn typeface(bytes: &[u8], weight: f64) -> Result<Value, PrepareError> {
    let data = sfnt(bytes)?;
    let font = first_font(&data)?;
    let head = font.head().map_err(|_| PrepareError::Unreadable)?;
    let hhea = font.hhea().map_err(|_| PrepareError::Unreadable)?;
    let (underline_position, underline_thickness) = font.post().map_or((0, 0), |post| {
        (
            post.underline_position().to_i16(),
            post.underline_thickness().to_i16(),
        )
    });
    Ok(json!({
        "glyphs": glyphs(&font, weight),
        "familyName": family_name(&font),
        "ascender": hhea.ascender().to_i16(),
        "descender": hhea.descender().to_i16(),
        "underlinePosition": underline_position,
        "underlineThickness": underline_thickness,
        "boundingBox": {
            "xMin": head.x_min(),
            "yMin": head.y_min(),
            "xMax": head.x_max(),
            "yMax": head.y_max(),
        },
        "resolution": head.units_per_em(),
        "original_font_information": {},
    }))
}
