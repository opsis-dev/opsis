use opsis_scene::clock::js_round;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Shadow;
use opsis_scene::style::{Align, BorderSide, BorderStyle, Display, Position, Style, WhiteSpace};
use opsis_scene::values::{Edges, Length};
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, css_color};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::NeutralTheme;
use crate::elements::oklch_mix::mix_oklch;
use crate::elements::parts::caret::{Caret, caret};
use crate::elements::parts::ui_steps::{Step, transition, tween};
use crate::elements::units::{utf16_len, utf16_slice};

const HEIGHT: f64 = 40.0;
const PADDING: f64 = 14.0;
const FONT_SIZE: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Idle,
    Hover,
    Active,
    Typing,
    Blur,
    Invalid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputLook {
    pub border: String,
    pub ring: String,
    pub ring_width: f64,
    pub background: String,
    pub caret: f64,
    pub reveal: f64,
    pub placeholder: f64,
}

fn look(state: InputState, theme: &NeutralTheme) -> InputLook {
    let get = |key: &str| theme.get(key).to_owned();
    let ring = mix_oklch(theme.get("background"), theme.get("ring"), 0.5);
    let plain =
        |border: String, ring_width: f64, caret: f64, reveal: f64, placeholder: f64| InputLook {
            border,
            ring: ring.clone(),
            ring_width,
            background: get("background"),
            caret,
            reveal,
            placeholder,
        };
    match state {
        InputState::Hover => InputLook {
            background: mix_oklch(theme.get("background"), theme.get("muted"), 0.4),
            ..plain(
                mix_oklch(theme.get("input"), theme.get("foreground"), 0.18),
                0.0,
                0.0,
                0.0,
                1.0,
            )
        },
        InputState::Active => plain(get("ring"), 3.0, 1.0, 0.0, 1.0),
        InputState::Typing => plain(get("ring"), 3.0, 1.0, 1.0, 0.0),
        InputState::Blur => plain(get("input"), 0.0, 0.0, 1.0, 0.0),
        InputState::Invalid => InputLook {
            ring: mix_oklch(theme.get("background"), theme.get("destructive"), 0.4),
            ..plain(get("destructive"), 3.0, 0.0, 1.0, 0.0)
        },
        InputState::Idle => plain(get("input"), 0.0, 0.0, 0.0, 1.0),
    }
}

pub fn input_transition(steps: &[Step<InputState>], theme: &NeutralTheme, frame: f64) -> InputLook {
    let moved = transition(steps, InputState::Idle, frame);
    let (a, b) = (look(moved.from, theme), look(moved.to, theme));
    let t = moved.eased;
    InputLook {
        ring_width: tween(a.ring_width, b.ring_width, t),
        caret: tween(a.caret, b.caret, t),
        reveal: tween(a.reveal, b.reveal, t),
        placeholder: tween(a.placeholder, b.placeholder, t),
        border: mix_oklch(&a.border, &b.border, t),
        ring: mix_oklch(&a.ring, &b.ring, t),
        background: mix_oklch(&a.background, &b.background, t),
    }
}

#[derive(Debug)]
pub struct InputParts<'a> {
    pub placeholder: &'a str,
    pub value: &'a str,
    pub look: &'a InputLook,
    pub theme: &'a NeutralTheme,
    pub width: Length,
}

pub fn input(parts: &InputParts<'_>, cx: &Cx<'_>) -> Node {
    let look = parts.look;
    let theme = parts.theme;
    let length = utf16_len(parts.value);
    let shown = js_round(float(length) * look.reveal).clamp(0.0, float(length));
    let revealed = utf16_slice(
        parts.value,
        0,
        usize::try_from(opsis_types::numbers::whole(shown)).unwrap_or(0),
    );
    let mark = caret(
        &Caret {
            color: css_color(theme.get("foreground")),
            height: Length::px(js_round(FONT_SIZE * 1.1)),
            radius: 1.0,
            opacity: Some(look.caret),
            margin_left: if revealed.is_empty() { 0.0 } else { 4.0 },
            ..Caret::default()
        },
        cx,
        &Style::default(),
    );
    div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(font_or("", FontRole::Main)))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .display(Display::Flex)
            .align_items(Align::Center)
            .width(parts.width.clone())
            .height(Length::px(HEIGHT))
            .padding(Edges::axes(Length::ZERO, Length::px(PADDING)))
            .font_size(FONT_SIZE)
            .letter_spacing(Length::em(-0.01))
            .background_color(css_color(&look.background))
            .border(Edges::all(BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color(&look.border)),
            }))
            .border_radius_all(theme.radius)
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::ZERO,
                spread: Length::px(look.ring_width),
                color: css_color(&look.ring),
            }]))
        .child(
            span(
                Style::default()
                    .position(Position::Absolute)
                    .left(Length::px(PADDING))
                    .color(css_color(theme.get("mutedForeground")))
                    .opacity(if look.reveal > 0.0 {
                        0.0
                    } else {
                        look.placeholder
                    })
                    .white_space(WhiteSpace::Nowrap),
            )
            .child(parts.placeholder.to_owned()),
        )
        .child(
            div(Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .min_width(Length::ZERO))
            .child(
                span(
                    Style::default()
                        .white_space(WhiteSpace::Nowrap)
                        .color(css_color(theme.get("foreground"))),
                )
                .child(revealed),
            )
            .child(mark),
        ),
    )
    .into()
}
