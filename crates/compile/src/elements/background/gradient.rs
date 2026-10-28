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
pub struct Gradient {
    #[default = "#1e1b4b"]
    #[schemars(
        description = "From",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub from: String,
    #[default = "#0f172a"]
    #[schemars(
        description = "To",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub to: String,
    #[default = 135]
    #[schemars(description = "Angle", range(min = 0, max = 360), extend("x-step" = 1))]
    pub angle: u32,
    #[default = 0.08]
    #[schemars(description = "Grain", range(min = 0, max = 0.4), extend("x-step" = 0.01))]
    pub grain: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Gradient>(
        element(
            ElementKind::Component,
            "gradient",
            "Gradient",
            "A gradient background.",
            ElementCategory::Background,
            90,
        )
        .look(NodeLook::of("lucide:rainbow")),
    )
}
