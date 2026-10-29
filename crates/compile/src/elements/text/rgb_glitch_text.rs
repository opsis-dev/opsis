use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, FontWeight};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    Multiply,
    Screen,
    Normal,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct RgbGlitchText {
    #[setting("Text", default = "GLITCH")]
    pub text: String,
    #[setting("Font size", default = 96, range(min = 12, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Glitch at",
        default = 0.6667,
        range(min = 0, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub glitch_at: f64,
    #[setting(
        "Glitch length",
        default = 0.2667,
        range(min = 0.0333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub glitch_duration: f64,
    #[setting("Intensity (px)", default = 6, range(min = 0, max = 30), step = 1)]
    pub intensity: u32,
    #[setting("Seed", default = "glitch")]
    pub seed: String,
    #[setting(
        "Glitch colours",
        default = vec!["#ff0040".into(), "#00ff80".into(), "#0080ff".into()],
        length(min = 3, max = 3),
        help = "The three copies that split off while the text glitches."
    )]
    pub glitch_colors: Vec<Colour>,
    #[setting(
        "Glitch blend",
        default = BlendMode::Multiply,
        help = "How the split copies mix with what is behind them.",
        labels = { "multiply": "On a light background", "screen": "On a dark background", "normal": "On any background" }
    )]
    pub blend: BlendMode,
    #[setting(
        "Background",
        default = "#fafafa",
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
}

pub fn definition() -> ElementMeta {
    meta::<RgbGlitchText>(
        element(
            ElementKind::Component,
            "rgb-glitch-text",
            "RGB glitch text",
            "Three RGB-offset text copies jitter for a brief glitch window.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Glitch)),
    )
}
