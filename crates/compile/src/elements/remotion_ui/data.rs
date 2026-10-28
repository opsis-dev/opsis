use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::DarkOrLight;
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FunnelChartStage {
    #[default = "Upgraded"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 1750]
    #[schemars(description = "Value", range(min = 0, max = 1000000000), extend("x-step" = 1))]
    pub value: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DonutChartSegment {
    #[default = "Referral"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default = 880.0]
    #[schemars(description = "Value")]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Delta", extend("type" = "string"))]
    pub delta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BarChartRaceSerie {
    #[default = "Partners"]
    #[schemars(description = "Label")]
    pub label: String,
    #[schemars(description = "Values")]
    pub values: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct StatCard {
    #[default = 98.0]
    #[schemars(description = "Value", extend("x-help" = "Number the counter lands on."))]
    pub value: f64,
    #[default = "%"]
    #[schemars(description = "Suffix", extend("x-help" = "Unit after the number."))]
    pub suffix: String,
    #[schemars(description = "Prefix", extend("x-help" = "Unit before the number."))]
    pub prefix: String,
    #[schemars(description = "Decimals", range(min = 0, max = 6), extend("x-step" = 1, "x-help" = "Decimal places while counting and at rest.", "x-advanced" = true))]
    pub decimals: u32,
    #[default = "Satisfaction"]
    #[schemars(description = "Label", extend("x-help" = "Metric label."))]
    pub label: String,
    #[default = "Customer survey, last 30 days"]
    #[schemars(description = "Caption", extend("x-help" = "Line under the label (source, window, cohort)."))]
    pub caption: String,
    #[default = 4]
    #[schemars(description = "Change", range(min = -1000000, max = 1000000), extend("x-step" = 1, "x-help" = "Change against the previous period; lands after the number settles.", "x-show-if" = {"showDelta":["true"]}))]
    pub delta: i32,
    #[default = "%"]
    #[schemars(description = "Change suffix", extend("x-help" = "Unit on the delta chip.", "x-show-if" = {"showDelta":["true"]}))]
    pub delta_suffix: String,
    #[default = "#2DD4BF"]
    #[schemars(description = "Accent color", extend("x-help" = "Ring, suffix, and delta colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[default = true]
    #[schemars(description = "Show change")]
    pub show_delta: bool,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Ring full at", range(min = 0, max = 1000000000), extend("x-zero-label" = "Full ring", "x-step" = 1, "x-help" = "The ring fills to the value out of this. Leave empty for a full ring.", "type" = "integer"))]
    pub max: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MetricTicker {
    #[default(vec![
        MetricTickerMetric { label: "Sign-ups".into(), value: 12400.0, delta: Some("+18%".into()), trend: Some(vec![62.0, 71.0, 68.0, 84.0, 96.0, 112.0, 124.0]), ..Default::default() },
        MetricTickerMetric { label: "Avg. session".into(), value: 48.0, suffix: Some("min".into()), delta: Some("+6%".into()), trend: Some(vec![39.0, 41.0, 40.0, 43.0, 45.0, 44.0, 48.0]), ..Default::default() },
        MetricTickerMetric { delta: Some("-4%".into()), trend: Some(vec![36.0, 35.0, 37.0, 34.0, 33.0, 33.5, 32.0]), ..Default::default() },
    ])]
    #[schemars(description = "Metrics", extend("x-help" = "Metric cards: label, value, and optional from, prefix, suffix, delta, trend, color. Set from above value for a metric that counts down."))]
    pub metrics: Vec<MetricTickerMetric>,
    #[default = "Growth this quarter"]
    #[schemars(description = "Title", extend("x-help" = "Scene title."))]
    pub title: String,
    #[default = "Quarter recap"]
    #[schemars(description = "Small heading", extend("x-help" = "Short label above the title."))]
    pub eyebrow: String,
    #[default = 4]
    #[schemars(description = "Most cards", extend("x-help" = "Cards beyond this count are dropped rather than squeezed."))]
    pub max_cards: u32,
    #[schemars(description = "Accent color", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Value colour, overridable per metric. Leave empty to follow the theme."))]
    pub accent_color: String,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette: text, cards and the change colours."))]
    pub theme: DarkOrLight,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(description = "Width", extend("x-help" = "Overall width: label gutter, funnel and drop-off gutter."))]
    pub width: f64,
    #[default = 560.0]
    #[schemars(description = "Height", extend("x-help" = "Overall height. Bands split it evenly."))]
    pub height: f64,
    #[default = 10.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between bands.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 0.5]
    #[schemars(description = "Last bar opacity", extend("x-step" = 0.01, "x-help" = "Opacity of the last band; earlier bands ramp toward 1.", "x-advanced" = true))]
    pub tail_opacity: f64,
    #[default = true]
    #[schemars(description = "Show drop-off", extend("x-help" = "Drop-off chip between consecutive stages."))]
    pub show_dropoff: bool,
    #[schemars(description = "Show conversion", extend("x-help" = "Share of the first stage, printed beside each value."))]
    pub show_conversion: bool,
    #[default = 22]
    #[schemars(description = "Animation length", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Length of one band's wipe.", "x-advanced" = true))]
    pub duration_in_frames: u32,
    #[default = 12]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between one band and the next.", "x-advanced" = true))]
    pub stagger_in_frames: u32,
    #[default = "#e8b86d"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "#fafafa"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ink_color: String,
    #[default = "rgba(250,250,250,0.55)"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
    #[default = "#f472b6"]
    #[schemars(description = "Drop-off color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub dropoff_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Label width", range(min = 0, max = 2000), extend("x-zero-label" = "Auto", "x-step" = 10, "x-unit" = "px", "x-advanced" = true, "type" = "integer"))]
    pub label_width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DonutChart {
    #[default(vec![
        DonutChartSegment { label: "Direct".into(), value: 4820.0, ..Default::default() },
        DonutChartSegment { label: "Search".into(), value: 3140.0, ..Default::default() },
        DonutChartSegment { label: "Social".into(), value: 1960.0, ..Default::default() },
        DonutChartSegment::default(),
    ])]
    #[schemars(description = "Segments")]
    pub segments: Vec<DonutChartSegment>,
    #[default = 520.0]
    #[schemars(description = "Size", extend("x-help" = "All type scales off it."))]
    pub size: f64,
    #[default = 46.0]
    #[schemars(description = "Thickness", extend("x-help" = "Ring thickness."))]
    pub thickness: f64,
    #[default(vec!["#e8b86d".into(), "#2dd4bf".into(), "#f472b6".into(), "#8b8bf5".into()])]
    #[schemars(description = "Colors", length(min = 1))]
    pub colors: Vec<Colour>,
    #[default = true]
    #[schemars(description = "Show legend", extend("x-help" = "Legend rows beside the ring, arriving with their own slice."))]
    pub show_legend: bool,
    #[default = true]
    #[schemars(description = "Show total", extend("x-help" = "Running total in the hole."))]
    pub show_total: bool,
    #[default = "Sessions"]
    #[schemars(description = "Total label", extend("x-help" = "Caption under the total."))]
    pub total_label: String,
    #[default = 26]
    #[schemars(description = "Animation length", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Length of one segment's sweep.", "x-advanced" = true))]
    pub duration_in_frames: u32,
    #[default = 9]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between one segment and the next.", "x-advanced" = true))]
    pub stagger_in_frames: u32,
    #[default = "rgba(250,250,250,0.08)"]
    #[schemars(description = "Track color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub track_color: String,
    #[default = "rgba(250,250,250,0.58)"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
    #[default = "#fafafa"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ink_color: String,
    #[default = "#2dd4bf"]
    #[schemars(description = "Increase color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub up_color: String,
    #[default = "#f87171"]
    #[schemars(description = "Decrease color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub down_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonBars {
    #[default(vec![
        ComparisonBarsRow { label: "Setup time (min)".into(), before: 240.0, after: 96.0, ..Default::default() },
        ComparisonBarsRow { label: "Tickets closed / week".into(), before: 120.0, after: 310.0, ..Default::default() },
        ComparisonBarsRow { label: "Active teams".into(), before: 34.0, after: 88.0, ..Default::default() },
        ComparisonBarsRow::default(),
    ])]
    #[schemars(description = "Rows", extend("x-help" = "The delta is computed when omitted."))]
    pub rows: Vec<ComparisonBarsRow>,
    #[default = 1100.0]
    #[schemars(description = "Width", extend("x-help" = "Overall width, label column included."))]
    pub width: f64,
    #[default = 84.0]
    #[schemars(description = "Row height", extend("x-help" = "Height of one pair, both bars and their gap included.", "x-advanced" = true))]
    pub row_height: f64,
    #[default = 26.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between rows.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 280.0]
    #[schemars(description = "Label width", extend("x-help" = "Width reserved for the row labels.", "x-advanced" = true))]
    pub label_width: f64,
    #[default = "rgba(250,250,250,0.22)"]
    #[schemars(description = "Before color", extend("x-help" = "Baseline bar. Muted on purpose: it is the thing being beaten.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub before_color: String,
    #[default = "#e8b86d"]
    #[schemars(description = "After color", extend("x-help" = "New-value bar, and the colour of a rising delta.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub after_color: String,
    #[default = "#f472b6"]
    #[schemars(description = "Decrease color", extend("x-help" = "Delta colour when the change is a fall.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub down_color: String,
    #[default(vec!["Before".into(), "After".into()])]
    #[schemars(description = "Series labels", length(max = 2), extend("x-help" = "Legend above the rows: the before bar, then the after bar."))]
    pub series_labels: Vec<String>,
    #[default = true]
    #[schemars(description = "Show change", extend("x-help" = "Percentage-change chip at the end of the second bar."))]
    pub show_delta: bool,
    #[default = 30]
    #[schemars(description = "Animation length", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Length of one bar's growth.", "x-advanced" = true))]
    pub duration_in_frames: u32,
    #[default = 12]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between one row and the next.", "x-advanced" = true))]
    pub stagger_in_frames: u32,
    #[default = 6]
    #[schemars(description = "Pair offset", extend("x-unit" = "frames", "x-help" = "Frames the second bar trails the first by.", "x-advanced" = true))]
    pub pair_offset_in_frames: u32,
    #[default = "#0b0b10"]
    #[schemars(description = "Bar text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub bar_ink_color: String,
    #[default = "rgba(250,250,250,0.55)"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BarChartRace {
    #[default(vec![
        BarChartRaceSerie { label: "Mobile".into(), values: vec![42.0, 58.0, 66.0, 72.0, 78.0, 84.0], ..Default::default() },
        BarChartRaceSerie { label: "Web".into(), values: vec![18.0, 34.0, 57.0, 76.0, 92.0, 108.0], ..Default::default() },
        BarChartRaceSerie { label: "Desktop".into(), values: vec![30.0, 40.0, 47.0, 52.0, 57.0, 61.0], ..Default::default() },
        BarChartRaceSerie { label: "API".into(), values: vec![12.0, 22.0, 36.0, 50.0, 63.0, 79.0], ..Default::default() },
        BarChartRaceSerie { label: "Email".into(), values: vec![24.0, 28.0, 31.0, 35.0, 38.0, 42.0], ..Default::default() },
        BarChartRaceSerie { values: vec![8.0, 12.0, 17.0, 21.0, 26.0, 30.0], ..Default::default() },
    ])]
    #[schemars(description = "Series")]
    pub series: Vec<BarChartRaceSerie>,
    #[default(vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into(), "Q5".into(), "Q6".into()])]
    #[schemars(description = "Steps", extend("x-help" = "Keyframe captions, e.g. years. Shown as the running clock."))]
    pub steps: Vec<String>,
    #[default = 18]
    #[schemars(description = "Time per step", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Frames spent travelling between two keyframes.", "x-advanced" = true))]
    pub frames_per_step: u32,
    #[default = 6]
    #[schemars(description = "Visible rows", range(min = 1, max = 20), extend("x-step" = 1, "x-help" = "How many rows stay on the board. Below that, series fade out.", "x-advanced" = true))]
    pub visible_rows: u32,
    #[default = 1200.0]
    #[schemars(description = "Width", extend("x-help" = "Overall width, label column included."))]
    pub width: f64,
    #[default = 76.0]
    #[schemars(description = "Row height", extend("x-help" = "Bar height. Type scales off it.", "x-advanced" = true))]
    pub row_height: f64,
    #[default = 14.0]
    #[schemars(description = "Gap", extend("x-help" = "Space between rows.", "x-advanced" = true))]
    pub gap: f64,
    #[default = 240.0]
    #[schemars(description = "Label width", extend("x-help" = "Width reserved for the row labels.", "x-advanced" = true))]
    pub label_width: f64,
    #[default = true]
    #[schemars(description = "Show step label", extend("x-help" = "Large step caption in the bottom-right corner."))]
    pub show_step_label: bool,
    #[default(vec!["#e8b86d".into(), "#2dd4bf".into(), "#f472b6".into(), "#8b8bf5".into(), "#f59e0b".into(), "#5eead4".into()])]
    #[schemars(description = "Colors", length(min = 1))]
    pub colors: Vec<Colour>,
    #[default = "#fafafa"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ink_color: String,
    #[default = "#0b0b10"]
    #[schemars(description = "Bar text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub bar_ink_color: String,
    #[default = "rgba(250,250,250,0.55)"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GaugeDial {
    #[default = 78.0]
    #[schemars(description = "Value", extend("x-help" = "Target the needle sweeps to."))]
    pub value: f64,
    #[schemars(description = "Min", extend("x-help" = "Value at the start of the arc."))]
    pub min: f64,
    #[default = 100.0]
    #[schemars(description = "Max", extend("x-help" = "Value at the end of the arc."))]
    pub max: f64,
    #[default = 480.0]
    #[schemars(description = "Size", extend("x-help" = "Type scales off it."))]
    pub size: f64,
    #[default = 26.0]
    #[schemars(description = "Thickness", extend("x-help" = "Track and fill thickness."))]
    pub thickness: f64,
    #[default = 250]
    #[schemars(description = "Sweep angle", range(min = 10, max = 360), extend("x-step" = 1, "x-help" = "Total travel, centred on twelve o'clock. 180 gives a half-moon meter.", "x-advanced" = true))]
    pub sweep_in_degrees: u32,
    #[default = "Weekly goal"]
    #[schemars(description = "Label", extend("x-help" = "Caption under the readout."))]
    pub label: String,
    #[default = "%"]
    #[schemars(description = "Unit", extend("x-help" = "Suffix on the readout."))]
    pub unit: String,
    #[default = 9]
    #[schemars(description = "Tick count", range(min = 0, max = 60), extend("x-step" = 1, "x-help" = "Tick marks around the arc. 0 hides them.", "x-advanced" = true))]
    pub tick_count: u32,
    #[default = 44]
    #[schemars(description = "Animation length", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Length of the sweep.", "x-advanced" = true))]
    pub duration_in_frames: u32,
    #[default = "#e8b86d"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "rgba(250,250,250,0.10)"]
    #[schemars(description = "Track color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub track_color: String,
    #[default = "#fafafa"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ink_color: String,
    #[default = "rgba(250,250,250,0.58)"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub label_color: String,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<StatCard>(
            element(
                ElementKind::Component,
                "stat-card",
                "Stat card",
                "One big number counts up while a ring fills, then its label and the change since last period appear.",
                C::Data,
                123,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/stat-card", "MIT")
                .look(NodeLook::of("lucide:circle-gauge")),
        ),
        meta::<MetricTicker>(
            element(
                ElementKind::Component,
                "metric-ticker",
                "Metric cards",
                "Key numbers on cards that count up, each with its change and a small trend line.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/metric-ticker", "MIT")
                .look(NodeLook::of("lucide:trending-up")),
        ),
        meta::<FunnelChart>(
            element(
                ElementKind::Component,
                "funnel-chart",
                "Funnel chart",
                "A bar for each step of a funnel, narrowing to show how many people drop off.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/funnel-chart", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:funnel")),
        ),
        meta::<DonutChart>(
            element(
                ElementKind::Component,
                "donut-chart",
                "Donut chart",
                "A ring chart that fills segment by segment, with a legend and a total.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/donut-chart", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:chart-pie")),
        ),
        meta::<ComparisonBars>(
            element(
                ElementKind::Component,
                "comparison-bars",
                "Before and after bars",
                "Pairs of bars comparing before and after, with the change called out.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/comparison-bars", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:chart-bar")),
        ),
        meta::<BarChartRace>(
            element(
                ElementKind::Component,
                "bar-chart-race",
                "Bar chart race",
                "Ranked bars that grow and swap places as the values change over time.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/bar-chart-race", "MIT")
                .design(1920, 1080)
                .look(NodeLook::of("lucide:chart-bar-decreasing")),
        ),
        meta::<GaugeDial>(
            element(
                ElementKind::Component,
                "gauge-dial",
                "Gauge",
                "A dial whose needle sweeps up to a value.",
                C::Data,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/gauge-dial", "MIT")
                .look(NodeLook::of("lucide:gauge")),
        ),
    ]
}
