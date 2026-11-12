use opsis_macros::settings;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::style::{Display, Overflow, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::brush::{
    Point, Taper, filter_id, grain_defs, length_or_one, n, reach as brush_reach, ribbon,
    sample_cubic, sample_line,
};
use crate::elements::stop_motion::{random_between, stepped_ramp};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct InkArrow {
    #[setting("Curvature", default = 0.35, range(min = -1, max = 1), step = 0.05)]
    pub curvature: f64,
    #[setting("Brush width", default = 8, range(min = 2, max = 24), step = 1)]
    pub stroke_width: u32,
    #[setting(
        "Opening pressure",
        default = 0.2,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub pressure: f64,
    #[setting(
        "Closing pressure",
        default = 1.0,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub release: f64,
    #[setting("Grain", default = 1.0, range(min = 0, max = 2), step = 0.1, advanced)]
    pub grain: f64,
    #[setting("Head size", default = 24, range(min = 8, max = 48), step = 1)]
    pub head_size: u32,
    #[setting(
        "Draw time",
        default = 1.2,
        range(min = 0.4, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_dur: f64,
    #[setting(
        "Arrowhead time",
        default = 0.4,
        range(min = 0.1333, max = 1.6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub head_dur: f64,
    #[setting("Ink color", default = "#26242c", check = colour)]
    pub color: String,
    #[setting("Seed", default = "arrow", advanced)]
    pub seed: String,
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
        "From",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub from_x: f64,
    #[setting(
        "From vertical",
        default = 0.72,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub from_y: f64,
    #[setting(
        "To",
        default = 0.48,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub to_x: f64,
    #[setting(
        "To vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub to_y: f64,
    #[setting(
        "Head angle",
        default = 41.25,
        range(min = 15, max = 75),
        step = 0.25,
        unit = "°",
        advanced
    )]
    pub head_angle: f64,
    #[setting("Wobble", default = 0.06, range(min = 0, max = 0.3), advanced)]
    pub wobble: f64,
    #[setting("Ink opacity", default = 0.85, range(min = 0.1, max = 1), advanced)]
    pub opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<InkArrow>(
            element(
                ElementKind::Component,
                "ink-arrow",
                "Ink arrow",
                "A hand-drawn brush arrow that sweeps itself from one point to another, thickening into the head that pops on the final poses.",
                ElementCategory::Overlay,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("tabler:arrow-curve-right").moves(NodeMotion::Draw)),
        )
}

const SHAFT_SAMPLES: usize = 40;
const ARM_SAMPLES: usize = 10;
const ARM_TAPER: f64 = 0.6;
const ARM_OVERSHOOT: f64 = 0.16;
const HEAD_MIN_RATIO: f64 = 2.6;

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn controls(from: Point, to: Point, curvature: f64, seed: &str, share: f64) -> (Point, Point) {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let distance = length_or_one(dx, dy);
    let (normal_x, normal_y) = (-dy / distance, dx / distance);
    let bow = curvature * distance;
    let wobble = distance * share;
    let at = |t: f64, key: &str| Point {
        x: from.x
            + dx * t
            + normal_x * bow
            + random_between(&format!("{seed}:{key}:x"), -wobble, wobble),
        y: from.y
            + dy * t
            + normal_y * bow
            + random_between(&format!("{seed}:{key}:y"), -wobble, wobble),
    };
    (at(0.3, "c1"), at(0.7, "c2"))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn head(c2: Point, to: Point, size: f64, angle: f64) -> (Point, Point) {
    let (dx, dy) = (to.x - c2.x, to.y - c2.y);
    let length = length_or_one(dx, dy);
    let (back_x, back_y) = (-dx / length, -dy / length);
    let arm = |a: f64| Point {
        x: to.x + size * (back_x * a.cos() - back_y * a.sin()),
        y: to.y + size * (back_x * a.sin() + back_y * a.cos()),
    };
    (arm(angle), arm(-angle))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn apex(c2: Point, to: Point, reach: f64) -> Point {
    let (dx, dy) = (to.x - c2.x, to.y - c2.y);
    let length = length_or_one(dx, dy);
    Point {
        x: to.x + dx / length * reach * ARM_OVERSHOOT,
        y: to.y + dy / length * reach * ARM_OVERSHOOT,
    }
}

fn head_reach(head_size: f64, stroke_width: f64) -> f64 {
    head_size.max(stroke_width * HEAD_MIN_RATIO)
}

pub fn draw(props: &InkArrow, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let stage = cx.at.stage;
    let from = Point {
        x: props.from_x * stage.width,
        y: props.from_y * stage.height,
    };
    let to = Point {
        x: props.to_x * stage.width,
        y: props.to_y * stage.height,
    };
    let frame = cx.authored_frame();
    let step = props.step;
    let stroke = f64::from(props.stroke_width);
    let progress = stepped_ramp(frame, 0.0, props.draw_dur, step, None);
    let (c1, c2) = controls(from, to, props.curvature, &props.seed, props.wobble);
    let reach_of_head = head_reach(f64::from(props.head_size), stroke);
    let (left, right) = head(c2, to, reach_of_head, props.head_angle.to_radians());
    let tip = apex(c2, to, reach_of_head);
    let head_start = props.draw_dur;
    let head_mid = head_start + props.head_dur / 2.0;
    let left_arm = stepped_ramp(frame, head_start, head_mid, step, None);
    let right_arm = stepped_ramp(frame, head_mid, head_start + props.head_dur, step, None);
    let distance = (to.x - from.x).hypot(to.y - from.y);
    let margin = props
        .curvature
        .abs()
        .mul_add(distance, head_reach(f64::from(props.head_size), stroke))
        + brush_reach(stroke, props.grain);
    let width = margin.max(from.x.max(to.x) + margin);
    let height = margin.max(from.y.max(to.y) + margin);
    let filter = (props.grain > 0.0)
        .then(|| format!("url(#{})", filter_id(&props.seed, stroke, props.grain)));
    let painted = |d: String| {
        let path = SvgElement::new("path")
            .attr("d", d)
            .attr("fill", props.color.as_str())
            .attr("opacity", props.opacity);
        match &filter {
            Some(f) => path.attr("filter", f.as_str()),
            None => path,
        }
    };
    let arm = |end: Point, progress: f64| {
        ribbon(
            &sample_line(tip, end, ARM_SAMPLES),
            &Taper {
                stroke_width: stroke * props.release,
                pressure: Some(1.0),
                release: Some(ARM_TAPER),
            },
            progress,
        )
    };
    let mut root = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", format!("0 0 {} {}", n(width), n(height)))
        .child(grain_defs(&props.seed, stroke, props.grain))
        .child(painted(ribbon(
            &sample_cubic(from, c1, c2, to, SHAFT_SAMPLES),
            &Taper {
                stroke_width: stroke,
                pressure: Some(props.pressure),
                release: Some(props.release),
            },
            progress,
        )));
    if left_arm > 0.0 {
        root = root.child(painted(arm(left, left_arm)));
    }
    if right_arm > 0.0 {
        root = root.child(painted(arm(right, right_arm)));
    }
    Ok(div(absolute_fill())
        .child(svg(
            root,
            Style::default()
                .display(Display::Block)
                .overflow(Overflow::Visible)
                .opacity(if progress > 0.0 { 1.0 } else { 0.0 }),
        ))
        .into())
}
