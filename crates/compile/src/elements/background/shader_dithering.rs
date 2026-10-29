use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
pub enum DitherPattern {
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "2x2")]
    Grid2x2,
    #[serde(rename = "4x4")]
    Grid4x4,
    #[serde(rename = "8x8")]
    Grid8x8,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderDithering {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Dot Size", default = 2, range(min = 1, max = 8), step = 1)]
    pub size: u32,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting("Front", default = "#6a6a85", check = colour)]
    pub color_front: String,
    #[setting("Pattern", default = DitherShape::Wave, advanced)]
    pub shape: DitherShape,
    #[setting(
        "Dot grid",
        default = DitherPattern::Grid4x4,
        advanced,
        labels = { "random": "Random", "2x2": "2 × 2", "4x4": "4 × 4", "8x8": "8 × 8" }
    )]
    pub r#type: DitherPattern,
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
