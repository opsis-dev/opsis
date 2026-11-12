use opsis_macros::settings;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{Align, Display, Position, Style};
use opsis_scene::values::Length;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::with_alpha;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct AnimatedBarChartBar {
    #[setting("Label")]
    pub label: String,
    #[setting("Value", default = 50, range(min = 0, max = 1_000_000), step = 1)]
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

const PAINTABLE_SCALE: f64 = 0.001;
const PADDING: f64 = 60.0;
const LABEL_SPACE: f64 = 40.0;
const LABEL_DROP: f64 = 28.0;
const BAR_SPRING: SpringConfig = SpringConfig {
    damping: 12.0,
    stiffness: 100.0,
    mass: 0.8,
    overshoot_clamping: false,
};

fn n(value: f64) -> String {
    js_number_text(value)
}

pub fn draw(props: &AnimatedBarChart, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let data: Vec<f64> = props.bars.iter().map(|b| f64::from(b.value)).collect();
    let labelled = props.bars.iter().any(|b| !b.label.is_empty());
    let face = font_or(&props.font_family, FontRole::Main);
    let frame = cx.authored_frame() * props.speed;
    let (width, height) = (f64::from(props.width), f64::from(props.height));
    let count = float(data.len());
    let inner_width = width - PADDING * 2.0;
    let inner_height = height - PADDING * 2.0 - if labelled { LABEL_SPACE } else { 0.0 };
    let largest = data.iter().copied().fold(0.0, f64::max);
    let max = if largest > 0.0 { largest } else { 1.0 };
    let fit_gap = f64::from(props.gap)
        .min((inner_width - count) / (count - 1.0).max(1.0))
        .max(0.0);
    let bar_width = ((inner_width - fit_gap * (count - 1.0)) / count).max(1.0);
    let base_y = PADDING + inner_height;
    let shadow = with_alpha(&props.bar_color, props.glow / 100.0);
    let mut root = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", format!("0 0 {} {}", n(width), n(height)))
        .child(
            SvgElement::new("line")
                .attr("x1", PADDING)
                .attr("x2", PADDING + inner_width)
                .attr("y1", base_y)
                .attr("y2", base_y)
                .attr("stroke", props.baseline_color.as_str())
                .attr("stroke-width", 2),
        );
    for (index, value) in data.iter().enumerate() {
        let target = value / max * inner_height;
        let x = float(index).mul_add(bar_width + fit_gap, PADDING);
        let y = base_y - target;
        let grow = spring(&Spring {
            config: BAR_SPRING,
            ..Spring::at(float(index).mul_add(-props.stagger_frames, frame), Cx::authored_fps())
        })?;
        let (ox, oy) = (x + (x + bar_width / 2.0), y + base_y);
        let mut group = SvgElement::new("g").child(
            SvgElement::new("rect")
                .attr("x", x)
                .attr("y", y)
                .attr("width", bar_width)
                .attr("height", target)
                .attr("rx", props.corner_radius)
                .attr("fill", props.bar_color.as_str())
                .attr(
                    "transform",
                    format!(
                        "translate({} {}) scale(1 {}) translate({} {})",
                        n(ox),
                        n(oy),
                        n(grow.max(PAINTABLE_SCALE)),
                        n(-ox),
                        n(-oy)
                    ),
                )
                .attr("filter", format!("drop-shadow(0 4px 16px {shadow})")),
        );
        let label = &props.bars[index].label;
        if labelled && !label.is_empty() {
            let size = f64::from(props.label_size)
                .min(fit_gap.mul_add(0.8, bar_width) / (float(label.encode_utf16().count()) * 0.56).max(1.0));
            group = group.child(
                SvgElement::new("text")
                    .attr("x", x + bar_width / 2.0)
                    .attr("y", base_y + LABEL_DROP)
                    .attr("fill", props.label_color.as_str())
                    .attr("font-size", size)
                    .attr("text-anchor", "middle")
                    .attr("opacity", grow)
                    .attr("font-family", face.join(", "))
                    .text(label.as_str()),
            );
        }
        root = root.child(group);
    }
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(face))
    .child(svg(root, Style::default()))
    .into())
}
