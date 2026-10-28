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
pub struct Paper {
    #[default = "#f5f1e8"]
    #[schemars(
        description = "Paper colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = 0.18]
    #[schemars(description = "Grain", range(min = 0, max = 0.5), extend("x-step" = 0.01))]
    pub grain: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Paper>(
        element(
            ElementKind::Component,
            "paper",
            "Paper",
            "A paper texture background.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:scroll")),
    )
}
