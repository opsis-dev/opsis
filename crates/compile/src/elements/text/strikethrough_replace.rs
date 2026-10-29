use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StrikethroughReplace {
    #[setting("From", default = "$49/mo")]
    pub from: String,
    #[setting("To", default = "Free")]
    pub to: String,
    #[setting("Line color", default = "#ff5e3a", check = colour)]
    pub line_color: String,
    #[setting("Font size", default = 96, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<StrikethroughReplace>(
        element(
            ElementKind::Component,
            "strikethrough-replace",
            "Strikethrough replace",
            "Draws a strike line across old text then reveals new text in its place.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:strikethrough").morphs("lucide:type")),
    )
}
