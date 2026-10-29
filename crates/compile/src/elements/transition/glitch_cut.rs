use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
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
pub struct GlitchCut {
    #[setting("Intensity", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub intensity: f64,
    #[setting("Slices", default = 24, range(min = 8, max = 64), step = 2)]
    pub slices: u32,
    #[setting("RGB split", default = 1.0, range(min = 0, max = 2), step = 0.1)]
    pub rgb_split: f64,
    #[setting("Block noise", default = 0.6, range(min = 0, max = 1), step = 0.05)]
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
