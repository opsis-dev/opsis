use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum HighlightPreset {
    Marker,
    Color,
    Underline,
    Strikethrough,
    Shimmer,
}

#[settings(export)]
pub struct TextHighlight {
    #[setting("Preset", default = HighlightPreset::Marker)]
    pub preset: HighlightPreset,
    #[setting("Before", default = "Made for ")]
    pub before: String,
    #[setting("Highlight", default = "builders")]
    pub highlight: String,
    #[setting("After", default = ".")]
    pub after: String,
    #[setting(
        "Replace with",
        default = "Free",
        show_if = { "preset": ["strikethrough"] }
    )]
    pub replace_with: String,
    #[setting("Base color", widget = "color", zero_label = "Theme")]
    pub base_color: String,
    #[setting(
        "Accent color",
        default = "#266DF0",
        widget = "color",
        zero_label = "Theme"
    )]
    pub accent_color: String,
    #[setting(
        "Highlighted text colour",
        widget = "color",
        zero_label = "Auto",
        advanced
    )]
    pub highlighted_text_color: String,
    #[setting(
        "Start at",
        default = 0.2,
        range(min = 0, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub start_at: f64,
    #[setting(
        "Draw time",
        default = 0.4667,
        range(min = 0.1667, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_duration: f64,
    #[setting(
        "Swap time",
        default = 0.2333,
        range(min = 0.0333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        show_if = { "preset": ["strikethrough"] },
        advanced
    )]
    pub swap_length: f64,
    #[setting(
        "Line thickness",
        default = 4,
        range(min = 1, max = 20),
        step = 1,
        show_if = { "preset": ["underline", "strikethrough"] },
        advanced
    )]
    pub thickness: u32,
    #[setting(
        "Marker strength",
        default = 16.0,
        range(min = 5, max = 100),
        unit = "%",
        show_if = { "preset": ["marker"] },
        advanced
    )]
    pub marker_opacity: f64,
    #[setting(
        "Marker height",
        default = 1.0,
        range(min = 0.2, max = 1.4),
        step = 0.05,
        unit = "× line",
        show_if = { "preset": ["marker"] },
        advanced
    )]
    pub marker_height: f64,
    #[setting(
        "Shine color",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub shine_color: String,
    #[setting(
        "Shine width",
        default = 40.0,
        range(min = 10, max = 80),
        unit = "%",
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub shine_width: f64,
    #[setting(
        "Sweeps",
        default = 1,
        range(min = 1, max = 6),
        step = 1,
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub sweeps: u32,
    #[setting(
        "Shine angle",
        default = 110.0,
        range(min = 0, max = 180),
        step = 5,
        unit = "°",
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub band_angle: f64,
    #[setting("Font size", default = 56, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Set it to transparent to lay the text over other layers."
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
        "Centre across",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub x: f64,
    #[setting(
        "Centre down",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub y: f64,
    #[setting("Letter spacing", default = -0.02, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 1.25, range(min = 0.9, max = 2), advanced)]
    pub line_height: f64,
    #[setting(
        "Text width",
        default = 80.0,
        range(min = 30, max = 100),
        unit = "%",
        advanced
    )]
    pub text_width: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TextHighlight>(
            element(
                ElementKind::Component,
                "text-highlight",
                "Text highlight",
                "Brings attention to one word in a sentence: a marker sweeps behind it, its colour changes, a line draws under it, it is struck out and replaced, or a shine passes over it.",
                ElementCategory::Text,
                56,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:baseline")),
        )
}
