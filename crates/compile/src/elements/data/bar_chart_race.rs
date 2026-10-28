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
pub struct BarChartRaceSerie {
    #[default = "Partners"]
    #[schemars(description = "Label")]
    pub label: String,
    #[schemars(description = "Values")]
    pub values: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Color",
        extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string")
    )]
    pub color: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct BarChartRace {
    #[default(vec![
        BarChartRaceSerie {
            label: "Mobile".into(),
            values: vec![42.0, 58.0, 66.0, 72.0, 78.0, 84.0],
            ..Default::default()
        },
        BarChartRaceSerie {
            label: "Web".into(),
            values: vec![18.0, 34.0, 57.0, 76.0, 92.0, 108.0],
            ..Default::default()
        },
        BarChartRaceSerie {
            label: "Desktop".into(),
            values: vec![30.0, 40.0, 47.0, 52.0, 57.0, 61.0],
            ..Default::default()
        },
        BarChartRaceSerie {
            label: "API".into(),
            values: vec![12.0, 22.0, 36.0, 50.0, 63.0, 79.0],
            ..Default::default()
        },
        BarChartRaceSerie {
            label: "Email".into(),
            values: vec![24.0, 28.0, 31.0, 35.0, 38.0, 42.0],
            ..Default::default()
        },
        BarChartRaceSerie { values: vec![8.0, 12.0, 17.0, 21.0, 26.0, 30.0], ..Default::default() },
    ])]
    #[schemars(description = "Series")]
    pub series: Vec<BarChartRaceSerie>,
    #[default(vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into(), "Q5".into(), "Q6".into()])]
    #[schemars(
        description = "Steps",
        extend("x-help" = "Keyframe captions, e.g. years. Shown as the running clock.")
    )]
    pub steps: Vec<String>,
    #[default = 18]
    #[schemars(
        description = "Time per step",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Frames spent travelling between two keyframes.",
            "x-advanced" = true
        )
    )]
    pub frames_per_step: u32,
    #[default = 6]
    #[schemars(
        description = "Visible rows",
        range(min = 1, max = 20),
        extend(
            "x-step" = 1,
            "x-help" = "How many rows stay on the board. Below that, series fade out.",
            "x-advanced" = true
        )
    )]
    pub visible_rows: u32,
    #[default = 1200.0]
    #[schemars(description = "Width", extend("x-help" = "Overall width, label column included."))]
    pub width: f64,
    #[default = 76.0]
    #[schemars(
        description = "Row height",
        extend("x-help" = "Bar height. Type scales off it.", "x-advanced" = true)
    )]
    pub row_height: f64,
    #[default = 14.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between rows.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 240.0]
    #[schemars(
        description = "Label width",
        extend("x-help" = "Width reserved for the row labels.", "x-advanced" = true)
    )]
    pub label_width: f64,
    #[default = true]
    #[schemars(
        description = "Show step label",
        extend("x-help" = "Large step caption in the bottom-right corner.")
    )]
    pub show_step_label: bool,
    #[default(vec![
        "#e8b86d".into(),
        "#2dd4bf".into(),
        "#f472b6".into(),
        "#8b8bf5".into(),
        "#f59e0b".into(),
        "#5eead4".into(),
    ])]
    #[schemars(description = "Colors", length(min = 1))]
    pub colors: Vec<Colour>,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ink_color: String,
    #[default = "#0b0b10"]
    #[schemars(
        description = "Bar text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub bar_ink_color: String,
    #[default = "rgba(250,250,250,0.55)"]
    #[schemars(
        description = "Label color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub label_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<BarChartRace>(
        element(
            ElementKind::Component,
            "bar-chart-race",
            "Bar chart race",
            "Ranked bars that grow and swap places as the values change over time.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar-decreasing")),
    )
}
