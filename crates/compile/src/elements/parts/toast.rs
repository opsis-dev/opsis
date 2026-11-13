use opsis_scene::color::Color;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, LineHeight, Style,
};
use opsis_scene::values::{Edges, Length, Num};

use crate::elements::draw::css_color;
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::NeutralTheme;
use crate::elements::parts::ui_steps::{Step, transition, tween};

const WIDTH: f64 = 356.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastState {
    Hidden,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Default,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToastLook {
    pub opacity: f64,
    pub translate_y: f64,
    pub scale: f64,
}

const fn look(state: ToastState) -> ToastLook {
    match state {
        ToastState::Visible => ToastLook {
            opacity: 1.0,
            translate_y: 0.0,
            scale: 1.0,
        },
        ToastState::Hidden => ToastLook {
            opacity: 0.0,
            translate_y: 16.0,
            scale: 0.97,
        },
    }
}

pub fn toast_transition(steps: &[Step<ToastState>], frame: f64) -> ToastLook {
    let moved = transition(steps, ToastState::Hidden, frame);
    let (a, b) = (look(moved.from), look(moved.to));
    let t = moved.eased;
    ToastLook {
        opacity: tween(a.opacity, b.opacity, t),
        translate_y: tween(a.translate_y, b.translate_y, t),
        scale: tween(a.scale, b.scale, t),
    }
}

fn icon(variant: ToastVariant, color: &str) -> Node {
    let ring = SvgElement::new("circle")
        .attr("cx", "12")
        .attr("cy", "12")
        .attr("r", "9")
        .attr("stroke", color)
        .attr("stroke-width", "2");
    let line = |d: &str| {
        SvgElement::new("path")
            .attr("d", d)
            .attr("stroke", color)
            .attr("stroke-width", "2")
            .attr("stroke-linecap", "round")
    };
    let dot = |cy: &str| {
        SvgElement::new("circle")
            .attr("cx", "12")
            .attr("cy", cy)
            .attr("r", "1.1")
            .attr("fill", color)
    };
    let root = SvgElement::new("svg")
        .attr("width", 18.0)
        .attr("height", 18.0)
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .child(ring);
    let root = match variant {
        ToastVariant::Success => {
            root.child(line("M8 12.5l2.6 2.6L16 9").attr("stroke-linejoin", "round"))
        }
        ToastVariant::Error => root.child(line("M12 7.5v5")).child(dot("16")),
        ToastVariant::Default => root.child(line("M12 11v5")).child(dot("8")),
    };
    svg(
        root,
        Style::default()
            .width(Length::px(18.0))
            .height(Length::px(18.0)),
    )
    .into()
}

pub fn toast(title: &str, variant: ToastVariant, look: &ToastLook, theme: &NeutralTheme) -> Node {
    let color = match variant {
        ToastVariant::Success => theme.get("success"),
        ToastVariant::Error => theme.get("destructive"),
        ToastVariant::Default => theme.get("mutedForeground"),
    };
    let shade = |y: f64, blur: f64, spread: f64| Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(y),
        blur: Length::px(blur),
        spread: Length::px(spread),
        color: Color::srgb_alpha(0.0, 0.0, 0.0, 0.1),
    };
    div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(12.0)
        .width(Length::px(WIDTH))
        .padding(Edges::all(Length::px(16.0)))
        .background_color(css_color(theme.get("popover")))
        .color(css_color(theme.get("popoverForeground")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(css_color(theme.get("border"))),
        }))
        .border_radius_all(theme.radius)
        .box_shadow(vec![shade(10.0, 15.0, -3.0), shade(4.0, 6.0, -4.0)])
        .opacity(look.opacity)
        .transform(vec![
            Transform::Translate(Length::ZERO, Length::px(look.translate_y), Length::ZERO),
            Transform::Scale(Num(look.scale), Num(look.scale), Num(1.0)),
        ])
        .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO))
        .font_family(font_or("", FontRole::Main)))
    .child(
        span(
            Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::Center),
        )
        .child(icon(variant, color)),
    )
    .child(
        div(Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(4.0)
            .min_width(Length::ZERO))
        .child(
            span(
                Style::default()
                    .font_size(14.0)
                    .font_weight(500.0)
                    .line_height(LineHeight::Number(Num(1.3)))
                    .letter_spacing(Length::em(-0.01)),
            )
            .child(title.to_owned()),
        ),
    )
    .into()
}
