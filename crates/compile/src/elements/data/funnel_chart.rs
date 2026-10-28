use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct FunnelChartStage {
    #[default = "Upgraded"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 1750]
    #[schemars(description = "Value", range(min = 0, max = 1000000000), extend("x-step" = 1))]
    pub value: u32,
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
pub struct FunnelChart {
    #[default(vec![
        FunnelChartStage { label: "Visited the site".into(), value: 42800, ..Default::default() },
        FunnelChartStage { label: "Signed up".into(), value: 18600, ..Default::default() },
        FunnelChartStage { label: "Created a project".into(), value: 9400, ..Default::default() },
        FunnelChartStage { label: "Invited a teammate".into(), value: 4100, ..Default::default() },
        FunnelChartStage::default(),
    ])]
    #[schemars(description = "Stages", length(min = 1))]
    pub stages: Vec<FunnelChartStage>,
    #[default = 1100.0]
    #[schemars(
        description = "Width",
        extend("x-help" = "Overall width: label gutter, funnel and drop-off gutter.")
    )]
    pub width: f64,
    #[default = 560.0]
    #[schemars(description = "Height", extend("x-help" = "Overall height. Bands split it evenly."))]
    pub height: f64,
    #[default = 10.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between bands.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 0.5]
    #[schemars(
        description = "Last bar opacity",
        extend(
            "x-step" = 0.01,
            "x-help" = "Opacity of the last band; earlier bands ramp toward 1.",
            "x-advanced" = true
        )
    )]
    pub tail_opacity: f64,
    #[default = true]
    #[schemars(
        description = "Show drop-off",
        extend("x-help" = "Drop-off chip between consecutive stages.")
    )]
    pub show_dropoff: bool,
    #[schemars(
        description = "Show conversion",
        extend("x-help" = "Share of the first stage, printed beside each value.")
    )]
    pub show_conversion: bool,
    #[default = 22]
    #[schemars(
        description = "Animation length",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Length of one band's wipe.",
            "x-advanced" = true
        )
    )]
    pub duration_in_frames: u32,
    #[default = 12]
    #[schemars(
        description = "Stagger",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames between one band and the next.",
            "x-advanced" = true
        )
    )]
    pub stagger_in_frames: u32,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ink_color: String,
    #[default = "rgba(250,250,250,0.55)"]
    #[schemars(
        description = "Label color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub label_color: String,
    #[default = "#f472b6"]
    #[schemars(
        description = "Drop-off color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub dropoff_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Label width",
        range(min = 0, max = 2000),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 10,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "integer"
        )
    )]
    pub label_width: Option<u32>,
}

pub fn definition() -> ElementMeta {
    meta::<FunnelChart>(
        element(
            ElementKind::Component,
            "funnel-chart",
            "Funnel chart",
            "A bar for each step of a funnel, narrowing to show how many people drop off.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:funnel")),
    )
}
