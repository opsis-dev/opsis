use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum DitherPattern {
    #[serde(rename = "random")]
    #[label("Random")]
    Random,
    #[serde(rename = "2x2")]
    #[label("2 × 2")]
    Grid2x2,
    #[serde(rename = "4x4")]
    #[label("4 × 4")]
    Grid4x4,
    #[serde(rename = "8x8")]
    #[label("8 × 8")]
    Grid8x8,
}

#[settings(export)]
pub struct ShaderDithering {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Dot Size", default = 2, range(min = 1, max = 8), step = 1)]
    pub size: u32,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting("Front", default = "#6a6a85", check = colour)]
    pub color_front: String,
    #[setting("Pattern", default = DitherShape::Wave)]
    pub shape: DitherShape,
    #[setting(
        "Dot grid",
        default = DitherPattern::Grid4x4,
        advanced,
        labels = { "random": "Random", "2x2": "2 × 2", "4x4": "4 × 4", "8x8": "8 × 8" }
    )]
    pub r#type: DitherPattern,
    #[setting("Scale", default = 0.6, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
    #[setting("Turn", default = 0.0, range(min = 0, max = 360), unit = "°", advanced)]
    pub rotation: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderDithering>(
        element(
            ElementKind::Component,
            "shader-dithering",
            "Shader dithering",
            "Ordered-dither shading over an animated gradient field.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:grip")),
    )
}
