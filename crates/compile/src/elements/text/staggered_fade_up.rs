use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct StaggeredFadeUp {
    #[setting("Text", default = "Ship your demo in minutes")]
    pub text: String,
    #[setting(
        "Stagger",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub stagger_delay: f64,
    #[setting(
        "Distance",
        default = 20,
        range(min = 0, max = 120),
        step = 1,
        advanced
    )]
    pub distance: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
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
        "Word fade time",
        default = 0.4,
        range(min = 0.1, max = 1.5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub word_duration: f64,
    #[setting("Letter spacing", default = -0.03, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<StaggeredFadeUp>(
        element(
            ElementKind::Component,
            "staggered-fade-up",
            "Staggered fade up",
            "Words fade in and slide up one after another with a configurable delay.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Rise)),
    )
}
