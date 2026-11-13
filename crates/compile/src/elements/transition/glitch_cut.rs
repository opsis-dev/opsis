use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{paired, rgb_uniform, value};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GlitchCut {
    #[setting("Intensity", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub intensity: f64,
    #[setting("Slices", default = 24, range(min = 8, max = 64), step = 2, advanced)]
    pub slices: u32,
    #[setting("RGB split", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub rgb_split: f64,
    #[setting(
        "Block noise",
        default = 0.6,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub block_noise: f64,
    #[setting("Flicker", default = 22, range(min = 4, max = 60), step = 1, advanced)]
    pub flicker: u32,
    #[setting(
        "Block columns",
        default = 16,
        range(min = 4, max = 64),
        step = 1,
        advanced
    )]
    pub block_columns: u32,
    #[setting("Tint", default = "#0d0417", widget = "color", check = colour, advanced)]
    pub tint: String,
}

pub fn definition() -> ElementMeta {
    meta::<GlitchCut>(
            element(
                ElementKind::Presentation,
                "glitch-cut",
                "Glitch cut",
                "A broadcast-glitch hard cut: the frame tears into displaced slices with channel split and corrupted blocks, and the incoming scene takes over slice by slice at the peak.",
                ElementCategory::Transition,
                12,
            )
                .look(NodeLook::of("lucide:zap")),
        )
}

pub fn present(props: &GlitchCut, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    Ok(paired(
        "glitch-cut",
        vec![
            value("progress", vec![side.progress]),
            value("intensity", vec![props.intensity]),
            value("slices", vec![f64::from(props.slices)]),
            value("rgbSplit", vec![props.rgb_split]),
            value("blockNoise", vec![props.block_noise]),
            value("steps", vec![f64::from(props.flicker)]),
            value("blockColumns", vec![f64::from(props.block_columns)]),
            value("tint", rgb_uniform(&props.tint)),
        ],
        side.entering,
        side.cut,
        side.children,
    ))
}
