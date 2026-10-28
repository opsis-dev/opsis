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
pub struct Polaroid {
    #[default = "first light"]
    #[schemars(description = "Caption")]
    pub caption: String,
    #[schemars(description = "Caption starts", range(min = 0, max = 60), extend("x-step" = 1))]
    pub caption_at: u32,
    #[default = 652]
    #[schemars(description = "Card width", range(min = 320, max = 900), extend("x-step" = 4))]
    pub width: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Caption size",
        range(min = 12, max = 56),
        extend(
            "x-zero-label" = "Fit",
            "x-step" = 1,
            "x-unit" = "px",
            "x-help" = "Leave empty to size the caption with the card.",
            "type" = "integer"
        )
    )]
    pub caption_size: Option<u32>,
    #[default = 0.1]
    #[schemars(
        description = "Time per pose",
        range(min = 0.0333, max = 0.2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub step: f64,
    #[default = "#fdfcf8"]
    #[schemars(
        description = "Frame colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub frame_color: String,
    #[default = "#26242c"]
    #[schemars(
        description = "Caption colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub caption_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Polaroid>(
            element(
                ElementKind::Wrapper,
                "polaroid",
                "Polaroid",
                "An instant-photo frame with a handwritten caption for video, images, or live compositions.",
                ElementCategory::Effect,
                150,
            )
                .fonts(&["Caveat"])
                .look(NodeLook::of("tabler:polaroid")),
        )
}
