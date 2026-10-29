use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
pub enum PhotoShape {
    #[serde(rename = "16:9")]
    Wide,
    #[serde(rename = "4:3")]
    Classic,
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "3:4")]
    Tall,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Polaroid {
    #[setting("Caption", default = "first light")]
    pub caption: String,
    #[setting("Caption starts", range(min = 0, max = 60), step = 1, advanced)]
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
        "type" = "integer",
        advanced
    )]
    pub caption_size: Option<u32>,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub step: f64,
    #[setting("Frame colour", default = "#fdfcf8", check = colour)]
    pub frame_color: String,
    #[setting("Caption colour", default = "#26242c", check = colour)]
    pub caption_color: String,
    #[setting(
        "Photo shape",
        default = PhotoShape::Wide,
        labels = { "16:9": "16:9", "4:3": "4:3", "1:1": "Square", "3:4": "3:4" },
        advanced
    )]
    pub photo_shape: PhotoShape,
    #[setting(
        "Caption strip height",
        default = 62,
        range(min = 30, max = 200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub caption_strip: u32,
    #[setting(
        "Rounded corners",
        default = 4,
        range(min = 0, max = 24),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Caption weight",
        default = 600,
        range(min = 400, max = 700),
        step = 100,
        advanced
    )]
    pub caption_weight: u32,
    #[setting("Letters per pose", default = 1.4, range(min = 0.5, max = 4), advanced)]
    pub write_speed: f64,
    #[setting("Caption font", widget = "font", zero_label = "Handwriting", advanced)]
    pub caption_font: String,
    #[setting("Photo background", default = "#100f14", widget = "color", check = colour, advanced)]
    pub photo_color: String,
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
