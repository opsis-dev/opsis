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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderDithering {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 2]
    #[schemars(description = "Dot Size", range(min = 1, max = 8), extend("x-step" = 1))]
    pub size: u32,
    #[default = "#12121a"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default = "#6a6a85"]
    #[schemars(
        description = "Front",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_front: String,
    #[default(DitherShape::Wave)]
    #[schemars(description = "Pattern", extend("x-advanced" = true))]
    pub shape: DitherShape,
    #[default(DitherPattern::Grid4x4)]
    #[schemars(
        description = "Dot grid",
        extend(
            "x-advanced" = true,
            "x-labels" = { "random": "Random", "2x2": "2 × 2", "4x4": "4 × 4", "8x8": "8 × 8" }
        )
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
