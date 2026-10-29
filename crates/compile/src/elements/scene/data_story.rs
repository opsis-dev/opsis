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
pub struct DataStoryBarData {
    #[setting("Label", default = "API")]
    pub label: String,
    #[setting("Value", default = 48000.0)]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoryMetric {
    #[setting("Label", default = "Support tickets")]
    pub label: String,
    #[setting("Value", default = 320.0)]
    pub value: f64,
    #[setting("Delta", default = "-4%")]
    pub delta: String,
    #[setting("Trend")]
    pub trend: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Suffix", "type" = "string")]
    pub suffix: Option<String>,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoryStep {
    #[setting("Title", default = "Next step")]
    pub title: String,
    #[setting("Description", default = "Close with one clear call to action.")]
    pub description: String,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStory {
    #[setting(
        "Title",
        default = "Quarterly reach by channel",
        help = "Opening hook headline (auto-fit)."
    )]
    pub title: String,
    #[setting(
        "Subtitle",
        default = "Short-form climbed while long-form held attention",
        help = "Supporting line under the hook."
    )]
    pub subtitle: String,
    #[setting(
        "Chart data",
        default = vec![
            DataStoryBarData {
                label: "Mobile".into(),
                value: 124000.0,
                delta: Some("+32%".into()),
                ..Default::default()
            },
            DataStoryBarData {
                label: "Web".into(),
                value: 82000.0,
                delta: Some("+11%".into()),
                ..Default::default()
            },
            DataStoryBarData {
                label: "Desktop".into(),
                value: 64000.0,
                delta: Some("+4%".into()),
                ..Default::default()
            },
            DataStoryBarData { delta: Some("-6%".into()), ..Default::default() },
        ],
        help = "Bar chart data."
    )]
    pub bar_data: Vec<DataStoryBarData>,
    #[setting(
        "Metrics",
        default = vec![
            DataStoryMetric {
                label: "Sign-ups".into(),
                value: 12400.0,
                delta: "+18%".into(),
                trend: vec![62.0, 71.0, 68.0, 84.0, 96.0, 112.0, 124.0],
                ..Default::default()
            },
            DataStoryMetric {
                label: "Avg. session".into(),
                value: 48.0,
                suffix: Some("min".into()),
                delta: "+6%".into(),
                trend: vec![39.0, 41.0, 40.0, 43.0, 45.0, 44.0, 48.0],
            },
            DataStoryMetric {
                trend: vec![36.0, 35.0, 37.0, 34.0, 33.0, 33.5, 32.0],
                ..Default::default()
            },
        ],
        help = "Metric cards."
    )]
    pub metrics: Vec<DataStoryMetric>,
    #[setting(
        "Steps",
        default = vec![
            DataStoryStep {
                title: "Hook".into(),
                description: "Lead with the sharpest insight.".into(),
            },
            DataStoryStep {
                title: "Proof".into(),
                description: "Show the chart and the numbers.".into(),
            },
            DataStoryStep::default(),
        ],
        help = "Context steps."
    )]
    pub steps: Vec<DataStoryStep>,
    #[setting(
        "Chart title",
        default = "Views by format",
        help = "Headline on the bar chart scene."
    )]
    pub chart_title: String,
    #[setting(
        "Metrics title",
        default = "Signals that mattered",
        help = "Headline on the metric ticker scene."
    )]
    pub metrics_title: String,
    #[setting(
        "Timeline title",
        default = "How we read the quarter",
        help = "Headline on the timeline scene."
    )]
    pub timeline_title: String,
    #[setting(
        "Takeaway",
        default = "Short-form drove reach; long-form kept attention.",
        help = "Takeaway quote in the insight bumper."
    )]
    pub insight: String,
    #[setting(
        "Takeaway heading",
        default = "Takeaway",
        help = "Eyebrow above the insight quote."
    )]
    pub insight_eyebrow: String,
    #[setting(
        "Closing title",
        default = "Turn your data into motion",
        help = "End card headline (separate from hook)."
    )]
    pub cta_title: String,
    #[setting(
        "Button label",
        default = "Get started",
        help = "End card CTA pill label."
    )]
    pub cta_label: String,
    #[setting(
        "Theme",
        default = DarkOrLight::Dark,
        help = "Light or dark surfaces for every scene."
    )]
    pub theme: DarkOrLight,
    #[setting(
        "Accent color",
        default = "#e8b86d",
        help = "Title and closing card.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Data color",
        default = "#2dd4bf",
        help = "Bars, key numbers and the takeaway.",
        check = colour
    )]
    pub data_color: String,
    #[setting("Timeline color", default = "#f59e0b", check = colour)]
    pub timeline_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<DataStory>(
            element(
                ElementKind::Component,
                "data-story",
                "Data story",
                "A whole data story: title, bar chart, key numbers, timeline, takeaway and a closing call to action.",
                ElementCategory::Scene,
                420,
            )
                .fonts(&["Inter"])
                .look(NodeLook::of("lucide:chart-area")),
        )
}
