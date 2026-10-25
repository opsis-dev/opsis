use std::sync::LazyLock;

use regex::Regex;

use crate::color::is_color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientKind {
    Linear,
    Radial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    pub color: String,
    pub at: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub angle: f64,
    pub stops: Vec<GradientStop>,
}

pub const MIN_STOPS: usize = 2;
pub const MAX_STOPS: usize = 6;

pub fn split_top_level(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if ch == ',' && depth == 0 {
            parts.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_owned());
    }
    parts
}

static STOP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)\s+(-?[\d.]+)%$").expect("pattern"));
static DEGREES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(-?[\d.]+)deg$").expect("pattern"));
static GRADIENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^(linear|radial)-gradient\((.*)\)$").expect("pattern"));

fn parse_stop(part: &str) -> Option<(String, Option<f64>)> {
    let caps = STOP.captures(part);
    let color = caps
        .as_ref()
        .map_or(part, |c| c.get(1).map_or("", |m| m.as_str()))
        .trim();
    if !is_color(color) {
        return None;
    }
    let at = caps
        .and_then(|c| c[2].parse::<f64>().ok())
        .map(|n| n.clamp(0.0, 100.0));
    Some((color.to_owned(), at))
}

fn side_angle(side: &str) -> Option<f64> {
    let normal = side
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    Some(match normal.as_str() {
        "to top" => 0.0,
        "to top right" | "to right top" => 45.0,
        "to right" => 90.0,
        "to bottom right" | "to right bottom" => 135.0,
        "to bottom" => 180.0,
        "to bottom left" | "to left bottom" => 225.0,
        "to left" => 270.0,
        "to top left" | "to left top" => 315.0,
        _ => return None,
    })
}

fn angle_of(part: &str) -> Option<f64> {
    match DEGREES.captures(part) {
        Some(c) => c[1]
            .parse::<f64>()
            .ok()
            .map(|d| (d % 360.0 + 360.0) % 360.0),
        None => side_angle(part),
    }
}

pub fn parse_gradient(text: &str) -> Option<Gradient> {
    let caps = GRADIENT.captures(text.trim())?;
    let kind = if &caps[1] == "radial" {
        GradientKind::Radial
    } else {
        GradientKind::Linear
    };
    let mut parts = split_top_level(&caps[2]);
    let mut angle = 180.0;
    let first = parts.first().cloned().unwrap_or_default();
    if !parts.is_empty() && parse_stop(&first).is_none() {
        parts.remove(0);
        if kind == GradientKind::Linear {
            angle = angle_of(&first).unwrap_or(angle);
        }
    }
    let stops: Option<Vec<(String, Option<f64>)>> = parts.iter().map(|p| parse_stop(p)).collect();
    let stops = stops.filter(|s| s.len() >= MIN_STOPS)?;
    let last = (stops.len() - 1).max(1) as f64;
    let stops = stops
        .into_iter()
        .enumerate()
        .map(|(i, (color, at))| GradientStop {
            color,
            at: at.unwrap_or_else(|| (i as f64 / last * 100.0).round()),
        })
        .collect();
    Some(Gradient { kind, angle, stops })
}
