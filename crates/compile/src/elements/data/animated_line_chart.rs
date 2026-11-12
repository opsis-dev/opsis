use opsis_macros::settings;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::{DEFAULT_SPRING, Spring, SpringConfig, spring};
use opsis_scene::style::{Align, Display, Overflow, Position, Style};
use opsis_scene::values::Length;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::{exact_eq, float};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::with_alpha;
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

const PADDING: f64 = 60.0;
const PROGRESS_SPRING: SpringConfig = SpringConfig {
    damping: 200.0,
    ..DEFAULT_SPRING
};

fn n(value: f64) -> String {
    js_number_text(value)
}

fn grid_line(from: (f64, f64), to: (f64, f64), color: &str, width: f64) -> SvgElement {
    SvgElement::new("line")
        .attr("x1", from.0)
        .attr("x2", to.0)
        .attr("y1", from.1)
        .attr("y2", to.1)
        .attr("stroke", color)
        .attr("stroke-width", width)
}

pub fn draw(props: &AnimatedLineChart, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let duration = cx.authored_duration() * props.speed;
    let (width, height) = (f64::from(props.width), f64::from(props.height));
    let inner_width = PADDING.mul_add(-2.0, width);
    let inner_height = PADDING.mul_add(-2.0, height);
    let data = &props.data;
    let lowest = data.iter().copied().fold(f64::INFINITY, f64::min);
    let min = if props.start_at_zero {
        lowest.min(0.0)
    } else {
        lowest
    };
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    let range = if exact_eq(span, 0.0) || span.is_nan() {
        1.0
    } else {
        span
    };
    let last = float(data.len()) - 1.0;
    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, value)| {
            (
                (float(i) / last).mul_add(inner_width, PADDING),
                f64::mul_add((value - min) / range, -inner_height, PADDING + inner_height),
            )
        })
        .collect();
    let length: f64 = points
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
        .sum();
    let d = points
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{}{:.2},{:.2}", if i == 0 { "M" } else { "L" }, p.0, p.1))
        .collect::<Vec<_>>()
        .join(" ");
    let progress = spring(&Spring {
        config: PROGRESS_SPRING,
        duration_in_frames: Some((duration * props.finish_at / 100.0).round()),
        ..Spring::at(frame, Cx::authored_fps())
    })?;
    let target = length * progress;
    let mut traveled = 0.0;
    let (mut dot_x, mut dot_y) = points.first().copied().unwrap_or((0.0, 0.0));
    for w in points.windows(2) {
        let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let segment = dx.hypot(dy);
        if traveled + segment >= target {
            let t = (target - traveled) / segment;
            dot_x = dx.mul_add(t, w[0].0);
            dot_y = dy.mul_add(t, w[0].1);
            break;
        }
        traveled += segment;
        (dot_x, dot_y) = w[1];
    }
    let grid = props.grid_color.as_str();
    let mut root = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", format!("0 0 {} {}", n(width), n(height)));
    let rows = f64::from(props.grid_rows);
    for i in 0..=props.grid_rows {
        let y = f64::mul_add(f64::from(i) / rows, inner_height, PADDING);
        root = root.child(grid_line(
            (PADDING, y),
            (PADDING + inner_width, y),
            grid,
            1.0,
        ));
    }
    if props.show_columns {
        let columns = data.len().saturating_sub(1);
        for i in 0..=columns {
            let x = (float(i) / float(columns)).mul_add(inner_width, PADDING);
            root = root.child(grid_line(
                (x, PADDING),
                (x, PADDING + inner_height),
                grid,
                1.0,
            ));
        }
    }
    root = root
        .child(grid_line(
            (PADDING, PADDING),
            (PADDING, PADDING + inner_height),
            grid,
            2.0,
        ))
        .child(grid_line(
            (PADDING, PADDING + inner_height),
            (PADDING + inner_width, PADDING + inner_height),
            grid,
            2.0,
        ))
        .child(
            SvgElement::new("path")
                .attr("d", d)
                .attr("fill", "none")
                .attr("stroke", props.stroke_color.as_str())
                .attr("stroke-width", props.stroke_width)
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round")
                .attr("stroke-dasharray", length)
                .attr("stroke-dashoffset", length * (1.0 - progress))
                .attr(
                    "filter",
                    format!(
                        "drop-shadow(0 0 12px {})",
                        with_alpha(&props.stroke_color, props.glow / 100.0)
                    ),
                ),
        );
    if props.show_dot && progress > 0.0 && progress < 1.0 {
        root = root.child(
            SvgElement::new("circle")
                .attr("cx", dot_x)
                .attr("cy", dot_y)
                .attr("r", f64::from(props.stroke_width) * 2.0)
                .attr("fill", props.stroke_color.as_str())
                .attr(
                    "filter",
                    format!("drop-shadow(0 0 8px {})", props.stroke_color),
                ),
        );
    }
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(font_or("", FontRole::Main)))
    .child(svg(root, Style::default().overflow(Overflow::Visible)))
    .into())
}
