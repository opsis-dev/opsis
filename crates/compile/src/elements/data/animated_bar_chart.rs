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
pub struct AnimatedBarChartBar {
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 50]
    #[schemars(description = "Value", range(min = 0, max = 1000000), extend("x-step" = 1))]
    pub value: u32,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedBarChart {
    #[default = "#0ea5e9"]
    #[schemars(
        description = "Bar color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub bar_color: String,
    #[default = 16]
    #[schemars(description = "Gap", range(min = 0, max = 80), extend("x-step" = 2))]
    pub gap: u32,
    #[default = 0.2]
    #[schemars(
        description = "Stagger",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub stagger_frames: f64,
    #[default(vec![
        AnimatedBarChartBar { value: 35, ..Default::default() },
        AnimatedBarChartBar { value: 60, ..Default::default() },
        AnimatedBarChartBar { value: 45, ..Default::default() },
        AnimatedBarChartBar { value: 80, ..Default::default() },
        AnimatedBarChartBar { value: 55, ..Default::default() },
        AnimatedBarChartBar { value: 70, ..Default::default() },
        AnimatedBarChartBar { value: 90, ..Default::default() },
        AnimatedBarChartBar { value: 65, ..Default::default() },
    ])]
    #[schemars(
        description = "Bars",
        length(min = 1, max = 60),
        extend("x-help" = "Leave every label empty to draw the bars without labels.")
    )]
    pub bars: Vec<AnimatedBarChartBar>,
    #[default = 1000]
    #[schemars(
        description = "Width",
        range(min = 200, max = 1920),
        extend("x-step" = 10, "x-advanced" = true)
    )]
    pub width: u32,
    #[default = 500]
    #[schemars(
        description = "Height",
        range(min = 100, max = 1080),
        extend("x-step" = 10, "x-advanced" = true)
    )]
    pub height: u32,
    #[default = "#27272a"]
    #[schemars(
        description = "Baseline colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub baseline_color: String,
    #[default = "#a1a1aa"]
    #[schemars(
        description = "Label colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub label_color: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<AnimatedBarChart>(
        element(
            ElementKind::Component,
            "animated-bar-chart",
            "Animated bar chart",
            "Bars spring up from the baseline in a staggered cascade.",
            ElementCategory::Data,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-column").moves(NodeMotion::Grow)),
    )
}
