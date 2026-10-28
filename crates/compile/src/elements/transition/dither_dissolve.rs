use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DitherShape;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DitherDissolve {
    #[default(DitherShape::Simplex)]
    #[schemars(description = "Shape")]
    pub shape: DitherShape,
    #[default = "#141318"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_back: String,
    #[default = "#8f88ae"]
    #[schemars(
        description = "Ink",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color_front: String,
    #[default = 1.5]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-advanced" = true)
    )]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<DitherDissolve>(
        element(
            ElementKind::Presentation,
            "dither-dissolve",
            "Dither dissolve",
            "A dithering shader transition that dissolves one scene into the next.",
            ElementCategory::Transition,
            100,
        )
        .look(NodeLook::of("lucide:grip").moves(NodeMotion::Dissolve)),
    )
}
