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
pub struct RippleZoom {
    #[default = 4.0]
    #[schemars(
        description = "Zoom",
        range(min = 1, max = 8),
        extend("x-step" = 0.1, "x-advanced" = true)
    )]
    pub zoom: f64,
    #[default = 0.5]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.5]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
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
    #[default = 0.5]
    #[schemars(
        description = "Grain",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-advanced" = true)
    )]
    pub noise: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RippleZoom>(
            element(
                ElementKind::Presentation,
                "ripple-zoom",
                "Ripple zoom",
                "A grain-gradient ripple transition that flies the camera through the center of the rings while the next scene approaches from the depth.",
                ElementCategory::Transition,
                116,
            )
                .look(NodeLook::of("tabler:ripple").moves(NodeMotion::Ripple)),
        )
}
