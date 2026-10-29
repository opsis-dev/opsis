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
pub struct ShaderWarp {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Distortion", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub distortion: f64,
    #[setting("Swirl", default = 0.4, range(min = 0, max = 1), step = 0.05)]
    pub swirl: f64,
    #[setting("Softness", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub softness: f64,
    #[setting("Proportion", default = 0.5, range(min = 0, max = 1), step = 0.05)]
    pub proportion: f64,
    #[setting(
        "Colours",
        default = vec!["#12121a".into(), "#3a3a5c".into(), "#12121a".into(), "#52527a".into()],
        length(min = 2, max = 10)
    )]
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
