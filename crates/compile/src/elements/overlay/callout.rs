use opsis_macros::{choice, settings};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::Transform;
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Position, Style, WhiteSpace};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped_eased, exit_opacity};
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::overlay_frame::overlay_frame;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[settings(export)]
pub struct Callout {
    #[setting("Text", default = "Click here")]
    pub text: String,
    #[setting("Target X (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Target Y (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Label side", default = Side::Top)]
    pub side: Side,
    #[setting("Label colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Arrow colour", default = "#f43f5e", check = colour)]
    pub accent: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Draw time",
        default = 0.6,
        range(min = 0.1, max = 3),
        unit = "s",
        advanced
    )]
    pub draw_time: f64,
    #[setting(
        "Label distance",
        default = 170,
        range(min = 40, max = 600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_distance: u32,
    #[setting("Arrow bend", default = 40, range(min = -200, max = 200), step = 1, unit = "px", advanced)]
    pub bend: i32,
    #[setting(
        "Arrow width",
        default = 7,
        range(min = 1, max = 30),
        step = 1,
        unit = "px",
        advanced
    )]
    pub arrow_width: u32,
    #[setting(
        "Dot size",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub dot_size: u32,
    #[setting(
        "Label corner radius",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_radius: u32,
    #[setting(
        "Text size",
        default = 34,
        range(min = 12, max = 120),
        step = 1,
        unit = "px"
    )]
    pub font_size: u32,
    #[setting(
        "Font weight",
        default = 700,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Arrow slant",
        default = 19.29,
        range(min = 0, max = 60),
        step = 0.5,
        unit = "°",
        advanced
    )]
    pub slant: f64,
    #[setting(
        "Fade out",
        default = 0.3333,
        range(min = 0.0333, max = 2),
        unit = "s",
        advanced
    )]
    pub fade_out: f64,
    #[setting(
        "Label background",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub label_background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Callout>(
        element(
            ElementKind::Component,
            "callout",
            "Callout",
            "A short note that points something out.",
            ElementCategory::Overlay,
            75,
        )
        .look(NodeLook::of("lucide:message-square-quote")),
    )
}

const DASH_SLACK: f64 = 1.3;
const LABEL_PAD_Y: f64 = 12.0;
const LABEL_PAD_X: f64 = 22.0;

fn direction_of(side: Side, slant: f64) -> (f64, f64) {
    let s = slant.to_radians().tan();
    match side {
        Side::Top => (s, -1.0),
        Side::Bottom => (s, 1.0),
        Side::Left => (-1.0, -s),
        Side::Right => (1.0, -s),
    }
}

pub fn draw(props: &Callout, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let k = overlay_frame(cx.at.stage).k;
    let drawn = clamped_eased(
        frame,
        [0.0, props.draw_time * cx.fps()],
        [0.0, 1.0],
        Easing::ease_out(Easing::Cubic),
    )?;
    let off = f64::from(props.label_distance) * k;
    let (dx, dy) = direction_of(props.side, props.slant);
    let (px, py) = (props.x * width, props.y * height);
    let (sx, sy) = (f64::mul_add(dx, off, px), f64::mul_add(dy, off, py));
    let len = (dx * off).hypot(dy * off);
    let accent = props.accent.as_str();
    let path = format!(
        "M {} {} Q {} {} {} {}",
        js_number_text(sx),
        js_number_text(sy),
        js_number_text(f64::mul_add(
            f64::from(props.bend),
            k,
            f64::midpoint(sx, px)
        )),
        js_number_text(f64::midpoint(sy, py)),
        js_number_text(px),
        js_number_text(py)
    );
    let arrow = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .child(
            SvgElement::new("path")
                .attr("d", path)
                .attr("fill", "none")
                .attr("stroke", accent)
                .attr("stroke-width", f64::from(props.arrow_width) * k)
                .attr("stroke-linecap", "round")
                .attr("stroke-dasharray", len * DASH_SLACK)
                .attr("stroke-dashoffset", len * DASH_SLACK * (1.0 - drawn)),
        )
        .child(
            SvgElement::new("circle")
                .attr("cx", px)
                .attr("cy", py)
                .attr("r", f64::from(props.dot_size) * k * drawn)
                .attr("fill", accent),
        );
    let label_fill = if props.label_background.is_empty() {
        accent
    } else {
        props.label_background.as_str()
    };
    let label = div(with_background(
        Style::default()
            .position(Position::Absolute)
            .left(sx)
            .top(sy)
            .transform(vec![Transform::Translate(
                Length::percent(-50.0),
                if dy > 0.0 {
                    Length::ZERO
                } else {
                    Length::percent(-100.0)
                },
                Length::ZERO,
            )])
            .padding(Edges::axes(
                Length::px(LABEL_PAD_Y * k),
                Length::px(LABEL_PAD_X * k),
            ))
            .border_radius_all(f64::from(props.label_radius) * k),
        label_fill,
    )
    .color(cx.color(&props.color))
    .font_family(font_or(&props.font_family, FontRole::Main))
    .font_weight(f64::from(props.font_weight))
    .font_size(f64::from(props.font_size) * k)
    .opacity(drawn)
    .white_space(WhiteSpace::Nowrap))
    .child(props.text.as_str());
    Ok(div(absolute_fill().opacity(exit_opacity(
        frame,
        cx.duration(),
        Some((props.fade_out * cx.fps()).round()),
    )?))
    .child(svg(
        arrow,
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO),
    ))
    .child(label)
    .into())
}
