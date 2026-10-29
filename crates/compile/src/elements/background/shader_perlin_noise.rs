use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderPerlinNoise {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Proportion", default = 0.35, range(min = 0, max = 1), step = 0.05)]
    pub proportion: f64,
    #[setting(
        "Softness",
        default = 0.1,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting("Front", default = "#6a6a85", check = colour)]
    pub color_front: String,
    #[setting("Detail", default = 1, range(min = 1, max = 8), step = 1, advanced)]
    pub octave_count: u32,
    #[setting("Roughness", default = 1.0, range(min = 0.3, max = 1), advanced)]
    pub persistence: f64,
    #[setting("Detail spacing", default = 1.5, range(min = 1.5, max = 10), advanced)]
    pub lacunarity: f64,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
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
