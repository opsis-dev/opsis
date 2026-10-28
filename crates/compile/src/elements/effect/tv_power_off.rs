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
pub struct TvPowerOff {
    #[default = 18]
    #[schemars(description = "Duration", range(min = 8, max = 30), extend("x-step" = 1))]
    pub duration_in_frames: u32,
    #[default = 1.0]
    #[schemars(description = "Surge", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub gain: f64,
    #[default = 1.0]
    #[schemars(description = "Afterglow", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub afterglow: f64,
    #[default = "#d8ecff"]
    #[schemars(
        description = "Phosphor",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub phosphor: String,
}

pub fn definition() -> ElementMeta {
    meta::<TvPowerOff>(
            element(
                ElementKind::Wrapper,
                "tv-power-off",
                "TV power off",
                "End a scene the way a tube does: the picture collapses into a glowing horizontal line, the line pinches to a phosphor dot, the dot decays, and the frame is left dead black.",
                ElementCategory::Effect,
                62,
            )
                .look(NodeLook::of("lucide:tv").moves(NodeMotion::Collapse)),
        )
}
