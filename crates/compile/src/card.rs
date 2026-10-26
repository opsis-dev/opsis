use std::sync::LazyLock;

use opsis_types::cards::{CardRow, Icon, Tone};
use regex::Regex;

use crate::asset_ref::base_name;
use crate::node::{RowExt, missing, row};
use crate::text::{format_duration, percent_text, truncate};
use crate::time::frames_to_ms;

pub const CARD_TEXT_MAX: usize = 42;
pub const QUOTED_TEXT_MAX: usize = CARD_TEXT_MAX - 2;
pub const CODE_TEXT_MAX: usize = 36;
pub const LABELLED_TEXT_MAX: usize = 24;
pub const TAG_TEXT_MAX: usize = 20;
const NAME_WITH_ROLE_MAX: usize = 34;
const TRANSITION_WARN_RATIO: f64 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cover {
    pub ratio: f64,
    pub shortest: u32,
}

pub fn cover(transition_frames: u32, clip_frames: &[Option<u32>]) -> Option<Cover> {
    let shortest = clip_frames
        .iter()
        .flatten()
        .copied()
        .filter(|f| *f > 0)
        .min()?;
    Some(Cover {
        ratio: f64::from(transition_frames) / f64::from(shortest),
        shortest,
    })
}

pub fn covers_too_much(ratio: f64) -> bool {
    ratio > TRANSITION_WARN_RATIO
}

pub fn squeezed_too_much(length: f64, designed: f64) -> bool {
    length < TRANSITION_WARN_RATIO * designed
}

pub fn frame_secs(frames: impl Into<f64>, fps: u32) -> String {
    format_duration(frames_to_ms(frames, fps))
}

pub fn cover_row(covered: Option<Cover>, fps: u32) -> Option<CardRow> {
    let covered = covered.filter(|c| covers_too_much(c.ratio))?;
    let share = percent_text(covered.ratio, 100.0);
    Some(
        row(
            Icon::TriangleAlert,
            format!(
                "Covers {share} of a {} clip",
                frame_secs(covered.shortest, fps)
            ),
        )
        .tone(Tone::Warn)
        .titled("Turn on Keep every clip whole, or make the transition shorter."),
    )
}

pub fn toggle_row(icon: Icon, on: bool) -> CardRow {
    row(icon, if on { "On" } else { "Off" }).muted_if(!on)
}

pub fn file_row(icon: Icon, path: &str) -> CardRow {
    named_file_row(icon, path, &base_name(path))
}

pub fn named_file_row(icon: Icon, path: &str, name: &str) -> CardRow {
    row(icon, name).titled(path)
}

pub fn range_row(start_ms: f64, end_ms: Option<f64>) -> CardRow {
    row(
        Icon::Scissors,
        match end_ms {
            None => format!("From {} to the end", format_duration(start_ms)),
            Some(end) => format!("{} → {}", format_duration(start_ms), format_duration(end)),
        },
    )
}

pub fn transition_length_text(frames: u32, fps: u32, held: bool) -> String {
    format!(
        "{}{}",
        frame_secs(frames, fps),
        if held { " · held" } else { "" }
    )
}

pub fn join_parts<S: AsRef<str>>(parts: impl IntoIterator<Item = Option<S>>) -> String {
    parts
        .into_iter()
        .flatten()
        .map(|p| p.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" · ")
}

pub fn quote(text: &str, max: usize) -> String {
    format!("“{}”", truncate(text, max))
}

pub fn quoted(text: &str) -> String {
    quote(text, QUOTED_TEXT_MAX)
}

pub fn quoted_row(icon: Icon, text: &str, empty: &str) -> CardRow {
    quoted_row_with(icon, text, empty, "")
}

pub fn quoted_row_with(icon: Icon, text: &str, empty: &str, suffix: &str) -> CardRow {
    if text.trim().is_empty() {
        missing(icon, empty)
    } else {
        row(
            icon,
            format!(
                "{}{suffix}",
                quote(text, QUOTED_TEXT_MAX - suffix.chars().count())
            ),
        )
    }
}

fn role_word(role: &str) -> Option<&'static str> {
    Some(match role {
        "button" => "button",
        "link" => "link",
        "textbox" => "field",
        "searchbox" => "search field",
        "checkbox" => "checkbox",
        "radio" => "option",
        "combobox" => "dropdown",
        "listbox" => "list",
        "option" => "option",
        "tab" => "tab",
        "menuitem" => "menu item",
        "heading" => "heading",
        "img" => "image",
        "switch" => "switch",
        "slider" => "slider",
        _ => return None,
    })
}

static POINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)$").expect("pattern"));

pub fn element_name(target: &str, role: Option<&str>) -> Option<String> {
    let t = target.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("css=") {
        return Some("Custom element".to_owned());
    }
    if let Some(p) = POINT.captures(t) {
        let n = |i: usize| p[i].parse::<f64>().unwrap_or(0.0).round();
        return Some(format!("Point {}, {}", n(1), n(2)));
    }
    Some(match role.and_then(role_word) {
        Some(word) => format!("{} {word}", quote(t, NAME_WITH_ROLE_MAX)),
        None => quoted(t),
    })
}

pub fn element_row(icon: Icon, target: &str, role: Option<&str>) -> CardRow {
    element_row_or(icon, target, role, "Pick an element")
}

pub fn element_row_or(icon: Icon, target: &str, role: Option<&str>, empty: &str) -> CardRow {
    match element_name(target, role) {
        Some(name) => row(icon, name).titled(target),
        None => missing(icon, empty),
    }
}

static KEEPS_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Z].*[A-Z]|\d").expect("pattern"));

pub fn sentence(title: &str) -> String {
    title
        .split(' ')
        .enumerate()
        .map(|(i, w)| {
            if i == 0 || KEEPS_CASE.is_match(w) {
                w.to_owned()
            } else {
                w.to_lowercase()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn truncate_card(text: &str) -> String {
    truncate(text, CARD_TEXT_MAX)
}
