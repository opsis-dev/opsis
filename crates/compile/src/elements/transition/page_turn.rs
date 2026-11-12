use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::style::Style;
use opsis_scene::values::{Angle, Length};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PageTurn {
    #[setting("Angle", default = -7, range(min = -20, max = 20), step = 1, unit = "°")]
    pub angle: i32,
    #[setting("Poses", default = 8, range(min = 2, max = 16), step = 1)]
    pub poses: u32,
    #[setting("Hinge position", default = 0.18, range(min = 0, max = 1), advanced)]
    pub hinge_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<PageTurn>(
            element(
                ElementKind::Presentation,
                "page-turn",
                "Page turn",
                "The exiting scene swings up and away like a notebook page in stop-motion poses, staying above the scene it reveals.",
                ElementCategory::Transition,
                24,
            )
                .look(NodeLook::of("lucide:book-open")),
        )
}

const LIFT_RATIO: f64 = 1.28;
const HINGE_Y: f64 = 1.0;

pub fn page_turn_lift(width: f64, height: f64, ox: f64, angle: f64) -> f64 {
    let theta = angle.to_radians();
    let lowest = [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)]
        .iter()
        .fold(0.0_f64, |low, (x, y)| {
            let dx = x - ox * width;
            let dy = y - HINGE_Y * height;
            low.max(dy.mul_add(theta.cos(), dx.mul_add(theta.sin(), HINGE_Y * height)))
        });
    (height * LIFT_RATIO).max(lowest * 1.02)
}

fn pose_of(progress: f64, poses: u32) -> f64 {
    let eased = progress.clamp(0.0, 1.0).powi(3);
    if poses <= 1 {
        return if eased >= 1.0 { 1.0 } else { 0.0 };
    }
    let count = f64::from(poses);
    let bucket = (eased * count).floor().min(count - 1.0);
    bucket / (count - 1.0)
}

pub fn present(props: &PageTurn, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if side.entering {
        return Ok(filled(Style::default(), side.children));
    }
    let pose = pose_of(side.progress, props.poses);
    let angle = f64::from(props.angle);
    let lift = page_turn_lift(cx.at.stage.width, cx.at.stage.height, props.hinge_x, angle);
    Ok(filled(
        Style::default()
            .z_index(2)
            .transform_origin((
                Length::percent(props.hinge_x * 100.0),
                Length::percent(HINGE_Y * 100.0),
                Length::ZERO,
            ))
            .translate(Length::ZERO, Length::px(-pose * lift))
            .rotate(Angle::deg(angle * pose)),
        side.children,
    ))
}
