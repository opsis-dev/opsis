use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::background::shader_dithering::DitherPattern;
use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct DitherDissolve {
    #[setting("Shape", default = DitherShape::Simplex)]
    pub shape: DitherShape,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting("Ink", default = "#8f88ae", check = colour)]
    pub color_front: String,
    #[setting(
        "Speed",
        default = 1.5,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Dot size",
        default = 1.6,
        range(min = 1, max = 8),
        unit = "px",
        advanced
    )]
    pub dot_size: f64,
    #[setting(
        "Dot grid",
        default = DitherPattern::Grid4x4,
        advanced
    )]
    pub pattern: DitherPattern,
}

pub fn definition() -> ElementMeta {
    meta::<DitherDissolve>(
        element(
            ElementKind::Presentation,
            "dither-dissolve",
            "Dither dissolve",
            "A dithering shader transition that dissolves one scene into the next.",
            ElementCategory::Transition,
            100,
        )
        .look(NodeLook::of("lucide:grip").moves(NodeMotion::Dissolve)),
    )
}
