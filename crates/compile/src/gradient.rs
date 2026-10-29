use std::sync::LazyLock;

use opsis_types::numbers;
use opsis_types::numbers::whole;
use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::color::is_color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum GradientKind {
    Linear,
    Radial,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GradientStop {
    pub color: String,
    pub at: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
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
    let last = numbers::float((stops.len() - 1).max(1));
    let stops = stops
        .into_iter()
        .enumerate()
        .map(|(i, (color, at))| GradientStop {
            color,
            at: at.unwrap_or_else(|| (numbers::float(i) / last * 100.0).round()),
        })
        .collect();
    Some(Gradient { kind, angle, stops })
}

const STRAIGHT_ACROSS: f64 = 90.0;
const FALLBACK_STOP_COLOR: &str = "#ffffff";

fn stop_list(stops: &[GradientStop]) -> String {
    stops
        .iter()
        .map(|s| format!("{} {}%", s.color, whole(s.at)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn format_gradient(gradient: &Gradient) -> String {
    match gradient.kind {
        GradientKind::Linear => format!(
            "linear-gradient({}deg, {})",
            whole(gradient.angle),
            stop_list(&gradient.stops)
        ),
        GradientKind::Radial => format!("radial-gradient(circle, {})", stop_list(&gradient.stops)),
    }
}

fn sorted_stops(stops: &[GradientStop]) -> Vec<GradientStop> {
    let mut sorted = stops.to_vec();
    sorted.sort_by(|a, b| a.at.total_cmp(&b.at));
    sorted
}

fn percent(at: f64) -> f64 {
    at.round().clamp(0.0, 100.0)
}

#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum GradientEdit {
    Kind { kind: GradientKind },
    Angle { angle: f64 },
    StopColor { index: usize, color: String },
    StopAt { index: usize, at: f64 },
    AddStop { at: f64 },
    RemoveStop { index: usize },
    Sort { index: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GradientView {
    pub gradient: Gradient,
    pub css: String,
    pub strip: String,
    pub can_add: bool,
    pub can_remove: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<usize>,
}

fn view_of(gradient: Gradient, selected: Option<usize>) -> GradientView {
    GradientView {
        css: format_gradient(&gradient),
        strip: format_gradient(&Gradient {
            kind: GradientKind::Linear,
            angle: STRAIGHT_ACROSS,
            stops: gradient.stops.clone(),
        }),
        can_add: gradient.stops.len() < MAX_STOPS,
        can_remove: gradient.stops.len() > MIN_STOPS,
        selected,
        gradient,
    }
}

pub fn gradient_view(css: &str) -> Option<GradientView> {
    parse_gradient(css).map(|g| view_of(g, None))
}

pub fn edit_gradient(css: &str, edit: &GradientEdit) -> Option<GradientView> {
    let mut gradient = parse_gradient(css)?;
    let mut selected = None;
    match edit {
        GradientEdit::Kind { kind } => gradient.kind = *kind,
        GradientEdit::Angle { angle } => gradient.angle = angle.round().rem_euclid(360.0),
        GradientEdit::StopColor { index, color } => {
            if is_color(color)
                && let Some(stop) = gradient.stops.get_mut(*index)
            {
                stop.color.clone_from(color);
            }
        }
        GradientEdit::StopAt { index, at } => {
            if let Some(stop) = gradient.stops.get_mut(*index) {
                stop.at = percent(*at);
            }
        }
        GradientEdit::AddStop { at } => {
            if gradient.stops.len() < MAX_STOPS {
                let at = percent(*at);
                let sorted = sorted_stops(&gradient.stops);
                let color = sorted
                    .iter()
                    .find(|s| s.at >= at)
                    .or(sorted.last())
                    .map_or_else(|| FALLBACK_STOP_COLOR.to_owned(), |s| s.color.clone());
                gradient.stops.push(GradientStop { color, at });
                selected = Some(gradient.stops.len() - 1);
            }
        }
        GradientEdit::RemoveStop { index } => {
            if gradient.stops.len() > MIN_STOPS && *index < gradient.stops.len() {
                gradient.stops.remove(*index);
                selected = Some(index.saturating_sub(1));
            }
        }
        GradientEdit::Sort { index } => {
            let mut order: Vec<usize> = (0..gradient.stops.len()).collect();
            order.sort_by(|a, b| gradient.stops[*a].at.total_cmp(&gradient.stops[*b].at));
            selected = order.iter().position(|i| i == index);
            gradient.stops = sorted_stops(&gradient.stops);
        }
    }
    Some(view_of(gradient, selected))
}

#[cfg(test)]
mod edits {
    use proptest::prelude::*;

    use super::*;

    const CSS: &str = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)";

    #[test]
    fn formats_what_it_parses() {
        let view = gradient_view(CSS).unwrap();
        assert_eq!(view.css, CSS);
        assert_eq!(
            view.strip,
            "linear-gradient(90deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)"
        );
        assert!(view.can_add && view.can_remove);
        assert_eq!(
            gradient_view("radial-gradient(red, blue)").unwrap().css,
            "radial-gradient(circle, red 0%, blue 100%)"
        );
        assert!(gradient_view("conic-gradient(red, blue)").is_none());
    }

    #[test]
    fn adds_a_stop_in_the_colour_after_it_and_stops_at_six() {
        let added = edit_gradient(CSS, &GradientEdit::AddStop { at: 20.4 }).unwrap();
        assert_eq!(
            added.gradient.stops.last(),
            Some(&GradientStop {
                color: "#f472b6".into(),
                at: 20.0
            })
        );
        assert_eq!(added.selected, Some(3));
        let mut css = CSS.to_owned();
        for _ in 0..5 {
            css = edit_gradient(&css, &GradientEdit::AddStop { at: 50.0 })
                .unwrap()
                .css;
        }
        let full = gradient_view(&css).unwrap();
        assert_eq!(full.gradient.stops.len(), MAX_STOPS);
        assert!(!full.can_add);
    }

    #[test]
    fn keeps_two_stops_and_follows_a_dragged_stop_when_sorting() {
        let two = "linear-gradient(90deg, red 0%, blue 100%)";
        let kept = edit_gradient(two, &GradientEdit::RemoveStop { index: 0 }).unwrap();
        assert_eq!(kept.gradient.stops.len(), 2);
        assert!(!kept.can_remove);
        let past = edit_gradient(
            CSS,
            &GradientEdit::StopAt {
                index: 0,
                at: 150.0,
            },
        )
        .unwrap();
        assert_eq!(past.gradient.stops[0].at, 100.0);
        let moved = edit_gradient(CSS, &GradientEdit::StopAt { index: 0, at: 90.0 }).unwrap();
        let sorted = edit_gradient(&moved.css, &GradientEdit::Sort { index: 0 }).unwrap();
        assert_eq!(sorted.selected, Some(1));
        assert_eq!(sorted.gradient.stops[1].color, "#e8b86d");
    }

    #[test]
    fn turns_round_and_wraps_the_angle() {
        let radial = edit_gradient(
            CSS,
            &GradientEdit::Kind {
                kind: GradientKind::Radial,
            },
        )
        .unwrap();
        assert!(radial.css.starts_with("radial-gradient(circle, "));
        let turned = edit_gradient(CSS, &GradientEdit::Angle { angle: 370.4 }).unwrap();
        assert_eq!(turned.gradient.angle, 10.0);
    }

    fn colour() -> impl Strategy<Value = String> {
        prop_oneof![
            "#[0-9a-f]{6}",
            "#[0-9a-f]{3}",
            prop::sample::select(vec!["red", "blue", "white", "transparent"])
                .prop_map(String::from),
            (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| format!("rgb({r}, {g}, {b})")),
        ]
    }

    fn gradient() -> impl Strategy<Value = Gradient> {
        (
            prop::bool::ANY,
            0u32..360,
            prop::collection::vec((colour(), 0u32..=100), MIN_STOPS..=MAX_STOPS),
        )
            .prop_map(|(radial, angle, stops)| Gradient {
                kind: if radial {
                    GradientKind::Radial
                } else {
                    GradientKind::Linear
                },
                angle: if radial { 180.0 } else { f64::from(angle) },
                stops: stops
                    .into_iter()
                    .map(|(color, at)| GradientStop {
                        color,
                        at: f64::from(at),
                    })
                    .collect(),
            })
    }

    fn edit() -> impl Strategy<Value = GradientEdit> {
        prop_oneof![
            prop::bool::ANY.prop_map(|radial| GradientEdit::Kind {
                kind: if radial {
                    GradientKind::Radial
                } else {
                    GradientKind::Linear
                }
            }),
            (-1000.0..1000.0f64).prop_map(|angle| GradientEdit::Angle { angle }),
            (0usize..8, colour())
                .prop_map(|(index, color)| GradientEdit::StopColor { index, color }),
            (0usize..8, -50.0..150.0f64).prop_map(|(index, at)| GradientEdit::StopAt { index, at }),
            (-50.0..150.0f64).prop_map(|at| GradientEdit::AddStop { at }),
            (0usize..8).prop_map(|index| GradientEdit::RemoveStop { index }),
            (0usize..8).prop_map(|index| GradientEdit::Sort { index }),
        ]
    }

    proptest! {
        #[test]
        fn reads_back_every_gradient_it_writes(gradient in gradient()) {
            let css = format_gradient(&gradient);
            prop_assert_eq!(parse_gradient(&css), Some(gradient));
        }

        #[test]
        fn keeps_two_to_six_whole_stops_inside_the_strip_after_any_edits(
            start in gradient(),
            edits in prop::collection::vec(edit(), 0..20),
        ) {
            let mut css = format_gradient(&start);
            for edit in &edits {
                let view = edit_gradient(&css, edit).unwrap();
                let stops = &view.gradient.stops;
                prop_assert!((MIN_STOPS..=MAX_STOPS).contains(&stops.len()));
                prop_assert!(stops.iter().all(|s| (0.0..=100.0).contains(&s.at) && s.at.fract() == 0.0));
                prop_assert!((0.0..360.0).contains(&view.gradient.angle));
                prop_assert_eq!(view.can_add, stops.len() < MAX_STOPS);
                prop_assert_eq!(view.can_remove, stops.len() > MIN_STOPS);
                prop_assert!(view.selected.is_none_or(|i| i < stops.len()));
                css = view.css;
            }
        }
    }
}
