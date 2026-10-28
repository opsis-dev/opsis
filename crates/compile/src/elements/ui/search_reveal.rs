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
pub struct SearchReveal {
    #[default = "acme"]
    #[schemars(description = "Search text")]
    pub text: String,
    #[default = 464]
    #[schemars(description = "Field width", range(min = 320, max = 640), extend("x-step" = 8))]
    pub field_width: u32,
    #[default = 34]
    #[schemars(description = "Font size", range(min = 18, max = 54), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 0.1333]
    #[schemars(
        description = "Time per letter",
        range(min = 0.0333, max = 0.4),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub frames_per_character: f64,
    #[default = true]
    #[schemars(description = "Graphic panel")]
    pub show_panel: bool,
    #[default = true]
    #[schemars(description = "Construction guides")]
    pub show_guides: bool,
    #[default = "#26232b"]
    #[schemars(
        description = "Text and markers",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#ffffff"]
    #[schemars(
        description = "Search field",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub field_color: String,
    #[default = "#cbc5cf"]
    #[schemars(
        description = "Circles and search icon",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ring_color: String,
    #[default = "#a800b7"]
    #[schemars(
        description = "Panel color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub panel_color: String,
    #[default = "#e8f99a"]
    #[schemars(
        description = "Graphic accents",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = "#f5f1f5"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background_color: String,
    #[schemars(description = "Static final pose")]
    pub reduced_motion: bool,
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
    meta::<SearchReveal>(
            element(
                ElementKind::Component,
                "search-reveal",
                "Search reveal",
                "Expanding construction circles snap into a search pill, type editable text, and reveal an abstract graphic panel.",
                ElementCategory::Ui,
                108,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:search")),
        )
}
