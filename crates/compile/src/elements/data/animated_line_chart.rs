use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedLineChart {
    #[default = "#22c55e"]
    #[schemars(
        description = "Stroke color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub stroke_color: String,
    #[default = 4]
    #[schemars(description = "Stroke width", range(min = 1, max = 16), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = "#27272a"]
    #[schemars(
        description = "Grid color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub grid_color: String,
    #[default = true]
    #[schemars(description = "Show leading dot")]
    pub show_dot: bool,
    #[default(vec![12.0, 19.0, 8.0, 15.0, 22.0, 18.0, 28.0, 25.0, 32.0])]
    #[schemars(
        description = "Values",
        length(min = 2),
        extend("x-help" = "Paste a column from a spreadsheet to fill the list.")
    )]
    pub data: Vec<f64>,
    #[default = 1000]
    #[schemars(
        description = "Width",
        range(min = 200, max = 3840),
        extend("x-step" = 10, "x-advanced" = true)
    )]
    pub width: u32,
    #[default = 500]
    #[schemars(
        description = "Height",
        range(min = 100, max = 2160),
        extend("x-step" = 10, "x-advanced" = true)
    )]
    pub height: u32,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<AnimatedLineChart>(
        element(
            ElementKind::Component,
            "animated-line-chart",
            "Animated line chart",
            "Line chart whose SVG path draws on from left to right with a leading dot.",
            ElementCategory::Data,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:chart-line").moves(NodeMotion::Draw)),
    )
}
