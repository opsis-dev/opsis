use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTableRow {
    #[default = "New feature"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default(vec!["yes".into(), "yes".into(), "yes".into()])]
    #[schemars(description = "Cells")]
    pub cells: Vec<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTable {
    #[default(vec!["Free".into(), "Studio".into(), "Scale".into()])]
    #[schemars(
        description = "Columns",
        length(min = 1),
        extend(
            "x-help" = "Column headings. The feature-label column is unnamed and sits ahead of these."
        )
    )]
    pub columns: Vec<String>,
    #[default(vec![
        ComparisonTableRow {
            label: "Render minutes".into(),
            cells: vec!["60".into(), "2,000".into(), "Unlimited".into()],
        },
        ComparisonTableRow { label: "1080p exports".into(), ..Default::default() },
        ComparisonTableRow {
            label: "4K exports".into(),
            cells: vec!["no".into(), "yes".into(), "yes".into()],
        },
        ComparisonTableRow {
            label: "Team seats".into(),
            cells: vec!["1".into(), "9".into(), "Unlimited".into()],
        },
        ComparisonTableRow {
            label: "Render API".into(),
            cells: vec!["no".into(), "yes".into(), "yes".into()],
        },
        ComparisonTableRow {
            label: "Priority queue".into(),
            cells: vec!["no".into(), "no".into(), "yes".into()],
        },
    ])]
    #[schemars(
        description = "Rows",
        extend("x-help" = "Type yes or no in a cell for a tick or a dash.")
    )]
    pub rows: Vec<ComparisonTableRow>,
    #[default = "What you get"]
    #[schemars(
        description = "Title",
        extend("x-help" = "Heading above the table. Omit to drop it.")
    )]
    pub title: String,
    #[default = 1]
    #[schemars(
        description = "Highlighted column",
        extend("x-help" = "Column kept lit throughout. Pass -1 for a table with no favourite.")
    )]
    pub highlight_column: u32,
    #[default = "Most picked"]
    #[schemars(
        description = "Highlight badge",
        extend("x-help" = "Chip pinned beside the title, above the highlighted column.")
    )]
    pub highlight_label: String,
    #[default = 0.5]
    #[schemars(
        description = "Start at (seconds)",
        extend("x-step" = 0.01, "x-help" = "Second the first row wipes in.", "x-advanced" = true)
    )]
    pub start_at_seconds: f64,
    #[default = 0.26]
    #[schemars(
        description = "Row stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between rows. Cells inside a row trail their label by another 0.07s each.",
            "x-advanced" = true
        )
    )]
    pub row_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Highlight band, its header, and the ticks inside it.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-help" = "Animation speed multiplier.")
    )]
    pub speed: f64,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<ComparisonTable>(
        element(
            ElementKind::Component,
            "comparison-table",
            "Plan comparison table",
            "A table comparing plans feature by feature, with one plan highlighted.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:table-2")),
    )
}
