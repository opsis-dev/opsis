use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::timeline::ChildAt;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::filtered;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct CameraLens {
    #[setting("Corner softness", default = 0.5, range(min = 0, max = 2), step = 0.05)]
    pub softness: f64,
    #[setting("Bloom", default = 0.5, range(min = 0, max = 2), step = 0.05)]
    pub bloom: f64,
    #[setting("Aberration", default = 0.5, range(min = 0, max = 3), step = 0.1)]
    pub aberration: f64,
    #[setting("Vignette", default = 0.5, range(min = 0, max = 1.5), step = 0.05)]
    pub vignette: f64,
    #[setting(
        "Distortion",
        default = 0.05,
        range(min = 0, max = 0.6),
        step = 0.01,
        advanced
    )]
    pub distortion: f64,
    #[setting("Sharp centre", default = 0.22, range(min = 0, max = 0.9), advanced)]
    pub sharp_centre: f64,
    #[setting(
        "Bloom from brightness",
        default = 0.62,
        range(min = 0, max = 1),
        advanced
    )]
    pub bloom_threshold: f64,
}

pub fn definition() -> ElementMeta {
    meta::<CameraLens>(
            element(
                ElementKind::Wrapper,
                "camera-lens",
                "Camera lens",
                "Put a scene behind real glass: resolution falls off toward the corners, highlights bloom through the optics, colour fringes at the edges — a screencast that reads as filmed rather than rendered.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:aperture")),
        )
}

pub fn wrap(props: &CameraLens, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let inner = child(cx.at.stage)?;
    Ok(filtered(
        "camera-lens",
        vec![
            ("softness", vec![props.softness]),
            ("bloom", vec![props.bloom]),
            ("aberration", vec![props.aberration]),
            ("vignette", vec![props.vignette]),
            ("distortion", vec![props.distortion]),
            ("sharpCentre", vec![props.sharp_centre]),
            ("bloomThreshold", vec![props.bloom_threshold]),
        ],
        inner,
    ))
}
