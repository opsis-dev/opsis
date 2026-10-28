use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum GrainGradientShape {
    Corners,
    Wave,
    Dots,
    Truchet,
    Ripple,
    Blob,
    Sphere,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderGrainGradient {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.6]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.2]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.15]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = "#12121a"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#5a5a7e".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default(GrainGradientShape::Corners)]
    #[schemars(description = "Pattern")]
    pub shape: GrainGradientShape,
    #[default = 1.0]
    #[schemars(
        description = "Scale",
        range(min = 0.1, max = 4),
        extend(
            "x-step" = 0.05,
            "x-help" = "How large the pattern is in the frame.",
            "x-advanced" = true
        )
    )]
    pub scale: f64,
    #[schemars(
        description = "Turn",
        range(min = 0, max = 360),
        extend(
            "x-step" = 1,
            "x-unit" = "°",
            "x-help" = "Turns the pattern around the middle of the frame.",
            "x-advanced" = true
        )
    )]
    pub rotation: u32,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderGrainGradient>(
        element(
            ElementKind::Component,
            "shader-grain-gradient",
            "Shader grain gradient",
            "Grainy multi-stop gradient that drifts slowly with film-grain texture over the blend.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("tabler:noise-reduction")),
    )
}
