use opsis_scene::css::font_family;
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::{Align, Display, LineHeight, Position, Style, TextAlign, WhiteSpace};
use opsis_scene::values::{Angle, Length, Num};
use opsis_types::numbers::float;

use crate::elements::draw::css_color;
use crate::elements::stop_motion::{quantised_frame, random_between, step_index};

pub const HAND_FAMILY: &str = "\"Caveat\"";

#[derive(Debug, Clone)]
pub struct Handwrite<'a> {
    pub text: &'a str,
    pub font_size: f64,
    pub color: &'a str,
    pub delay: f64,
    pub per_step: f64,
    pub weight: f64,
    pub centered: bool,
    pub font_family: &'a str,
    pub step: f64,
    pub wobble: f64,
}

pub fn handwrite_duration(text: &str, per_step: f64, step: f64) -> f64 {
    (float(text.chars().count()) / per_step)
        .ceil()
        .mul_add(step, step)
}

pub fn handwrite(hand: &Handwrite<'_>, frame: f64) -> Node {
    let steps = step_index(
        (quantised_frame(frame, hand.step) - hand.delay).max(0.0),
        hand.step,
    )
    .max(0.0);
    let shown = (steps * hand.per_step).floor();
    let fresh_from = shown - hand.per_step.ceil();
    let chars: Vec<Node> = hand
        .text
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let at = float(i);
            let visible = at < shown;
            let mut style = Style::default()
                .display(Display::InlineBlock)
                .white_space(WhiteSpace::Pre)
                .opacity(if visible { 1.0 } else { 0.0 });
            if visible {
                let slant =
                    random_between(&format!("hw:{}:{i}:r", hand.text), -3.2, 3.2) * hand.wobble;
                let dy =
                    random_between(&format!("hw:{}:{i}:y", hand.text), -1.8, 1.8) * hand.wobble;
                let grow = if at >= fresh_from { 1.06 } else { 1.0 };
                style = style
                    .translate(Length::ZERO, Length::px(dy))
                    .rotate(Angle::deg(slant))
                    .scale(grow, grow);
            }
            span(style).child(c.to_string()).into()
        })
        .collect();
    div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(if hand.centered {
            Align::Center
        } else {
            Align::FlexStart
        }))
    .child(
        span(
            Style::default()
                .display(Display::InlineBlock)
                .font_family(font_family(&format!("{}, cursive", hand.font_family)))
                .font_weight(hand.weight)
                .font_size(hand.font_size)
                .line_height(LineHeight::Number(Num(1.15)))
                .color(css_color(hand.color))
                .white_space(WhiteSpace::PreWrap)
                .text_align(if hand.centered {
                    TextAlign::Center
                } else {
                    TextAlign::Left
                }),
        )
        .children(chars),
    )
    .into()
}
