use opsis_macros::settings;
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Captions {
    #[setting("Style", default = CaptionStyle::WordByWord)]
    pub style: CaptionStyle,
    #[setting("Position", default = CaptionPosition::Bottom)]
    pub position: CaptionPosition,
    #[setting("Font size", default = 56, range(min = 16, max = 140), step = 1)]
    pub font_size: u32,
    #[setting("Text colour", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Highlight colour", default = "#facc15", check = colour)]
    pub highlight_color: String,
    #[setting(
        "Panel colour",
        default = "rgba(0,0,0,0.35)",
        help = "The box behind the words. Lower its opacity to let the video show through.",
        check = colour
    )]
    pub panel: String,
    #[setting("Font weight", default = 800, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Words per line", default = 6, range(min = 1, max = 20), step = 1)]
    pub words_per_line: u32,
    #[setting(
        "Stay after the last word",
        default = 250,
        range(min = 0, max = 2000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub linger_ms: u32,
    #[setting(
        "Distance from the edge",
        default = 90,
        range(min = 0, max = 400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub edge_offset: u32,
    #[setting(
        "Side margin",
        default = 10.0,
        range(min = 0, max = 40),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Panel padding",
        default = 22,
        range(min = 0, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_padding: u32,
    #[setting(
        "Panel corners",
        default = 16,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_radius: u32,
    #[setting("Text shadow", default = true, advanced)]
    pub text_shadow: bool,
    #[setting(
        "Unspoken words",
        default = 55.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub unspoken_opacity: f64,
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
