use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SearchReveal {
    #[setting("Search text", default = "acme")]
    pub text: String,
    #[setting("Field width", default = 464, range(min = 320, max = 640), step = 8)]
    pub field_width: u32,
    #[setting("Font size", default = 34, range(min = 18, max = 54), step = 1)]
    pub font_size: u32,
    #[setting(
        "Time per letter",
        default = 0.1333,
        range(min = 0.0333, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub frames_per_character: f64,
    #[setting("Graphic panel", default = true)]
    pub show_panel: bool,
    #[setting("Construction guides", default = true)]
    pub show_guides: bool,
    #[setting("Text and markers", default = "#26232b", check = colour)]
    pub color: String,
    #[setting("Search field", default = "#ffffff", check = colour, advanced)]
    pub field_color: String,
    #[setting("Circles and search icon", default = "#cbc5cf", check = colour, advanced)]
    pub ring_color: String,
    #[setting("Panel color", default = "#a800b7", check = colour)]
    pub panel_color: String,
    #[setting("Graphic accents", default = "#e8f99a", check = colour)]
    pub accent_color: String,
    #[setting("Background", default = "#f5f1f5", check = colour)]
    pub background_color: String,
    #[setting("Static final pose", advanced)]
    pub reduced_motion: bool,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Font weight", default = FontWeight::W400, advanced)]
    pub weight: FontWeight,
}

pub fn definition() -> ElementMeta {
    meta::<SearchReveal>(
            element(
                ElementKind::Component,
                "search-reveal",
                "Search reveal",
                "Expanding construction circles snap into a search pill, type editable text, and reveal an abstract graphic panel.",
                ElementCategory::Ui,
                108,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:search")),
        )
}
