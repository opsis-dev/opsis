use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum HighlightPreset {
    LogoWipe,
    Marker,
    Color,
    Underline,
    Strikethrough,
    Shimmer,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TextHighlight {
    #[setting(
        "Before",
        default = "Powered by ",
        show_if = { "preset": ["marker", "color", "underline", "strikethrough", "shimmer"] }
    )]
    pub before: String,
    #[setting("Highlight / wordmark", default = "Northwind")]
    pub highlight: String,
    #[setting(
        "After",
        show_if = { "preset": ["marker", "color", "underline", "strikethrough", "shimmer"] }
    )]
    pub after: String,
    #[setting("Preset", default = HighlightPreset::LogoWipe)]
    pub preset: HighlightPreset,
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
    #[setting("Replace with (strikethrough)", default = "Acme")]
    pub replace_with: String,
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
        "Line thickness",
        default = 4,
        range(min = 1, max = 20),
        step = 1,
        advanced
    )]
    pub thickness: u32,
    #[setting(
        "Shine color (shimmer)",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        advanced
    )]
    pub shine_color: String,
    #[setting(
        "Logo",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*",
        help = "The mark the logo wipe uses."
    )]
    pub logo_src: String,
    #[setting(
        "Mark size (× font)",
        default = 1.05,
        range(min = 0.4, max = 3),
        step = 0.05,
        advanced
    )]
    pub logo_scale: f64,
    #[setting(
        "Mark spin (turns)",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.25,
        advanced
    )]
    pub spin_turns: f64,
    #[setting(
        "Lockup hold",
        default = 0.4,
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub hold_duration: f64,
    #[setting(
        "Rush time",
        default = 0.8,
        range(min = 0.2, max = 2.6667),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub rush_duration: f64,
    #[setting(
        "Rush scale (× mark)",
        default = 42,
        range(min = 2, max = 80),
        step = 1,
        advanced
    )]
    pub rush_scale: u32,
    #[setting("Swing across (× width)", default = 0.55, range(min = -0.8, max = 0.8), step = 0.05, advanced)]
    pub rush_drift: f64,
    #[setting(
        "Swing arc (× width)",
        default = 0.25,
        range(min = 0, max = 0.33),
        step = 0.01,
        advanced
    )]
    pub swing_out: f64,
    #[setting(
        "Ink color (end of rush)",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub cover_color: String,
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
    #[setting(
        "Marker strength",
        default = 16.0,
        range(min = 5, max = 100),
        unit = "%",
        advanced
    )]
    pub marker_opacity: f64,
    #[setting(
        "Shine width",
        default = 40.0,
        range(min = 10, max = 80),
        unit = "%",
        advanced
    )]
    pub shine_width: f64,
    #[setting(
        "Gap after mark",
        default = 0.3,
        range(min = 0, max = 1.5),
        unit = "× font size",
        advanced
    )]
    pub mark_gap: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TextHighlight>(
            element(
                ElementKind::Component,
                "text-highlight",
                "Text highlight",
                "Text highlight animation: marker highlighter, underline, strikethrough and shimmer on a word. Animated emphasis on one span inside a static sentence — marker sweep, color shift, underline, strikethrough-replace, or shimmer.",
                ElementCategory::Text,
                56,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:baseline")),
        )
}
