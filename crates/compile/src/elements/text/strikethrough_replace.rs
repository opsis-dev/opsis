use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct StrikethroughReplace {
    #[setting("From", default = "$49/mo")]
    pub from: String,
    #[setting("To", default = "Free")]
    pub to: String,
    #[setting("Line color", default = "#ff5e3a", check = colour)]
    pub line_color: String,
    #[setting("Font size", default = 96, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
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
        "Strike ends at",
        default = 0.4,
        range(min = 0.1, max = 0.9),
        unit = "× length",
        advanced
    )]
    pub strike_end: f64,
    #[setting(
        "Swap time",
        default = 0.2,
        range(min = 0.05, max = 0.6),
        unit = "× length",
        advanced
    )]
    pub swap_length: f64,
    #[setting("Letter spacing", default = -0.03, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Line thickness",
        range(min = 1, max = 40),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        "type" = "integer",
        advanced
    )]
    pub thickness: Option<u32>,
    #[setting(
        "New text colour",
        widget = "color",
        zero_label = "Same as the text",
        check = colour_or_empty,
        advanced
    )]
    pub to_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<StrikethroughReplace>(
        element(
            ElementKind::Component,
            "strikethrough-replace",
            "Strikethrough replace",
            "Draws a strike line across old text then reveals new text in its place.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:strikethrough").morphs("lucide:type")),
    )
}
