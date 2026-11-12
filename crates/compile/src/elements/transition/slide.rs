use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::paint::Transform;
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct Slide {
    #[setting("Direction", default = Direction::FromRight)]
    pub direction: Direction,
}

pub fn definition() -> ElementMeta {
    meta::<Slide>(
        element(
            ElementKind::Presentation,
            "slide",
            "Slide",
            "",
            ElementCategory::Transition,
            20,
        )
        .look(NodeLook::of("tabler:transition-right")),
    )
}

const fn axis_of(direction: Direction) -> (bool, f64) {
    match direction {
        Direction::FromLeft => (true, -1.0),
        Direction::FromRight => (true, 1.0),
        Direction::FromTop => (false, -1.0),
        Direction::FromBottom => (false, 1.0),
    }
}

pub fn slide_offset(direction: Direction, entering: bool, progress: f64) -> f64 {
    let (_, side) = axis_of(direction);
    if entering {
        side * (1.0 - progress)
    } else {
        -side * progress
    }
}

pub fn present(props: &Slide, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let offset =
        Length::percent(slide_offset(props.direction, side.entering, side.progress) * 100.0);
    let (horizontal, _) = axis_of(props.direction);
    let shift = if horizontal {
        Transform::Translate(offset, Length::ZERO, Length::ZERO)
    } else {
        Transform::Translate(Length::ZERO, offset, Length::ZERO)
    };
    Ok(filled(
        &Style::default().transform(vec![shift]),
        side.children,
    ))
}
