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
pub struct InlineHighlight {
    #[setting("Before", default = "Ship faster with ")]
    pub before: String,
    #[setting("Highlight", default = "Acme")]
    pub highlight: String,
    #[setting("After", default = ".")]
    pub after: String,
    #[setting("Base color", default = "#171717", check = colour)]
    pub base_color: String,
    #[setting("Highlight color", default = "#ff5e3a", check = colour)]
    pub highlight_color: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
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
}

pub fn definition() -> ElementMeta {
    meta::<InlineHighlight>(
        element(
            ElementKind::Component,
            "inline-highlight",
            "Inline highlight",
            "Animates one word inside a sentence from a base color to a brand color.",
            ElementCategory::Overlay,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:square-dashed-text")),
    )
}
