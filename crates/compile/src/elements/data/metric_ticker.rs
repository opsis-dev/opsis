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
pub struct MetricTickerMetric {
    #[default = "Support tickets"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 320.0]
    #[schemars(description = "Value")]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Delta", extend("type" = "string"))]
    pub delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Trend", extend("type" = "array"))]
    pub trend: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Suffix", extend("type" = "string"))]
    pub suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "From", extend("type" = "number"))]
    pub from: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Prefix", extend("type" = "string"))]
    pub prefix: Option<String>,
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
pub struct MetricTicker {
    #[default(vec![
        MetricTickerMetric {
            label: "Sign-ups".into(),
            value: 12400.0,
            delta: Some("+18%".into()),
            trend: Some(vec![62.0, 71.0, 68.0, 84.0, 96.0, 112.0, 124.0]),
            ..Default::default()
        },
        MetricTickerMetric {
            label: "Avg. session".into(),
            value: 48.0,
            suffix: Some("min".into()),
            delta: Some("+6%".into()),
            trend: Some(vec![39.0, 41.0, 40.0, 43.0, 45.0, 44.0, 48.0]),
            ..Default::default()
        },
        MetricTickerMetric {
            delta: Some("-4%".into()),
            trend: Some(vec![36.0, 35.0, 37.0, 34.0, 33.0, 33.5, 32.0]),
            ..Default::default()
        },
    ])]
    #[schemars(
        description = "Metrics",
        extend(
            "x-help" = "Metric cards: label, value, and optional from, prefix, suffix, delta, trend, color. Set from above value for a metric that counts down."
        )
    )]
    pub metrics: Vec<MetricTickerMetric>,
    #[default = "Growth this quarter"]
    #[schemars(description = "Title", extend("x-help" = "Scene title."))]
    pub title: String,
    #[default = "Quarter recap"]
    #[schemars(description = "Small heading", extend("x-help" = "Short label above the title."))]
    pub eyebrow: String,
    #[default = 4]
    #[schemars(
        description = "Most cards",
        extend("x-help" = "Cards beyond this count are dropped rather than squeezed.")
    )]
    pub max_cards: u32,
    #[schemars(
        description = "Accent color",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Value colour, overridable per metric. Leave empty to follow the theme."
        )
    )]
    pub accent_color: String,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
    )]
    pub background_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(
        description = "Theme",
        extend("x-help" = "Surface palette: text, cards and the change colours.")
    )]
    pub theme: DarkOrLight,
}

pub fn definition() -> ElementMeta {
    meta::<MetricTicker>(
        element(
            ElementKind::Component,
            "metric-ticker",
            "Metric cards",
            "Key numbers on cards that count up, each with its change and a small trend line.",
            ElementCategory::Data,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:trending-up")),
    )
}
