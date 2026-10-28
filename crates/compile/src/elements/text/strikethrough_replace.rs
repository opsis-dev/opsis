use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StrikethroughReplace {
    #[default = "$49/mo"]
    #[schemars(description = "From")]
    pub from: String,
    #[default = "Free"]
    #[schemars(description = "To")]
    pub to: String,
    #[default = "#ff5e3a"]
    #[schemars(
        description = "Line color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub line_color: String,
    #[default = 96]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default(FontWeight::W700)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = "white"]
    #[schemars(
        description = "Background",
        extend(
            "x-help" = "Set it to transparent to lay the text over other layers.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background: String,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
