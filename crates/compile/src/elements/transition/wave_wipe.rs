use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct WaveWipe {
    #[setting("Intensity", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting("Softness", default = 0.7, range(min = 0, max = 1), step = 0.05)]
    pub softness: f64,
    #[setting("Noise", default = 0.4, range(min = 0, max = 1), step = 0.05)]
    pub noise: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()],
        length(min = 2, max = 7)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Zoom",
        default = 1.16,
        range(min = 0.5, max = 4),
        step = 0.01,
        advanced
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
