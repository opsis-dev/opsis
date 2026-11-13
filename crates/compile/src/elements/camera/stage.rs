use opsis_scene::css::with_background;
use opsis_scene::node::{Element, Node, div, img};
use opsis_scene::paint::{Filter, Shadow, Transform};
use opsis_scene::style::{Align, ObjectFit, Overflow, Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};

use crate::backdrops::{Backdrop, DEFAULT_BACKDROP, Tone};
use crate::elements::camera::window::{WINDOW_RADIUS, ink};
use crate::elements::draw::css_color;
use crate::elements::oklch_mix::parsed_rgb;

const LIGHT_LUMA: f64 = 0.6;
const GLOW_RADIUS: f64 = 60.0;
const GLOW_BLUR: f64 = 90.0;
const RIM_RADIUS: f64 = WINDOW_RADIUS + 1.0;
const PAINT_PREFIXES: [&str; 7] = [
    "#",
    "rgb",
    "hsl",
    "oklch",
    "linear-gradient",
    "radial-gradient",
    "conic-gradient",
];
const ADDRESS_PREFIXES: [&str; 4] = ["http:", "https:", "data:", "blob:"];

fn named(value: &str) -> Option<Backdrop> {
    Backdrop::named(value)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film weighs the channels with separate multiplies and adds"
)]
fn luma(colour: &str) -> Option<f64> {
    let rgb = parsed_rgb(colour.trim())?;
    Some(0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b)
}

pub fn is_light_backdrop(backdrop: &str) -> bool {
    let value = backdrop.trim();
    named(value).map_or_else(
        || luma(value).unwrap_or(0.0) > LIGHT_LUMA,
        |b| b.tone() == Tone::Light,
    )
}

fn backdrop_file(name: &str) -> String {
    let file = if name.trim().is_empty() {
        DEFAULT_BACKDROP.id()
    } else {
        name.trim()
    };
    if file.contains('.') {
        format!("backdrops/{file}")
    } else {
        format!("backdrops/{file}.jpg")
    }
}

pub fn backdrop(name: &str) -> Node {
    let value = name.trim();
    let paint = PAINT_PREFIXES.iter().any(|p| value.starts_with(p))
        || (named(value).is_none() && !value.is_empty() && parsed_rgb(value).is_some());
    if paint {
        return div(with_background(absolute_fill(), value)).into();
    }
    let source = if ADDRESS_PREFIXES.iter().any(|p| value.starts_with(p)) || value.starts_with('/')
    {
        value.to_owned()
    } else {
        format!("app:{}", backdrop_file(value))
    };
    div(absolute_fill())
        .child(img(
            source,
            Style::default()
                .width(Length::percent(100.0))
                .height(Length::percent(100.0))
                .object_fit(ObjectFit::Cover),
        ))
        .into()
}

pub fn stage(children: Vec<Node>) -> Node {
    div(absolute_fill()
        .justify_content(Align::Center)
        .align_items(Align::Center)
        .overflow(Overflow::Hidden))
    .children(children)
    .into()
}

pub fn rim_light(light: bool, strength: f64, k: f64) -> Vec<Node> {
    let (glow, lift, rim) = if light {
        ("rgba(30,20,10,0.30)", 2.5, ink(0.16))
    } else {
        (
            "rgba(255,255,255,0.10)",
            2.0,
            "rgba(255,255,255,0.14)".to_owned(),
        )
    };
    let halo: Element = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .border_radius_all(GLOW_RADIUS * k)
        .background_color(css_color(glow))
        .transform(vec![
            Transform::Translate(Length::ZERO, Length::percent(lift), Length::ZERO),
            Transform::Scale(Num(0.97), Num(0.96), Num(1.0)),
        ])
        .filter(vec![Filter::Blur(Length::px(GLOW_BLUR * k))])
        .opacity(strength));
    let edge: Element = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::px(-k))
        .border_radius_all(RIM_RADIUS * k)
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::ZERO,
            spread: Length::px(k),
            color: css_color(&rim),
        }]));
    vec![halo.into(), edge.into()]
}
