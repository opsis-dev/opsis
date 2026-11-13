use opsis_scene::clock::js_round;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::Transform;
use opsis_scene::style::{Align, BorderSide, BorderStyle, Display, Position, Style};
use opsis_scene::values::{Edges, Length, Num};

use crate::elements::draw::css_color;
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::NeutralTheme;
use crate::elements::oklch_mix::mix_oklch;
use crate::elements::parts::spinner::spinner;
use crate::elements::parts::ui_steps::{Step, transition, tween};
use crate::elements::shared::ButtonVariant;

const TRANSPARENT: &str = "transparent";
const FONT_SIZE: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Hover,
    Press,
    Loading,
    Success,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonLook {
    pub translate_y: f64,
    pub scale: f64,
    pub background: String,
    pub label_opacity: f64,
    pub spinner_opacity: f64,
    pub check_opacity: f64,
}

struct Tokens {
    bg: String,
    fg: String,
    hover: String,
    border: String,
}

fn tokens(variant: ButtonVariant, theme: &NeutralTheme) -> Tokens {
    let t = |key: &str| theme.get(key).to_owned();
    match variant {
        ButtonVariant::Secondary => Tokens {
            bg: t("secondary"),
            fg: t("secondaryForeground"),
            hover: mix_oklch(theme.get("secondary"), theme.get("muted"), 1.0),
            border: TRANSPARENT.to_owned(),
        },
        ButtonVariant::Destructive => Tokens {
            bg: t("destructive"),
            fg: t("destructiveForeground"),
            hover: mix_oklch(theme.get("destructive"), theme.get("foreground"), 0.12),
            border: TRANSPARENT.to_owned(),
        },
        ButtonVariant::Outline => Tokens {
            bg: TRANSPARENT.to_owned(),
            fg: t("foreground"),
            hover: t("accent"),
            border: t("border"),
        },
        ButtonVariant::Ghost => Tokens {
            bg: TRANSPARENT.to_owned(),
            fg: t("foreground"),
            hover: t("accent"),
            border: TRANSPARENT.to_owned(),
        },
        ButtonVariant::Default => Tokens {
            bg: t("primary"),
            fg: t("primaryForeground"),
            hover: mix_oklch(theme.get("primary"), theme.get("foreground"), 0.1),
            border: TRANSPARENT.to_owned(),
        },
    }
}

fn look(state: ButtonState, variant: ButtonVariant, theme: &NeutralTheme) -> ButtonLook {
    let tokens = tokens(variant, theme);
    let see_through = tokens.bg == TRANSPARENT;
    let rest = if see_through {
        theme.get("background").to_owned()
    } else {
        tokens.bg.clone()
    };
    let press = if see_through {
        tokens.hover.clone()
    } else {
        mix_oklch(&tokens.hover, theme.get("foreground"), 0.08)
    };
    let (translate_y, scale, background, label, spin, check) = match state {
        ButtonState::Hover => (-1.0, 1.0, tokens.hover, 1.0, 0.0, 0.0),
        ButtonState::Press => (-1.0, 0.97, press, 1.0, 0.0, 0.0),
        ButtonState::Loading => (-1.0, 1.0, tokens.hover, 0.0, 1.0, 0.0),
        ButtonState::Success => (-1.0, 1.0, theme.get("primary").to_owned(), 0.0, 0.0, 1.0),
        ButtonState::Idle => (0.0, 1.0, rest, 1.0, 0.0, 0.0),
    };
    ButtonLook {
        translate_y,
        scale,
        background,
        label_opacity: label,
        spinner_opacity: spin,
        check_opacity: check,
    }
}

pub fn button_transition(
    steps: &[Step<ButtonState>],
    variant: ButtonVariant,
    theme: &NeutralTheme,
    frame: f64,
) -> ButtonLook {
    let moved = transition(steps, ButtonState::Idle, frame);
    let (a, b) = (
        look(moved.from, variant, theme),
        look(moved.to, variant, theme),
    );
    let t = moved.eased;
    ButtonLook {
        translate_y: tween(a.translate_y, b.translate_y, t),
        scale: tween(a.scale, b.scale, t),
        background: mix_oklch(&a.background, &b.background, t),
        label_opacity: tween(a.label_opacity, b.label_opacity, t),
        spinner_opacity: tween(a.spinner_opacity, b.spinner_opacity, t),
        check_opacity: tween(a.check_opacity, b.check_opacity, t),
    }
}

fn overlay(opacity: f64, child: Node) -> Node {
    span(
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .opacity(opacity),
    )
    .child(child)
    .into()
}

fn check(size: f64, color: &str) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr("d", "M5 12.5l4.5 4.5L19 7")
                    .attr("stroke", color)
                    .attr("stroke-width", "2.6")
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round"),
            ),
        Style::default()
            .width(Length::px(size))
            .height(Length::px(size)),
    )
    .into()
}

#[derive(Debug)]
pub struct ButtonParts<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub look: &'a ButtonLook,
    pub theme: &'a NeutralTheme,
    pub spin: f64,
}

pub fn button(parts: &ButtonParts<'_>) -> Node {
    let tokens = tokens(parts.variant, parts.theme);
    let look = parts.look;
    let icon = js_round(FONT_SIZE * 1.1);
    let border = if parts.variant == ButtonVariant::Outline {
        tokens.border.as_str()
    } else {
        TRANSPARENT
    };
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
            .transform(vec![
                Transform::Translate(Length::ZERO, Length::px(look.translate_y), Length::ZERO),
                Transform::Scale(Num(look.scale), Num(look.scale), Num(1.0)),
            ])
            .display(Display::InlineFlex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .gap(8.0)
            .height(Length::px(40.0))
            .padding(Edges::axes(Length::ZERO, Length::px(20.0)))
            .font_size(FONT_SIZE)
            .font_weight(500.0)
            .letter_spacing(Length::em(-0.01))
            .color(css_color(&tokens.fg))
            .background_color(css_color(&look.background))
            .border(Edges::all(BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color(border)),
            }))
            .border_radius_all(parts.theme.radius))
        .child(
            span(
                Style::default()
                    .position(Position::Relative)
                    .display(Display::InlineFlex),
            )
            .child(span(Style::default().opacity(look.label_opacity)).child(parts.label.to_owned()))
            .child(overlay(
                look.spinner_opacity,
                spinner(icon, &tokens.fg, parts.spin),
            ))
            .child(overlay(look.check_opacity, check(icon, &tokens.fg))),
        ),
    )
    .into()
}
