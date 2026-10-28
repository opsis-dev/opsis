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
pub struct CameraLens {
    #[default = 0.5]
    #[schemars(description = "Corner softness", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.5]
    #[schemars(description = "Bloom", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub bloom: f64,
    #[default = 0.5]
    #[schemars(description = "Aberration", range(min = 0, max = 3), extend("x-step" = 0.1))]
    pub aberration: f64,
    #[default = 0.5]
    #[schemars(description = "Vignette", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub vignette: f64,
    #[default = 0.05]
    #[schemars(description = "Distortion", range(min = 0, max = 0.6), extend("x-step" = 0.01))]
    pub distortion: f64,
}

pub fn definition() -> ElementMeta {
    meta::<CameraLens>(
            element(
                ElementKind::Wrapper,
                "camera-lens",
                "Camera lens",
                "Put a scene behind real glass: resolution falls off toward the corners, highlights bloom through the optics, colour fringes at the edges — a screencast that reads as filmed rather than rendered.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:aperture").moves(NodeMotion::RackFocus)),
        )
}
