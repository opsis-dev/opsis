use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderGodRays {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.8]
    #[schemars(description = "Intensity", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.3]
    #[schemars(description = "Density", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub density: f64,
    #[default = 0.4]
    #[schemars(description = "Bloom", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bloom: f64,
    #[default = "#12121a"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#5a5a7e".into(), "#8a8a95".into(), "#ffffff".into(), "#3a3a5c".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 5))]
    pub colors: Vec<Colour>,
    #[default = "#3a3a5c"]
    #[schemars(
        description = "Glow colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_bloom: String,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderGodRays>(
        element(
            ElementKind::Component,
            "shader-god-rays",
            "Shader god rays",
            "Volumetric light rays radiating from an off-screen source.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:sunrise")),
    )
}
