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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct RgbGlitchText {
    #[default = "GLITCH"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 96]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default(FontWeight::W700)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 0.6667]
    #[schemars(
        description = "Glitch at",
        range(min = 0, max = 4),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub glitch_at: f64,
    #[default = 0.2667]
    #[schemars(
        description = "Glitch length",
        range(min = 0.0333, max = 2),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub glitch_duration: f64,
    #[default = 6]
    #[schemars(description = "Intensity (px)", range(min = 0, max = 30), extend("x-step" = 1))]
    pub intensity: u32,
    #[default = "glitch"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default(vec!["#ff0040".into(), "#00ff80".into(), "#0080ff".into()])]
    #[schemars(
        description = "Glitch colours",
        length(min = 3, max = 3),
        extend("x-help" = "The three copies that split off while the text glitches.")
    )]
    pub glitch_colors: Vec<Colour>,
    #[default(BlendMode::Multiply)]
    #[schemars(
        description = "Glitch blend",
        extend(
            "x-help" = "How the split copies mix with what is behind them.",
            "x-labels" = { "multiply": "On a light background", "screen": "On a dark background", "normal": "On any background" }
        )
    )]
    pub blend: BlendMode,
    #[default = "#fafafa"]
    #[schemars(
        description = "Background",
        extend(
            "x-help" = "Set it to transparent to lay the text over other layers.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
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
