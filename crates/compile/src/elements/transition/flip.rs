use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::paint::Transform;
use opsis_scene::style::{Style, Visibility};
use opsis_scene::values::Angle;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::shared::Direction;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct Flip {
    #[setting("Direction", default = Direction::FromRight)]
    pub direction: Direction,
    #[setting(
        "Perspective",
        default = 1000,
        range(min = 200, max = 5000),
        step = 1,
        unit = "px",
        advanced
    )]
    pub perspective: u32,
}

pub fn definition() -> ElementMeta {
    meta::<Flip>(
        element(
            ElementKind::Presentation,
            "flip",
            "Flip",
            "",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of(
            "lucide:triangles-centerline-dashed-horizontal",
        )),
    )
}

const fn turn_of(direction: Direction) -> (bool, f64) {
    match direction {
        Direction::FromLeft => (true, -1.0),
        Direction::FromRight => (true, 1.0),
        Direction::FromTop => (false, 1.0),
        Direction::FromBottom => (false, -1.0),
    }
}

pub fn flip_angle(direction: Direction, entering: bool, progress: f64) -> f64 {
    let (_, side) = turn_of(direction);
    if entering {
        side * 180.0 * (1.0 - progress)
    } else {
        -side * 180.0 * progress
    }
}

pub fn present(props: &Flip, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let angle = Angle::deg(flip_angle(props.direction, side.entering, side.progress));
    let (around_y, _) = turn_of(props.direction);
    let turn = if around_y {
        Transform::RotateY(angle)
    } else {
        Transform::RotateX(angle)
    };
    Ok(filled(
        Style::default().perspective(f64::from(props.perspective)),
        vec![filled(
            Style::default()
                .transform(vec![turn])
                .backface_visibility(Visibility::Hidden),
            side.children,
        )],
    ))
}
