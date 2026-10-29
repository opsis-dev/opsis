use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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
    #[setting("Symbol path")]
    pub symbol_path: String,
    #[setting(
        "Symbol colours",
        length(min = 0, max = 8),
        help = "Leave empty for the built-in colours."
    )]
    pub symbol_colors: Vec<Colour>,
    #[setting("Symbol size", default = 1.0, range(min = 0, max = 3), step = 0.05)]
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
        step = 0.005
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
