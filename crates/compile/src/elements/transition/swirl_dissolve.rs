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
pub struct SwirlDissolve {
    #[default = 10]
    #[schemars(description = "Bands", range(min = 1, max = 12), extend("x-step" = 1))]
    pub band_count: u32,
    #[default = 0.35]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#141318"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default(vec!["#1f1d29".into(), "#413d56".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
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
    meta::<SwirlDissolve>(
        element(
            ElementKind::Presentation,
            "swirl-dissolve",
            "Swirl dissolve",
            "A swirl shader transition that unwinds open and winds shut to reveal the next scene.",
            ElementCategory::Transition,
            116,
        )
        .look(NodeLook::of("tabler:spiral").moves(NodeMotion::Dissolve)),
    )
}
