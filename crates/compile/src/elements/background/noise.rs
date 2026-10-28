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
pub struct Noise {
    #[default = "#0b1020"]
    #[schemars(
        description = "Base",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color1: String,
    #[default = "#4f46e5"]
    #[schemars(
        description = "Highlight",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color2: String,
    #[default = 0.004]
    #[schemars(description = "Scale", range(min = 0.0005, max = 0.02), extend("x-step" = 0.0005))]
    pub scale: f64,
    #[default = 0.004]
    #[schemars(description = "Speed", range(min = 0, max = 0.05), extend("x-step" = 0.001))]
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
