use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedBarChartBar {
    #[setting("Label")]
    pub label: String,
    #[setting("Value", default = 50, range(min = 0, max = 1000000), step = 1)]
    pub value: u32,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimatedBarChart {
    #[setting("Bar color", default = "#0ea5e9", check = colour)]
    pub bar_color: String,
    #[setting("Gap", default = 16, range(min = 0, max = 80), step = 2)]
    pub gap: u32,
    #[setting(
        "Stagger",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub stagger_frames: f64,
    #[setting(
        "Bars",
        default = vec![
            AnimatedBarChartBar { value: 35, ..Default::default() },
            AnimatedBarChartBar { value: 60, ..Default::default() },
            AnimatedBarChartBar { value: 45, ..Default::default() },
            AnimatedBarChartBar { value: 80, ..Default::default() },
            AnimatedBarChartBar { value: 55, ..Default::default() },
            AnimatedBarChartBar { value: 70, ..Default::default() },
            AnimatedBarChartBar { value: 90, ..Default::default() },
            AnimatedBarChartBar { value: 65, ..Default::default() },
        ],
        length(min = 1, max = 60),
        help = "Leave every label empty to draw the bars without labels."
    )]
    pub bars: Vec<AnimatedBarChartBar>,
    #[setting(
        "Width",
        default = 1000,
        range(min = 200, max = 1920),
        step = 10,
        advanced
    )]
    pub width: u32,
    #[setting(
        "Height",
        default = 500,
        range(min = 100, max = 1080),
        step = 10,
        advanced
    )]
    pub height: u32,
    #[setting("Baseline colour", default = "#27272a", check = colour)]
    pub baseline_color: String,
    #[setting("Label colour", default = "#a1a1aa", check = colour)]
    pub label_color: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<AnimatedBarChart>(
        element(
            ElementKind::Component,
            "animated-bar-chart",
            "Animated bar chart",
            "Bars spring up from the baseline in a staggered cascade.",
            ElementCategory::Data,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-column").moves(NodeMotion::Grow)),
    )
}
