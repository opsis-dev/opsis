use opsis_macros::{choice, settings};
use opsis_scene::node::Node;
use opsis_scene::paint::{ClipPath, FillRule};
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::exact_eq;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum WipeDirection {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
    FromTopLeft,
    FromTopRight,
    FromBottomLeft,
    FromBottomRight,
}

#[settings(export, eq)]
pub struct Wipe {
    #[setting("Direction", default = WipeDirection::FromLeft)]
    pub direction: WipeDirection,
}

pub fn definition() -> ElementMeta {
    meta::<Wipe>(
        element(
            ElementKind::Presentation,
            "wipe",
            "Wipe",
            "",
            ElementCategory::Transition,
            20,
        )
        .look(NodeLook::of("lucide:square-split-horizontal")),
    )
}

fn edge_wipe(direction: WipeDirection, shown: f64) -> Option<Vec<(f64, f64)>> {
    Some(match direction {
        WipeDirection::FromLeft => vec![(0.0, 0.0), (shown, 0.0), (shown, 1.0), (0.0, 1.0)],
        WipeDirection::FromRight => vec![(1.0 - shown, 0.0), (1.0, 0.0), (1.0, 1.0), (1.0 - shown, 1.0)],
        WipeDirection::FromTop => vec![(0.0, 0.0), (1.0, 0.0), (1.0, shown), (0.0, shown)],
        WipeDirection::FromBottom => vec![(0.0, 1.0 - shown), (1.0, 1.0 - shown), (1.0, 1.0), (0.0, 1.0)],
        _ => return None,
    })
}

fn corner_wipe(direction: WipeDirection, shown: f64) -> Vec<(f64, f64)> {
    let reach = shown * 2.0;
    let (x, y) = match direction {
        WipeDirection::FromTopRight => (1.0, 0.0),
        WipeDirection::FromBottomLeft => (0.0, 1.0),
        WipeDirection::FromBottomRight => (1.0, 1.0),
        _ => (0.0, 0.0),
    };
    let across = if exact_eq(x, 0.0) { reach } else { 1.0 - reach };
    let down = if exact_eq(y, 0.0) { reach } else { 1.0 - reach };
    vec![(x, y), (across, y), (x, down)]
}

pub fn wipe_corners(direction: WipeDirection, progress: f64) -> Vec<(f64, f64)> {
    edge_wipe(direction, progress).unwrap_or_else(|| corner_wipe(direction, progress))
}

pub fn present(props: &Wipe, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    if !side.entering {
        return Ok(filled(Style::default(), side.children));
    }
    let points = wipe_corners(props.direction, side.progress)
        .into_iter()
        .map(|(x, y)| (Length::percent(x * 100.0), Length::percent(y * 100.0)))
        .collect();
    Ok(filled(
        Style::default().clip_path(ClipPath::Polygon {
            fill: FillRule::Nonzero,
            points,
        }),
        side.children,
    ))
}
