use opsis_scene::node::{Effect, EffectUniform, Element, Kind, Node};
use opsis_scene::style::Style;
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;

use crate::elements::oklch_mix::parsed_or_black;

const FILTER_SCALE: f64 = 1.0;
const QUAD: u32 = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub name: &'static str,
    pub values: Vec<f64>,
    pub columns: u32,
}

pub fn value(name: &'static str, values: Vec<f64>) -> Setting {
    let columns = u32::try_from(values.len()).unwrap_or(1);
    Setting {
        name,
        values,
        columns,
    }
}

pub const fn quads(name: &'static str, values: Vec<f64>) -> Setting {
    Setting {
        name,
        values,
        columns: QUAD,
    }
}

pub fn rgb_uniform(css: &str) -> Vec<f64> {
    let rgb = parsed_or_black(css);
    vec![rgb.r, rgb.g, rgb.b]
}

fn effect(program: &str, settings: Vec<Setting>, scale: f64, style: Style) -> Element {
    Element::new(
        Kind::Effect(Effect {
            program: program.to_owned(),
            uniforms: settings
                .into_iter()
                .map(|s| EffectUniform {
                    name: s.name.to_owned(),
                    values: s.values.into_iter().map(Num).collect(),
                    columns: s.columns,
                })
                .collect(),
            scale: Num(scale),
        }),
        style,
    )
}

pub fn filtered(program: &str, settings: Vec<(&'static str, Vec<f64>)>, child: Node) -> Node {
    effect(
        program,
        settings.into_iter().map(|(n, v)| value(n, v)).collect(),
        FILTER_SCALE,
        absolute_fill(),
    )
    .child(child)
    .into()
}

pub fn generated(program: &str, settings: Vec<Setting>, scale: f64) -> Node {
    effect(program, settings, scale, absolute_fill()).into()
}

pub const PAPER_SCALE: f64 = 2.0;
const PAPER_FALLBACK: [f64; 4] = [0.5, 0.5, 0.5, 1.0];
const COVER: f64 = 2.0;

fn hex_rgba(hex: &str) -> Option<[f64; 4]> {
    let digits: String = match hex.len() {
        3 | 4 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 => format!("{hex}ff"),
        _ => hex.to_owned(),
    };
    if digits.len() != 8 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| {
        u8::from_str_radix(digits.get(i..i + 2).unwrap_or("00"), 16)
            .map(|v| f64::from(v) / 255.0)
            .ok()
    };
    Some([channel(0)?, channel(2)?, channel(4)?, channel(6)?])
}

fn rgb_function(text: &str) -> Option<[f64; 4]> {
    let inner = text
        .strip_prefix("rgba")
        .or_else(|| text.strip_prefix("rgb"))?
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let byte = |s: &str| s.parse::<u32>().ok().map(|v| f64::from(v) / 255.0);
    let alpha = parts.get(3).map_or(Some(1.0), |s| s.parse::<f64>().ok())?;
    Some([byte(parts[0])?, byte(parts[1])?, byte(parts[2])?, alpha])
}

pub fn paper_color(css: &str) -> [f64; 4] {
    let text = css.trim();
    let parsed = text
        .strip_prefix('#')
        .map_or_else(|| rgb_function(text), hex_rgba);
    parsed.map_or(PAPER_FALLBACK, |c| c.map(|v| v.clamp(0.0, 1.0)))
}

pub fn paper_colors(colors: &[String], most: usize) -> Vec<Setting> {
    let shown: Vec<&String> = colors.iter().take(most).collect();
    let count = opsis_types::numbers::float(shown.len());
    let mut settings = vec![value("colorsCount", vec![count])];
    if !shown.is_empty() {
        settings.push(quads(
            "colors",
            shown.iter().flat_map(|c| paper_color(c)).collect(),
        ));
    }
    settings
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperSizing {
    pub scale: f64,
    pub rotation: f64,
    pub offset: (f64, f64),
}

pub fn paper_sizing(sizing: PaperSizing) -> Vec<Setting> {
    vec![
        value("fit", vec![COVER]),
        value("sizingScale", vec![sizing.scale]),
        value("rotation", vec![sizing.rotation]),
        value("originX", vec![0.5]),
        value("originY", vec![0.5]),
        value("offsetX", vec![sizing.offset.0]),
        value("offsetY", vec![sizing.offset.1]),
        value("worldWidth", vec![0.0]),
        value("worldHeight", vec![0.0]),
    ]
}
