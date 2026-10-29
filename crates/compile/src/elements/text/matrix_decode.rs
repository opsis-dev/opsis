use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct MatrixDecode {
    #[setting("Text", default = "DECRYPTED")]
    pub text: String,
    #[setting(
        "Scramble characters",
        default = "!@#$%^&*()_+-=<>?/\\|",
        zero_label = "Built-in",
        help = "Characters the text flickers through before it settles."
    )]
    pub charset: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#22c55e", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Reveal time",
        default = 2.0,
        range(min = 0.3333, max = 8),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub reveal_duration: f64,
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
