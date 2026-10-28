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
pub struct ComparisonBarsRow {
    #[default = "Manual steps"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 62.0]
    #[schemars(description = "Before")]
    pub before: f64,
    #[default = 18.0]
    #[schemars(description = "After")]
    pub after: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Delta", extend("type" = "string"))]
    pub delta: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonBars {
    #[default(vec![
        ComparisonBarsRow {
            label: "Setup time (min)".into(),
            before: 240.0,
            after: 96.0,
            ..Default::default()
        },
        ComparisonBarsRow {
            label: "Tickets closed / week".into(),
            before: 120.0,
            after: 310.0,
            ..Default::default()
        },
        ComparisonBarsRow {
            label: "Active teams".into(),
            before: 34.0,
            after: 88.0,
            ..Default::default()
        },
        ComparisonBarsRow::default(),
    ])]
    #[schemars(description = "Rows", extend("x-help" = "The delta is computed when omitted."))]
    pub rows: Vec<ComparisonBarsRow>,
    #[default = 1100.0]
    #[schemars(description = "Width", extend("x-help" = "Overall width, label column included."))]
    pub width: f64,
    #[default = 84.0]
    #[schemars(
        description = "Row height",
        extend(
            "x-help" = "Height of one pair, both bars and their gap included.",
            "x-advanced" = true
        )
    )]
    pub row_height: f64,
    #[default = 26.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between rows.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 280.0]
    #[schemars(
        description = "Label width",
        extend("x-help" = "Width reserved for the row labels.", "x-advanced" = true)
    )]
    pub label_width: f64,
    #[default = "rgba(250,250,250,0.22)"]
    #[schemars(
        description = "Before color",
        extend(
            "x-help" = "Baseline bar. Muted on purpose: it is the thing being beaten.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub before_color: String,
    #[default = "#e8b86d"]
    #[schemars(
        description = "After color",
        extend(
            "x-help" = "New-value bar, and the colour of a rising delta.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub after_color: String,
    #[default = "#f472b6"]
    #[schemars(
        description = "Decrease color",
        extend(
            "x-help" = "Delta colour when the change is a fall.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub down_color: String,
    #[default(vec!["Before".into(), "After".into()])]
    #[schemars(
        description = "Series labels",
        length(max = 2),
        extend("x-help" = "Legend above the rows: the before bar, then the after bar.")
    )]
    pub series_labels: Vec<String>,
    #[default = true]
    #[schemars(
        description = "Show change",
        extend("x-help" = "Percentage-change chip at the end of the second bar.")
    )]
    pub show_delta: bool,
    #[default = 30]
    #[schemars(
        description = "Animation length",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Length of one bar's growth.",
            "x-advanced" = true
        )
    )]
    pub duration_in_frames: u32,
    #[default = 12]
    #[schemars(
        description = "Stagger",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames between one row and the next.",
            "x-advanced" = true
        )
    )]
    pub stagger_in_frames: u32,
    #[default = 6]
    #[schemars(
        description = "Pair offset",
        extend(
            "x-unit" = "frames",
            "x-help" = "Frames the second bar trails the first by.",
            "x-advanced" = true
        )
    )]
    pub pair_offset_in_frames: u32,
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
    meta::<ComparisonBars>(
        element(
            ElementKind::Component,
            "comparison-bars",
            "Before and after bars",
            "Pairs of bars comparing before and after, with the change called out.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar")),
    )
}
