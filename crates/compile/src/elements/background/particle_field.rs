use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleField {
    #[setting(
        "Background color",
        default = "#05070f",
        help = "Plate behind the field.",
        check = colour
    )]
    pub background_color: String,
    #[setting("Color", default = "#e8b86d", help = "Particle colour.", check = colour)]
    pub color: String,
    #[setting(
        "Particles",
        default = 70,
        range(min = 1, max = 400),
        step = 1,
        help = "How many particles."
    )]
    pub count: u32,
    #[setting(
        "Direction",
        default = 8.0,
        help = "Direction of travel in degrees. 0 drifts straight up."
    )]
    pub angle: f64,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Traverse speed. 1 crosses the frame in about eight seconds."
    )]
    pub speed: f64,
    #[setting("Size", default = 14.0)]
    pub size: f64,
    #[setting("Smallest size", default = 2.0, advanced)]
    pub min_size: f64,
    #[setting(
        "Depth blur",
        default = 3.0,
        help = "Defocus on the furthest particles. 0 flattens the field.",
        advanced
    )]
    pub depth_blur: f64,
    #[setting(
        "Drift",
        default = 4.0,
        help = "Sideways wander, in percent of the frame.",
        advanced
    )]
    pub drift: f64,
    #[setting("Intensity", default = 1.0, help = "Overall brightness.")]
    pub intensity: f64,
    #[setting(
        "Glow",
        default = 1.6,
        step = 0.1,
        help = "Halo per particle, as a multiple of its size. 0 draws hard dots."
    )]
    pub glow: f64,
    #[setting("Variation", default = 1, advanced)]
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
