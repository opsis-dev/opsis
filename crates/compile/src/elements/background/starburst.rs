use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{ColorStop, Gradient, RadialShape};
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Length, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Starburst {
    #[setting("Ray A", default = "#18181b", check = colour)]
    pub color1: String,
    #[setting("Ray B", default = "#27272a", check = colour)]
    pub color2: String,
    #[setting("Rays", default = 24, range(min = 6, max = 64), step = 1)]
    pub rays: u32,
    #[setting(
        "Rotation speed",
        default = 0.15,
        range(min = 0, max = 2),
        step = 0.05,
        unit = "°/frame"
    )]
    pub rotation_speed: f64,
    #[setting(
        "Edge fade",
        default = 75.0,
        range(min = 20, max = 150),
        unit = "%",
        advanced
    )]
    pub edge_fade: f64,
    #[setting(
        "Ray width",
        default = 50.0,
        range(min = 10, max = 90),
        unit = "%",
        advanced
    )]
    pub ray_width: f64,
    #[setting(
        "Centre across",
        default = 50.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub center_x: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Starburst>(
        element(
            ElementKind::Component,
            "starburst",
            "Starburst",
            "Rays spreading from the centre.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:sun")),
    )
}

const OVERSCAN_PERCENT: f64 = -50.0;
const TURN_DEGREES: f64 = 360.0;
const MIN_RAYS: u32 = 2;

fn of_turn(degrees: f64) -> Length {
    Length::percent(degrees / TURN_DEGREES * 100.0)
}

pub fn draw(props: &Starburst, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (ray_a, ray_b) = (cx.color(&props.color1), cx.color(&props.color2));
    let step = TURN_DEGREES / f64::from(props.rays.max(MIN_RAYS));
    let split = step * props.ray_width / 100.0;
    let across = f64::midpoint(props.center_x, 50.0);
    let rays = Gradient::conic(
        Angle::deg(cx.frame() * props.rotation_speed),
        Position2 {
            x: Length::percent(across),
            y: Length::percent(50.0),
        },
        vec![
            ColorStop::span(ray_a.clone(), of_turn(0.0), of_turn(split)),
            ColorStop::span(ray_b, of_turn(split), of_turn(step)),
        ],
    )
    .repeating();
    let fade = Gradient::radial(
        RadialShape::Circle,
        Position2 {
            x: Length::percent(props.center_x),
            y: Length::percent(50.0),
        },
        vec![
            ColorStop::at(Color::TRANSPARENT, Length::percent(0.0)),
            ColorStop::at(ray_a.clone(), Length::percent(props.edge_fade)),
        ],
    );
    Ok(div(absolute_fill()
        .overflow(Overflow::Hidden)
        .background_color(ray_a))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::percent(OVERSCAN_PERCENT))
        .background(vec![rays.layer()])))
    .child(div(absolute_fill().background(vec![fade.layer()])))
    .into())
}
