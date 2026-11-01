use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::NumberFormat;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct FunnelChartStage {
    #[setting("Label", default = "Upgraded")]
    pub label: String,
    #[setting("Value", default = 1750, range(min = 0, max = 1_000_000_000), step = 1)]
    pub value: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct FunnelChart {
    #[setting(
        "Stages",
        default = vec![
            FunnelChartStage { label: "Visited the site".into(), value: 42800, ..Default::default() },
            FunnelChartStage { label: "Signed up".into(), value: 18600, ..Default::default() },
            FunnelChartStage { label: "Created a project".into(), value: 9400, ..Default::default() },
            FunnelChartStage { label: "Invited a teammate".into(), value: 4100, ..Default::default() },
            FunnelChartStage::default(),
        ],
        length(min = 1)
    )]
    pub stages: Vec<FunnelChartStage>,
    #[setting(
        "Width",
        default = 1100.0,
        help = "Overall width: label gutter, funnel and drop-off gutter."
    )]
    pub width: f64,
    #[setting(
        "Height",
        default = 560.0,
        help = "Overall height. Bands split it evenly."
    )]
    pub height: f64,
    #[setting("Gap", default = 10.0, help = "Space between bands.", advanced)]
    pub gap: f64,
    #[setting(
        "Last bar opacity",
        default = 0.5,
        step = 0.01,
        help = "Opacity of the last band; earlier bands ramp toward 1.",
        advanced
    )]
    pub tail_opacity: f64,
    #[setting(
        "Show drop-off",
        default = true,
        help = "Drop-off chip between consecutive stages."
    )]
    pub show_dropoff: bool,
    #[setting(
        "Show conversion",
        help = "Share of the first stage, printed beside each value."
    )]
    pub show_conversion: bool,
    #[setting(
        "Animation length",
        default = 22,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one band's wipe.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 12,
        unit = "frames",
        help = "Frames between one band and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Color", default = "#e8b86d", check = colour)]
    pub color: String,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.55)", check = colour, advanced)]
    pub label_color: String,
    #[setting("Drop-off color", default = "#f472b6", check = colour, advanced)]
    pub dropoff_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Label width",
        range(min = 0, max = 2000),
        zero_label = "Auto",
        step = 10,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub label_width: Option<u32>,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting("Drop-off heading", default = "Drop-off", advanced)]
    pub dropoff_heading: String,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<FunnelChart>(
        element(
            ElementKind::Component,
            "funnel-chart",
            "Funnel chart",
            "A bar for each step of a funnel, narrowing to show how many people drop off.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:funnel")),
    )
}
