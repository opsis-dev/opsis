use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Align, Display, FlexDirection, Overflow, Position, Style, TextAlign};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::handwrite::{HAND_FAMILY, Handwrite, handwrite, handwrite_duration};
use crate::elements::path_utils::path_length;
use crate::elements::shared::FontWeight;
use crate::elements::stop_motion::{random_between, stepped_ramp};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct CheckListItem {
    #[setting("Text", default = "New item")]
    pub text: String,
    #[setting("Ticked", default = true)]
    pub checked: bool,
}

#[settings(export)]
pub struct CheckList {
    #[setting("Font size", default = 40, range(min = 24, max = 64), step = 2)]
    pub font_size: u32,
    #[setting(
        "Time between items",
        default = 0.6,
        range(min = 0.2, max = 1.6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub item_gap: f64,
    #[setting(
        "Time between ticks",
        default = 0.3,
        range(min = 0.1, max = 0.8),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub close_gap: f64,
    #[setting(
        "Letters / pose",
        default = 1.6,
        range(min = 0.8, max = 3),
        step = 0.2,
        advanced
    )]
    pub per_step: f64,
    #[setting(
        "Stroke width",
        default = 3,
        range(min = 1, max = 8),
        step = 1,
        advanced
    )]
    pub stroke_width: u32,
    #[setting("Label color", default = "#26242c", check = colour)]
    pub color: String,
    #[setting("Tick color", default = "#6f7f35", check = colour)]
    pub tick_color: String,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub step: f64,
    #[setting(
        "Items",
        default = vec![
            CheckListItem { text: "Record the walkthrough".into(), ..Default::default() },
            CheckListItem { text: "Add captions and zooms".into(), ..Default::default() },
            CheckListItem { text: "Export for the launch".into(), checked: false },
        ]
    )]
    pub items: Vec<CheckListItem>,
    #[setting(
        "Width",
        default = 720,
        range(min = 200, max = 1800),
        step = 10,
        unit = "px"
    )]
    pub width: u32,
    #[setting("Box colour", default = "#26242c", check = colour)]
    pub box_color: String,
    #[setting("Font weight", default = FontWeight::W600, advanced)]
    pub weight: FontWeight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Row gap",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub row_gap: Option<u32>,
    #[setting(
        "Variation",
        default = "checklist",
        help = "Change it to draw the handwriting a little differently.",
        advanced
    )]
    pub seed: String,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Wobble",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.1,
        help = "How much the boxes, strikes and letters wander.",
        advanced
    )]
    pub wobble: f64,
    #[setting(
        "Font",
        widget = "font",
        zero_label = "Handwriting",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Strike colour",
        widget = "color",
        zero_label = "Tick colour",
        check = colour_or_empty,
        advanced
    )]
    pub strike_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<CheckList>(
            element(
                ElementKind::Component,
                "check-list",
                "Check list",
                "A handwritten checklist that writes itself out in full, then ticks and strikes through every done item, one at a time.",
                ElementCategory::Ui,
                360,
            )
                .authored_clock()
                .fonts(&["Caveat"])
                .look(NodeLook::of("lucide:list-checks")),
        )
}

const CAVEAT_ADVANCE: f64 = 0.36;

fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

#[derive(Debug, Clone, Copy)]
struct Geometry {
    box_size: f64,
    box_gap: f64,
    row_height: f64,
    label_width: f64,
}

fn geometry(width: f64, font_size: f64) -> Geometry {
    let box_size = js_round(font_size * 0.62);
    let box_gap = js_round(font_size * 0.4);
    Geometry {
        box_size,
        box_gap,
        row_height: js_round(font_size * 1.15),
        label_width: (font_size * 2.0).max(width - box_size - box_gap),
    }
}

#[derive(Debug, Clone, Copy)]
struct Beat {
    box_from: f64,
    box_to: f64,
    label_at: f64,
    close: Option<(f64, f64)>,
}

fn schedule(props: &CheckList) -> Vec<Beat> {
    let step = props.step;
    let draw_time = |text: &str| handwrite_duration(text, props.per_step, step);
    let enter_end = props
        .items
        .iter()
        .enumerate()
        .fold(0.0f64, |end, (i, item)| {
            end.max(float(i).mul_add(props.item_gap, step * 2.0) + draw_time(&item.text))
        });
    let mut closed = 0.0_f64;
    props
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let box_from = float(i) * props.item_gap;
            let box_to = step.mul_add(2.0, box_from);
            let close = item.checked.then(|| {
                let tick_from = closed.mul_add(props.close_gap, enter_end);
                closed += 1.0;
                (tick_from, step.mul_add(2.0, tick_from))
            });
            Beat {
                box_from,
                box_to,
                label_at: box_to,
                close,
            }
        })
        .collect()
}

fn box_path(size: f64, seed: &str, index: usize, wobble: f64) -> String {
    let nudge = |n: u32| random_between(&format!("{seed}:{index}:c{n}"), -1.5, 1.5) * wobble;
    let lo = size * 0.08;
    let hi = size - lo;
    let n = js_number_text;
    format!(
        "M {} {} L {} {} L {} {} L {} {} Z",
        n(lo + nudge(0)),
        n(lo + nudge(1)),
        n(hi + nudge(2)),
        n(lo + nudge(3)),
        n(hi + nudge(4)),
        n(hi + nudge(5)),
        n(lo + nudge(6)),
        n(hi + nudge(7))
    )
}

fn tick_path(size: f64) -> String {
    let n = js_number_text;
    format!(
        "M {} {} L {} {} L {} {}",
        n(size * 0.2),
        n(size * 0.52),
        n(size * 0.42),
        n(size * 0.78),
        n(size * 0.88),
        n(size * 0.14)
    )
}

struct Strike<'a> {
    reach: f64,
    font_size: f64,
    row_height: f64,
    seed: &'a str,
    index: usize,
    wobble: f64,
}

fn strike_path(s: &Strike<'_>) -> String {
    let lift =
        |n: u32| random_between(&format!("{}:{}:s{n}", s.seed, s.index), -1.6, 1.6) * s.wobble;
    let overshoot = s.font_size * 0.07;
    let mid = s.row_height / 2.0;
    let n = js_number_text;
    format!(
        "M {} {} C {} {}, {} {}, {} {}",
        n(-overshoot),
        n(mid + lift(0)),
        n(s.reach * 0.35),
        n(mid + lift(1)),
        n(s.reach * 0.7),
        n(mid + lift(2)),
        n(s.reach + overshoot),
        n(mid + lift(3))
    )
}

struct Stroke<'a> {
    d: String,
    color: &'a str,
    width: f64,
    progress: f64,
    shown: f64,
    join: bool,
}

fn stroke(s: Stroke<'_>) -> SvgElement {
    let length = path_length(&s.d);
    let mut path = SvgElement::new("path")
        .attr("stroke", s.color)
        .attr("stroke-width", s.width)
        .attr("stroke-linecap", "round")
        .attr("fill", "none")
        .attr("stroke-dasharray", length)
        .attr("stroke-dashoffset", (1.0 - s.progress) * length)
        .attr("opacity", if s.progress > 0.0 { s.shown } else { 0.0 });
    if s.join {
        path = path.attr("stroke-linejoin", "round");
    }
    path.attr("d", s.d)
}

fn ramp(frame: f64, span: Option<(f64, f64)>, step: f64) -> f64 {
    span.map_or(0.0, |(from, to)| {
        stepped_ramp(frame, from, to, step, Some(&ease_out_cubic))
    })
}

pub fn draw(props: &CheckList, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame();
    let font_size = f64::from(props.font_size);
    let g = geometry(f64::from(props.width), font_size);
    let beats = schedule(props);
    let stroke_width = f64::from(props.stroke_width);
    let label_size = |text: &str| {
        font_size.min(g.label_width / (float(text.chars().count()) * CAVEAT_ADVANCE).max(1.0))
    };
    let family = if props.font_family.is_empty() {
        HAND_FAMILY
    } else {
        &props.font_family
    };
    let rows: Vec<Node> = props
        .items
        .iter()
        .zip(&beats)
        .enumerate()
        .map(|(i, (item, beat))| {
            let box_progress = stepped_ramp(
                frame,
                beat.box_from,
                beat.box_to,
                props.step,
                Some(&ease_out_cubic),
            );
            let tick_progress = ramp(frame, beat.close, props.step);
            let strike_span = beat
                .close
                .map(|(_, tick_to)| (tick_to, props.step.mul_add(2.0, tick_to)));
            let strike_progress = ramp(frame, strike_span, props.step);
            let size = g.box_size;
            let mut mark = SvgElement::new("svg")
                .attr("width", size)
                .attr("height", size)
                .attr(
                    "viewBox",
                    format!("0 0 {} {}", js_number_text(size), js_number_text(size)),
                )
                .child(stroke(Stroke {
                    d: box_path(size, &props.seed, i, props.wobble),
                    color: &props.box_color,
                    width: stroke_width,
                    progress: box_progress,
                    shown: 0.9,
                    join: true,
                }));
            if item.checked {
                mark = mark.child(stroke(Stroke {
                    d: tick_path(size),
                    color: &props.tick_color,
                    width: stroke_width + 1.0,
                    progress: tick_progress,
                    shown: 1.0,
                    join: true,
                }));
            }
            let text_size = label_size(&item.text);
            let mut label = div(Style::default()
                .position(Position::Relative)
                .flex_shrink(0.0)
                .width(Length::px(g.label_width))
                .height(Length::px(g.row_height)))
            .child(handwrite(
                &Handwrite {
                    text: &item.text,
                    font_size: text_size,
                    color: &props.color,
                    delay: beat.label_at,
                    per_step: props.per_step,
                    weight: props.weight.value(),
                    centered: false,
                    font_family: family,
                    step: props.step,
                    wobble: props.wobble,
                },
                frame,
            ));
            if item.checked {
                let reach = g
                    .label_width
                    .min(float(item.text.chars().count()) * text_size * CAVEAT_ADVANCE);
                let line = stroke(Stroke {
                    d: strike_path(&Strike {
                        reach,
                        font_size: text_size,
                        row_height: g.row_height,
                        seed: &props.seed,
                        index: i,
                        wobble: props.wobble,
                    }),
                    color: if props.strike_color.is_empty() {
                        &props.tick_color
                    } else {
                        &props.strike_color
                    },
                    width: stroke_width,
                    progress: strike_progress,
                    shown: 0.85,
                    join: false,
                });
                label = label.child(svg(
                    SvgElement::new("svg")
                        .attr("width", g.label_width)
                        .attr("height", g.row_height)
                        .attr(
                            "viewBox",
                            format!(
                                "0 0 {} {}",
                                js_number_text(g.label_width),
                                js_number_text(g.row_height)
                            ),
                        )
                        .child(line),
                    Style::default()
                        .position(Position::Absolute)
                        .inset(Length::ZERO)
                        .display(Display::Block)
                        .overflow(Overflow::Visible),
                ));
            }
            div(Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(g.box_gap)
                .height(Length::px(g.row_height)))
            .child(svg(
                mark,
                Style::default()
                    .display(Display::Block)
                    .overflow(Overflow::Visible)
                    .flex_shrink(0.0),
            ))
            .child(label)
            .into()
        })
        .collect();
    let row_gap = props.row_gap.map_or(font_size * 0.55, f64::from);
    Ok(div(with_background(
        absolute_fill()
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .text_align(TextAlign::Center),
        &props.background,
    ))
    .child(
        div(Style::default()
            .width(Length::px(g.box_size + g.box_gap + g.label_width))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(row_gap))
        .children(rows),
    )
    .into())
}
