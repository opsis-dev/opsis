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
pub struct DataStoryBarData {
    #[default = "API"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 48000.0]
    #[schemars(description = "Value")]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Delta", extend("type" = "string"))]
    pub delta: Option<String>,
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
pub struct DataStoryMetric {
    #[default = "Support tickets"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 320.0]
    #[schemars(description = "Value")]
    pub value: f64,
    #[default = "-4%"]
    #[schemars(description = "Delta")]
    pub delta: String,
    #[schemars(description = "Trend")]
    pub trend: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Suffix", extend("type" = "string"))]
    pub suffix: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoryStep {
    #[default = "Next step"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Close with one clear call to action."]
    #[schemars(description = "Description")]
    pub description: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStory {
    #[default = "Quarterly reach by channel"]
    #[schemars(description = "Title", extend("x-help" = "Opening hook headline (auto-fit)."))]
    pub title: String,
    #[default = "Short-form climbed while long-form held attention"]
    #[schemars(description = "Subtitle", extend("x-help" = "Supporting line under the hook."))]
    pub subtitle: String,
    #[default(vec![
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
    ])]
    #[schemars(description = "Chart data", extend("x-help" = "Bar chart data."))]
    pub bar_data: Vec<DataStoryBarData>,
    #[default(vec![
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
    ])]
    #[schemars(description = "Metrics", extend("x-help" = "Metric cards."))]
    pub metrics: Vec<DataStoryMetric>,
    #[default(vec![
        DataStoryStep {
            title: "Hook".into(),
            description: "Lead with the sharpest insight.".into(),
        },
        DataStoryStep {
            title: "Proof".into(),
            description: "Show the chart and the numbers.".into(),
        },
        DataStoryStep::default(),
    ])]
    #[schemars(description = "Steps", extend("x-help" = "Context steps."))]
    pub steps: Vec<DataStoryStep>,
    #[default = "Views by format"]
    #[schemars(description = "Chart title", extend("x-help" = "Headline on the bar chart scene."))]
    pub chart_title: String,
    #[default = "Signals that mattered"]
    #[schemars(
        description = "Metrics title",
        extend("x-help" = "Headline on the metric ticker scene.")
    )]
    pub metrics_title: String,
    #[default = "How we read the quarter"]
    #[schemars(
        description = "Timeline title",
        extend("x-help" = "Headline on the timeline scene.")
    )]
    pub timeline_title: String,
    #[default = "Short-form drove reach; long-form kept attention."]
    #[schemars(
        description = "Takeaway",
        extend("x-help" = "Takeaway quote in the insight bumper.")
    )]
    pub insight: String,
    #[default = "Takeaway"]
    #[schemars(
        description = "Takeaway heading",
        extend("x-help" = "Eyebrow above the insight quote.")
    )]
    pub insight_eyebrow: String,
    #[default = "Turn your data into motion"]
    #[schemars(
        description = "Closing title",
        extend("x-help" = "End card headline (separate from hook).")
    )]
    pub cta_title: String,
    #[default = "Get started"]
    #[schemars(description = "Button label", extend("x-help" = "End card CTA pill label."))]
    pub cta_label: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Light or dark surfaces for every scene."))]
    pub theme: DarkOrLight,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Title and closing card.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default = "#2dd4bf"]
    #[schemars(
        description = "Data color",
        extend(
            "x-help" = "Bars, key numbers and the takeaway.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub data_color: String,
    #[default = "#f59e0b"]
    #[schemars(
        description = "Timeline color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub timeline_color: String,
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
    meta::<DataStory>(
            element(
                ElementKind::Component,
                "data-story",
                "Data story",
                "A whole data story: title, bar chart, key numbers, timeline, takeaway and a closing call to action.",
                ElementCategory::Scene,
                420,
            )
                .look(NodeLook::of("lucide:chart-area")),
        )
}
