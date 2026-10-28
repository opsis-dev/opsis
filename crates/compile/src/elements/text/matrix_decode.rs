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
pub struct MatrixDecode {
    #[default = "DECRYPTED"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = "!@#$%^&*()_+-=<>?/\\|"]
    #[schemars(
        description = "Scramble characters",
        extend(
            "x-zero-label" = "Built-in",
            "x-help" = "Characters the text flickers through before it settles."
        )
    )]
    pub charset: String,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#22c55e"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 2.0]
    #[schemars(
        description = "Reveal time",
        range(min = 0.3333, max = 8),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub reveal_duration: f64,
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
    meta::<MatrixDecode>(
        element(
            ElementKind::Component,
            "matrix-decode",
            "Matrix decode",
            "Random scramble resolves left-to-right into the target text.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:binary").morphs("lucide:type")),
    )
}
