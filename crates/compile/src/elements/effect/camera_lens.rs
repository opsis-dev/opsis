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
pub struct CameraLens {
    #[setting("Corner softness", default = 0.5, range(min = 0, max = 2), step = 0.05)]
    pub softness: f64,
    #[setting("Bloom", default = 0.5, range(min = 0, max = 2), step = 0.05)]
    pub bloom: f64,
    #[setting("Aberration", default = 0.5, range(min = 0, max = 3), step = 0.1)]
    pub aberration: f64,
    #[setting("Vignette", default = 0.5, range(min = 0, max = 1.5), step = 0.05)]
    pub vignette: f64,
    #[setting(
        "Distortion",
        default = 0.05,
        range(min = 0, max = 0.6),
        step = 0.01,
        advanced
    )]
    pub distortion: f64,
    #[setting("Sharp centre", default = 0.22, range(min = 0, max = 0.9), advanced)]
    pub sharp_centre: f64,
    #[setting(
        "Bloom from brightness",
        default = 0.62,
        range(min = 0, max = 1),
        advanced
    )]
    pub bloom_threshold: f64,
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
