use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderGodRays {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Intensity", default = 0.8, range(min = 0, max = 2), step = 0.05)]
    pub intensity: f64,
    #[setting("Density", default = 0.3, range(min = 0, max = 1), step = 0.05)]
    pub density: f64,
    #[setting("Bloom", default = 0.4, range(min = 0, max = 1), step = 0.05)]
    pub bloom: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#5a5a7e".into(), "#8a8a95".into(), "#ffffff".into(), "#3a3a5c".into()],
        length(min = 1, max = 5)
    )]
    pub colors: Vec<Colour>,
    #[setting("Glow colour", default = "#3a3a5c", check = colour)]
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
