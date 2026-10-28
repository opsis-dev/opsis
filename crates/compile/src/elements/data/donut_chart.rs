use opsis_types::looks::NodeLook;
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
pub struct DonutChartSegment {
    #[default = "Referral"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 880.0]
    #[schemars(description = "Value")]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Color",
        extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string")
    )]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Delta", extend("type" = "string"))]
    pub delta: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DonutChart {
    #[default(vec![
        DonutChartSegment { label: "Direct".into(), value: 4820.0, ..Default::default() },
        DonutChartSegment { label: "Search".into(), value: 3140.0, ..Default::default() },
        DonutChartSegment { label: "Social".into(), value: 1960.0, ..Default::default() },
        DonutChartSegment::default(),
    ])]
    #[schemars(description = "Segments")]
    pub segments: Vec<DonutChartSegment>,
    #[default = 520.0]
    #[schemars(description = "Size", extend("x-help" = "All type scales off it."))]
    pub size: f64,
    #[default = 46.0]
    #[schemars(description = "Thickness", extend("x-help" = "Ring thickness."))]
    pub thickness: f64,
    #[default(vec!["#e8b86d".into(), "#2dd4bf".into(), "#f472b6".into(), "#8b8bf5".into()])]
    #[schemars(description = "Colors", length(min = 1))]
    pub colors: Vec<Colour>,
    #[default = true]
    #[schemars(
        description = "Show legend",
        extend("x-help" = "Legend rows beside the ring, arriving with their own slice.")
    )]
    pub show_legend: bool,
    #[default = true]
    #[schemars(description = "Show total", extend("x-help" = "Running total in the hole."))]
    pub show_total: bool,
    #[default = "Sessions"]
    #[schemars(description = "Total label", extend("x-help" = "Caption under the total."))]
    pub total_label: String,
    #[default = 26]
    #[schemars(
        description = "Animation length",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Length of one segment's sweep.",
            "x-advanced" = true
        )
    )]
    pub duration_in_frames: u32,
    #[default = 9]
    #[schemars(
        description = "Stagger",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames between one segment and the next.",
            "x-advanced" = true
        )
    )]
    pub stagger_in_frames: u32,
    #[default = "rgba(250,250,250,0.08)"]
    #[schemars(
        description = "Track color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub track_color: String,
    #[default = "rgba(250,250,250,0.58)"]
    #[schemars(
        description = "Label color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub label_color: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ink_color: String,
    #[default = "#2dd4bf"]
    #[schemars(
        description = "Increase color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub up_color: String,
    #[default = "#f87171"]
    #[schemars(
        description = "Decrease color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub down_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<DonutChart>(
        element(
            ElementKind::Component,
            "donut-chart",
            "Donut chart",
            "A ring chart that fills segment by segment, with a legend and a total.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-pie")),
    )
}
