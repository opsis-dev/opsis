use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GradientScaleCutText {
    #[setting("Text", default = "Introducing")]
    pub text: String,
    #[setting(
        "Giant font size",
        default = 520,
        range(min = 160, max = 720),
        step = 4
    )]
    pub giant_font_size: u32,
    #[setting(
        "Compact font size",
        default = 132,
        range(min = 40, max = 240),
        step = 2
    )]
    pub compact_font_size: u32,
    #[setting(
        "Final horizontal anchor offset",
        default = -16,
        range(min = -400, max = 400),
        step = 4,
        advanced
    )]
    pub anchor_offset_x: i32,
    #[setting(
        "Giant phase left travel",
        default = 400,
        range(min = 0, max = 800),
        step = 4,
        advanced
    )]
    pub giant_travel: u32,
    #[setting(
        "Compact entrance travel",
        default = 160,
        range(min = 0, max = 400),
        step = 4,
        advanced
    )]
    pub settle_travel: u32,
    #[setting(
        "Hard cut at",
        default = 0.4333,
        range(min = 0.1333, max = 0.8),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub cut_frame: f64,
    #[setting(
        "Reveal edge softness",
        default = 14,
        range(min = 0, max = 40),
        step = 1,
        advanced
    )]
    pub reveal_softness: u32,
    #[setting(
        "Blur after the cut",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        advanced
    )]
    pub compact_blur: u32,
    #[setting("Gradient start", default = "#f04a14", check = colour)]
    pub gradient_start: String,
    #[setting("Gradient end", default = "#f3f1f1", check = colour)]
    pub gradient_end: String,
    #[setting("Unrevealed text color", default = "#170b0a", check = colour, advanced)]
    pub ghost_color: String,
    #[setting("Background color", default = "#000000", check = colour)]
    pub background_color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Reveal length",
        default = 1.1,
        range(min = 0.3333, max = 3),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub reveal_end: f64,
    #[setting(
        "Compact text width",
        default = 87.0,
        range(min = 50, max = 120),
        unit = "%",
        advanced
    )]
    pub compact_width: f64,
    #[setting(
        "Gradient spread",
        default = 26.0,
        range(min = 5, max = 100),
        unit = "%",
        advanced
    )]
    pub gradient_spread: f64,
    #[setting("Letter spacing", default = 0.0, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GradientScaleCutText>(
            element(
                ElementKind::Component,
                "gradient-scale-cut-text",
                "Gradient scale cut text",
                "An oversized gradient reveal snaps to a separate compact blurred line that settles left while the reveal continues.",
                ElementCategory::Text,
                36,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type")),
        )
}
