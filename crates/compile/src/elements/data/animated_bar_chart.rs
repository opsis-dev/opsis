use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AnimatedBarChartBar {
    #[setting("Label")]
    pub label: String,
    #[setting("Value", default = 50, range(min = 0, max = 1000000), step = 1)]
    pub value: u32,
}

#[settings(export)]
pub struct AnimatedBarChart {
    #[setting("Bar color", default = "#0ea5e9", check = colour)]
    pub bar_color: String,
    #[setting("Gap", default = 16, range(min = 0, max = 80), step = 2, advanced)]
    pub gap: u32,
    #[setting(
        "Stagger",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
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
    #[setting("Width", default = 1000, range(min = 200, max = 1920), step = 10)]
    pub width: u32,
    #[setting("Height", default = 500, range(min = 100, max = 1080), step = 10)]
    pub height: u32,
    #[setting("Baseline colour", default = "#27272a", check = colour, advanced)]
    pub baseline_color: String,
    #[setting("Label colour", default = "#a1a1aa", check = colour, advanced)]
    pub label_color: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Corner radius",
        default = 6,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Glow",
        default = 33.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
    #[setting(
        "Label size",
        default = 16,
        range(min = 8, max = 64),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_size: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
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
