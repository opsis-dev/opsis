use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AnnounceTitle {
    #[setting("Eyebrow", default = "Introducing")]
    pub eyebrow: String,
    #[setting("Title", default = "Acme 2.0")]
    pub title: String,
    #[setting(
        "Tagline",
        default = "Everything your team needs to ship, in one place."
    )]
    pub tagline: String,
    #[setting("Symbol path", advanced)]
    pub symbol_path: String,
    #[setting(
        "Symbol colours",
        length(min = 0, max = 8),
        help = "Leave empty for the built-in colours.",
        advanced
    )]
    pub symbol_colors: Vec<Colour>,
    #[setting(
        "Symbol size",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.05,
        advanced
    )]
    pub symbol_scale: f64,
    #[setting("Colour field", default = "#5600f5", check = colour)]
    pub field_color: String,
    #[setting("Title ink", default = "#4800c9", check = colour)]
    pub title_color: String,
    #[setting("Title paper", default = "#fcfcfa", check = colour)]
    pub paper_color: String,
    #[setting("Opening ground", default = "#100022", check = colour)]
    pub void_color: String,
    #[setting("Closing ground", default = "#000028", check = colour)]
    pub night_color: String,
    #[setting("Closing glow", default = "#08ff4b", check = colour)]
    pub glow_color: String,
    #[setting(
        "Glow strength",
        default = 0.139,
        range(min = 0, max = 0.4),
        step = 0.005,
        advanced
    )]
    pub glow_strength: f64,
    #[setting("Tagline ink", default = "#f2f8ff", check = colour)]
    pub tagline_color: String,
    #[setting("Shot ink", default = "#ffffff", check = colour)]
    pub ink_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Font weight",
        default = 400,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting("Text size", default = 1.0, range(min = 0.5, max = 2), advanced)]
    pub text_scale: f64,
    #[setting("Symbol glow", default = 1.0, range(min = 0, max = 3), advanced)]
    pub symbol_glow: f64,
    #[setting(
        "Tagline tint",
        default = vec!["#8a2447".into(), "#862547".into(), "#7b2a4f".into(), "#633864".into(), "#4f467d".into(), "#3c519f".into(), "#2f5ed2".into(), "#3266e5".into()],
        length(min = 1, max = 12),
        help = "The colours the tagline shows before it settles, from left to right.",
        advanced
    )]
    pub tagline_tint: Vec<Colour>,
}

pub fn definition() -> ElementMeta {
    meta::<AnnounceTitle>(
            element(
                ElementKind::Component,
                "announce-title",
                "Announce title",
                "Launch title animation, product announcement intro with kinetic typography. A four-shot launch title: the eyebrow rushes past the camera on a receding type plane and lands on a colour field, the name assembles word by word on paper, and a macro pan across the tagline cuts wide as the line builds itself last word first while a white sweep drains it out of colour.",
                ElementCategory::Text,
                170,
            )
                .authored_clock()
                .fonts(&["Geist", "Google Sans", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:megaphone")),
        )
}
