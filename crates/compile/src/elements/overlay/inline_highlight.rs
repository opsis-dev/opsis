use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
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
    #[setting(
        "Change starts at",
        default = 0.2,
        range(min = 0, max = 0.9),
        unit = "×",
        advanced
    )]
    pub change_start: f64,
    #[setting(
        "Change ends at",
        default = 0.7,
        range(min = 0.1, max = 1),
        unit = "×",
        advanced
    )]
    pub change_end: f64,
    #[setting("Letter spacing", default = -0.03, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
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
