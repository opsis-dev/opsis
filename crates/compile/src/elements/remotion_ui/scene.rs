use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{ChangeKind, DarkOrLight};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStoryStep {
    #[default = "Next step"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Close with one clear call to action."]
    #[schemars(description = "Description")]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTableRow {
    #[default = "New feature"]
    #[schemars(description = "Label")]
    pub label: String,
    #[default(vec!["yes".into(), "yes".into(), "yes".into()])]
    #[schemars(description = "Cells")]
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntryChange {
    #[default(ChangeKind::Removed)]
    #[schemars(description = "Kind")]
    pub kind: ChangeKind,
    #[default = "Legacy waveform atom"]
    #[schemars(description = "Text")]
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWallLogo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Name and glyph", "type" = "string"))]
    pub image: Option<String>,
    #[default = "Marrow"]
    #[schemars(description = "Name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Color", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Glyph", extend("type" = "string"))]
    pub glyph: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStory {
    #[default = "Quarterly reach by channel"]
    #[schemars(description = "Title", extend("x-help" = "Opening hook headline (auto-fit)."))]
    pub title: String,
    #[default = "Short-form climbed while long-form held attention"]
    #[schemars(description = "Subtitle", extend("x-help" = "Supporting line under the hook."))]
    pub subtitle: String,
    #[default(vec![
        DataStoryBarData { label: "Mobile".into(), value: 124000.0, delta: Some("+32%".into()), ..Default::default() },
        DataStoryBarData { label: "Web".into(), value: 82000.0, delta: Some("+11%".into()), ..Default::default() },
        DataStoryBarData { label: "Desktop".into(), value: 64000.0, delta: Some("+4%".into()), ..Default::default() },
        DataStoryBarData { delta: Some("-6%".into()), ..Default::default() },
    ])]
    #[schemars(description = "Chart data", extend("x-help" = "Bar chart data."))]
    pub bar_data: Vec<DataStoryBarData>,
    #[default(vec![
        DataStoryMetric { label: "Sign-ups".into(), value: 12400.0, delta: "+18%".into(), trend: vec![62.0, 71.0, 68.0, 84.0, 96.0, 112.0, 124.0], ..Default::default() },
        DataStoryMetric { label: "Avg. session".into(), value: 48.0, suffix: Some("min".into()), delta: "+6%".into(), trend: vec![39.0, 41.0, 40.0, 43.0, 45.0, 44.0, 48.0] },
        DataStoryMetric { trend: vec![36.0, 35.0, 37.0, 34.0, 33.0, 33.5, 32.0], ..Default::default() },
    ])]
    #[schemars(description = "Metrics", extend("x-help" = "Metric cards."))]
    pub metrics: Vec<DataStoryMetric>,
    #[default(vec![
        DataStoryStep { title: "Hook".into(), description: "Lead with the sharpest insight.".into() },
        DataStoryStep { title: "Proof".into(), description: "Show the chart and the numbers.".into() },
        DataStoryStep::default(),
    ])]
    #[schemars(description = "Steps", extend("x-help" = "Context steps."))]
    pub steps: Vec<DataStoryStep>,
    #[default = "Views by format"]
    #[schemars(description = "Chart title", extend("x-help" = "Headline on the bar chart scene."))]
    pub chart_title: String,
    #[default = "Signals that mattered"]
    #[schemars(description = "Metrics title", extend("x-help" = "Headline on the metric ticker scene."))]
    pub metrics_title: String,
    #[default = "How we read the quarter"]
    #[schemars(description = "Timeline title", extend("x-help" = "Headline on the timeline scene."))]
    pub timeline_title: String,
    #[default = "Short-form drove reach; long-form kept attention."]
    #[schemars(description = "Takeaway", extend("x-help" = "Takeaway quote in the insight bumper."))]
    pub insight: String,
    #[default = "Takeaway"]
    #[schemars(description = "Takeaway heading", extend("x-help" = "Eyebrow above the insight quote."))]
    pub insight_eyebrow: String,
    #[default = "Turn your data into motion"]
    #[schemars(description = "Closing title", extend("x-help" = "End card headline (separate from hook)."))]
    pub cta_title: String,
    #[default = "Get started"]
    #[schemars(description = "Button label", extend("x-help" = "End card CTA pill label."))]
    pub cta_label: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Light or dark surfaces for every scene."))]
    pub theme: DarkOrLight,
    #[default = "#e8b86d"]
    #[schemars(description = "Accent color", extend("x-help" = "Title and closing card.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = "#2dd4bf"]
    #[schemars(description = "Data color", extend("x-help" = "Bars, key numbers and the takeaway.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub data_color: String,
    #[default = "#f59e0b"]
    #[schemars(description = "Timeline color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub timeline_color: String,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoReveal {
    #[default = "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z"]
    #[schemars(description = "Logo outline", extend("x-widget" = "code", "x-help" = "Logo path, or paths drawn in sequence."))]
    pub path_d: String,
    #[default = "Acme"]
    #[schemars(description = "Name", extend("x-help" = "Brand name rising after the mark completes."))]
    pub wordmark: String,
    #[default = "Ship your next release faster"]
    #[schemars(description = "Tagline", extend("x-help" = "Supporting line, entering last."))]
    pub tagline: String,
    #[default = "#e8b86d"]
    #[schemars(description = "Line color", extend("x-help" = "Mark stroke color.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub stroke: String,
    #[default = "#080810"]
    #[schemars(description = "Background color", extend("x-help" = "Scene background.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[default = "#f4f4f5"]
    #[schemars(description = "Text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub text_color: String,
    #[schemars(description = "Outline area", extend("x-zero-label" = "Fit", "x-help" = "Leave empty to fit the outline.", "x-advanced" = true))]
    pub view_box: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Logo size", range(min = 20, max = 2000), extend("x-zero-label" = "Auto", "x-step" = 10, "x-unit" = "px", "type" = "integer"))]
    pub size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Line width", range(min = 1, max = 40), extend("x-zero-label" = "Auto", "x-step" = 0.5, "x-unit" = "px", "x-advanced" = true, "type" = "number"))]
    pub stroke_width: Option<f64>,
    #[schemars(description = "Fill colour", extend("x-widget" = "color", "x-zero-label" = "None"))]
    pub fill: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PricingCard {
    #[default = "Studio"]
    #[schemars(description = "Plan name", extend("x-help" = "Tier name across the top of the card."))]
    pub tier: String,
    #[default = 32.0]
    #[schemars(description = "Price", extend("x-help" = "The number the price rolls up to."))]
    pub price: f64,
    #[default = "$"]
    #[schemars(description = "Currency", extend("x-help" = "Symbol set ahead of the number."))]
    pub currency: String,
    #[default = "/mo"]
    #[schemars(description = "Billing period", extend("x-help" = "Cadence after the price: \"/mo\", \"per seat\", anything short."))]
    pub period: String,
    #[default = "Most picked"]
    #[schemars(description = "Badge", extend("x-help" = "Chip beside the tier name. Omit to drop it."))]
    pub badge: String,
    #[default = "Billed annually. Cancel whenever."]
    #[schemars(description = "Small print", extend("x-help" = "Line under the price."))]
    pub note: String,
    #[default(vec!["2,000 render minutes a month".into(), "4K exports, no watermark".into(), "9 team seats".into(), "Render API and webhooks".into(), "Priority render queue".into()])]
    #[schemars(description = "Features", extend("x-help" = "Ticked lines under the price, revealed in order."))]
    pub features: Vec<String>,
    #[default = "Start rendering"]
    #[schemars(description = "Button label", extend("x-help" = "Button text. Omit to drop the button."))]
    pub cta_label: String,
    #[default = 0.42]
    #[schemars(description = "Price at (seconds)", extend("x-step" = 0.01, "x-help" = "Second the price starts rolling.", "x-advanced" = true))]
    pub price_at_seconds: f64,
    #[default = 0.9]
    #[schemars(description = "Roll (seconds)", range(min = 0.05, max = 10), extend("x-step" = 0.01, "x-help" = "How long the roll takes.", "x-advanced" = true))]
    pub roll_seconds: f64,
    #[default = 1.05]
    #[schemars(description = "Features at (seconds)", extend("x-step" = 0.1, "x-help" = "Second the first feature ticks in.", "x-advanced" = true))]
    pub features_at_seconds: f64,
    #[default = 0.22]
    #[schemars(description = "Feature stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between features. The CTA lands 0.16s after the last one.", "x-advanced" = true))]
    pub feature_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Badge, ticks, card border wash and the CTA fill.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Old price", range(min = 0, max = 1000000), extend("x-zero-label" = "None", "x-step" = 1, "x-help" = "Shown struck out next to the price.", "type" = "integer"))]
    pub was_price: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ComparisonTable {
    #[default(vec!["Free".into(), "Studio".into(), "Scale".into()])]
    #[schemars(description = "Columns", length(min = 1), extend("x-help" = "Column headings. The feature-label column is unnamed and sits ahead of these."))]
    pub columns: Vec<String>,
    #[default(vec![
        ComparisonTableRow { label: "Render minutes".into(), cells: vec!["60".into(), "2,000".into(), "Unlimited".into()] },
        ComparisonTableRow { label: "1080p exports".into(), ..Default::default() },
        ComparisonTableRow { label: "4K exports".into(), cells: vec!["no".into(), "yes".into(), "yes".into()] },
        ComparisonTableRow { label: "Team seats".into(), cells: vec!["1".into(), "9".into(), "Unlimited".into()] },
        ComparisonTableRow { label: "Render API".into(), cells: vec!["no".into(), "yes".into(), "yes".into()] },
        ComparisonTableRow { label: "Priority queue".into(), cells: vec!["no".into(), "no".into(), "yes".into()] },
    ])]
    #[schemars(description = "Rows", extend("x-help" = "Type yes or no in a cell for a tick or a dash."))]
    pub rows: Vec<ComparisonTableRow>,
    #[default = "What you get"]
    #[schemars(description = "Title", extend("x-help" = "Heading above the table. Omit to drop it."))]
    pub title: String,
    #[default = 1]
    #[schemars(description = "Highlighted column", extend("x-help" = "Column kept lit throughout. Pass -1 for a table with no favourite."))]
    pub highlight_column: u32,
    #[default = "Most picked"]
    #[schemars(description = "Highlight badge", extend("x-help" = "Chip pinned beside the title, above the highlighted column."))]
    pub highlight_label: String,
    #[default = 0.5]
    #[schemars(description = "Start at (seconds)", extend("x-step" = 0.01, "x-help" = "Second the first row wipes in.", "x-advanced" = true))]
    pub start_at_seconds: f64,
    #[default = 0.26]
    #[schemars(description = "Row stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between rows. Cells inside a row trail their label by another 0.07s each.", "x-advanced" = true))]
    pub row_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Highlight band, its header, and the ticks inside it.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangelogEntry {
    #[default(vec![
        ChangelogEntryChange { kind: ChangeKind::Added, text: "Caption presets for nine languages".into() },
        ChangelogEntryChange { kind: ChangeKind::Added, text: "Render API webhooks on job completion".into() },
        ChangelogEntryChange { kind: ChangeKind::Fixed, text: "Retry backoff no longer stalls the queue".into() },
        ChangelogEntryChange { kind: ChangeKind::Changed, text: "Theme tokens moved to a single source".into() },
        ChangelogEntryChange::default(),
    ])]
    #[schemars(description = "Changes", extend("x-help" = "A kind (added, fixed, changed or removed) and the sentence beside it."))]
    pub changes: Vec<ChangelogEntryChange>,
    #[default = "v2.4.0"]
    #[schemars(description = "Version", extend("x-help" = "Version string, set at display size."))]
    pub version: String,
    #[default = "16 August"]
    #[schemars(description = "Date", extend("x-help" = "Date beside the version."))]
    pub date: String,
    #[default = "Captions, webhooks, and a queue that stops sulking."]
    #[schemars(description = "Summary", extend("x-help" = "Headline under the version. Omit for releases without one."))]
    pub summary: String,
    #[default = 0.66]
    #[schemars(description = "Start at (seconds)", extend("x-step" = 0.01, "x-help" = "Second the first change row arrives.", "x-advanced" = true))]
    pub start_at_seconds: f64,
    #[default = 0.28]
    #[schemars(description = "Stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between rows. Each sentence trails its own tag by 0.08s.", "x-advanced" = true))]
    pub stagger_seconds: f64,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
    #[schemars(description = "Added colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub kind_added: String,
    #[schemars(description = "Fixed colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub kind_fixed: String,
    #[schemars(description = "Changed colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub kind_changed: String,
    #[schemars(description = "Removed colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub kind_removed: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWall {
    #[default(vec![
        LogoWallLogo { name: "Northstar".into(), color: Some("#E8B86D".into()), ..Default::default() },
        LogoWallLogo { name: "Halcyon".into(), color: Some("#7DD3E8".into()), ..Default::default() },
        LogoWallLogo { name: "Fernweh".into(), color: Some("#9BD4A0".into()), ..Default::default() },
        LogoWallLogo { name: "Orbital".into(), color: Some("#C99BE8".into()), ..Default::default() },
        LogoWallLogo { name: "Lumen".into(), color: Some("#8FB8F0".into()), ..Default::default() },
        LogoWallLogo { name: "Tidewater".into(), color: Some("#E89B9B".into()), ..Default::default() },
        LogoWallLogo { name: "Kestrel".into(), color: Some("#E8B86D".into()), ..Default::default() },
        LogoWallLogo { color: Some("#7DD3E8".into()), ..Default::default() },
    ])]
    #[schemars(description = "Logos", extend("x-help" = "Wordmark, brand colour and the glyph in the mark beside it, or a logo image in their place."))]
    pub logos: Vec<LogoWallLogo>,
    #[default = "Trusted by"]
    #[schemars(description = "Small heading", extend("x-help" = "Small line above the heading. Omit to drop it."))]
    pub eyebrow: String,
    #[default = "Teams that ship with us"]
    #[schemars(description = "Title", extend("x-help" = "Heading above the wall. Omit to drop it."))]
    pub title: String,
    #[default = 4]
    #[schemars(description = "Columns", range(min = 1), extend("x-step" = 1, "x-help" = "Tiles per row."))]
    pub columns: u32,
    #[default = 0.32]
    #[schemars(description = "Start at (seconds)", extend("x-step" = 0.01, "x-help" = "Second the first tile arrives.", "x-advanced" = true))]
    pub start_at_seconds: f64,
    #[default = 0.06]
    #[schemars(description = "Arrive stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between tiles arriving, in reading order. Deliberately fast.", "x-advanced" = true))]
    pub arrive_stagger_seconds: f64,
    #[default = 0.24]
    #[schemars(description = "Stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between diagonals of the colour sweep.", "x-advanced" = true))]
    pub stagger_seconds: f64,
    #[default = 0.6]
    #[schemars(description = "Colour fade (seconds)", range(min = 0.05, max = 10), extend("x-step" = 0.01, "x-help" = "How long one tile takes to go from grey to brand colour.", "x-advanced" = true))]
    pub warm_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Eyebrow, and the fallback for logos with no colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct EndCard {
    #[default = "Ready to try it?"]
    #[schemars(description = "Title", extend("x-help" = "Closing headline."))]
    pub title: String,
    #[default = "Start free. No credit card needed."]
    #[schemars(description = "Subtitle", extend("x-help" = "Line under the title."))]
    pub subtitle: String,
    #[schemars(description = "Small heading", extend("x-help" = "Chip above the title."))]
    pub eyebrow: String,
    #[default = "Start free trial"]
    #[schemars(description = "Button label", extend("x-help" = "Button label. Omit to end on the title alone."))]
    pub cta: String,
    #[default = "example.com/signup"]
    #[schemars(description = "Address", extend("x-help" = "Address typed under the button."))]
    pub url: String,
    #[default(vec!["@acme".into(), "acme.example/blog".into()])]
    #[schemars(description = "Handles", extend("x-help" = "Handles or channels listed along the foot."))]
    pub handles: Vec<String>,
    #[schemars(description = "Logo", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub logo_src: String,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Button, pulse, and eyebrow colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Logo size", range(min = 10, max = 600), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "type" = "integer"))]
    pub logo_size: Option<u32>,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<DataStory>(
            element(
                ElementKind::Component,
                "data-story",
                "Data story",
                "A whole data story: title, bar chart, key numbers, timeline, takeaway and a closing call to action.",
                C::Scene,
                420,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/data-story", "MIT")
                .look(NodeLook::of("lucide:chart-area")),
        ),
        meta::<LogoReveal>(
            element(
                ElementKind::Component,
                "logo-reveal",
                "Logo reveal",
                "Draws your logo outline, then brings in the name and a tagline.",
                C::Scene,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/logo-reveal", "MIT")
                .look(NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw)),
        ),
        meta::<PricingCard>(
            element(
                ElementKind::Component,
                "pricing-card",
                "Pricing card",
                "A pricing plan card: the price rolls in, then each feature is ticked off.",
                C::Scene,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/pricing-card", "MIT")
                .look(NodeLook::of("lucide:badge-dollar-sign")),
        ),
        meta::<ComparisonTable>(
            element(
                ElementKind::Component,
                "comparison-table",
                "Plan comparison table",
                "A table comparing plans feature by feature, with one plan highlighted.",
                C::Scene,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/comparison-table", "MIT")
                .look(NodeLook::of("lucide:table-2")),
        ),
        meta::<ChangelogEntry>(
            element(
                ElementKind::Component,
                "changelog-entry",
                "Changelog",
                "A release note: version and date, then each change appears with its own tag.",
                C::Scene,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/changelog-entry", "MIT")
                .look(NodeLook::of("lucide:rotate-ccw-clock")),
        ),
        meta::<LogoWall>(
            element(
                ElementKind::Component,
                "logo-wall",
                "Logo wall",
                "A grid of customer logos that light up from grey to colour.",
                C::Scene,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/logo-wall", "MIT")
                .look(NodeLook::of("lucide:layout-grid")),
        ),
        meta::<EndCard>(
            element(
                ElementKind::Component,
                "end-card",
                "End card",
                "A closing card with a title, a button, your web address and social handles.",
                C::Scene,
                150,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/end-card", "MIT")
                .look(NodeLook::of("lucide:flag")),
        ),
    ]
}
