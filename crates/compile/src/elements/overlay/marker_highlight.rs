use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MarkerHighlight {
    #[setting("Before", default = "Made for ")]
    pub before: String,
    #[setting("Highlight", default = "builders")]
    pub highlight: String,
    #[setting("After", default = ".")]
    pub after: String,
    #[setting("Marker color", default = "#facc15", check = colour)]
    pub marker_color: String,
    #[setting("Base color", default = "#171717", check = colour)]
    pub base_color: String,
    #[setting("Highlighted text color", default = "#171717", check = colour, advanced)]
    pub highlighted_text_color: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = FontWeight::W600, advanced)]
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
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Marker starts",
        default = 0.5,
        range(min = 0, max = 5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub marker_at: f64,
    #[setting("Marker height", default = 1.0, range(min = 0.1, max = 1), advanced)]
    pub marker_height: f64,
    #[setting("Centre across", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Centre down", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
}

pub fn definition() -> ElementMeta {
    meta::<MarkerHighlight>(
        element(
            ElementKind::Component,
            "marker-highlight",
            "Marker highlight",
            "Colored marker block draws behind a phrase as the text color shifts.",
            ElementCategory::Overlay,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:highlighter")),
    )
}
