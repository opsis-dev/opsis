use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::NumberFormat;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ComparisonBarsRow {
    #[setting("Label", default = "Manual steps")]
    pub label: String,
    #[setting("Before", default = 62.0)]
    pub before: f64,
    #[setting("After", default = 18.0)]
    pub after: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
}

#[settings(export)]
pub struct ComparisonBars {
    #[setting(
        "Rows",
        default = vec![
            ComparisonBarsRow {
                label: "Setup time (min)".into(),
                before: 240.0,
                after: 96.0,
                ..Default::default()
            },
            ComparisonBarsRow {
                label: "Tickets closed / week".into(),
                before: 120.0,
                after: 310.0,
                ..Default::default()
            },
            ComparisonBarsRow {
                label: "Active teams".into(),
                before: 34.0,
                after: 88.0,
                ..Default::default()
            },
            ComparisonBarsRow::default(),
        ],
        help = "The delta is computed when omitted."
    )]
    pub rows: Vec<ComparisonBarsRow>,
    #[setting(
        "Width",
        default = 1100.0,
        help = "Overall width, label column included."
    )]
    pub width: f64,
    #[setting(
        "Row height",
        default = 84.0,
        help = "Height of one pair, both bars and their gap included.",
        advanced
    )]
    pub row_height: f64,
    #[setting("Gap", default = 26.0, help = "Space between rows.", advanced)]
    pub gap: f64,
    #[setting(
        "Label width",
        default = 280.0,
        help = "Width reserved for the row labels.",
        advanced
    )]
    pub label_width: f64,
    #[setting(
        "Before color",
        default = "rgba(250,250,250,0.22)",
        help = "Baseline bar. Muted on purpose: it is the thing being beaten.",
        check = colour
    )]
    pub before_color: String,
    #[setting(
        "After color",
        default = "#e8b86d",
        help = "New-value bar, and the colour of a rising delta.",
        check = colour
    )]
    pub after_color: String,
    #[setting(
        "Decrease color",
        default = "#f472b6",
        help = "Delta colour when the change is a fall.",
        check = colour,
        advanced
    )]
    pub down_color: String,
    #[setting(
        "Series labels",
        default = vec!["Before".into(), "After".into()],
        length(max = 2),
        help = "Legend above the rows: the before bar, then the after bar."
    )]
    pub series_labels: Vec<String>,
    #[setting(
        "Show change",
        default = true,
        help = "Percentage-change chip at the end of the second bar."
    )]
    pub show_delta: bool,
    #[setting(
        "Animation length",
        default = 30,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one bar's growth.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 12,
        unit = "frames",
        help = "Frames between one row and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting(
        "Pair offset",
        default = 6,
        unit = "frames",
        help = "Frames the second bar trails the first by.",
        advanced
    )]
    pub pair_offset_in_frames: u32,
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
        default = 28.0,
        range(min = 0, max = 50),
        unit = "%",
        advanced
    )]
    pub corner_radius: f64,
    #[setting("Show before value", advanced)]
    pub show_before_value: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<ComparisonBars>(
        element(
            ElementKind::Component,
            "comparison-bars",
            "Before and after bars",
            "Pairs of bars comparing before and after, with the change called out.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar")),
    )
}
