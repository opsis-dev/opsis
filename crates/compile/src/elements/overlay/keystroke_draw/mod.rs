pub mod keys;
pub mod timeline;

use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, BoxSizing, Display, LineHeight, Overflow, Position, Style,
    WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::numbers::float;

use crate::elements::anim::smoothstep;
use crate::elements::code_theme::{Palette, palette_of};
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::overlay::keystroke::Keystroke;
use crate::elements::overlay::keystroke_draw::keys::{Legend, WORD_FONT, icon_shape};
use crate::elements::overlay::keystroke_draw::timeline::{
    EXIT, Group, depth, eased, glide, rise, round3, timeline,
};
use crate::elements::theme::ThemeMode;

const STAGGER: f64 = 1.0;
const FADE_IN: f64 = 4.0;
const SLIDE: f64 = 10.0;
const DIM: f64 = 6.0;
const DIM_OPACITY: f64 = 0.45;
const PUSH: f64 = 10.0;
const MIN_SIZE: f64 = 8.0;
const MAX_SIZE: f64 = 400.0;
const WALL: f64 = 0.16;
const TRAVEL: f64 = 0.12;
const RISE: f64 = 0.42;
const CHAR_FONT: f64 = 0.4;
const ICON_SIZE: f64 = 0.36;
const SEPARATOR_FONT: f64 = 0.25;
const SLOT_RISE: f64 = 1.09;
const SLOT_SCALE: f64 = 0.86;
const SLOT_OPACITY: f64 = 0.55;
const EXIT_DRIFT: f64 = 0.22;
const EXIT_SCALE: f64 = 0.04;
const SHADOW_Y: f64 = 0.07;
const SHADOW_Y_PRESSED: f64 = 0.02;
const SHADOW_BLUR: f64 = 0.11;
const SHADOW_BLUR_PRESSED: f64 = 0.03;

const LIGHT: [(&str, &str); 6] = [
    ("cap", "#fcfcfb"),
    ("edge", "#dfdfda"),
    ("wall", "#cfcfca"),
    ("ink", "#1d1d1f"),
    ("separator", "#8e8e89"),
    ("shadow", "rgba(20, 20, 16, 0.18)"),
];

const DARK: [(&str, &str); 6] = [
    ("cap", "#38393e"),
    ("edge", "#48494f"),
    ("wall", "#1f2023"),
    ("ink", "#f5f5f7"),
    ("separator", "#9d9da4"),
    ("shadow", "rgba(0, 0, 0, 0.5)"),
];

struct Metrics {
    height: f64,
    travel: f64,
    keytop: f64,
    radius: f64,
    rise: f64,
    border: f64,
    char_font: f64,
    word_font: f64,
    icon: f64,
    separator_font: f64,
}

fn metrics(size: f64, unit: f64, roundness: f64) -> Metrics {
    Metrics {
        height: round3(size),
        travel: round3(size * TRAVEL),
        keytop: round3(size * (1.0 - WALL)),
        radius: round3(size * roundness),
        rise: round3(size * RISE),
        border: round3(unit.max(1.0)),
        char_font: round3(size * CHAR_FONT),
        word_font: round3(size * WORD_FONT),
        icon: round3(size * ICON_SIZE),
        separator_font: round3(size * SEPARATOR_FONT),
    }
}

fn legend_color(ink: &str, accent: &str, amount: f64) -> Color {
    let share = (amount.clamp(0.0, 1.0) * 100.0).floor();
    if share <= 0.0 {
        return css_color(ink);
    }
    if share >= 100.0 {
        return css_color(accent);
    }
    Color::mix(
        ColorInterpolation {
            space: ColorSpace::Srgb,
            ..ColorInterpolation::DEFAULT
        },
        (css_color(accent), Some(share)),
        (css_color(ink), None),
    )
}

fn fade_in(frames: f64) -> f64 {
    eased(frames, FADE_IN, &|t| (1.0 - t).mul_add(-(1.0 - t), 1.0))
}

fn legend(legend: &Legend, metrics: &Metrics, weight: f64) -> Node {
    match legend {
        Legend::Icon(icon) => {
            let (view_box, paths) = icon_shape(*icon);
            let root = paths.iter().fold(
                SvgElement::new("svg")
                    .attr("width", metrics.icon)
                    .attr("height", metrics.icon)
                    .attr("viewBox", view_box)
                    .attr("fill", "none")
                    .attr("stroke", "currentColor")
                    .attr("stroke-width", 2)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round"),
                |root, d| root.child(SvgElement::new("path").attr("d", *d)),
            );
            svg(root, Style::default().display(Display::Block)).into()
        }
        Legend::Char(text) | Legend::Word(text) => {
            let size = if matches!(legend, Legend::Word(_)) {
                metrics.word_font
            } else {
                metrics.char_font
            };
            span(
                Style::default()
                    .font_size(size)
                    .font_weight(weight)
                    .line_height(LineHeight::Number(Num(1.0)))
                    .white_space(WhiteSpace::Nowrap),
            )
            .child(text.as_str())
            .into()
        }
    }
}

struct Look<'a> {
    metrics: &'a Metrics,
    palette: &'a Palette,
    accent: &'a str,
    weight: f64,
    then_text: &'a str,
    size: f64,
    x: f64,
    y: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn group_node(group: &Group, time: f64, look: &Look<'_>) -> Node {
    let size = look.size;
    let metrics = look.metrics;
    let slot: f64 = group
        .pushes
        .iter()
        .map(|at| eased(time - at, PUSH, &glide))
        .sum();
    let leaving = ((time - group.exit) / EXIT).clamp(0.0, 1.0);
    let pushed_away = group.pushes.iter().any(|at| *at <= group.exit);
    let drift = (if pushed_away { -1.0 } else { 1.0 }) * EXIT_DRIFT * size * leaving.powi(3);
    let dimmed = 1.0 - (1.0 - SLOT_OPACITY) * slot.min(1.0);
    let mut visible = group.layout.widths.first().copied().unwrap_or(0.0);
    for chord in 1..group.chords.len() {
        let added = group.layout.widths[chord] - group.layout.widths[chord - 1];
        visible += added * eased(time - group.timing[chord].enter, SLIDE, &glide);
    }
    let left = -visible / 2.0;
    let mut children = Vec::new();
    for separator in &group.layout.separators {
        let enter = group.timing[separator.chord].enter;
        if time <= enter {
            continue;
        }
        let y = round3((1.0 - rise(time - enter)) * metrics.rise);
        children.push(
            div(Style::default()
                .position(Position::Absolute)
                .left(round3((left + separator.x) * size))
                .top(y - metrics.height / 2.0)
                .width(round3(separator.width * size))
                .height(metrics.keytop)
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::Center)
                .opacity(round3(fade_in(time - enter)))
                .color(css_color(look.palette.get("separator")))
                .font_size(metrics.separator_font)
                .font_weight(look.weight)
                .line_height(LineHeight::Number(Num(1.0)))
                .white_space(WhiteSpace::Nowrap))
            .child(look.then_text)
            .into(),
        );
    }
    for cap in &group.layout.caps {
        let timing = &group.timing[cap.chord];
        let enter = timing.enter + float(cap.index) * STAGGER;
        if time <= enter {
            continue;
        }
        let key = &group.chords[cap.chord][cap.index];
        let pressed = depth(time, timing.keys[cap.index]);
        let landing = group
            .timing
            .get(cap.chord + 1)
            .map_or(f64::INFINITY, |t| t.fire);
        let dim = smoothstep((time - (landing - DIM)) / DIM);
        let lean = pressed.clamp(-1.0, 1.0);
        let shadow_y = SHADOW_Y - (SHADOW_Y - SHADOW_Y_PRESSED) * lean;
        let shadow_blur = SHADOW_BLUR - (SHADOW_BLUR - SHADOW_BLUR_PRESSED) * lean;
        let y = round3((1.0 - rise(time - enter)) * metrics.rise);
        let key_lift = round3(pressed * metrics.travel);
        let wall = div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .top(metrics.travel)
            .bottom(Length::ZERO)
            .border_radius_all(metrics.radius)
            .background_color(css_color(look.palette.get("wall")))
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(round3(shadow_y * size)),
                blur: Length::px(round3(shadow_blur * size)),
                spread: Length::ZERO,
                color: css_color(look.palette.get("shadow")),
            }]));
        let top = div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .top(Length::ZERO)
            .height(metrics.keytop)
            .box_sizing(BoxSizing::BorderBox)
            .border_radius_all(metrics.radius)
            .border(Edges::all(BorderSide {
                width: Length::px(metrics.border),
                style: BorderStyle::Solid,
                color: Some(css_color(look.palette.get("edge"))),
            }))
            .background_color(css_color(look.palette.get("cap")))
            .color(legend_color(
                look.palette.get("ink"),
                look.accent,
                pressed.clamp(0.0, 1.0),
            ))
            .transform(vec![Transform::Translate(
                Length::ZERO,
                Length::px(key_lift),
                Length::ZERO,
            )])
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center))
        .child(legend(&key.legend, metrics, look.weight));
        children.push(
            div(Style::default()
                .position(Position::Absolute)
                .left(round3((left + cap.x) * size))
                .top(y - metrics.height / 2.0)
                .width(round3(cap.width * size))
                .height(metrics.height)
                .opacity(round3(
                    fade_in(time - enter) * (1.0 - (1.0 - DIM_OPACITY) * dim),
                )))
            .child(wall)
            .child(top)
            .into(),
        );
    }
    let scale = round3((1.0 - (1.0 - SLOT_SCALE) * slot) * (1.0 - EXIT_SCALE * leaving));
    div(Style::default()
        .position(Position::Absolute)
        .left(Length::percent(look.x * 100.0))
        .top(Length::percent(look.y * 100.0))
        .width(Length::ZERO)
        .height(Length::ZERO)
        .opacity(round3(dimmed * (1.0 - leaving.powi(2))))
        .transform(vec![
            Transform::Translate(
                Length::ZERO,
                Length::px(round3(-slot * SLOT_RISE * size + drift)),
                Length::ZERO,
            ),
            Transform::Scale(Num(scale), Num(scale), Num(1.0)),
        ])
        .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO)))
    .children(children)
    .into()
}

pub fn draw(props: &Keystroke, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let fps = Cx::authored_fps();
    let (width, height) = (cx.at.stage.width.max(1.0), cx.at.stage.height.max(1.0));
    let unit = (height / 720.0).min(width / 1280.0);
    let reference = f64::from(props.size).clamp(MIN_SIZE, MAX_SIZE);
    let size = reference * unit;
    let metrics = metrics(size, unit, props.roundness);
    let time = cx.authored_frame().max(0.0) * props.speed.max(0.0) * 30.0 / fps;
    let base = match props.theme.mode {
        ThemeMode::Light => LIGHT,
        ThemeMode::Dark => DARK,
    };
    let palette = palette_of(&base, &[], &props.theme.colors);
    let accent = if props.accent.trim().is_empty() {
        "#0a84ff"
    } else {
        props.accent.trim()
    };
    let size = round3(size);
    let look = Look {
        metrics: &metrics,
        palette: &palette,
        accent,
        weight: f64::from(props.font_weight),
        then_text: &props.then_text,
        size,
        x: props.x,
        y: props.y,
    };
    let groups: Vec<Node> = timeline(props, fps)
        .iter()
        .filter(|g| time > g.enter && time < g.end)
        .map(|g| group_node(g, time, &look))
        .collect();
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .overflow(Overflow::Hidden)
        .font_family(font_or(&props.font_family, FontRole::Main)))
    .children(groups)
    .into())
}
