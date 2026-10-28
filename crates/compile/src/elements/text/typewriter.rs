use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct Typewriter {
    #[default = "console.log('hello, world')"]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[default = true]
    #[schemars(description = "Show cursor")]
    pub cursor: bool,
    #[default = 22]
    #[schemars(description = "Chars / sec", range(min = 4, max = 60), extend("x-step" = 1))]
    pub chars_per_second: u32,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#171717"]
    #[schemars(
        description = "Cursor color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub cursor_color: String,
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
