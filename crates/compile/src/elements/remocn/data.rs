use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedBarChartBar {
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 50.0]
    #[schemars(description = "Value", range(min = 0, max = 1000000), extend("x-step" = 1))]
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedBarChart {
    #[default = "#0ea5e9"]
    #[schemars(description = "Bar color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub bar_color: String,
    #[default = 16.0]
    #[schemars(description = "Gap", range(min = 0, max = 80), extend("x-step" = 2))]
    pub gap: f64,
    #[default = 0.2]
    #[schemars(description = "Stagger", range(min = 0, max = 1), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub stagger_frames: f64,
    #[default(vec![
        AnimatedBarChartBar { value: 35.0, ..Default::default() },
        AnimatedBarChartBar { value: 60.0, ..Default::default() },
        AnimatedBarChartBar { value: 45.0, ..Default::default() },
        AnimatedBarChartBar { value: 80.0, ..Default::default() },
        AnimatedBarChartBar { value: 55.0, ..Default::default() },
        AnimatedBarChartBar { value: 70.0, ..Default::default() },
        AnimatedBarChartBar { value: 90.0, ..Default::default() },
        AnimatedBarChartBar { value: 65.0, ..Default::default() },
    ])]
    #[schemars(description = "Bars", length(min = 1, max = 60), extend("x-help" = "Leave every label empty to draw the bars without labels."))]
    pub bars: Vec<AnimatedBarChartBar>,
    #[default = 1000.0]
    #[schemars(description = "Width", range(min = 200, max = 1920), extend("x-step" = 10, "x-advanced" = true))]
    pub width: f64,
    #[default = 500.0]
    #[schemars(description = "Height", range(min = 100, max = 1080), extend("x-step" = 10, "x-advanced" = true))]
    pub height: f64,
    #[default = "#27272a"]
    #[schemars(description = "Baseline colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub baseline_color: String,
    #[default = "#a1a1aa"]
    #[schemars(description = "Label colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedLineChart {
    #[default = "#22c55e"]
    #[schemars(description = "Stroke color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub stroke_color: String,
    #[default = 4.0]
    #[schemars(description = "Stroke width", range(min = 1, max = 16), extend("x-step" = 1))]
    pub stroke_width: f64,
    #[default = "#27272a"]
    #[schemars(description = "Grid color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub grid_color: String,
    #[default = true]
    #[schemars(description = "Show leading dot")]
    pub show_dot: bool,
    #[default(vec![12.0, 19.0, 8.0, 15.0, 22.0, 18.0, 28.0, 25.0, 32.0])]
    #[schemars(description = "Values", length(min = 2), extend("x-help" = "Paste a column from a spreadsheet to fill the list."))]
    pub data: Vec<f64>,
    #[default = 1000.0]
    #[schemars(description = "Width", range(min = 200, max = 3840), extend("x-step" = 10, "x-advanced" = true))]
    pub width: f64,
    #[default = 500.0]
    #[schemars(description = "Height", range(min = 100, max = 2160), extend("x-step" = 10, "x-advanced" = true))]
    pub height: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RollingNumber {
    #[schemars(description = "From", range(min = 0, max = 999999999999.0), extend("x-step" = 1))]
    pub from: f64,
    #[default = 24813.0]
    #[schemars(description = "To", range(min = 0, max = 999999999999.0), extend("x-step" = 1))]
    pub to: f64,
    #[default = 120.0]
    #[schemars(description = "Font size", range(min = 24, max = 280), extend("x-step" = 4))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "white"]
    #[schemars(description = "Background", extend("x-help" = "Set it to transparent to lay the text over other layers.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<AnimatedBarChart>(
            element(
                ElementKind::Component,
                "remocn/animated-bar-chart",
                "Animated bar chart",
                "Bars spring up from the baseline in a staggered cascade.",
                C::Data,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/animated-bar-chart", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:chart-column").moves(NodeMotion::Grow)),
        ),
        meta::<AnimatedLineChart>(
            element(
                ElementKind::Component,
                "remocn/animated-line-chart",
                "Animated line chart",
                "Line chart whose SVG path draws on from left to right with a leading dot.",
                C::Data,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/animated-line-chart", "MIT")
                .look(NodeLook::of("lucide:chart-line").moves(NodeMotion::Draw)),
        ),
        meta::<RollingNumber>(
            element(
                ElementKind::Component,
                "remocn/rolling-number",
                "Rolling number",
                "Odometer where every place rolls at its own speed, new places rising into view, settling exactly on the target value.",
                C::Data,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/rolling-number", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:hash").moves(NodeMotion::Roll)),
        ),
    ]
}
