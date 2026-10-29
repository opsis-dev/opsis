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
pub struct Noise {
    #[setting("Base", default = "#0b1020", check = colour)]
    pub color1: String,
    #[setting("Highlight", default = "#4f46e5", check = colour)]
    pub color2: String,
    #[setting(
        "Scale",
        default = 0.004,
        range(min = 0.0005, max = 0.02),
        step = 0.0005
    )]
    pub scale: f64,
    #[setting("Speed", default = 0.004, range(min = 0, max = 0.05), step = 0.001)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Noise>(
        element(
            ElementKind::Component,
            "noise",
            "Noise",
            "A soft moving noise background.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("tabler:grain")),
    )
}
