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
pub struct ShaderWarp {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.2]
    #[schemars(description = "Distortion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub distortion: f64,
    #[default = 0.4]
    #[schemars(description = "Swirl", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub swirl: f64,
    #[default = 1.0]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.5]
    #[schemars(description = "Proportion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub proportion: f64,
    #[default(vec!["#12121a".into(), "#3a3a5c".into(), "#12121a".into(), "#52527a".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderWarp>(
        element(
            ElementKind::Component,
            "shader-warp",
            "Shader warp",
            "Domain-warped color field that folds and stretches like liquid.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:droplets")),
    )
}
