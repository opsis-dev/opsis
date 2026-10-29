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
pub struct Polaroid {
    #[setting("Caption", default = "first light")]
    pub caption: String,
    #[setting("Caption starts", range(min = 0, max = 60), step = 1)]
    pub caption_at: u32,
    #[setting("Card width", default = 652, range(min = 320, max = 900), step = 4)]
    pub width: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Caption size",
        range(min = 12, max = 56),
        zero_label = "Fit",
        step = 1,
        unit = "px",
        help = "Leave empty to size the caption with the card.",
        "type" = "integer"
    )]
    pub caption_size: Option<u32>,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub step: f64,
    #[setting("Frame colour", default = "#fdfcf8", check = colour)]
    pub frame_color: String,
    #[setting("Caption colour", default = "#26242c", check = colour)]
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
