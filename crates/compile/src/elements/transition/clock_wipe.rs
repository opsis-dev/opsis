use opsis_scene::node::Node;
use opsis_scene::paint::{ClipPath, FillRule};
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::truncate;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "clock-wipe",
            "Clock wipe",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:clock-fading")),
    )
}

const STEP_DEGREES: f64 = 3.0;

pub fn clock_points(width: f64, height: f64, progress: f64) -> Vec<(f64, f64)> {
    let reach = 1.5 * width.hypot(height) / 2.0;
    let sweep = progress * 360.0;
    let steps: u32 = truncate((sweep / STEP_DEGREES).ceil().max(1.0));
    let mut points = vec![(width / 2.0, height / 2.0)];
    for i in 0..=steps {
        let angle = (-90.0 + sweep * f64::from(i) / f64::from(steps)).to_radians();
        points.push((
            reach.mul_add(angle.cos(), width / 2.0),
            reach.mul_add(angle.sin(), height / 2.0),
        ));
    }
    points
}

pub fn present(_: &NoSettings, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if !side.entering {
        return Ok(filled(Style::default(), side.children));
    }
    let points = clock_points(cx.at.stage.width, cx.at.stage.height, side.progress)
        .into_iter()
        .map(|(x, y)| (Length::px(x), Length::px(y)))
        .collect();
    Ok(filled(
        Style::default().clip_path(ClipPath::Polygon {
            fill: FillRule::Nonzero,
            points,
        }),
        side.children,
    ))
}
