use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, Corner, NumberFormat};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct BarChartRaceSerie {
    #[setting("Label", default = "Partners")]
    pub label: String,
    #[setting("Values")]
    pub values: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct BarChartRace {
    #[setting(
        "Series",
        default = vec![
            BarChartRaceSerie {
                label: "Mobile".into(),
                values: vec![42.0, 58.0, 66.0, 72.0, 78.0, 84.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Web".into(),
                values: vec![18.0, 34.0, 57.0, 76.0, 92.0, 108.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Desktop".into(),
                values: vec![30.0, 40.0, 47.0, 52.0, 57.0, 61.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "API".into(),
                values: vec![12.0, 22.0, 36.0, 50.0, 63.0, 79.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Email".into(),
                values: vec![24.0, 28.0, 31.0, 35.0, 38.0, 42.0],
                ..Default::default()
            },
            BarChartRaceSerie { values: vec![8.0, 12.0, 17.0, 21.0, 26.0, 30.0], ..Default::default() },
        ]
    )]
    pub series: Vec<BarChartRaceSerie>,
    #[setting(
        "Steps",
        default = vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into(), "Q5".into(), "Q6".into()],
        help = "Keyframe captions, e.g. years. Shown as the running clock."
    )]
    pub steps: Vec<String>,
    #[setting(
        "Time per step",
        default = 18,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Frames spent travelling between two keyframes."
    )]
    pub frames_per_step: u32,
    #[setting(
        "Visible rows",
        default = 6,
        range(min = 1, max = 20),
        step = 1,
        help = "How many rows stay on the board. Below that, series fade out.",
        advanced
    )]
    pub visible_rows: u32,
    #[setting(
        "Width",
        default = 1200.0,
        help = "Overall width, label column included."
    )]
    pub width: f64,
    #[setting(
        "Row height",
        default = 76.0,
        help = "Bar height. Type scales off it.",
        advanced
    )]
    pub row_height: f64,
    #[setting("Gap", default = 14.0, help = "Space between rows.", advanced)]
    pub gap: f64,
    #[setting(
        "Label width",
        default = 240.0,
        help = "Width reserved for the row labels.",
        advanced
    )]
    pub label_width: f64,
    #[setting(
        "Show step label",
        default = true,
        help = "Large step caption in the bottom-right corner."
    )]
    pub show_step_label: bool,
    #[setting(
        "Colors",
        default = vec![
            "#e8b86d".into(),
            "#2dd4bf".into(),
            "#f472b6".into(),
            "#8b8bf5".into(),
            "#f59e0b".into(),
            "#5eead4".into(),
        ],
        length(min = 1)
    )]
    pub colors: Vec<Colour>,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Bar text color", default = "#0b0b10", check = colour, advanced)]
    pub bar_ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.55)", check = colour, advanced)]
    pub label_color: String,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting(
        "Corner rounding",
        default = 22.0,
        range(min = 0, max = 50),
        unit = "%",
        advanced
    )]
    pub corner_radius: f64,
    #[setting(
        "Step label position",
        default = Corner::BottomRight,
        advanced
    )]
    pub step_label_position: Corner,
    #[setting(
        "Step label size",
        default = 90.0,
        range(min = 20, max = 300),
        unit = "%",
        advanced
    )]
    pub step_label_size: f64,
    #[setting(
        "Step label opacity",
        default = 22.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub step_label_opacity: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<BarChartRace>(
        element(
            ElementKind::Component,
            "bar-chart-race",
            "Bar chart race",
            "Ranked bars that grow and swap places as the values change over time.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar-decreasing")),
    )
}
