use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AuroraBg {
    #[default = "#050505"]
    #[schemars(
        description = "Background color",
        extend(
            "x-help" = "Plate behind the curtains.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background_color: String,
    #[default(vec!["#e4ac59".into(), "#e07a5f".into(), "#c2557a".into()])]
    #[schemars(
        description = "Colors",
        length(min = 1),
        extend("x-help" = "Curtain colours, cycled.")
    )]
    pub colors: Vec<Colour>,
    #[default = 4]
    #[schemars(
        description = "Ribbon count",
        range(min = 1, max = 12),
        extend("x-step" = 1, "x-help" = "How many curtains.")
    )]
    pub ribbon_count: u32,
    #[default = 9.0]
    #[schemars(
        description = "Amplitude",
        extend("x-help" = "Vertical travel of a fold, in percent of the frame.")
    )]
    pub amplitude: f64,
    #[default = 15.0]
    #[schemars(
        description = "Thickness",
        extend("x-help" = "Curtain height at its thickest point, in percent.")
    )]
    pub thickness: f64,
    #[default = 46.0]
    #[schemars(
        description = "Vertical position",
        extend("x-help" = "Where the band sits, in percent of the frame height.")
    )]
    pub center_y: f64,
    #[default = 34.0]
    #[schemars(description = "Spread")]
    pub spread: f64,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0, max = 4),
        extend("x-step" = 0.25, "x-help" = "Drift speed. 0 freezes the sky.")
    )]
    pub speed: f64,
    #[default = 14.0]
    #[schemars(
        description = "Blur",
        extend("x-help" = "Much above 20 and the folds become blobs.")
    )]
    pub blur: f64,
    #[default = 1.3]
    #[schemars(description = "Intensity", extend("x-step" = 0.1, "x-help" = "Overall brightness."))]
    pub intensity: f64,
    #[default = 0.5]
    #[schemars(
        description = "Horizon glow",
        extend(
            "x-step" = 0.01,
            "x-help" = "Ground glow under the band. 0 removes it.",
            "x-advanced" = true
        )
    )]
    pub horizon_glow: f64,
    #[default = 0.55]
    #[schemars(
        description = "Striation",
        extend(
            "x-step" = 0.01,
            "x-help" = "Vertical ray structure through the curtains, 0-1.",
            "x-advanced" = true
        )
    )]
    pub striation: f64,
    #[default = 1]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: u32,
}

pub fn definition() -> ElementMeta {
    meta::<AuroraBg>(
        element(
            ElementKind::Component,
            "aurora-bg",
            "Aurora",
            "Soft ribbons of light drifting across a dark sky.",
            ElementCategory::Background,
            120,
        )
        .look(NodeLook::of("tabler:wave-sine")),
    )
}
