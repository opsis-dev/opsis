use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum GrainDissolveShape {
    Wave,
    Dots,
    Truchet,
    Corners,
    Ripple,
    Blob,
    Sphere,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GrainDissolve {
    #[default(GrainDissolveShape::Blob)]
    #[schemars(description = "Shape")]
    pub shape: GrainDissolveShape,
    #[default = 0.3]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = 2.0]
    #[schemars(description = "Zoom", range(min = 0.5, max = 4), extend("x-step" = 0.25))]
    pub zoom: f64,
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
}

pub fn definition() -> ElementMeta {
    meta::<GrainDissolve>(
            element(
                ElementKind::Presentation,
                "grain-dissolve",
                "Grain dissolve",
                "A grain-gradient shader transition that disintegrates the outgoing scene into soft grainy shapes and condenses the next scene out of the noise.",
                ElementCategory::Transition,
                116,
            )
                .look(NodeLook::of("tabler:grain").moves(NodeMotion::Dissolve)),
        )
}
