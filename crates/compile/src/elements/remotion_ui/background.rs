use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AuroraBg {
    #[default = "#050505"]
    #[schemars(description = "Background color", extend("x-help" = "Plate behind the curtains.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[default(vec!["#e4ac59".into(), "#e07a5f".into(), "#c2557a".into()])]
    #[schemars(description = "Colors", length(min = 1), extend("x-help" = "Curtain colours, cycled."))]
    pub colors: Vec<Colour>,
    #[default = 4.0]
    #[schemars(description = "Ribbon count", range(min = 1, max = 12), extend("x-step" = 1, "x-help" = "How many curtains."))]
    pub ribbon_count: f64,
    #[default = 9.0]
    #[schemars(description = "Amplitude", extend("x-help" = "Vertical travel of a fold, in percent of the frame."))]
    pub amplitude: f64,
    #[default = 15.0]
    #[schemars(description = "Thickness", extend("x-help" = "Curtain height at its thickest point, in percent."))]
    pub thickness: f64,
    #[default = 46.0]
    #[schemars(description = "Vertical position", extend("x-help" = "Where the band sits, in percent of the frame height."))]
    pub center_y: f64,
    #[default = 34.0]
    #[schemars(description = "Spread")]
    pub spread: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0, max = 4), extend("x-step" = 0.25, "x-help" = "Drift speed. 0 freezes the sky."))]
    pub speed: f64,
    #[default = 14.0]
    #[schemars(description = "Blur", extend("x-help" = "Much above 20 and the folds become blobs."))]
    pub blur: f64,
    #[default = 1.3]
    #[schemars(description = "Intensity", extend("x-step" = 0.1, "x-help" = "Overall brightness."))]
    pub intensity: f64,
    #[default = 0.5]
    #[schemars(description = "Horizon glow", extend("x-step" = 0.01, "x-help" = "Ground glow under the band. 0 removes it.", "x-advanced" = true))]
    pub horizon_glow: f64,
    #[default = 0.55]
    #[schemars(description = "Striation", extend("x-step" = 0.01, "x-help" = "Vertical ray structure through the curtains, 0-1.", "x-advanced" = true))]
    pub striation: f64,
    #[default = 1.0]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleField {
    #[default = "#05070f"]
    #[schemars(description = "Background color", extend("x-help" = "Plate behind the field.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[default = "#e8b86d"]
    #[schemars(description = "Color", extend("x-help" = "Particle colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = 70.0]
    #[schemars(description = "Particles", range(min = 1, max = 400), extend("x-step" = 1, "x-help" = "How many particles."))]
    pub count: f64,
    #[default = 8.0]
    #[schemars(description = "Direction", extend("x-help" = "Direction of travel in degrees. 0 drifts straight up."))]
    pub angle: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Traverse speed. 1 crosses the frame in about eight seconds."))]
    pub speed: f64,
    #[default = 14.0]
    #[schemars(description = "Size")]
    pub size: f64,
    #[default = 2.0]
    #[schemars(description = "Smallest size", extend("x-advanced" = true))]
    pub min_size: f64,
    #[default = 3.0]
    #[schemars(description = "Depth blur", extend("x-help" = "Defocus on the furthest particles. 0 flattens the field.", "x-advanced" = true))]
    pub depth_blur: f64,
    #[default = 4.0]
    #[schemars(description = "Drift", extend("x-help" = "Sideways wander, in percent of the frame.", "x-advanced" = true))]
    pub drift: f64,
    #[default = 1.0]
    #[schemars(description = "Intensity", extend("x-help" = "Overall brightness."))]
    pub intensity: f64,
    #[default = 1.6]
    #[schemars(description = "Glow", extend("x-step" = 0.1, "x-help" = "Halo per particle, as a multiple of its size. 0 draws hard dots."))]
    pub glow: f64,
    #[default = 1.0]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<AuroraBg>(
            element(
                ElementKind::Component,
                "aurora-bg",
                "Aurora",
                "Soft ribbons of light drifting across a dark sky.",
                C::Background,
                120,
            )
            .source(
                "RemotionUI",
                "https://remotionui.com/docs/components/aurora-bg",
                "MIT",
            )
            .look(NodeLook::of("tabler:wave-sine")),
        ),
        meta::<ParticleField>(
            element(
                ElementKind::Component,
                "particle-field",
                "Particle field",
                "Glowing particles drifting at different depths.",
                C::Background,
                120,
            )
            .source(
                "RemotionUI",
                "https://remotionui.com/docs/components/particle-field",
                "MIT",
            )
            .look(NodeLook::of("lucide:sparkles")),
        ),
    ]
}
