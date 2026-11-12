use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{
    ClipPath, ColorStop, Gradient, RadialShape, RadialSize, Shadow, Transform,
};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{Align, BorderSide, BorderStyle, Display, Overflow, Position, Style};
use opsis_scene::values::{Corners, Edges, Length, Num, Position2};

use crate::elements::anim::sprung;
use crate::elements::code_theme::Palette;
use crate::elements::draw::{DrawFailure, css_color, tinted};

#[derive(Debug, Clone, Copy)]
pub struct CodeWindow {
    pub u: f64,
    pub width: f64,
    pub radius: f64,
    pub open: f64,
    pub rise: f64,
}

pub fn bottom_line(color: &str) -> Edges<BorderSide> {
    Edges {
        top: BorderSide::default(),
        right: BorderSide::default(),
        bottom: BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(css_color(color)),
        },
        left: BorderSide::default(),
    }
}

pub fn code_window(palette: &Palette, window: CodeWindow) -> Style {
    let u = window.u;
    Style::default()
        .width(Length::px(window.width))
        .border_radius_all(window.radius)
        .background_color(css_color(palette.get("window")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(css_color(palette.get("border"))),
        }))
        .box_shadow(vec![
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::px(1.0),
                blur: Length::ZERO,
                spread: Length::ZERO,
                color: css_color(palette.get("highlight")),
            },
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(24.0 * u),
                blur: Length::px(60.0 * u),
                spread: Length::ZERO,
                color: css_color(palette.get("shadow")),
            },
        ])
        .overflow(Overflow::Hidden)
        .clip_path(ClipPath::Inset {
            edges: Edges {
                top: Length::ZERO,
                right: Length::ZERO,
                bottom: Length::percent((1.0 - window.open) * 100.0),
                left: Length::ZERO,
            },
            round: Some(Corners::all((Length::px(18.0 * u), Length::px(18.0 * u)))),
        })
        .translate(Length::ZERO, Length::px(window.rise))
}

const LIGHTS: [&str; 3] = ["#FF5F57", "#FEBC2E", "#28C840"];
const DARK_VIGNETTE: &str =
    "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 45%, rgba(0,0,0,0.55) 100%)";
const LIGHT_VIGNETTE: &str =
    "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 55%, rgba(15,18,25,0.07) 100%)";
const ENTER_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 120.0,
    mass: 0.8,
    overshoot_clamping: false,
};

pub fn editor_enter(frame: f64, fps: f64) -> Result<f64, DrawFailure> {
    sprung(frame, fps, ENTER_SPRING)
}

pub fn backdrop_glow(accent: &str, glow: bool, opacity: f64) -> Node {
    let gradient = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(70.0), Length::percent(50.0)),
        center: Position2 {
            x: Length::percent(50.0),
            y: Length::percent(38.0),
        },
        stops: vec![
            ColorStop::new(tinted(accent, if glow { 11.0 } else { 0.0 })),
            ColorStop::at(Color::TRANSPARENT, Length::percent(70.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![gradient.layer()])
        .opacity(opacity))
    .into()
}

pub fn backdrop_vignette(vignette: bool, dark: bool) -> Node {
    let style = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO);
    let style = match (vignette, dark) {
        (false, _) => style,
        (true, true) => with_background(style, DARK_VIGNETTE),
        (true, false) => with_background(style, LIGHT_VIGNETTE),
    };
    div(style).into()
}

pub fn traffic_lights(show: bool) -> Node {
    div(Style::default()
        .display(if show { Display::Flex } else { Display::None })
        .gap(8.0))
    .children(
        LIGHTS
            .iter()
            .map(|c| {
                div(Style::default()
                    .width(Length::px(11.0))
                    .height(Length::px(11.0))
                    .border_radius_all(Length::percent(50.0))
                    .background_color(css_color(c)))
                .into()
            })
            .collect::<Vec<Node>>(),
    )
    .into()
}

pub fn editor_window(
    palette: &Palette,
    width: f64,
    scale: f64,
    enter: f64,
    radius: f64,
) -> Result<Style, DrawFailure> {
    let shrink = interpolate(enter, &[0.0, 1.0], &[0.965, 1.0], &Interpolation::default())?;
    let rise = interpolate(enter, &[0.0, 1.0], &[26.0, 0.0], &Interpolation::default())?;
    Ok(Style::default()
        .width(Length::px(width))
        .transform(vec![
            Transform::Scale(Num(scale * shrink), Num(scale * shrink), Num(1.0)),
            Transform::Translate(Length::ZERO, Length::px(rise), Length::ZERO),
        ])
        .opacity(enter)
        .border_radius_all(radius)
        .background_color(css_color(palette.get("window")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(css_color(palette.get("border"))),
        }))
        .box_shadow(vec![
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::px(1.0),
                blur: Length::ZERO,
                spread: Length::ZERO,
                color: css_color(palette.get("highlight")),
            },
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(34.0),
                blur: Length::px(90.0),
                spread: Length::ZERO,
                color: css_color(palette.get("shadow")),
            },
        ])
        .overflow(Overflow::Hidden))
}

pub fn editor_stage(width: f64, height: f64, page: &str, font: Vec<String>) -> Style {
    with_background(
        Style::default()
            .width(Length::px(width))
            .height(Length::px(height)),
        page,
    )
    .font_family(font)
    .display(Display::Flex)
    .align_items(Align::Center)
    .justify_content(Align::Center)
    .overflow(Overflow::Hidden)
    .position(Position::Relative)
}
