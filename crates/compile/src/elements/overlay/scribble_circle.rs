use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::style::Style;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::{float, truncate};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::brush::{
    BrushView, Point, Taper, brush_view, placed_on_stage, reach as brush_reach, ribbon,
};
use crate::elements::stop_motion::{random_between, stepped_ramp};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ScribbleCircle {
    #[setting("Laps", default = 1.15, range(min = 0.5, max = 3), step = 0.05)]
    pub laps: f64,
    #[setting("Brush width", default = 14, range(min = 4, max = 32), step = 1)]
    pub stroke_width: u32,
    #[setting(
        "Opening pressure",
        default = 0.2,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub pressure: f64,
    #[setting("Grain", default = 1.0, range(min = 0, max = 2), step = 0.1, advanced)]
    pub grain: f64,
    #[setting(
        "Poses to draw",
        default = 10,
        range(min = 2, max = 20),
        step = 1,
        advanced
    )]
    pub duration_steps: u32,
    #[setting("Ink color", default = "#6f7f35", check = colour)]
    pub color: String,
    #[setting("Seed", default = "scribble", advanced)]
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
        "Around",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub x: f64,
    #[setting(
        "Around vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Width",
        default = 420,
        range(min = 20, max = 3840),
        step = 10,
        unit = "px"
    )]
    pub width: u32,
    #[setting(
        "Height",
        default = 220,
        range(min = 20, max = 2160),
        step = 10,
        unit = "px"
    )]
    pub height: u32,
    #[setting(
        "Closing pressure",
        default = 1.0,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub release: f64,
    #[setting("Wobble", default = 0.035, range(min = 0, max = 0.15), advanced)]
    pub wobble: f64,
    #[setting(
        "Start at",
        default = 0.0,
        range(min = 0, max = 360),
        unit = "°",
        advanced
    )]
    pub start_angle: f64,
    #[setting("Clockwise", default = true, advanced)]
    pub clockwise: bool,
    #[setting("Ink opacity", default = 0.85, range(min = 0.1, max = 1), advanced)]
    pub opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ScribbleCircle>(
            element(
                ElementKind::Component,
                "scribble-circle",
                "Scribble circle",
                "A brush circle swept around a region of the frame, opening thin and closing thick, with grain breaking up the dry opening.",
                ElementCategory::Overlay,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:lasso").moves(NodeMotion::Draw)),
        )
}

const SAMPLES_PER_LAP: f64 = 48.0;
const CENTRE: f64 = 0.02;

fn points(props: &ScribbleCircle) -> Vec<Point> {
    let (width, height) = (f64::from(props.width), f64::from(props.height));
    let count: usize = truncate((props.laps * SAMPLES_PER_LAP).ceil().max(2.0));
    let (rx, ry) = (width / 2.0, height / 2.0);
    let seed = &props.seed;
    let cx = random_between(&format!("{seed}:cx"), -CENTRE, CENTRE).mul_add(width, width / 2.0);
    let cy = random_between(&format!("{seed}:cy"), -CENTRE, CENTRE).mul_add(height, height / 2.0);
    let start = props.start_angle.to_radians();
    let turn = if props.clockwise { 1.0 } else { -1.0 };
    (0..count)
        .map(|i| {
            let angle = (turn * props.laps * PI * 2.0).mul_add(float(i) / float(count - 1), start);
            let scale = 1.0 + random_between(&format!("{seed}:r{i}"), -props.wobble, props.wobble);
            Point {
                x: (rx * scale).mul_add(angle.cos(), cx),
                y: (ry * scale).mul_add(angle.sin(), cy),
            }
        })
        .collect()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
pub fn draw(props: &ScribbleCircle, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let progress = stepped_ramp(
        cx.authored_frame(),
        0.0,
        f64::from(props.duration_steps) * props.step,
        props.step,
        None,
    );
    let (width, height) = (f64::from(props.width), f64::from(props.height));
    let stroke = f64::from(props.stroke_width);
    let reach = brush_reach(stroke, props.grain);
    let margin_x = reach + width / 2.0 * props.wobble + width * CENTRE;
    let margin_y = reach + height / 2.0 * props.wobble + height * CENTRE;
    let d = ribbon(
        &points(props),
        &Taper {
            stroke_width: stroke,
            pressure: Some(props.pressure),
            release: Some(props.release),
        },
        progress,
    );
    let view = brush_view(BrushView {
        width,
        height,
        margin: (margin_x, margin_y),
        d,
        color: &props.color,
        opacity: props.opacity,
        seed: &props.seed,
        stroke_width: stroke,
        grain: props.grain,
        shown: progress > 0.0,
    });
    Ok(placed_on_stage(
        props.x,
        props.y,
        cx.at.stage,
        &Style::default(),
        view,
    ))
}
