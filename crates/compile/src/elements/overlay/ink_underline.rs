use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::style::Style;
use opsis_scene::values::Angle;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::brush::{
    BrushView, Point, Taper, brush_view, placed_on_stage, reach as brush_reach, ribbon,
    sample_cubic,
};
use crate::elements::stop_motion::{random_between, stepped_ramp};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct InkUnderline {
    #[setting("Width", default = 420, range(min = 100, max = 800), step = 10)]
    pub width: u32,
    #[setting("Ink color", default = "#6f7f35", check = colour)]
    pub color: String,
    #[setting("Brush width", default = 9, range(min = 2, max = 24), step = 1)]
    pub thickness: u32,
    #[setting(
        "Opening pressure",
        default = 1.0,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub pressure: f64,
    #[setting(
        "Closing pressure",
        default = 0.15,
        range(min = 0.05, max = 1),
        step = 0.05,
        advanced
    )]
    pub release: f64,
    #[setting("Grain", default = 1.0, range(min = 0, max = 2), step = 0.1, advanced)]
    pub grain: f64,
    #[setting(
        "Poses to draw",
        default = 5,
        range(min = 2, max = 10),
        step = 1,
        advanced
    )]
    pub duration_steps: u32,
    #[setting("Seed", default = "ink", advanced)]
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
        "Where",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub x: f64,
    #[setting(
        "Where vertical",
        default = 0.62,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Wobble",
        default = 1.2,
        range(min = 0, max = 10),
        unit = "px",
        advanced
    )]
    pub wobble: f64,
    #[setting("Tilt", default = 0.0, range(min = -45, max = 45), unit = "°", advanced)]
    pub angle: f64,
    #[setting("Ink opacity", default = 0.85, range(min = 0.1, max = 1), advanced)]
    pub opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<InkUnderline>(
            element(
                ElementKind::Component,
                "ink-underline",
                "Ink underline",
                "A hand-dragged brush underline that sweeps itself across in a few stop-motion poses, thinning to a dry tail.",
                ElementCategory::Overlay,
                120,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:underline").moves(NodeMotion::Draw)),
        )
}

const SAMPLES: usize = 40;
const EDGE_INSET: f64 = 2.0;

fn spine(width: f64, thickness: f64, seed: &str, wobble_size: f64) -> Vec<Point> {
    let wobble = random_between(&format!("{seed}:w"), -wobble_size, wobble_size);
    let mid = thickness / 2.0 + EDGE_INSET;
    sample_cubic(
        Point { x: EDGE_INSET, y: mid },
        Point {
            x: width * 0.3,
            y: mid + wobble,
        },
        Point {
            x: width * 0.7,
            y: mid - wobble,
        },
        Point {
            x: width - EDGE_INSET,
            y: wobble.mul_add(0.6, mid),
        },
        SAMPLES,
    )
}

pub fn draw(props: &InkUnderline, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame();
    let progress = stepped_ramp(
        frame,
        0.0,
        f64::from(props.duration_steps) * props.step,
        props.step,
        None,
    );
    let (width, thickness) = (f64::from(props.width), f64::from(props.thickness));
    let reach = brush_reach(thickness, props.grain);
    let height = thickness + 4.0;
    let d = ribbon(
        &spine(width, thickness, &props.seed, props.wobble),
        &Taper {
            stroke_width: thickness,
            pressure: Some(props.pressure),
            release: Some(props.release),
        },
        progress,
    );
    let view = brush_view(BrushView {
        width,
        height,
        margin: (reach, reach),
        d,
        color: &props.color,
        opacity: props.opacity,
        seed: &props.seed,
        stroke_width: thickness,
        grain: props.grain,
        shown: progress > 0.0,
    });
    Ok(placed_on_stage(
        props.x,
        props.y,
        cx.at.stage,
        &Style::default().rotate(Angle::deg(props.angle)),
        view,
    ))
}
