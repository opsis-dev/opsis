use std::fmt::Write;

use opsis_scene::node::{Element, Node, SvgElement, div, svg};
use opsis_scene::random::{js_number_text, random};
use opsis_scene::style::{Display, Overflow, Position, Style};
use opsis_scene::timeline::{Stage, absolute_fill};
use opsis_scene::values::Length;
use opsis_types::numbers::{exact_eq, float};

const GRAIN_RATIO: f64 = 0.5;
const GRAIN_FREQUENCY: f64 = 0.7;
const GRAIN_OCTAVES: u32 = 3;
const DEFAULT_PRESSURE: f64 = 0.2;
const DEFAULT_RELEASE: f64 = 1.0;
const ID_SCALE: f64 = 1e9;
const SEED_SCALE: f64 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Taper {
    pub stroke_width: f64,
    pub pressure: Option<f64>,
    pub release: Option<f64>,
}

fn half_width(taper: &Taper, t: f64) -> f64 {
    let pressure = taper.pressure.unwrap_or(DEFAULT_PRESSURE);
    let release = taper.release.unwrap_or(DEFAULT_RELEASE);
    taper.stroke_width / 2.0 * (release - pressure).mul_add(t, pressure)
}

fn grain_scale(stroke_width: f64, grain: f64) -> f64 {
    stroke_width * GRAIN_RATIO * grain
}

pub fn reach(stroke_width: f64, grain: f64) -> f64 {
    stroke_width / 2.0 + grain_scale(stroke_width, grain) / 2.0
}

pub fn filter_id(seed: &str, stroke_width: f64, grain: f64) -> String {
    let key = format!(
        "{seed}:{}:{}",
        js_number_text(stroke_width),
        js_number_text(grain)
    );
    format!("brush-{}", js_number_text((random(key) * ID_SCALE).floor()))
}

#[expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "the film page's JavaScript rounds every step and names its curve terms so, and the pictures have to match it"
)]
pub fn sample_cubic(from: Point, c1: Point, c2: Point, to: Point, points: usize) -> Vec<Point> {
    let count = points.max(2);
    (0..count)
        .map(|i| {
            let t = float(i) / float(count - 1);
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            Point {
                x: a * from.x + b * c1.x + c * c2.x + d * to.x,
                y: a * from.y + b * c1.y + c * c2.y + d * to.y,
            }
        })
        .collect()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
pub fn sample_line(from: Point, to: Point, points: usize) -> Vec<Point> {
    let count = points.max(2);
    (0..count)
        .map(|i| {
            let t = float(i) / float(count - 1);
            Point {
                x: from.x + (to.x - from.x) * t,
                y: from.y + (to.y - from.y) * t,
            }
        })
        .collect()
}

fn normal_at(points: &[Point], i: usize) -> Point {
    let here = points[i];
    let before = i
        .checked_sub(1)
        .and_then(|j| points.get(j))
        .copied()
        .unwrap_or(here);
    let after = points.get(i + 1).copied().unwrap_or(here);
    let (tx, ty) = (after.x - before.x, after.y - before.y);
    let length = length_or_one(tx, ty);
    Point {
        x: -ty / length,
        y: tx / length,
    }
}

pub fn length_or_one(x: f64, y: f64) -> f64 {
    let length = x.hypot(y);
    if exact_eq(length, 0.0) || length.is_nan() {
        1.0
    } else {
        length
    }
}

pub fn n(value: f64) -> String {
    js_number_text(value)
}

fn curve_segments(points: &[Point]) -> String {
    let mut out = Vec::new();
    for i in 0..points.len().saturating_sub(1) {
        let p1 = points[i];
        let p0 = i.checked_sub(1).map_or(p1, |j| points[j]);
        let p2 = points[i + 1];
        let p3 = points.get(i + 2).copied().unwrap_or(p2);
        out.push(format!(
            "C {} {}, {} {}, {} {}",
            n(p1.x + (p2.x - p0.x) / 6.0),
            n(p1.y + (p2.y - p0.y) / 6.0),
            n(p2.x - (p3.x - p1.x) / 6.0),
            n(p2.y - (p3.y - p1.y) / 6.0),
            n(p2.x),
            n(p2.y)
        ));
    }
    out.join(" ")
}

pub fn ribbon(spine: &[Point], taper: &Taper, progress: f64) -> String {
    let last = spine.len().saturating_sub(1);
    let rounded: usize = opsis_types::numbers::truncate((progress * float(last)).round().max(0.0));
    let drawn = (rounded + 1).max(2).min(spine.len());
    let mut left = Vec::with_capacity(drawn);
    let mut right = Vec::with_capacity(drawn);
    for i in 0..drawn {
        let half = half_width(taper, float(i) / float(last.max(1)));
        let normal = normal_at(spine, i);
        left.push(Point {
            x: normal.x.mul_add(half, spine[i].x),
            y: normal.y.mul_add(half, spine[i].y),
        });
        right.push(Point {
            x: normal.x.mul_add(-half, spine[i].x),
            y: normal.y.mul_add(-half, spine[i].y),
        });
    }
    right.reverse();
    let mut out = String::new();
    let _ = write!(
        out,
        "M {} {} {} L {} {} {} Z",
        n(left[0].x),
        n(left[0].y),
        curve_segments(&left),
        n(right[0].x),
        n(right[0].y),
        curve_segments(&right)
    );
    out
}

pub fn grain_defs(seed: &str, stroke_width: f64, grain: f64) -> SvgElement {
    SvgElement::new("defs").child(
        SvgElement::new("filter")
            .attr("id", filter_id(seed, stroke_width, grain))
            .attr("x", "-30%")
            .attr("y", "-30%")
            .attr("width", "160%")
            .attr("height", "160%")
            .child(
                SvgElement::new("feTurbulence")
                    .attr("type", "fractalNoise")
                    .attr("baseFrequency", GRAIN_FREQUENCY)
                    .attr("numOctaves", GRAIN_OCTAVES)
                    .attr(
                        "seed",
                        (random(format!("{seed}:grain")) * SEED_SCALE).floor(),
                    )
                    .attr("result", "brushGrain"),
            )
            .child(
                SvgElement::new("feDisplacementMap")
                    .attr("in", "SourceGraphic")
                    .attr("in2", "brushGrain")
                    .attr("scale", grain_scale(stroke_width, grain))
                    .attr("xChannelSelector", "R")
                    .attr("yChannelSelector", "G"),
            ),
    )
}

#[derive(Debug, Clone)]
pub struct BrushView<'a> {
    pub width: f64,
    pub height: f64,
    pub margin: (f64, f64),
    pub d: String,
    pub color: &'a str,
    pub opacity: f64,
    pub seed: &'a str,
    pub stroke_width: f64,
    pub grain: f64,
    pub shown: bool,
}

pub fn brush_view(view: BrushView<'_>) -> Element {
    let (mx, my) = view.margin;
    let mut path = SvgElement::new("path")
        .attr("d", view.d)
        .attr("fill", view.color)
        .attr("opacity", view.opacity);
    if view.grain > 0.0 {
        path = path.attr(
            "filter",
            format!(
                "url(#{})",
                filter_id(view.seed, view.stroke_width, view.grain)
            ),
        );
    }
    let root = SvgElement::new("svg")
        .attr("width", f64::mul_add(mx, 2.0, view.width))
        .attr("height", f64::mul_add(my, 2.0, view.height))
        .attr(
            "viewBox",
            format!(
                "{} {} {} {}",
                n(-mx),
                n(-my),
                n(f64::mul_add(mx, 2.0, view.width)),
                n(f64::mul_add(my, 2.0, view.height))
            ),
        )
        .child(grain_defs(view.seed, view.stroke_width, view.grain))
        .child(path);
    div(Style::default()
        .position(Position::Relative)
        .width(view.width)
        .height(view.height)
        .opacity(if view.shown { 1.0 } else { 0.0 }))
    .child(svg(
        root,
        Style::default()
            .position(Position::Absolute)
            .left(-mx)
            .top(-my)
            .display(Display::Block)
            .overflow(Overflow::Visible),
    ))
}

pub fn placed_on_stage(x: f64, y: f64, stage: Stage, extra: &Style, child: Element) -> Node {
    div(absolute_fill())
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(x * stage.width)
                .top(y * stage.height)
                .translate(Length::percent(-50.0), Length::percent(-50.0))
                .merged(extra))
            .child(child),
        )
        .into()
}
