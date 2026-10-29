use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DonutChartSegment {
    #[setting("Label", default = "Referral")]
    pub label: String,
    #[setting("Value", default = 880.0)]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DonutChart {
    #[setting(
        "Segments",
        default = vec![
            DonutChartSegment { label: "Direct".into(), value: 4820.0, ..Default::default() },
            DonutChartSegment { label: "Search".into(), value: 3140.0, ..Default::default() },
            DonutChartSegment { label: "Social".into(), value: 1960.0, ..Default::default() },
            DonutChartSegment::default(),
        ]
    )]
    pub segments: Vec<DonutChartSegment>,
    #[setting("Size", default = 520.0, help = "All type scales off it.")]
    pub size: f64,
    #[setting("Thickness", default = 46.0, help = "Ring thickness.")]
    pub thickness: f64,
    #[setting(
        "Colors",
        default = vec!["#e8b86d".into(), "#2dd4bf".into(), "#f472b6".into(), "#8b8bf5".into()],
        length(min = 1)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Show legend",
        default = true,
        help = "Legend rows beside the ring, arriving with their own slice."
    )]
    pub show_legend: bool,
    #[setting("Show total", default = true, help = "Running total in the hole.")]
    pub show_total: bool,
    #[setting("Total label", default = "Sessions", help = "Caption under the total.")]
    pub total_label: String,
    #[setting(
        "Animation length",
        default = 26,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one segment's sweep.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 9,
        unit = "frames",
        help = "Frames between one segment and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Track color", default = "rgba(250,250,250,0.08)", check = colour)]
    pub track_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.58)", check = colour)]
    pub label_color: String,
    #[setting("Text color", default = "#fafafa", check = colour)]
    pub ink_color: String,
    #[setting("Increase color", default = "#2dd4bf", check = colour)]
    pub up_color: String,
    #[setting("Decrease color", default = "#f87171", check = colour)]
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
