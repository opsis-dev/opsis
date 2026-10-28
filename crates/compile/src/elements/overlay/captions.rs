use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub const CAPTIONS_ELEMENT: &str = "captions";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum CaptionStyle {
    WordByWord,
    Karaoke,
    Line,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum CaptionPosition {
    Bottom,
    Center,
    Top,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Captions {
    #[default(CaptionStyle::WordByWord)]
    #[schemars(description = "Style")]
    pub style: CaptionStyle,
    #[default(CaptionPosition::Bottom)]
    #[schemars(description = "Position")]
    pub position: CaptionPosition,
    #[default = 56]
    #[schemars(description = "Font size", range(min = 16, max = 140), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#ffffff"]
    #[schemars(
        description = "Text colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#facc15"]
    #[schemars(
        description = "Highlight colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub highlight_color: String,
    #[default = "rgba(0,0,0,0.35)"]
    #[schemars(
        description = "Panel colour",
        extend(
            "x-help" = "The box behind the words. Lower its opacity to let the video show through.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub panel: String,
    #[default = 800]
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

impl crate::params::Params for Captions {}

pub fn definition() -> ElementMeta {
    meta::<Captions>(element(
        ElementKind::Component,
        CAPTIONS_ELEMENT,
        "Captions",
        "Captions shown over the video.",
        ElementCategory::Overlay,
        150,
    ))
}
