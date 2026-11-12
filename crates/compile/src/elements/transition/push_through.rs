use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::Filter;
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PushThrough {
    #[setting("Zoom", default = 2.4, range(min = 1.5, max = 4), step = 0.1)]
    pub zoom: f64,
    #[setting("Blur", default = 14, range(min = 0, max = 30), step = 1, unit = "px")]
    pub blur: u32,
    #[setting(
        "Approach from",
        default = 0.68,
        range(min = 0.3, max = 1),
        unit = "×",
        advanced
    )]
    pub approach_from: f64,
}

pub fn definition() -> ElementMeta {
    meta::<PushThrough>(
            element(
                ElementKind::Presentation,
                "push-through",
                "Push through",
                "A dolly transition that blows the outgoing scene past the camera while the next one approaches from the depth.",
                ElementCategory::Transition,
                40,
            )
                .look(NodeLook::of("lucide:arrow-right-to-line")),
        )
}

pub fn present(props: &PushThrough, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    let blur = f64::from(props.blur);
    if !side.entering {
        let opacity = clamped_eased(p, [0.4, 0.62], [1.0, 0.0], Easing::Bezier(0.42, 0.0, 0.58, 1.0))?;
        let scale = clamped_eased(p, [0.0, 0.65], [1.0, props.zoom], Easing::ease_in(Easing::Cubic))?;
        let soft = clamped(p, [0.15, 0.6], [0.0, blur])?;
        return Ok(filled(
            Style::default()
                .opacity(opacity)
                .scale(scale, scale)
                .filter(vec![Filter::Blur(Length::px(soft))]),
            side.children,
        ));
    }
    let opacity = clamped(p, [0.3, 0.5], [0.0, 1.0])?;
    let scale = interpolate(
        p,
        &[0.3, 0.88, 1.0],
        &[props.approach_from, 1.02, 1.0],
        &Interpolation::clamped().eased(Easing::ease_out(Easing::Cubic)),
    )?;
    let soft = clamped(p, [0.3, 0.75], [blur * 0.7, 0.0])?;
    Ok(filled(
        Style::default()
            .opacity(opacity)
            .scale(scale, scale)
            .filter(vec![Filter::Blur(Length::px(soft))]),
        side.children,
    ))
}
