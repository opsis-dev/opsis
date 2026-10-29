use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AnimatedLineChart {
    #[setting("Stroke color", default = "#22c55e", check = colour)]
    pub stroke_color: String,
    #[setting("Stroke width", default = 4, range(min = 1, max = 16), step = 1)]
    pub stroke_width: u32,
    #[setting("Grid color", default = "#27272a", check = colour, advanced)]
    pub grid_color: String,
    #[setting("Show leading dot", default = true, advanced)]
    pub show_dot: bool,
    #[setting(
        "Values",
        default = vec![12.0, 19.0, 8.0, 15.0, 22.0, 18.0, 28.0, 25.0, 32.0],
        length(min = 2),
        help = "Paste a column from a spreadsheet to fill the list."
    )]
    pub data: Vec<f64>,
    #[setting("Width", default = 1000, range(min = 200, max = 3840), step = 10)]
    pub width: u32,
    #[setting("Height", default = 500, range(min = 100, max = 2160), step = 10)]
    pub height: u32,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Start at zero", advanced)]
    pub start_at_zero: bool,
    #[setting(
        "Finish drawing at",
        default = 85.0,
        range(min = 10, max = 100),
        unit = "%",
        advanced
    )]
    pub finish_at: f64,
    #[setting("Grid rows", default = 4, range(min = 0, max = 20), step = 1, advanced)]
    pub grid_rows: u32,
    #[setting("Show vertical grid lines", default = true, advanced)]
    pub show_columns: bool,
    #[setting(
        "Glow",
        default = 33.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
}

pub fn definition() -> ElementMeta {
    meta::<AnimatedLineChart>(
        element(
            ElementKind::Component,
            "animated-line-chart",
            "Animated line chart",
            "Line chart whose SVG path draws on from left to right with a leading dot.",
            ElementCategory::Data,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:chart-line").moves(NodeMotion::Draw)),
    )
}
