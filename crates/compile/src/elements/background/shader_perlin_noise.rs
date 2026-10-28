use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderPerlinNoise {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.35]
    #[schemars(description = "Proportion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub proportion: f64,
    #[default = 0.1]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
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
}

pub fn definition() -> ElementMeta {
    meta::<ShaderPerlinNoise>(
        element(
            ElementKind::Component,
            "shader-perlin-noise",
            "Shader perlin noise",
            "Classic Perlin noise clouds drifting across the frame.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:cloudy")),
    )
}
