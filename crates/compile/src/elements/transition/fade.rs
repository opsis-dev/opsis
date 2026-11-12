use opsis_scene::node::Node;
use opsis_scene::style::Style;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::NoSettings;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub fn definition() -> ElementMeta {
    meta::<NoSettings>(
        element(
            ElementKind::Presentation,
            "fade",
            "Fade",
            "",
            ElementCategory::Transition,
            15,
        )
        .look(NodeLook::of("lucide:blend").moves(NodeMotion::FadeThrough)),
    )
}

pub fn present(_: &NoSettings, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let opacity = if side.entering { side.progress } else { 1.0 };
    Ok(filled(&Style::default().opacity(opacity), side.children))
}
