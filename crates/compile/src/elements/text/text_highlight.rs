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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TextHighlight {
    #[default = "Powered by "]
    #[schemars(
        description = "Before",
        extend(
            "x-show-if" = { "preset": ["marker", "color", "underline", "strikethrough", "shimmer"] }
        )
    )]
    pub before: String,
    #[default = "Northwind"]
    #[schemars(description = "Highlight / wordmark")]
    pub highlight: String,
    #[schemars(
        description = "After",
        extend(
            "x-show-if" = { "preset": ["marker", "color", "underline", "strikethrough", "shimmer"] }
        )
    )]
    pub after: String,
    #[default(HighlightPreset::LogoWipe)]
    #[schemars(description = "Preset")]
    pub preset: HighlightPreset,
    #[schemars(description = "Base color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub base_color: String,
    #[default = "#266DF0"]
    #[schemars(
        description = "Accent color",
        extend("x-widget" = "color", "x-zero-label" = "Theme")
    )]
    pub accent_color: String,
    #[schemars(
        description = "Highlighted text colour",
        extend("x-widget" = "color", "x-zero-label" = "Auto")
    )]
    pub highlighted_text_color: String,
    #[default = "Acme"]
    #[schemars(description = "Replace with (strikethrough)")]
    pub replace_with: String,
    #[default = 0.2]
    #[schemars(
        description = "Start at",
        range(min = 0, max = 4),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub start_at: f64,
    #[default = 0.4667]
    #[schemars(
        description = "Draw time",
        range(min = 0.1667, max = 4),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub draw_duration: f64,
    #[default = 4]
    #[schemars(description = "Line thickness", range(min = 1, max = 20), extend("x-step" = 1))]
    pub thickness: u32,
    #[default = "#FAFAFA"]
    #[schemars(
        description = "Shine color (shimmer)",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub shine_color: String,
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(
        description = "Logo",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-help" = "The mark the logo wipe uses."
        )
    )]
    pub logo_src: String,
    #[default = 1.05]
    #[schemars(
        description = "Mark size (× font)",
        range(min = 0.4, max = 3),
        extend("x-step" = 0.05)
    )]
    pub logo_scale: f64,
    #[default = 1.0]
    #[schemars(description = "Mark spin (turns)", range(min = 0, max = 4), extend("x-step" = 0.25))]
    pub spin_turns: f64,
    #[default = 0.4]
    #[schemars(
        description = "Lockup hold",
        range(min = 0, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub hold_duration: f64,
    #[default = 0.8]
    #[schemars(
        description = "Rush time",
        range(min = 0.2, max = 2.6667),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub rush_duration: f64,
    #[default = 42]
    #[schemars(description = "Rush scale (× mark)", range(min = 2, max = 80), extend("x-step" = 1))]
    pub rush_scale: u32,
    #[default = 0.55]
    #[schemars(
        description = "Swing across (× width)",
        range(min = -0.8, max = 0.8),
        extend("x-step" = 0.05)
    )]
    pub rush_drift: f64,
    #[default = 0.25]
    #[schemars(
        description = "Swing arc (× width)",
        range(min = 0, max = 0.33),
        extend("x-step" = 0.01)
    )]
    pub swing_out: f64,
    #[default = "#266DF0"]
    #[schemars(
        description = "Ink color (end of rush)",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub cover_color: String,
    #[default = 56]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Set it to transparent to lay the text over other layers."
        )
    )]
    pub background: String,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
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
