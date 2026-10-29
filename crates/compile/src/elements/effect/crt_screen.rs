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
pub struct CrtScreen {
    #[setting("Curvature", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub curvature: f64,
    #[setting("Scanlines", default = 1.0, range(min = 0, max = 1.5), step = 0.05)]
    pub scanlines: f64,
    #[setting("Mask pitch", default = 2, range(min = 1, max = 6), step = 1)]
    pub mask_scale: u32,
    #[setting("Vignette", default = 1.0, range(min = 0, max = 1.5), step = 0.05)]
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
