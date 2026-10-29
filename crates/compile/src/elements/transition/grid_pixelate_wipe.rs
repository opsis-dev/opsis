use opsis_macros::settings;
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct GridPixelateWipe {
    #[setting(
        "Columns",
        default = 12,
        range(min = 1, max = 48),
        step = 1,
        help = "Grid column count."
    )]
    pub cols: u32,
    #[setting(
        "Rows",
        default = 8,
        range(min = 1, max = 48),
        step = 1,
        help = "Grid row count."
    )]
    pub rows: u32,
    #[setting(
        "Order",
        default = PixelateOrder::FromLeft,
        help = "Which axis or point the cells light up from."
    )]
    pub order: PixelateOrder,
    #[setting(
        "Shape",
        default = PixelShape::Square,
        help = "Cells fill as blocks, or as points that grow with their own reveal."
    )]
    pub shape: PixelShape,
    #[setting(
        "Stagger",
        default = 0.82,
        range(min = 0, max = 1),
        step = 0.01,
        help = "0 pops every cell together; 1 spreads them across the whole window.",
        advanced
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
