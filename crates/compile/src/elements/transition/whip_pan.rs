use std::f64::consts::PI;

use opsis_macros::{choice, settings};
use opsis_scene::easing::Easing;
use opsis_scene::node::Node;
use opsis_scene::paint::Filter;
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum WhipDirection {
    Left,
    Right,
    Up,
    Down,
}

#[settings(export)]
pub struct WhipPan {
    #[setting("Direction", default = WhipDirection::Left)]
    pub direction: WhipDirection,
    #[setting("Blur", default = 24, range(min = 0, max = 48), step = 2, unit = "px")]
    pub blur: u32,
    #[setting(
        "Stretch",
        default = 12.0,
        range(min = 0, max = 40),
        unit = "%",
        advanced
    )]
    pub stretch: f64,
}

pub fn definition() -> ElementMeta {
    meta::<WhipPan>(
            element(
                ElementKind::Presentation,
                "whip-pan",
                "Whip pan",
                "A whip-pan transition that flies both scenes through the frame in one move with velocity-based motion blur.",
                ElementCategory::Transition,
                26,
            )
                .look(NodeLook::of("lucide:move-horizontal")),
        )
}

const TRAVEL_PERCENT: f64 = 110.0;

pub fn present(props: &WhipPan, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let travel = clamped_eased(
        side.progress,
        [0.0, 1.0],
        [0.0, 1.0],
        Easing::Bezier(0.7, 0.0, 0.2, 1.0),
    )?;
    let velocity = (PI * travel).sin();
    let horizontal = matches!(props.direction, WhipDirection::Left | WhipDirection::Right);
    let sign = if matches!(props.direction, WhipDirection::Left | WhipDirection::Up) {
        -1.0
    } else {
        1.0
    };
    let offset = if side.entering {
        (travel - 1.0) * TRAVEL_PERCENT * sign
    } else {
        travel * TRAVEL_PERCENT * sign
    };
    let stretch = 1.0 + velocity * props.stretch / 100.0;
    let style = if horizontal {
        Style::default()
            .translate(Length::percent(offset), Length::ZERO)
            .scale(stretch, 1.0)
    } else {
        Style::default()
            .translate(Length::ZERO, Length::percent(offset))
            .scale(1.0, stretch)
    };
    Ok(filled(
        &style.filter(vec![Filter::Blur(Length::px(
            velocity * f64::from(props.blur),
        ))]),
        side.children,
    ))
}
