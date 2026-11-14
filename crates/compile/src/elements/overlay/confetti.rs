use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div};
use opsis_scene::random::random;
use opsis_scene::style::{Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Confetti {
    #[setting("Particles", default = 140, range(min = 10, max = 400), step = 10)]
    pub particle_count: u32,
    #[setting("Power", default = 17, range(min = 4, max = 40), step = 1, advanced)]
    pub power: u32,
    #[setting(
        "Gravity",
        default = 0.45,
        range(min = 0, max = 1.5),
        step = 0.05,
        advanced
    )]
    pub gravity: f64,
    #[setting("Size", default = 13, range(min = 4, max = 30), step = 1)]
    pub size: u32,
    #[setting("Seed", default = 1, range(min = 1), step = 1, advanced)]
    pub seed: u32,
    #[setting(
        "Colours",
        default = vec![
            "#1d9bf0".into(),
            "#ff5da2".into(),
            "#ffd23f".into(),
            "#22c55e".into(),
            "#a855f7".into(),
            "#ff7a45".into(),
        ],
        length(min = 1, max = 12)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Burst from",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Click the frame to choose the spot.",
        widget = "spot"
    )]
    pub origin_x: f64,
    #[setting(
        "Burst from vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub origin_y: f64,
    #[setting(
        "How long pieces last",
        default = 3.0,
        range(min = 0.3333, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub lifetime: f64,
    #[setting(
        "Spread",
        default = 360.0,
        range(min = 10, max = 360),
        unit = "°",
        advanced
    )]
    pub spread: f64,
    #[setting("Direction", default = 180.0, range(min = -180, max = 180), unit = "°", advanced)]
    pub direction: f64,
    #[setting("Spin", default = 1.0, range(min = 0, max = 3), unit = "×", advanced)]
    pub spin: f64,
    #[setting("Piece height", default = 0.62, range(min = 0.1, max = 1), advanced)]
    pub piece_shape: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Confetti>(
        element(
            ElementKind::Component,
            "confetti",
            "Confetti",
            "A burst of confetti to celebrate a moment.",
            ElementCategory::Overlay,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:party-popper")),
    )
}

const SCALE_HEIGHT: f64 = 720.0;
const FADE_IN_FRAMES: f64 = 4.0;
const FADE_OUT_SHARE: f64 = 0.7;
const PIECE_RADIUS: f64 = 2.0;

#[derive(Debug, Clone)]
struct Particle {
    angle: f64,
    speed: f64,
    color: String,
    size: f64,
    rot0: f64,
    spin: f64,
    drift: f64,
}

fn particles(props: &Confetti) -> Vec<Particle> {
    let mut draws = 0_u32;
    let mut next = || {
        draws += 1;
        random(format!("{}:{draws}", props.seed))
    };
    (0..props.particle_count)
        .map(|_| {
            let angle = f64::mul_add(next(), props.spread, props.direction - props.spread / 2.0)
                .to_radians();
            let speed = 0.65_f64.mul_add(next(), 0.35);
            let pick: usize = truncate((next() * float(props.colors.len())).floor());
            let color = props.colors.get(pick).map_or_default(|c| c.0.clone());
            let size = 0.6_f64.mul_add(next(), 0.7);
            let rot0 = next() * PI * 2.0;
            let spin = (next() - 0.5) * 0.6 * props.spin;
            let drift = (next() - 0.5) * 2.0;
            Particle {
                angle,
                speed,
                color,
                size,
                rot0,
                spin,
                drift,
            }
        })
        .collect()
}

fn particle_opacity(tt: f64, lifetime: f64) -> Result<f64, DrawFailure> {
    if tt < 0.0 || tt > lifetime {
        return Ok(0.0);
    }
    let fade_in = clamped(tt, [0.0, FADE_IN_FRAMES], [0.0, 1.0])?;
    let fade_out = clamped(tt, [lifetime * FADE_OUT_SHARE, lifetime], [1.0, 0.0])?;
    Ok(fade_in.min(fade_out))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn draw(props: &Confetti, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let tt = cx.authored_frame();
    if tt < 0.0 || tt > props.lifetime {
        return Ok(div(absolute_fill()).into());
    }
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let sc = height / SCALE_HEIGHT;
    let (ox, oy) = (width * props.origin_x, height * props.origin_y);
    let power = f64::from(props.power);
    let opacity = particle_opacity(tt, props.lifetime)?;
    let mut pieces = Vec::new();
    if opacity > 0.0 {
        for p in particles(props) {
            let vx = p.angle.cos() * p.speed * power + p.drift;
            let vy = p.angle.sin() * p.speed * power;
            let x = ox + vx * tt * sc;
            let y = oy + (vy * tt + 0.5 * props.gravity * tt * tt) * sc;
            let rot = p.rot0 + p.spin * tt;
            let flutter = (p.rot0 + p.spin * tt * 1.6).cos();
            let w = f64::from(props.size) * p.size * sc;
            pieces.push(
                div(with_background(
                    Style::default()
                        .position(Position::Absolute)
                        .left(x)
                        .top(y)
                        .width(w)
                        .height(w * props.piece_shape)
                        .border_radius_all(PIECE_RADIUS),
                    &p.color,
                )
                .opacity(opacity)
                .translate(Length::percent(-50.0), Length::percent(-50.0))
                .rotate(Angle::Rad(Num(rot)))
                .scale(flutter, 1.0))
                .into(),
            );
        }
    }
    Ok(div(absolute_fill()).children(pieces).into())
}
