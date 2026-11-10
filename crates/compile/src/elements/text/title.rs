use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum TitleAlign {
    Center,
    Left,
}

#[choice(export)]
pub enum Entrance {
    #[label("Rise")]
    Rise,
    #[label("Fade")]
    Fade,
    #[label("Blur")]
    Blur,
    #[label("Scale")]
    Scale,
    #[label("None")]
    None,
    #[label("Soft blur in")]
    SoftBlur,
    #[label("Typewriter")]
    Typewriter,
    #[label("Staggered fade up")]
    FadeUp,
    #[label("Word push")]
    WordPush,
    #[label("Mask reveal up")]
    MaskReveal,
    #[label("Tracking in")]
    Tracking,
    #[label("Colour glitch")]
    Glitch,
    #[label("Decode")]
    Decode,
    #[label("Line by line slide")]
    LineByLine,
    #[label("Animated headline")]
    Headline,
    #[label("Handwriting")]
    Handwriting,
    #[label("Neon sign")]
    Neon,
    #[label("Gradient scale cut")]
    ScaleCut,
}

#[settings(export)]
pub struct Title {
    #[setting("Eyebrow")]
    pub eyebrow: String,
    #[setting("Title", default = "Sign up in 10 seconds")]
    pub title: String,
    #[setting("Subtitle")]
    pub subtitle: String,
    #[setting("Alignment", default = TitleAlign::Center)]
    pub align: TitleAlign,
    #[setting(
        "Entrance",
        default = Entrance::Rise,
        help = "Rise, Fade, Blur and Scale bring every line in. The others are Animated text's animations: the title plays it and the other lines fade in."
    )]
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
