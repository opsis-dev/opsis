use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, Display, LineHeight, NumericVariant, Position, Style, TextAlign, TextTransform,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float};

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::{Direction, format_number, read_delta};
use crate::elements::shared::{Colour, LegendValue, NumberFormat};
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct DonutChartSegment {
    #[setting("Label", default = "Referral")]
    pub label: String,
    #[setting("Value", default = 880.0)]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
}

#[settings(export)]
pub struct DonutChart {
    #[setting(
        "Segments",
        default = vec![
            DonutChartSegment { label: "Direct".into(), value: 4820.0, ..Default::default() },
            DonutChartSegment { label: "Search".into(), value: 3140.0, ..Default::default() },
            DonutChartSegment { label: "Social".into(), value: 1960.0, ..Default::default() },
            DonutChartSegment::default(),
        ]
    )]
    pub segments: Vec<DonutChartSegment>,
    #[setting("Size", default = 520.0, help = "All type scales off it.")]
    pub size: f64,
    #[setting("Thickness", default = 46.0, help = "Ring thickness.")]
    pub thickness: f64,
    #[setting(
        "Colors",
        default = vec!["#e8b86d".into(), "#2dd4bf".into(), "#f472b6".into(), "#8b8bf5".into()],
        length(min = 1)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Show legend",
        default = true,
        help = "Legend rows beside the ring, arriving with their own slice."
    )]
    pub show_legend: bool,
    #[setting("Show total", default = true, help = "Running total in the hole.")]
    pub show_total: bool,
    #[setting("Total label", default = "Sessions", help = "Caption under the total.")]
    pub total_label: String,
    #[setting(
        "Animation length",
        default = 26,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one segment's sweep.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 9,
        unit = "frames",
        help = "Frames between one segment and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Track color", default = "rgba(250,250,250,0.08)", check = colour, advanced)]
    pub track_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.58)", check = colour, advanced)]
    pub label_color: String,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Increase color", default = "#2dd4bf", check = colour, advanced)]
    pub up_color: String,
    #[setting("Decrease color", default = "#f87171", check = colour, advanced)]
    pub down_color: String,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting(
        "Segment gap",
        default = 1.2,
        range(min = 0, max = 5),
        unit = "%",
        advanced
    )]
    pub segment_gap: f64,
    #[setting("Rounded ends", advanced)]
    pub round_ends: bool,
    #[setting(
        "Start angle",
        default = 0.0,
        range(min = 0, max = 359),
        unit = "deg",
        advanced
    )]
    pub start_angle: f64,
    #[setting("Capitalise label", default = true, advanced)]
    pub uppercase_label: bool,
    #[setting(
        "Legend shows",
        default = LegendValue::Percent,
        advanced
    )]
    pub legend_shows: LegendValue,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<DonutChart>(
        element(
            ElementKind::Component,
            "donut-chart",
            "Donut chart",
            "A ring chart that fills segment by segment, with a legend and a total.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-pie")),
    )
}

fn n(value: f64) -> String {
    js_number_text(value)
}

fn tabular() -> NumericVariant {
    NumericVariant {
        tabular: true,
        ..NumericVariant::default()
    }
}

struct Drawn<'a> {
    segment: &'a DonutChartSegment,
    start: f64,
    fraction: f64,
    progress: f64,
    color: String,
}

fn legend_row(props: &DonutChart, drawn: &Drawn<'_>, cx: &Cx<'_>) -> Node {
    let size = props.size;
    let shown = if props.legend_shows == LegendValue::Percent {
        format!(
            "{}%",
            js_number_text(js_round(drawn.fraction * drawn.progress * 100.0))
        )
    } else {
        format_number(drawn.segment.value * drawn.progress, props.number_format)
    };
    let mut row = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(size * 0.045)
        .opacity((drawn.progress * 1.6).clamp(0.0, 1.0))
        .translate(
            Length::px((1.0 - drawn.progress) * size * 0.05),
            Length::ZERO,
        ))
    .child(span(with_background(
        Style::default()
            .width(size * 0.05)
            .height(size * 0.05)
            .border_radius_all(999.0)
            .flex_shrink(0.0),
        &drawn.color,
    )))
    .child(
        span(
            Style::default()
                .color(cx.color(&props.ink_color))
                .font_size(size * 0.062)
                .font_weight(600.0)
                .min_width(size * 0.42),
        )
        .child(drawn.segment.label.as_str()),
    )
    .child(
        span(
            Style::default()
                .color(cx.color(&props.label_color))
                .font_size(size * 0.062)
                .font_weight(600.0)
                .font_variant_numeric(tabular()),
        )
        .child(shown),
    );
    if let Some(delta) = drawn.segment.delta.as_deref().filter(|d| !d.is_empty()) {
        let (direction, text) = read_delta(delta);
        let color = match direction {
            Direction::Down => &props.down_color,
            Direction::Up => &props.up_color,
            Direction::Flat => &props.label_color,
        };
        row = row.child(
            span(
                Style::default()
                    .color(cx.color(color))
                    .font_size(size * 0.052)
                    .font_weight(600.0)
                    .font_variant_numeric(tabular()),
            )
            .child(text),
        );
    }
    row.into()
}

pub fn draw(props: &DonutChart, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let sum: f64 = props.segments.iter().map(|s| s.value).sum();
    let total = if exact_eq(sum, 0.0) || sum.is_nan() {
        1.0
    } else {
        sum
    };
    let size = props.size;
    let radius = (size - props.thickness) / 2.0;
    let circumference = 2.0 * PI * radius;
    let gap = (circumference * props.segment_gap / 100.0).min(props.thickness * 0.22);
    let duration = f64::from(props.duration_in_frames);
    let mut drawn = Vec::new();
    let mut before = 0.0;
    for (index, segment) in props.segments.iter().enumerate() {
        let begin = float(index) * f64::from(props.stagger_in_frames);
        drawn.push(Drawn {
            segment,
            start: before,
            fraction: segment.value / total,
            progress: clamped_eased(frame, [begin, begin + duration], [0.0, 1.0], easing_enter())?,
            color: segment.color.clone().unwrap_or_else(|| {
                props
                    .colors
                    .get(index % props.colors.len().max(1))
                    .map_or_default(|c| c.0.clone())
            }),
        });
        before += segment.value / total;
    }
    let landed: f64 = drawn.iter().map(|d| d.segment.value * d.progress).sum();
    let centre = size / 2.0;
    let mut ring = SvgElement::new("svg")
        .attr("width", size)
        .attr("height", size)
        .attr("viewBox", format!("0 0 {} {}", n(size), n(size)))
        .child(
            SvgElement::new("circle")
                .attr("cx", centre)
                .attr("cy", centre)
                .attr("r", radius)
                .attr("fill", "none")
                .attr("stroke", props.track_color.as_str())
                .attr("stroke-width", props.thickness),
        );
    for segment in &drawn {
        let length = (segment.fraction * circumference)
            .mul_add(segment.progress, -gap)
            .max(0.0);
        ring = ring.child(
            SvgElement::new("circle")
                .attr("cx", centre)
                .attr("cy", centre)
                .attr("r", radius)
                .attr("fill", "none")
                .attr("stroke", segment.color.as_str())
                .attr("stroke-width", props.thickness)
                .attr(
                    "stroke-linecap",
                    if props.round_ends { "round" } else { "butt" },
                )
                .attr(
                    "stroke-dasharray",
                    format!("{} {}", n(length), n(circumference)),
                )
                .attr(
                    "transform",
                    format!(
                        "rotate({} {} {})",
                        n(f64::mul_add(
                            segment.start,
                            360.0,
                            -90.0 + props.start_angle
                        )),
                        n(centre),
                        n(centre)
                    ),
                ),
        );
    }
    let mut chart = div(Style::default()
        .position(Position::Relative)
        .width(size)
        .height(size))
    .child(svg(ring, Style::default()));
    if props.show_total {
        let mut caption = Style::default()
            .color(cx.color(&props.label_color))
            .font_size(size * 0.065)
            .font_weight(600.0)
            .margin_top(size * 0.03);
        if props.uppercase_label {
            caption = caption
                .letter_spacing(Length::em(0.06))
                .text_transform(TextTransform::Uppercase);
        }
        chart = chart.child(
            div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .display(Display::Grid)
                .align_items(Align::Center)
                .justify_items(Align::Center)
                .text_align(TextAlign::Center))
            .child(
                div(Style::default())
                    .child(
                        div(Style::default()
                            .color(cx.color(&props.ink_color))
                            .font_size(size * 0.17)
                            .font_weight(700.0)
                            .line_height(LineHeight::Number(Num(1.0)))
                            .font_variant_numeric(tabular()))
                        .child(format_number(landed, props.number_format)),
                    )
                    .child(div(caption).child(props.total_label.as_str())),
            ),
        );
    }
    let mut body = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .font_family(font_or(&props.font_family, FontRole::Main))
        .gap(size * 0.16)
        .opacity(1.0)
        .scale(1.0, 1.0))
    .child(chart);
    if props.show_legend {
        body = body.child(
            div(Style::default().display(Display::Grid).gap(size * 0.055))
                .children(drawn.iter().map(|d| legend_row(props, d, cx))),
        );
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(body)
    .into())
}
