use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MarkLook {
    Underline,
    Circle,
    Box,
    Marker,
    CrossedOff,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum TitleAlign {
    Center,
    Left,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Entrance {
    Rise,
    Fade,
    Blur,
    Scale,
    None,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Title {
    #[schemars(description = "Eyebrow")]
    pub eyebrow: String,
    #[default = "Sign up in 10 seconds"]
    #[schemars(description = "Title")]
    pub title: String,
    #[schemars(description = "Subtitle")]
    pub subtitle: String,
    #[schemars(
        description = "Mark a word",
        extend("x-help" = "Words from the title to draw a hand-drawn mark on, after the title comes in. Leave empty for none.")
    )]
    pub mark: String,
    #[default(MarkLook::Underline)]
    #[schemars(
        description = "Mark",
        extend(
            "x-labels" = {
                "underline": "Underline",
                "circle": "Circle",
                "box": "Box",
                "marker": "Marker",
                "crossed-off": "Crossed out"
            }
        )
    )]
    pub mark_look: MarkLook,
    #[default = "#facc15"]
    #[schemars(
        description = "Mark colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub mark_color: String,
    #[default(TitleAlign::Center)]
    #[schemars(description = "Alignment")]
    pub align: TitleAlign,
    #[default(Entrance::Rise)]
    #[schemars(description = "Entrance")]
    pub entrance: Entrance,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#a78bfa"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent: String,
    #[default = "transparent"]
    #[schemars(description = "Background (colour, gradient or 'transparent')")]
    pub background: String,
    #[default = 112]
    #[schemars(description = "Font size", range(min = 24, max = 260), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 760]
    #[schemars(description = "Font weight", range(min = 100, max = 900), extend("x-step" = 10))]
    pub font_weight: u32,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<Title>(
        element(
            ElementKind::Component,
            "title",
            "Title card",
            "A full-screen title with an optional eyebrow and subtitle.",
            ElementCategory::Text,
            75,
        )
        .look(NodeLook::of("lucide:heading")),
    )
}
