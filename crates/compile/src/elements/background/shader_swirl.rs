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
pub struct ShaderSwirl {
    #[setting("Speed", default = 1.0, range(min = 0.1, max = 4), step = 0.1)]
    pub speed: f64,
    #[setting("Bands", default = 4, range(min = 1, max = 12), step = 1)]
    pub band_count: u32,
    #[setting("Twist", default = 0.1, range(min = 0, max = 1), step = 0.05)]
    pub twist: f64,
    #[setting(
        "Softness",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Background", default = "#12121a", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#52527a".into(), "#3a3a5c".into(), "#232338".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting("Centre size", default = 0.2, range(min = 0, max = 1), advanced)]
    pub center: f64,
    #[setting("Proportion", default = 0.5, range(min = 0, max = 1), advanced)]
    pub proportion: f64,
    #[setting("Wobble", default = 0.2, range(min = 0, max = 1), advanced)]
    pub noise: f64,
    #[setting("Wobble size", default = 0.4, range(min = 0, max = 1), advanced)]
    pub noise_frequency: f64,
    #[setting("Scale", default = 1.0, range(min = 0.1, max = 4), advanced)]
    pub scale: f64,
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
