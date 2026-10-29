use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::NumberFormat;
use crate::elements::theme::{Theme, ThemeMode, data_story_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
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

#[settings(export)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Prefix", "type" = "string")]
    pub prefix: Option<String>,
}

#[settings(export)]
pub struct DataStoryStep {
    #[setting("Title", default = "Next step")]
    pub title: String,
    #[setting("Description", default = "Close with one clear call to action.")]
    pub description: String,
}

#[settings(export)]
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
                ..Default::default()
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
        default = Theme::of(ThemeMode::Dark),
        transform = data_story_themes,
        widget = "theme"
    )]
    pub theme: Theme,
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
    #[setting("Timeline color", default = "#f59e0b", check = colour, advanced)]
    pub timeline_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        advanced
    )]
    pub background_color: String,
    #[setting(
        "Chart scale",
        default = true,
        help = "The value axis under the bars.",
        advanced
    )]
    pub show_scale: bool,
    #[setting(
        "Highlighted bar",
        zero_label = "None",
        help = "The label of the bar drawn in the accent color.",
        advanced
    )]
    pub highlight_bar: String,
    #[setting("Chart subtitle", advanced)]
    pub chart_subtitle: String,
    #[setting("Numbers", default = NumberFormat::Compact, advanced)]
    pub number_format: NumberFormat,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        help = "Scales every scene's headline together.",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Logo",
        widget = "asset",
        accept = "image/*",
        zero_label = "None",
        help = "Shown above the opening title."
    )]
    pub logo_src: String,
    #[setting("Closing web address", zero_label = "None")]
    pub cta_url: String,
    #[setting("Closing subtitle", zero_label = "None", advanced)]
    pub cta_subtitle: String,
    #[setting("Uppercase small headings", default = true, advanced)]
    pub eyebrow_caps: bool,
    #[setting(
        "Glow",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.05,
        help = "How strongly each scene's accent glows behind it.",
        advanced
    )]
    pub glow: f64,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
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
