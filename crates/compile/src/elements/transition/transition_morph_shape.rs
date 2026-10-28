use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MorphShape {
    Circle,
    Square,
    Squircle,
    Triangle,
    Diamond,
    Blob,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionMorphShape {
    #[default(MorphShape::Circle)]
    #[schemars(
        description = "From",
        extend("x-help" = "Shape the reveal starts as, at zero size. A raw path d also works.")
    )]
    pub from: MorphShape,
    #[default(MorphShape::Squircle)]
    #[schemars(
        description = "To",
        extend("x-help" = "Shape it has become once it covers the frame.")
    )]
    pub to: MorphShape,
    #[default = 0.5]
    #[schemars(
        description = "Horizontal origin",
        extend(
            "x-step" = 0.01,
            "x-help" = "Where the shape grows from, 0→1 of the frame width.",
            "x-advanced" = true
        )
    )]
    pub origin_x: f64,
    #[default = 0.5]
    #[schemars(
        description = "Vertical origin",
        extend(
            "x-step" = 0.01,
            "x-help" = "Where the shape grows from, 0→1 of the frame height.",
            "x-advanced" = true
        )
    )]
    pub origin_y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Overshoot",
        range(min = 0.8, max = 3),
        extend(
            "x-zero-label" = "Fit the shape",
            "x-step" = 0.05,
            "x-help" = "Final size as a multiple of the frame diagonal. Leave empty to grow each shape until it covers the frame.",
            "x-advanced" = true,
            "type" = "number"
        )
    )]
    pub overshoot: Option<f64>,
    #[schemars(
        description = "Warp",
        extend("x-help" = "0 keeps the silhouette crisp.", "x-advanced" = true)
    )]
    pub warp: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TransitionMorphShape>(
            element(
                ElementKind::Presentation,
                "transition-morph-shape",
                "Shape morph",
                "The next scene shows through a shape that grows and changes form until it fills the frame.",
                ElementCategory::Transition,
                24,
            )
                .look(NodeLook::of("lucide:square").morphs("lucide:circle")),
        )
}
