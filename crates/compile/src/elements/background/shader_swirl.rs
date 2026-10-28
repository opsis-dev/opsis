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
pub struct ShaderSwirl {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 4]
    #[schemars(description = "Bands", range(min = 1, max = 12), extend("x-step" = 1))]
    pub band_count: u32,
    #[default = 0.1]
    #[schemars(description = "Twist", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub twist: f64,
    #[default = 0.2]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#12121a"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#52527a".into(), "#3a3a5c".into(), "#232338".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
}

pub fn definition() -> ElementMeta {
    meta::<ShaderSwirl>(
        element(
            ElementKind::Component,
            "shader-swirl",
            "Shader swirl",
            "Concentric bands swirling around a center point.",
            ElementCategory::Background,
            150,
        )
        .authored_clock()
        .look(NodeLook::of("tabler:spiral")),
    )
}
