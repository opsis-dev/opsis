use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::timeline::ChildAt;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{filtered, rgb_uniform};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct TvPowerOff {
    #[setting("Duration", default = 18, range(min = 8, max = 30), step = 1)]
    pub duration_in_frames: u32,
    #[setting("Surge", default = 1.0, range(min = 0, max = 2), step = 0.05, advanced)]
    pub gain: f64,
    #[setting("Afterglow", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub afterglow: f64,
    #[setting("Phosphor", default = "#d8ecff", check = colour)]
    pub phosphor: String,
    #[setting("Screen off colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub screen_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<TvPowerOff>(
            element(
                ElementKind::Wrapper,
                "tv-power-off",
                "TV power off",
                "End a scene the way a tube does: the picture collapses into a glowing horizontal line, the line pinches to a phosphor dot, the dot decays, and the frame is left dead black.",
                ElementCategory::Effect,
                62,
            )
                .look(NodeLook::of("lucide:tv").moves(NodeMotion::Collapse)),
        )
}

fn power_progress(props: &TvPowerOff, frame: f64, film_frames: f64) -> f64 {
    let frames = f64::from(props.duration_in_frames);
    let delay = (film_frames - frames).max(0.0);
    ((frame - delay) / frames.max(1.0)).clamp(0.0, 1.0)
}

pub fn wrap(props: &TvPowerOff, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let inner = child(cx.at.stage)?;
    Ok(filtered(
        "tv-power-off",
        vec![
            (
                "progress",
                vec![power_progress(props, cx.frame(), cx.duration())],
            ),
            ("gain", vec![props.gain]),
            ("afterglow", vec![props.afterglow]),
            ("phosphor", rgb_uniform(&props.phosphor)),
            ("screen", rgb_uniform(&props.screen_color)),
        ],
        inner,
    ))
}
