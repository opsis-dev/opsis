use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{BlendMode, Filter, Transform};
use opsis_scene::random::random;
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::whole;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::units::to_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ParticleField {
    #[setting(
        "Background color",
        default = "#05070f",
        help = "Plate behind the field.",
        check = colour
    )]
    pub background_color: String,
    #[setting("Color", default = "#e8b86d", help = "Particle colour.", check = colour)]
    pub color: String,
    #[setting(
        "Particles",
        default = 70,
        range(min = 1, max = 400),
        step = 1,
        help = "How many particles."
    )]
    pub count: u32,
    #[setting(
        "Direction",
        default = 8.0,
        help = "Direction of travel in degrees. 0 drifts straight up."
    )]
    pub angle: f64,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Traverse speed. 1 crosses the frame in about eight seconds."
    )]
    pub speed: f64,
    #[setting("Size", default = 14.0)]
    pub size: f64,
    #[setting("Smallest size", default = 2.0, advanced)]
    pub min_size: f64,
    #[setting(
        "Depth blur",
        default = 3.0,
        help = "Defocus on the furthest particles. 0 flattens the field.",
        advanced
    )]
    pub depth_blur: f64,
    #[setting(
        "Drift",
        default = 4.0,
        help = "Sideways wander, in percent of the frame.",
        advanced
    )]
    pub drift: f64,
    #[setting("Intensity", default = 1.0, help = "Overall brightness.")]
    pub intensity: f64,
    #[setting(
        "Glow",
        default = 1.6,
        step = 0.1,
        help = "Halo per particle, as a multiple of its size. 0 draws hard dots."
    )]
    pub glow: f64,
    #[setting("Variation", default = 1, advanced)]
    pub seed: u32,
    #[setting("Twinkle", default = 0.3, range(min = 0, max = 1), advanced)]
    pub twinkle: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ParticleField>(
        element(
            ElementKind::Component,
            "particle-field",
            "Particle field",
            "Glowing particles drifting at different depths.",
            ElementCategory::Background,
            120,
        )
        .look(NodeLook::of("lucide:sparkles")),
    )
}

const TRACK: f64 = 140.0;
const CROSSING_SECONDS: f64 = 8.0;
const DESIGN_WIDTH: f64 = 1920.0;

#[derive(Debug, Clone, Copy)]
struct Particle {
    depth: f64,
    offset: f64,
    lane: f64,
    drift_amount: f64,
    drift_period: f64,
    drift_phase: f64,
    twinkle_period: f64,
    twinkle_phase: f64,
}

fn particles(count: u32, seed: u32) -> Vec<Particle> {
    let draw = |index: u32, n: u32| random(format!("{seed}:{index}:{n}"));
    (0..count)
        .map(|i| Particle {
            depth: draw(i, 1).powi(2),
            offset: draw(i, 2),
            lane: draw(i, 3),
            drift_amount: draw(i, 4).mul_add(1.6, 0.4),
            drift_period: draw(i, 5).mul_add(1.3, 1.1),
            drift_phase: draw(i, 6) * PI * 2.0,
            twinkle_period: draw(i, 7).mul_add(1.2, 0.9),
            twinkle_phase: draw(i, 8) * PI * 2.0,
        })
        .collect()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places each particle with these exact steps, and the frames have to match it"
)]
fn particle(props: &ParticleField, p: &Particle, time: f64, pixel: f64) -> Node {
    let radians = props.angle * PI / 180.0;
    let (dir_x, dir_y) = (radians.sin(), -radians.cos());
    let (perp_x, perp_y) = (radians.cos(), radians.sin());
    let rate = (0.35 + p.depth * 0.95) / CROSSING_SECONDS;
    let along = (p.offset + time * rate) % 1.0;
    let wander =
        (time / p.drift_period * PI * 2.0 + p.drift_phase).sin() * props.drift * p.drift_amount;
    let across = p.lane * TRACK - TRACK / 2.0 + wander;
    let forward = along * TRACK - TRACK / 2.0;
    let x = 50.0 + dir_x * forward + perp_x * across;
    let y = 50.0 + dir_y * forward + perp_y * across;
    let diameter = (props.min_size + (props.size - props.min_size) * p.depth) * pixel;
    let twinkle = 1.0 - props.twinkle
        + props.twinkle * (time / p.twinkle_period * PI * 2.0 + p.twinkle_phase).sin();
    let mut style = Style::default()
        .position(Position::Absolute)
        .left(Length::percent(to_fixed(x, 3)))
        .top(Length::percent(to_fixed(y, 3)))
        .width(Length::px(diameter))
        .height(Length::px(diameter))
        .margin_left(Length::px(-diameter / 2.0))
        .margin_top(Length::px(-diameter / 2.0))
        .border_radius_all(Length::percent(50.0))
        .opacity((0.18 + p.depth * 0.7) * twinkle * props.intensity)
        .mix_blend_mode(BlendMode::Screen);
    style = if props.glow > 0.0 {
        with_background(
            style,
            &format!(
                "radial-gradient(circle, {0} 0%, {0} 34%, transparent 70%)",
                props.color
            ),
        )
        .transform(vec![Transform::Scale(
            Num(1.0 + props.glow),
            Num(1.0 + props.glow),
            Num(1.0),
        )])
    } else {
        with_background(style, &props.color)
    };
    if props.depth_blur > 0.0 {
        style = style.filter(vec![Filter::Blur(Length::px(to_fixed(
            (1.0 - p.depth) * props.depth_blur * pixel,
            2,
        )))]);
    }
    div(style).into()
}

pub fn draw(props: &ParticleField, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let pixel = cx.at.stage.width / DESIGN_WIDTH;
    let time = cx.frame() / cx.fps() * props.speed;
    let field = particles(whole(f64::from(props.count)).max(1), props.seed);
    Ok(
        div(with_background(absolute_fill(), &props.background_color).overflow(Overflow::Hidden))
            .children(field.iter().map(|p| particle(props, p, time, pixel)))
            .into(),
    )
}
