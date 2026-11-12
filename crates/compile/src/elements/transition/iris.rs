use opsis_scene::node::Node;
use opsis_scene::paint::{ClipPath, ShapeRadius};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "iris",
            "Iris",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:aperture")),
    )
}

pub fn iris_radius(width: f64, height: f64, progress: f64) -> f64 {
    progress * width.hypot(height) / 2.0
}

pub fn present(_: &NoSettings, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if !side.entering {
        return Ok(filled(&Style::default(), side.children));
    }
    let radius = iris_radius(cx.at.stage.width, cx.at.stage.height, side.progress);
    Ok(filled(
        &Style::default().clip_path(ClipPath::Circle {
            radius: ShapeRadius::Length(Length::px(radius)),
            center: Position2::center(),
        }),
        side.children,
    ))
}
