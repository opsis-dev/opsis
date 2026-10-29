use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::NumberFormat;
use crate::elements::theme::{Theme, ThemeMode, metric_ticker_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MetricTickerMetric {
    #[setting("Label", default = "Support tickets")]
    pub label: String,
    #[setting("Value", default = 320.0)]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Trend", "type" = "array")]
    pub trend: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Suffix", "type" = "string")]
    pub suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("From", "type" = "number")]
    pub from: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Prefix", "type" = "string")]
    pub prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct MetricTicker {
    #[setting(
        "Metrics",
        default = vec![
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
        ],
        help = "Metric cards: label, value, and optional from, prefix, suffix, delta, trend, color. Set from above value for a metric that counts down."
    )]
    pub metrics: Vec<MetricTickerMetric>,
    #[setting("Title", default = "Growth this quarter", help = "Scene title.")]
    pub title: String,
    #[setting(
        "Small heading",
        default = "Quarter recap",
        help = "Short label above the title."
    )]
    pub eyebrow: String,
    #[setting(
        "Most cards",
        default = 4,
        help = "Cards beyond this count are dropped rather than squeezed.",
        advanced
    )]
    pub max_cards: u32,
    #[setting(
        "Accent color",
        widget = "color",
        zero_label = "Theme",
        help = "Value colour, overridable per metric. Leave empty to follow the theme."
    )]
    pub accent_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        advanced
    )]
    pub background_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = metric_ticker_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Show arrows", default = true, advanced)]
    pub show_arrows: bool,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting("Glow", default = 7.0, range(min = 0, max = 100), unit = "%", advanced)]
    pub glow: f64,
    #[setting("Capitalise small heading", default = true, advanced)]
    pub uppercase_eyebrow: bool,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Fill under trend", default = true, advanced)]
    pub trend_fill: bool,
    #[setting("Font", widget = "font", zero_label = "Main font", advanced)]
    pub font_family: String,
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
