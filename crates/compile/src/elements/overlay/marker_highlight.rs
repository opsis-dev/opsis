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
pub struct MarkerHighlight {
    #[default = "Made for "]
    #[schemars(description = "Before")]
    pub before: String,
    #[default = "builders"]
    #[schemars(description = "Highlight")]
    pub highlight: String,
    #[default = "."]
    #[schemars(description = "After")]
    pub after: String,
    #[default = "#facc15"]
    #[schemars(
        description = "Marker color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub marker_color: String,
    #[default = "#171717"]
    #[schemars(
        description = "Base color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub base_color: String,
    #[default = "#171717"]
    #[schemars(
        description = "Highlighted text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub highlighted_text_color: String,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W600)]
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
    meta::<MarkerHighlight>(
        element(
            ElementKind::Component,
            "marker-highlight",
            "Marker highlight",
            "Colored marker block draws behind a phrase as the text color shifts.",
            ElementCategory::Overlay,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:highlighter")),
    )
}
