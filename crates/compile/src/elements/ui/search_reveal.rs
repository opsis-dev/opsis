use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Align, Display, Overflow, Position, Style, WhiteSpace};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float, truncate};

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::units::graphemes;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SearchReveal {
    #[setting("Search text", default = "acme")]
    pub text: String,
    #[setting("Field width", default = 464, range(min = 320, max = 640), step = 8)]
    pub field_width: u32,
    #[setting("Font size", default = 34, range(min = 18, max = 54), step = 1)]
    pub font_size: u32,
    #[setting(
        "Time per letter",
        default = 0.1333,
        range(min = 0.0333, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub frames_per_character: f64,
    #[setting("Graphic panel", default = true)]
    pub show_panel: bool,
    #[setting("Construction guides", default = true)]
    pub show_guides: bool,
    #[setting("Text and markers", default = "#26232b", check = colour)]
    pub color: String,
    #[setting("Search field", default = "#ffffff", check = colour, advanced)]
    pub field_color: String,
    #[setting("Circles and search icon", default = "#cbc5cf", check = colour, advanced)]
    pub ring_color: String,
    #[setting("Panel color", default = "#a800b7", check = colour)]
    pub panel_color: String,
    #[setting("Graphic accents", default = "#e8f99a", check = colour)]
    pub accent_color: String,
    #[setting("Background", default = "#f5f1f5", check = colour)]
    pub background_color: String,
    #[setting("Static final pose", advanced)]
    pub reduced_motion: bool,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Font weight", default = FontWeight::W400, advanced)]
    pub weight: FontWeight,
}

pub fn definition() -> ElementMeta {
    meta::<SearchReveal>(
            element(
                ElementKind::Component,
                "search-reveal",
                "Search reveal",
                "Expanding construction circles snap into a search pill, type editable text, and reveal an abstract graphic panel.",
                ElementCategory::Ui,
                108,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:search")),
        )
}

const REVEAL_LENGTH: f64 = 90.0;
const MIN_FIELD_FONT: f64 = 16.0;
const STAGE: (f64, f64) = (1280.0, 720.0);
const CURVE: Easing = Easing::Bezier(0.22, 1.0, 0.36, 1.0);
const TYPING_START: f64 = 54.0;
const CARET_START: f64 = 52.0;
const ADVANCE: f64 = 0.65;

fn tween(frame: f64, [start, end]: [f64; 2], [from, to]: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &[start, end],
        &[from, to],
        &Interpolation {
            easing: Some(CURVE),
            ..Interpolation::clamped()
        },
    )?)
}

#[derive(Debug, Clone)]
struct State {
    text: String,
    circle_radius: f64,
    circle_offset: f64,
    circles_visible: bool,
    field_visible: bool,
    field_width: f64,
    field_height: f64,
    tilt: f64,
    icon_opacity: f64,
    panel_width: f64,
    panel_height: f64,
    panel_radius: f64,
    panel_opacity: f64,
    reveal: f64,
    caret_visible: bool,
}

fn state(props: &SearchReveal, frame: f64) -> Result<State, DrawFailure> {
    let characters = graphemes(&props.text);
    let count = float(characters.len());
    let step = props.frames_per_character.clamp(1.0, 12.0);
    let end = REVEAL_LENGTH.max(count.mul_add(step, TYPING_START) + 10.0);
    let speed = props.speed.max(0.0);
    let t = if props.reduced_motion {
        end
    } else {
        frame.max(0.0) * speed
    };
    let typed = ((t - TYPING_START) / step).floor().clamp(0.0, count);
    let width = f64::from(props.field_width).clamp(320.0, 640.0);
    let reveal = if props.show_panel {
        tween(t, [56.0, 90.0], [0.0, 1.0])?
    } else {
        0.0
    };
    let field_width = tween(t, [26.0, 48.0], [130.0, width])?;
    let field_height = tween(t, [26.0, 46.0], [24.0, 80.0])?;
    let caret_blink = exact_eq(((t - CARET_START) / 12.0).floor() % 2.0, 0.0);
    Ok(State {
        text: characters[..truncate::<usize>(typed)].concat(),
        circle_radius: tween(t, [0.0, 26.0], [250.0, 410.0])?,
        circle_offset: tween(t, [0.0, 26.0], [62.0, 185.0])?,
        circles_visible: t < 26.0,
        field_visible: t >= 26.0,
        field_width,
        field_height,
        tilt: tween(t, [26.0, 47.0], [-4.0, 0.0])?,
        icon_opacity: tween(t, [34.0, 46.0], [0.0, 1.0])?,
        panel_width: (774.0f64.max(width + 170.0) - field_width).mul_add(reveal, field_width),
        panel_height: (374.0 - field_height).mul_add(reveal, field_height),
        panel_radius: 40.0 * (1.0 - reveal),
        panel_opacity: tween(t, [56.0, 60.0], [0.0, 1.0])?,
        reveal,
        caret_visible: t >= CARET_START && t < end && (typed < count || caret_blink),
    })
}

fn cross(x: f64, y: f64, size: f64) -> SvgElement {
    let n = js_number_text;
    SvgElement::new("path").attr(
        "d",
        format!(
            "M {} {} H {} M {} {} V {}",
            n(x - size),
            n(y),
            n(x + size),
            n(x),
            n(y - size),
            n(y + size)
        ),
    )
}

fn circles(props: &SearchReveal, s: &State) -> Node {
    let rings = [-1.0, 1.0].into_iter().fold(
        SvgElement::new("g")
            .attr("fill", "none")
            .attr("stroke", props.ring_color.as_str())
            .attr("stroke-width", "1"),
        |group, side: f64| {
            let mut ring = SvgElement::new("g").child(
                SvgElement::new("circle")
                    .attr("cx", side * s.circle_offset)
                    .attr("r", s.circle_radius),
            );
            if props.show_guides {
                for edge in [-1.0, 1.0] {
                    ring = ring.child(cross(
                        side.mul_add(s.circle_offset, edge * s.circle_radius),
                        0.0,
                        6.0,
                    ));
                }
            }
            group.child(ring)
        },
    );
    svg(
        SvgElement::new("svg")
            .attr("width", "1280")
            .attr("height", "720")
            .attr("viewBox", "-640 -360 1280 720")
            .child(rings),
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .overflow(Overflow::Visible),
    )
    .into()
}

fn panel(props: &SearchReveal, s: &State) -> Node {
    let reveal = s.reveal;
    let loops = (0..30).fold(
        SvgElement::new("g")
            .attr(
                "transform",
                format!(
                    "translate(310 -45) rotate({})",
                    js_number_text(10.0f64.mul_add(reveal, -24.0))
                ),
            )
            .attr("fill", "none")
            .attr("stroke", props.field_color.as_str()),
        |g, i| {
            let i = f64::from(i);
            g.child(
                SvgElement::new("ellipse")
                    .attr("rx", i.mul_add(3.0, 82.0))
                    .attr("ry", 118.0)
                    .attr("transform", format!("rotate({})", js_number_text(i * 6.0)))
                    .attr("stroke-width", "1.4")
                    .attr("opacity", 0.35 + i / 70.0),
            )
        },
    );
    let petals = (0..10).fold(
        SvgElement::new("g")
            .attr(
                "transform",
                format!(
                    "translate(65 233) rotate({})",
                    js_number_text(reveal * 25.0)
                ),
            )
            .attr("fill", props.accent_color.as_str()),
        |g, i| {
            g.child(
                SvgElement::new("ellipse")
                    .attr("cy", "-41")
                    .attr("rx", "19")
                    .attr("ry", "48")
                    .attr(
                        "transform",
                        format!("rotate({})", js_number_text(f64::from(i) * 36.0)),
                    ),
            )
        },
    );
    let mark = SvgElement::new("g")
        .attr(
            "transform",
            format!(
                "translate(720 357) rotate({})",
                js_number_text(-18.0 * reveal)
            ),
        )
        .attr("fill", "none")
        .attr("stroke", props.accent_color.as_str())
        .attr("stroke-width", "22")
        .child(SvgElement::new("path").attr(
            "d",
            "M -68 -115 V 80 M -122 -60 L -14 26 M -125 18 L -12 -61",
        ))
        .child(
            SvgElement::new("circle")
                .attr("cx", "103")
                .attr("cy", "-35")
                .attr("r", "54"),
        );
    let art = svg(
        SvgElement::new("svg")
            .attr("width", "844")
            .attr("height", "374")
            .attr("viewBox", "0 0 844 374")
            .child(loops)
            .child(petals)
            .child(mark),
        Style::default()
            .position(Position::Absolute)
            .left(Length::percent(50.0))
            .top(Length::percent(50.0))
            .transform(vec![
                Transform::Translate(Length::percent(-50.0), Length::percent(-50.0), Length::ZERO),
                Transform::Scale(
                    Num(reveal.mul_add(-0.07, 1.07)),
                    Num(reveal.mul_add(-0.07, 1.07)),
                    Num(1.0),
                ),
            ]),
    );
    div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(-s.panel_width / 2.0))
        .top(Length::px(-s.panel_height / 2.0))
        .width(Length::px(s.panel_width))
        .height(Length::px(s.panel_height))
        .border_radius_all(s.panel_radius)
        .overflow(Overflow::Hidden)
        .opacity(s.panel_opacity)
        .background_color(css_color(&props.panel_color)))
    .child(art)
    .into()
}

fn guides(props: &SearchReveal, s: &State) -> Node {
    let n = js_number_text;
    let (w, h) = if props.show_panel {
        (s.panel_width, s.panel_height)
    } else {
        (s.field_width, s.field_height)
    };
    let lines = SvgElement::new("g")
        .attr("fill", "none")
        .attr("stroke", props.ring_color.as_str())
        .attr("stroke-width", "1")
        .attr("opacity", 0.5 * s.icon_opacity)
        .child(SvgElement::new("path").attr(
            "d",
            format!(
                "M {} {} V -360 M {} {} V 360",
                n(w / 2.0),
                n(-h / 2.0 - 14.0),
                n(-w / 2.0),
                n(h / 2.0 + 14.0)
            ),
        ));
    let marks = SvgElement::new("g")
        .attr("fill", "none")
        .attr("stroke", props.color.as_str())
        .attr("stroke-width", "1.5")
        .child(cross(-w / 2.0, h / 2.0, 9.0))
        .child(cross(w / 2.0, -h / 2.0, 9.0));
    svg(
        SvgElement::new("svg")
            .attr("width", "1280")
            .attr("height", "720")
            .attr("viewBox", "-640 -360 1280 720")
            .child(lines)
            .child(marks),
        Style::default()
            .position(Position::Absolute)
            .left(Length::px(-640.0))
            .top(Length::px(-360.0))
            .overflow(Overflow::Visible),
    )
    .into()
}

fn field(props: &SearchReveal, s: &State) -> Node {
    let font_size =
        MIN_FIELD_FONT.max(f64::from(props.font_size).clamp(18.0, 54.0).min(
            (s.field_width - 120.0) / (float(graphemes(&props.text).len()) * ADVANCE).max(1.0),
        ));
    let overflows = float(graphemes(&s.text).len()) * ADVANCE * font_size > s.field_width - 120.0;
    let icon = svg(
        SvgElement::new("svg")
            .attr("width", "28")
            .attr("height", "28")
            .attr("viewBox", "0 0 28 28")
            .attr("fill", "none")
            .attr("stroke", props.ring_color.as_str())
            .attr("stroke-width", "1.5")
            .attr("stroke-linecap", "round")
            .child(
                SvgElement::new("circle")
                    .attr("cx", "11.5")
                    .attr("cy", "11.5")
                    .attr("r", "7.5"),
            )
            .child(SvgElement::new("path").attr("d", "m17 17 7 7")),
        Style::default()
            .position(Position::Absolute)
            .right(Length::px(24.0))
            .opacity(s.icon_opacity),
    );
    let typed = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(if overflows {
            Align::FlexEnd
        } else {
            Align::Center
        })
        .max_width(Length::px(s.field_width - 116.0))
        .overflow(Overflow::Hidden)
        .white_space(WhiteSpace::Pre))
    .child(span(Style::default()).child(s.text.clone()))
    .child(span(
        Style::default()
            .width(Length::px(1.5))
            .height(Length::px(font_size * 1.12))
            .margin_left(Length::px(2.0))
            .flex_shrink(0.0)
            .background_color(css_color(&props.color))
            .opacity(if s.caret_visible { 1.0 } else { 0.0 }),
    ));
    div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(-s.field_width / 2.0))
        .top(Length::px(-s.field_height / 2.0))
        .width(Length::px(s.field_width))
        .height(Length::px(s.field_height))
        .border_radius_all(999.0)
        .background_color(css_color(&props.field_color))
        .box_shadow(vec![
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(3.0),
                blur: Length::px(12.0),
                spread: Length::ZERO,
                color: css_color("#26232b08"),
            },
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::ZERO,
                spread: Length::px(1.0),
                color: css_color("#26232b0a"),
            },
        ])
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .overflow(Overflow::Hidden)
        .color(css_color(&props.color))
        .font_family(font_or(&props.font_family, FontRole::Main))
        .font_size(font_size)
        .font_weight(props.weight.value()))
    .child(typed)
    .child(icon)
    .into()
}

pub fn draw(props: &SearchReveal, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let s = state(props, cx.authored_frame())?;
    let scale = (cx.at.stage.width / STAGE.0).min(cx.at.stage.height / STAGE.1);
    let mut stage = div(Style::default()
        .width(Length::px(STAGE.0))
        .height(Length::px(STAGE.1))
        .flex_shrink(0.0)
        .position(Position::Relative)
        .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]));
    if s.circles_visible {
        stage = stage.child(circles(props, &s));
    }
    if s.field_visible {
        let mut holder = div(Style::default()
            .position(Position::Absolute)
            .left(Length::percent(50.0))
            .top(Length::percent(50.0))
            .width(Length::ZERO)
            .height(Length::ZERO)
            .transform(vec![Transform::Rotate(Angle::deg(s.tilt))]));
        if props.show_panel {
            holder = holder.child(panel(props, &s));
        }
        if props.show_guides {
            holder = holder.child(guides(props, &s));
        }
        stage = stage.child(holder.child(field(props, &s)));
    }
    Ok(div(absolute_fill()
        .background_color(css_color(&props.background_color))
        .overflow(Overflow::Hidden)
        .align_items(Align::Center)
        .justify_content(Align::Center))
    .child(stage)
    .into())
}
