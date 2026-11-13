use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::timeline::ChildAt;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::filtered;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct VhsFilter {
    #[setting("Chroma bleed", default = 1.0, range(min = 0, max = 3), step = 0.1)]
    pub bleed: f64,
    #[setting("Tape wobble", default = 1.0, range(min = 0, max = 3), step = 0.1)]
    pub wobble: f64,
    #[setting("Noise", default = 1.0, range(min = 0, max = 3), step = 0.1)]
    pub noise: f64,
    #[setting("Intensity", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting("Bottom tear", default = 1.0, range(min = 0, max = 3), advanced)]
    pub head_switch: f64,
    #[setting("Scanlines", default = 1.0, range(min = 0, max = 3), advanced)]
    pub scanlines: f64,
}

pub fn definition() -> ElementMeta {
    meta::<VhsFilter>(
            element(
                ElementKind::Wrapper,
                "vhs-filter",
                "VHS filter",
                "Hold a scene under analog tape: chroma bleeds sideways off every edge, rows wobble out of line, the bottom of the frame tears on the head switch, and luma noise settles over all of it.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:cassette-tape")),
        )
}

pub fn wrap(props: &VhsFilter, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let inner = child(cx.at.stage)?;
    Ok(filtered(
        "vhs-filter",
        vec![
            ("frame", vec![cx.frame()]),
            ("time", vec![cx.frame() / cx.fps()]),
            ("bleed", vec![props.bleed]),
            ("wobble", vec![props.wobble]),
            ("noise", vec![props.noise]),
            ("intensity", vec![props.intensity]),
            ("headSwitch", vec![props.head_switch]),
            ("scanlines", vec![props.scanlines]),
        ],
        inner,
    ))
}
