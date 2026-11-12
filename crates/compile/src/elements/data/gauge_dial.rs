use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, LineHeight, NumericVariant, Position, Style, TextAlign, TextTransform,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::exact_eq;

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::{arc_path, polar};
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GaugeDial {
    #[setting("Value", default = 78.0, help = "Target the needle sweeps to.")]
    pub value: f64,
    #[setting("Min", help = "Value at the start of the arc.", advanced)]
    pub min: f64,
    #[setting("Max", default = 100.0, help = "Value at the end of the arc.")]
    pub max: f64,
    #[setting("Size", default = 480.0, help = "Type scales off it.")]
    pub size: f64,
    #[setting("Thickness", default = 26.0, help = "Track and fill thickness.")]
    pub thickness: f64,
    #[setting(
        "Sweep angle",
        default = 250,
        range(min = 10, max = 360),
        step = 1,
        help = "Total travel, centred on twelve o'clock. 180 gives a half-moon meter.",
        advanced
    )]
    pub sweep_in_degrees: u32,
    #[setting("Label", default = "Weekly goal", help = "Caption under the readout.")]
    pub label: String,
    #[setting("Unit", default = "%", help = "Suffix on the readout.")]
    pub unit: String,
    #[setting(
        "Tick count",
        default = 9,
        range(min = 0, max = 60),
        step = 1,
        help = "Tick marks around the arc. 0 hides them.",
        advanced
    )]
    pub tick_count: u32,
    #[setting(
        "Animation length",
        default = 44,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of the sweep.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting("Color", default = "#e8b86d", check = colour)]
    pub color: String,
    #[setting("Track color", default = "rgba(250,250,250,0.10)", check = colour, advanced)]
    pub track_color: String,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.58)", check = colour, advanced)]
    pub label_color: String,
    #[setting("Rounded ends", default = true, advanced)]
    pub round_ends: bool,
    #[setting("Show needle", default = true, advanced)]
    pub show_needle: bool,
    #[setting("Capitalise label", default = true, advanced)]
    pub uppercase_label: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Decimals",
        range(min = 0, max = 3),
        zero_label = "From the value",
        step = 1,
        "type" = "integer",
        advanced
    )]
    pub decimals: Option<u32>,
}

pub fn definition() -> ElementMeta {
    meta::<GaugeDial>(
        element(
            ElementKind::Component,
            "gauge-dial",
            "Gauge",
            "A dial whose needle sweeps up to a value.",
            ElementCategory::Data,
            120,
        )
        .look(NodeLook::of("lucide:gauge")),
    )
}

const MAX_DECIMALS: usize = 3;

fn decimals_of(value: f64) -> usize {
    js_number_text(value)
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
        .min(MAX_DECIMALS)
}

const fn pop_easing() -> Easing {
    Easing::Bezier(0.34, 1.56, 0.64, 1.0)
}

fn stroke(d: String, color: &str, width: f64, cap: &str) -> SvgElement {
    SvgElement::new("path")
        .attr("d", d)
        .attr("fill", "none")
        .attr("stroke", color)
        .attr("stroke-width", width)
        .attr("stroke-linecap", cap)
}

fn line(from: (f64, f64), to: (f64, f64), color: &str, width: f64) -> SvgElement {
    SvgElement::new("line")
        .attr("x1", from.0)
        .attr("y1", from.1)
        .attr("x2", to.0)
        .attr("y2", to.1)
        .attr("stroke", color)
        .attr("stroke-width", width)
        .attr("stroke-linecap", "round")
}

pub fn draw(props: &GaugeDial, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let cap = if props.round_ends { "round" } else { "butt" };
    let sweep = f64::from(props.sweep_in_degrees.clamp(1, 360));
    let full_circle = sweep >= 360.0;
    let ticks = f64::from(props.tick_count);
    let steps = if full_circle { ticks } else { ticks - 1.0 };
    let decimals = props.decimals.map_or_else(
        || decimals_of(props.value),
        |d| usize::try_from(d).unwrap_or(0),
    );
    let size = props.size;
    let centre = size / 2.0;
    let radius = (size - props.thickness) / 2.0;
    let (start, end) = (-sweep / 2.0, sweep / 2.0);
    let span = props.max - props.min;
    let span = if exact_eq(span, 0.0) || span.is_nan() {
        1.0
    } else {
        span
    };
    let target = ((props.value - props.min) / span).clamp(0.0, 1.0);
    let window = [0.0, f64::from(props.duration_in_frames)];
    let arc_progress = clamped_eased(frame, window, [0.0, 1.0], easing_enter())?;
    let needle_progress = clamped_eased(frame, window, [0.0, 1.0], pop_easing())?;
    let arc_fraction = target * arc_progress;
    let needle_fraction = target * needle_progress;
    let needle_angle = needle_fraction.mul_add(sweep, start);
    let tip = polar(
        centre,
        centre,
        props.thickness.mul_add(-0.55, radius),
        needle_angle,
    );
    let tail = polar(centre, centre, props.thickness * 0.5, needle_angle + 180.0);
    let mut dial = SvgElement::new("svg")
        .attr("width", size)
        .attr("height", size)
        .attr(
            "viewBox",
            format!("0 0 {} {}", js_number_text(size), js_number_text(size)),
        )
        .child(stroke(
            arc_path(centre, centre, radius, start, end),
            &props.track_color,
            props.thickness,
            cap,
        ))
        .child(stroke(
            arc_path(
                centre,
                centre,
                radius,
                start,
                arc_fraction.mul_add(sweep, start),
            ),
            &props.color,
            props.thickness,
            cap,
        ));
    for index in 0..props.tick_count {
        let fraction = if steps > 0.0 {
            f64::from(index) / steps
        } else {
            0.0
        };
        let angle = fraction.mul_add(sweep, start);
        let outer = polar(
            centre,
            centre,
            props.thickness.mul_add(-0.72, radius),
            angle,
        );
        let inner = polar(
            centre,
            centre,
            props.thickness.mul_add(-1.05, radius),
            angle,
        );
        let passed = fraction <= needle_fraction;
        dial = dial.child(
            line(
                inner,
                outer,
                if passed {
                    &props.ink_color
                } else {
                    &props.label_color
                },
                size * 0.007,
            )
            .attr("opacity", if passed { 0.85 } else { 0.35 }),
        );
    }
    if props.show_needle {
        dial = dial
            .child(line(tail, tip, &props.ink_color, size * 0.018))
            .child(
                SvgElement::new("circle")
                    .attr("cx", centre)
                    .attr("cy", centre)
                    .attr("r", size * 0.038)
                    .attr("fill", props.ink_color.as_str()),
            )
            .child(
                SvgElement::new("circle")
                    .attr("cx", centre)
                    .attr("cy", centre)
                    .attr("r", size * 0.016)
                    .attr("fill", props.color.as_str()),
            );
    }
    let shown = (props.value - props.min).mul_add(arc_progress, props.min);
    let mut readout = div(Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .right(Length::ZERO)
        .top(size * 0.62)
        .text_align(TextAlign::Center))
    .child(
        div(Style::default()
            .color(cx.color(&props.ink_color))
            .font_size(size * 0.14)
            .font_weight(700.0)
            .line_height(LineHeight::Number(Num(1.0)))
            .font_variant_numeric(NumericVariant {
                tabular: true,
                ..NumericVariant::default()
            }))
        .child(format!("{shown:.decimals$}{}", props.unit)),
    );
    if !props.label.is_empty() {
        let mut caption = Style::default()
            .color(cx.color(&props.label_color))
            .font_size(size * 0.055)
            .font_weight(600.0)
            .margin_top(size * 0.028);
        if props.uppercase_label {
            caption = caption
                .letter_spacing(Length::em(0.06))
                .text_transform(TextTransform::Uppercase);
        }
        readout = readout.child(div(caption).child(props.label.as_str()));
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(size)
            .height(size)
            .opacity(1.0)
            .font_family(font_or(&props.font_family, FontRole::Main)))
        .child(svg(dial, Style::default()))
        .child(readout),
    )
    .into())
}
