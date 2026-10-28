use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleField {
    #[default = "#05070f"]
    #[schemars(
        description = "Background color",
        extend(
            "x-help" = "Plate behind the field.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background_color: String,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Color",
        extend(
            "x-help" = "Particle colour.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub color: String,
    #[default = 70]
    #[schemars(
        description = "Particles",
        range(min = 1, max = 400),
        extend("x-step" = 1, "x-help" = "How many particles.")
    )]
    pub count: u32,
    #[default = 8.0]
    #[schemars(
        description = "Direction",
        extend("x-help" = "Direction of travel in degrees. 0 drifts straight up.")
    )]
    pub angle: f64,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend(
            "x-step" = 0.25,
            "x-help" = "Traverse speed. 1 crosses the frame in about eight seconds."
        )
    )]
    pub speed: f64,
    #[default = 14.0]
    #[schemars(description = "Size")]
    pub size: f64,
    #[default = 2.0]
    #[schemars(description = "Smallest size", extend("x-advanced" = true))]
    pub min_size: f64,
    #[default = 3.0]
    #[schemars(
        description = "Depth blur",
        extend(
            "x-help" = "Defocus on the furthest particles. 0 flattens the field.",
            "x-advanced" = true
        )
    )]
    pub depth_blur: f64,
    #[default = 4.0]
    #[schemars(
        description = "Drift",
        extend("x-help" = "Sideways wander, in percent of the frame.", "x-advanced" = true)
    )]
    pub drift: f64,
    #[default = 1.0]
    #[schemars(description = "Intensity", extend("x-help" = "Overall brightness."))]
    pub intensity: f64,
    #[default = 1.6]
    #[schemars(
        description = "Glow",
        extend(
            "x-step" = 0.1,
            "x-help" = "Halo per particle, as a multiple of its size. 0 draws hard dots."
        )
    )]
    pub glow: f64,
    #[default = 1]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: u32,
}

pub fn definition() -> ElementMeta {
    meta::<ParticleField>(
        element(
            ElementKind::Component,
            "particle-field",
            "Particle field",
            "Glowing particles drifting at different depths.",
            ElementCategory::Background,
            120,
        )
        .look(NodeLook::of("lucide:sparkles")),
    )
}
