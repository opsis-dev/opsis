use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AnnounceTitle {
    #[default = "Introducing"]
    #[schemars(description = "Eyebrow")]
    pub eyebrow: String,
    #[default = "Acme 2.0"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Everything your team needs to ship, in one place."]
    #[schemars(description = "Tagline")]
    pub tagline: String,
    #[schemars(description = "Symbol path")]
    pub symbol_path: String,
    #[schemars(
        description = "Symbol colours",
        length(min = 0, max = 8),
        extend("x-help" = "Leave empty for the built-in colours.")
    )]
    pub symbol_colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Symbol size", range(min = 0, max = 3), extend("x-step" = 0.05))]
    pub symbol_scale: f64,
    #[default = "#5600f5"]
    #[schemars(
        description = "Colour field",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub field_color: String,
    #[default = "#4800c9"]
    #[schemars(
        description = "Title ink",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub title_color: String,
    #[default = "#fcfcfa"]
    #[schemars(
        description = "Title paper",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub paper_color: String,
    #[default = "#100022"]
    #[schemars(
        description = "Opening ground",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub void_color: String,
    #[default = "#000028"]
    #[schemars(
        description = "Closing ground",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub night_color: String,
    #[default = "#08ff4b"]
    #[schemars(
        description = "Closing glow",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub glow_color: String,
    #[default = 0.139]
    #[schemars(description = "Glow strength", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub glow_strength: f64,
    #[default = "#f2f8ff"]
    #[schemars(
        description = "Tagline ink",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub tagline_color: String,
    #[default = "#ffffff"]
    #[schemars(
        description = "Shot ink",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ink_color: String,
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
    meta::<AnnounceTitle>(
            element(
                ElementKind::Component,
                "announce-title",
                "Announce title",
                "Launch title animation, product announcement intro with kinetic typography. A four-shot launch title: the eyebrow rushes past the camera on a receding type plane and lands on a colour field, the name assembles word by word on paper, and a macro pan across the tagline cuts wide as the line builds itself last word first while a white sweep drains it out of colour.",
                ElementCategory::Text,
                170,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:megaphone")),
        )
}
