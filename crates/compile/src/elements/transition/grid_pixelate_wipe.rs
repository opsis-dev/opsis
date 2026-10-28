use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum PixelateOrder {
    FromLeft,
    FromTop,
    Diagonal,
    Center,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum PixelShape {
    Square,
    Dot,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GridPixelateWipe {
    #[default = 12]
    #[schemars(
        description = "Columns",
        range(min = 1, max = 48),
        extend("x-step" = 1, "x-help" = "Grid column count.")
    )]
    pub cols: u32,
    #[default = 8]
    #[schemars(
        description = "Rows",
        range(min = 1, max = 48),
        extend("x-step" = 1, "x-help" = "Grid row count.")
    )]
    pub rows: u32,
    #[default(PixelateOrder::FromLeft)]
    #[schemars(
        description = "Order",
        extend("x-help" = "Which axis or point the cells light up from.")
    )]
    pub order: PixelateOrder,
    #[default(PixelShape::Square)]
    #[schemars(
        description = "Shape",
        extend("x-help" = "Cells fill as blocks, or as points that grow with their own reveal.")
    )]
    pub shape: PixelShape,
    #[default = 0.82]
    #[schemars(
        description = "Stagger",
        range(min = 0, max = 1),
        extend(
            "x-step" = 0.01,
            "x-help" = "0 pops every cell together; 1 spreads them across the whole window.",
            "x-advanced" = true
        )
    )]
    pub stagger: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GridPixelateWipe>(
        element(
            ElementKind::Presentation,
            "grid-pixelate-wipe",
            "Pixel grid wipe",
            "The next scene appears one grid cell at a time.",
            ElementCategory::Transition,
            26,
        )
        .look(NodeLook::of("lucide:grid-2x2").moves(NodeMotion::WipeReveal)),
    )
}
