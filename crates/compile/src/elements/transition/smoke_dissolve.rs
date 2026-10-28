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
pub struct SmokeDissolve {
    #[default = "#8f88ae"]
    #[schemars(
        description = "Ring",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ring_color: String,
    #[default = "#141318"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 10))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-advanced" = true)
    )]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SmokeDissolve>(
            element(
                ElementKind::Presentation,
                "smoke-dissolve",
                "Smoke dissolve",
                "A smoke-ring shader transition that expands the ring outward to reveal the next scene.",
                ElementCategory::Transition,
                116,
            )
                .look(NodeLook::of("lucide:cloud-fog").moves(NodeMotion::Dissolve)),
        )
}
