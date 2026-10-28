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
pub struct CrtScreen {
    #[default = 1.0]
    #[schemars(description = "Curvature", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub curvature: f64,
    #[default = 1.0]
    #[schemars(description = "Scanlines", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub scanlines: f64,
    #[default = 2]
    #[schemars(description = "Mask pitch", range(min = 1, max = 6), extend("x-step" = 1))]
    pub mask_scale: u32,
    #[default = 1.0]
    #[schemars(description = "Vignette", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub vignette: f64,
}

pub fn definition() -> ElementMeta {
    meta::<CrtScreen>(
            element(
                ElementKind::Wrapper,
                "crt-screen",
                "CRT screen",
                "Put a scene behind curved glass: the picture bulges out to the corners, an RGB phosphor mask and scanlines ride the curve, bright areas flood the mask, and the tube falls away into darkness at the rim.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:tv-minimal")),
        )
}
