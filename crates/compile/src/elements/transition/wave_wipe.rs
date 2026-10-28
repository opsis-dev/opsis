use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct WaveWipe {
    #[default = 0.2]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.7]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.4]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = "#141318"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-advanced" = true)
    )]
    pub speed: f64,
    #[default = 1.16]
    #[schemars(
        description = "Zoom",
        range(min = 0.5, max = 4),
        extend("x-step" = 0.01, "x-advanced" = true)
    )]
    pub zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<WaveWipe>(
            element(
                ElementKind::Presentation,
                "wave-wipe",
                "Wave wipe",
                "A grain-gradient wave that sweeps up across the frame, covering the outgoing scene and revealing the next one from below.",
                ElementCategory::Transition,
                116,
            )
                .look(NodeLook::of("lucide:waves-horizontal").moves(NodeMotion::WipeReveal)),
        )
}
