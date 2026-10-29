use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{FontWeight, TextAlign};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Typewriter {
    #[setting("Text", default = "console.log('hello, world')", widget = "textarea")]
    pub text: String,
    #[setting("Show cursor", default = true)]
    pub cursor: bool,
    #[setting("Chars / sec", default = 22, range(min = 4, max = 60), step = 1)]
    pub chars_per_second: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Cursor color", default = "#171717", check = colour, advanced)]
    pub cursor_color: String,
    #[setting("Font weight", default = FontWeight::W600)]
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
    #[setting(
        "Cursor width",
        default = 0.08,
        range(min = 0.02, max = 0.6),
        unit = "em",
        advanced
    )]
    pub cursor_width: f64,
    #[setting(
        "Blinks per second",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "/s",
        advanced
    )]
    pub blink_rate: f64,
    #[setting(
        "Align",
        default = TextAlign::Center,
        labels = { "left": "Left", "center": "Centre", "right": "Right" },
        advanced
    )]
    pub align: TextAlign,
    #[setting("Letter spacing", default = -0.03, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Typewriter>(
        element(
            ElementKind::Component,
            "typewriter",
            "Typewriter",
            "Types the text out letter by letter with a blinking cursor.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:text-cursor").moves(NodeMotion::Caret)),
    )
}
