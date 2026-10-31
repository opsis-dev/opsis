use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MarkLook {
    #[label("Underline")]
    Underline,
    #[label("Circle")]
    Circle,
    #[label("Box")]
    Box,
    #[label("Marker")]
    Marker,
    #[label("Crossed out")]
    CrossedOff,
}

#[choice(export)]
pub enum TitleAlign {
    Center,
    Left,
}

#[choice(export)]
pub enum Entrance {
    Rise,
    Fade,
    Blur,
    Scale,
    None,
}

#[settings(export)]
pub struct Title {
    #[setting("Eyebrow")]
    pub eyebrow: String,
    #[setting("Title", default = "Sign up in 10 seconds")]
    pub title: String,
    #[setting("Subtitle")]
    pub subtitle: String,
    #[setting(
        "Mark a word",
        help = "Words from the title to draw a hand-drawn mark on, after the title comes in. Leave empty for none."
    )]
    pub mark: String,
    #[setting(
        "Mark",
        default = MarkLook::Underline
    )]
    pub mark_look: MarkLook,
    #[setting("Mark colour", default = "#facc15", check = colour)]
    pub mark_color: String,
    #[setting("Alignment", default = TitleAlign::Center)]
    pub align: TitleAlign,
    #[setting("Entrance", default = Entrance::Rise)]
    pub entrance: Entrance,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting(
        "Background (colour, gradient or 'transparent')",
        default = "transparent"
    )]
    pub background: String,
    #[setting("Font size", default = 112, range(min = 24, max = 260), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = 760, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting(
        "Side margin",
        default = 9.0,
        range(min = 0, max = 30),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Line gap",
        default = 0.22,
        range(min = 0, max = 1),
        unit = "em",
        advanced
    )]
    pub line_gap: f64,
    #[setting(
        "Eyebrow size",
        default = 0.26,
        range(min = 0.1, max = 0.8),
        unit = "× title",
        advanced
    )]
    pub eyebrow_size: f64,
    #[setting("Uppercase eyebrow", default = true, advanced)]
    pub eyebrow_uppercase: bool,
    #[setting(
        "Subtitle size",
        default = 0.34,
        range(min = 0.15, max = 0.8),
        unit = "× title",
        advanced
    )]
    pub subtitle_size: f64,
    #[setting("Letter spacing", default = -0.035, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting(
        "Line stagger",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "×",
        advanced
    )]
    pub line_stagger: f64,
    #[setting(
        "Mark thickness",
        default = 0.045,
        range(min = 0.01, max = 0.15),
        unit = "× font size",
        advanced
    )]
    pub mark_thickness: f64,
    #[setting(
        "Marker strength",
        default = 55.0,
        range(min = 10, max = 100),
        unit = "%",
        advanced
    )]
    pub marker_opacity: f64,
    #[setting(
        "Mark variation",
        default = 1,
        range(min = 1, max = 100),
        step = 1,
        advanced
    )]
    pub mark_seed: u32,
    #[setting(
        "Mark draw time",
        default = 0.45,
        range(min = 0.1, max = 2),
        unit = "s",
        advanced
    )]
    pub mark_draw_time: f64,
    #[setting(
        "Fade out",
        default = 0.3333,
        range(min = 0, max = 2),
        unit = "s",
        advanced
    )]
    pub fade_out: f64,
    #[setting(
        "Subtitle colour",
        widget = "color",
        zero_label = "Text colour, faded",
        check = colour_or_empty,
        advanced
    )]
    pub subtitle_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Title>(
        element(
            ElementKind::Component,
            "title",
            "Title card",
            "A full-screen title with an optional eyebrow and subtitle.",
            ElementCategory::Text,
            75,
        )
        .look(NodeLook::of("lucide:letter-h")),
    )
}
