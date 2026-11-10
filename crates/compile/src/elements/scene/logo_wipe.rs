use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoWipe {
    #[setting("Name", default = "Northwind")]
    pub wordmark: String,
    #[setting(
        "Logo",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*",
        help = "The mark that spins in front of the name."
    )]
    pub logo_src: String,
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
        "Name end colour",
        widget = "color",
        zero_label = "Base color",
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
        "Wipe time",
        default = 0.4667,
        range(min = 0.1667, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_duration: f64,
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
        help = "Set it to transparent to lay the logo over other layers."
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
    meta::<LogoWipe>(
        element(
            ElementKind::Component,
            "logo-wipe",
            "Logo wipe",
            "Your name wipes in behind a spinning mark, then the mark rushes at the camera and floods the frame.",
            ElementCategory::Scene,
            56,
        )
        .authored_clock()
        .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
        .look(NodeLook::of("lucide:pen-tool")),
    )
}
