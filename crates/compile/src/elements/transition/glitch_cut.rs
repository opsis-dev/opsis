use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GlitchCut {
    #[default = 1.0]
    #[schemars(description = "Intensity", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub intensity: f64,
    #[default = 24]
    #[schemars(description = "Slices", range(min = 8, max = 64), extend("x-step" = 2))]
    pub slices: u32,
    #[default = 1.0]
    #[schemars(description = "RGB split", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub rgb_split: f64,
    #[default = 0.6]
    #[schemars(description = "Block noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub block_noise: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GlitchCut>(
            element(
                ElementKind::Presentation,
                "glitch-cut",
                "Glitch cut",
                "A broadcast-glitch hard cut: the frame tears into displaced slices with channel split and corrupted blocks, and the incoming scene takes over slice by slice at the peak.",
                ElementCategory::Transition,
                78,
            )
                .look(NodeLook::of("lucide:zap").moves(NodeMotion::Glitch)),
        )
}
