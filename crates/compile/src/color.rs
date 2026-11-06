use serde_json::Value;

use crate::vars::strings_in;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsva {
    pub h: f64,
    pub s: f64,
    pub v: f64,
    pub a: f64,
}

fn parsed(value: &str) -> Option<csscolorparser::Color> {
    let text = value.trim();
    if text.is_empty() || text.starts_with("var(") {
        return None;
    }
    csscolorparser::parse(text).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Oklch,
}

impl ColorFormat {
    pub const ALL: [Self; 4] = [Self::Hex, Self::Rgb, Self::Hsl, Self::Oklch];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hex => "Hex",
            Self::Rgb => "RGB",
            Self::Hsl => "HSL",
            Self::Oklch => "OKLCH",
        }
    }

    pub fn of(value: &str) -> Self {
        let text = value.trim().to_lowercase();
        if text.starts_with("rgb") {
            Self::Rgb
        } else if text.starts_with("hsl") {
            Self::Hsl
        } else if text.starts_with("oklch") {
            Self::Oklch
        } else {
            Self::Hex
        }
    }
}

fn number(value: f64, places: i32) -> String {
    let scale = 10f64.powi(places);
    let rounded = (value * scale).round() / scale;
    let text = format!(
        "{rounded:.prec$}",
        prec = usize::try_from(places).unwrap_or(0)
    );
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

pub fn format_color(color: Hsva, format: ColorFormat) -> String {
    let parsed = csscolorparser::Color::from_hsva(
        opsis_types::numbers::narrow(color.h),
        opsis_types::numbers::narrow(color.s),
        opsis_types::numbers::narrow(color.v),
        opsis_types::numbers::narrow(color.a),
    );
    let opaque = color.a >= 1.0;
    let alpha = number(color.a, 2);
    match format {
        ColorFormat::Hex => {
            let [r, g, b, a] = parsed.to_rgba8();
            if opaque {
                format!("#{r:02x}{g:02x}{b:02x}")
            } else {
                format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
            }
        }
        ColorFormat::Rgb => {
            let [r, g, b, _] = parsed.to_rgba8();
            if opaque {
                format!("rgb({r}, {g}, {b})")
            } else {
                format!("rgba({r}, {g}, {b}, {alpha})")
            }
        }
        ColorFormat::Hsl => {
            let [h, s, l, _] = parsed.to_hsla();
            let h = if h.is_finite() { f64::from(h) } else { 0.0 };
            let (s, l) = (f64::from(s) * 100.0, f64::from(l) * 100.0);
            if opaque {
                format!(
                    "hsl({}, {}%, {}%)",
                    number(h, 0),
                    number(s, 0),
                    number(l, 0)
                )
            } else {
                format!(
                    "hsla({}, {}%, {}%, {alpha})",
                    number(h, 0),
                    number(s, 0),
                    number(l, 0)
                )
            }
        }
        ColorFormat::Oklch => {
            let [l, c, h, _] = parsed.to_oklcha();
            let h = if h.is_finite() {
                f64::from(h).to_degrees().rem_euclid(360.0)
            } else {
                0.0
            };
            let body = format!(
                "{} {} {}",
                number(f64::from(l), 3),
                number(f64::from(c), 3),
                number(h, 1)
            );
            if opaque {
                format!("oklch({body})")
            } else {
                format!("oklch({body} / {alpha})")
            }
        }
    }
}

pub fn is_color(value: &str) -> bool {
    parsed(value).is_some()
}

pub fn parse_color(value: &str) -> Option<Hsva> {
    let color = parsed(value)?;
    let [h, s, v, a] = color.to_hsva();
    Some(Hsva {
        h: if h.is_finite() { f64::from(h) } else { 0.0 },
        s: f64::from(s),
        v: f64::from(v),
        a: f64::from(a),
    })
}

pub fn css_hex(value: &str) -> Option<String> {
    parsed(value).map(|c| c.to_css_hex().to_string())
}

pub fn hue_and_saturation(value: &str) -> Option<(f64, f64)> {
    let [h, s, _, _] = parsed(value)?.to_hsla();
    let hue = if h.is_finite() { f64::from(h) } else { 0.0 };
    Some((hue, f64::from(s).clamp(0.0, 1.0)))
}

pub fn color_key(color: &str) -> String {
    parsed(color).map_or_else(|| color.trim().to_owned(), |c| c.to_css_hex().to_string())
}

pub fn same_color(a: &str, b: &str) -> bool {
    color_key(a) == color_key(b)
}

pub fn looks_like_color(value: &str) -> bool {
    let text = value.trim();
    let lower = text.to_ascii_lowercase();
    let shaped = (text.starts_with('#')
        && (4..=9).contains(&text.len())
        && text[1..].chars().all(|c| c.is_ascii_hexdigit()))
        || [
            "rgb(", "rgba(", "hsl(", "hsla(", "oklch(", "oklab(", "hwb(", "lab(", "lch(",
        ]
        .iter()
        .any(|p| lower.starts_with(p) && lower.ends_with(')'));
    shaped && is_color(text)
}

pub fn color_or(color: &str, fallback: &str) -> String {
    let t = color.trim();
    if t.is_empty() {
        fallback.to_owned()
    } else {
        t.to_owned()
    }
}

pub fn project_swatches(params: &[&Value], max: usize) -> Vec<String> {
    let mut counts: indexmap::IndexMap<String, usize> = indexmap::IndexMap::new();
    for value in params {
        let mut found = Vec::new();
        strings_in(value, &looks_like_color, &mut found);
        for color in found {
            *counts.entry(color.trim().to_owned()).or_default() += 1;
        }
    }
    let mut by_use: Vec<(String, usize)> = counts.into_iter().collect();
    by_use.sort_by_key(|e| std::cmp::Reverse(e.1));
    let mut seen = std::collections::HashSet::new();
    by_use
        .into_iter()
        .map(|(c, _)| c)
        .filter(|c| seen.insert(color_key(c)))
        .take(max)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub fn rgb_of(value: &str) -> Option<Rgb> {
    let c = parsed(value)?;
    let unit = |v: f32| f64::from(v).clamp(0.0, 1.0);
    Some(Rgb {
        r: unit(c.r),
        g: unit(c.g),
        b: unit(c.b),
    })
}
