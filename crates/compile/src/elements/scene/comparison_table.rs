use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTableRow {
    #[setting("Label", default = "New feature")]
    pub label: String,
    #[setting("Cells", default = vec!["yes".into(), "yes".into(), "yes".into()])]
    pub cells: Vec<String>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTable {
    #[setting(
        "Columns",
        default = vec!["Free".into(), "Studio".into(), "Scale".into()],
        length(min = 1),
        help = "Column headings. The feature-label column is unnamed and sits ahead of these."
    )]
    pub columns: Vec<String>,
    #[setting(
        "Rows",
        default = vec![
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
        ],
        help = "Type yes or no in a cell for a tick or a dash."
    )]
    pub rows: Vec<ComparisonTableRow>,
    #[setting(
        "Title",
        default = "What you get",
        help = "Heading above the table. Omit to drop it."
    )]
    pub title: String,
    #[setting(
        "Highlighted column",
        default = 1,
        help = "Column kept lit throughout. Pass -1 for a table with no favourite."
    )]
    pub highlight_column: u32,
    #[setting(
        "Highlight badge",
        default = "Most picked",
        help = "Chip pinned beside the title, above the highlighted column."
    )]
    pub highlight_label: String,
    #[setting(
        "Start at (seconds)",
        default = 0.5,
        step = 0.01,
        help = "Second the first row wipes in.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Row stagger (seconds)",
        default = 0.26,
        step = 0.01,
        help = "Seconds between rows. Cells inside a row trail their label by another 0.07s each.",
        advanced
    )]
    pub row_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Highlight band, its header, and the ticks inside it.",
        check = colour
    )]
    pub accent_color: String,
    #[setting("Theme", default = DarkOrLight::Dark, help = "Surface palette.")]
    pub theme: DarkOrLight,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
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
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:table-2")),
    )
}
