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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Title {
    #[setting("Eyebrow")]
    pub eyebrow: String,
    #[setting("Title", default = "Sign up in 10 seconds")]
    pub title: String,
    #[setting("Subtitle")]
    pub subtitle: String,
    #[setting(
        "Mark a word",
        help = "Words from the title to draw a hand-drawn mark on, after the title comes in. Leave empty for none."
    )]
    pub mark: String,
    #[setting(
        "Mark",
        default = MarkLook::Underline,
        labels = {
            "underline": "Underline",
            "circle": "Circle",
            "box": "Box",
            "marker": "Marker",
            "crossed-off": "Crossed out"
        }
    )]
    pub mark_look: MarkLook,
    #[setting("Mark colour", default = "#facc15", check = colour)]
    pub mark_color: String,
    #[setting("Alignment", default = TitleAlign::Center)]
    pub align: TitleAlign,
    #[setting("Entrance", default = Entrance::Rise)]
    pub entrance: Entrance,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting(
        "Background (colour, gradient or 'transparent')",
        default = "transparent"
    )]
    pub background: String,
    #[setting("Font size", default = 112, range(min = 24, max = 260), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = 760, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
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
