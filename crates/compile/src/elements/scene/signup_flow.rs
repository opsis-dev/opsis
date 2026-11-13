use opsis_macros::{choice, settings};
use opsis_scene::color::Color;
use opsis_scene::node::{Element, Node, SvgElement, div, svg};
use opsis_scene::paint::{Filter, Shadow, Transform};
use opsis_scene::style::{
    BorderSide, BorderStyle, Display, FlexDirection, LineHeight, Overflow, Position, Style,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::{NeutralTheme, neutral_theme};
use crate::elements::parts::button::{ButtonParts, ButtonState, button, button_transition};
use crate::elements::parts::input::{InputParts, InputState, input, input_transition};
use crate::elements::parts::toast::{ToastState, ToastVariant, toast, toast_transition};
use crate::elements::parts::ui_steps::{Step, ease_out, transition, tween};
use crate::elements::shared::{ButtonVariant, Corner};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum CursorVariant {
    Arrow,
    Pointer,
}

#[settings(export)]
pub struct SignupFlow {
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Title", default = "Create an account")]
    pub title: String,
    #[setting(
        "Description",
        default = "Enter your information below to create your account"
    )]
    pub description: String,
    #[setting("Name typed in", default = "John Doe")]
    pub full_name: String,
    #[setting("Email typed in", default = "m@example.com")]
    pub email: String,
    #[setting("Password shown", default = "••••••••")]
    pub password: String,
    #[setting("Button label", default = "Create account")]
    pub create_label: String,
    #[setting("Notice", default = "Account created")]
    pub toast_title: String,
    #[setting("Name label", default = "Full Name")]
    pub name_label: String,
    #[setting("Email label", default = "Email")]
    pub email_label: String,
    #[setting("Email help", default = "We'll use this to contact you.")]
    pub email_help: String,
    #[setting("Password label", default = "Password")]
    pub password_label: String,
    #[setting("Password help", default = "Must be at least 8 characters long.")]
    pub password_help: String,
    #[setting("Confirm label", default = "Confirm Password")]
    pub confirm_label: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Loading time",
        default = 1.4,
        range(min = 0.2, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub loading_seconds: f64,
    #[setting(
        "Notice shown for",
        default = 2.2,
        range(min = 0.5, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub notice_seconds: f64,
    #[setting("Cursor", default = CursorVariant::Pointer, advanced)]
    pub cursor_shape: CursorVariant,
    #[setting(
        "Cursor size",
        default = 28,
        range(min = 16, max = 64),
        step = 1,
        unit = "px",
        advanced
    )]
    pub cursor_size: u32,
    #[setting("Notice position", default = Corner::BottomRight, advanced)]
    pub notice_corner: Corner,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background_color: String,
    #[setting(
        "Corner radius",
        default = 10,
        range(min = 0, max = 32),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<SignupFlow>(
        element(
            ElementKind::Component,
            "signup-flow",
            "Signup flow",
            "A sign-up form that fills itself in, step by step.",
            ElementCategory::Scene,
            240,
        )
        .authored_clock()
        .exact_design(1280, 720)
        .look(NodeLook::of("lucide:user-plus")),
    )
}

const STAGE_W: f64 = 1280.0;
const CARD_W: f64 = 376.0;
const CARD_TOP: f64 = 48.0;
const CENTER_X: f64 = STAGE_W / 2.0;
const NAME_Y: f64 = 216.0;
const EMAIL_Y: f64 = 296.0;
const PASS_Y: f64 = 398.0;
const CONFIRM_Y: f64 = 500.0;
const CREATE_Y: f64 = 564.0;
const DEMO: f64 = 48.0;
const NOTICE_INSET: f64 = 32.0;
const BLUR: f64 = 8.0;
const RISE: f64 = 12.0;
const CLICK_FRAMES: f64 = 16.0;
const PRESS_FRAMES: f64 = 8.0;
const ARROW_PATH: &str = "M1 1 L1 18.5 L5.6 14.4 L8.7 21.6 L11.6 20.4 L8.5 13.2 L14.5 13.2 Z";
const POINTER_PATH: &str = "M8 2.2 C8 1.3 8.7 0.7 9.5 0.7 C10.3 0.7 11 1.3 11 2.2 L11 9 C11 9 11.3 8.4 12.2 8.4 C13 8.4 13.4 9 13.4 9.6 C13.4 9.6 13.9 9.1 14.7 9.1 C15.5 9.1 15.9 9.7 15.9 10.3 C15.9 10.3 16.4 9.9 17.1 9.9 C17.9 9.9 18.3 10.5 18.3 11.2 L18.3 16.8 C18.3 19.8 16.3 22.3 12.8 22.3 L11.4 22.3 C9 22.3 7.9 21.2 6.6 19.2 L4 15.2 C3.5 14.4 3.7 13.4 4.5 12.9 C5.1 12.5 5.9 12.6 6.4 13.2 L8 15 Z";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reveal {
    Hidden,
    Revealed,
}

#[derive(Debug, Clone, Copy)]
struct BlurIn {
    blur: f64,
    opacity: f64,
    rise: f64,
}

const fn blur_look(state: Reveal, distance: f64) -> BlurIn {
    match state {
        Reveal::Revealed => BlurIn {
            blur: 0.0,
            opacity: 1.0,
            rise: 0.0,
        },
        Reveal::Hidden => BlurIn {
            blur: BLUR,
            opacity: 0.0,
            rise: distance,
        },
    }
}

fn blur_in(at: f64, duration: f64, distance: f64, frame: f64) -> BlurIn {
    let moved = transition(
        &[Step {
            at,
            state: Reveal::Revealed,
            duration,
        }],
        Reveal::Hidden,
        frame,
    );
    let (a, b) = (
        blur_look(moved.from, distance),
        blur_look(moved.to, distance),
    );
    BlurIn {
        blur: tween(a.blur, b.blur, moved.eased),
        opacity: tween(a.opacity, b.opacity, moved.eased),
        rise: tween(a.rise, b.rise, moved.eased),
    }
}

fn blurred(style: Style, look: &BlurIn) -> Style {
    let style = style.opacity(look.opacity);
    if look.blur > 0.0 {
        style.filter(vec![Filter::Blur(Length::px(look.blur))])
    } else {
        style
    }
}

fn revealed(look: &BlurIn, child: impl Into<Node>) -> Node {
    div(
        blurred(Style::default().display(Display::Block), look).transform(vec![
            Transform::Translate(Length::ZERO, Length::px(look.rise), Length::ZERO),
        ]),
    )
    .child(child)
    .into()
}

struct Waypoint {
    at: f64,
    x: f64,
    y: f64,
    duration: f64,
}

#[derive(Debug, Clone, Copy)]
struct CursorLook {
    x: f64,
    y: f64,
    ripple_opacity: f64,
    ripple_scale: f64,
    press: f64,
}

fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0f64).mul_add(t, 2.0).powi(3) / 2.0
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film moves the cursor with separate multiplies and adds"
)]
fn cursor_at(points: &[Waypoint], frame: f64) -> CursorLook {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return CursorLook {
            x: 0.0,
            y: 0.0,
            ripple_opacity: 0.0,
            ripple_scale: 0.0,
            press: 1.0,
        };
    };
    if frame <= first.at {
        return CursorLook {
            x: first.x,
            y: first.y,
            ripple_opacity: 0.0,
            ripple_scale: 0.0,
            press: 1.0,
        };
    }
    let past = frame >= last.at;
    let index = points
        .iter()
        .skip(1)
        .position(|p| p.at > frame)
        .map_or(points.len() - 1, |i| i + 1);
    let (from, to) = if past {
        (last, last)
    } else {
        (&points[index - 1], &points[index])
    };
    let start = to.at - to.duration;
    let t = if past || to.duration <= 0.0 {
        1.0
    } else {
        ease_in_out(((frame - start) / to.duration).clamp(0.0, 1.0))
    };
    let clicked = points
        .iter()
        .skip(1)
        .filter(|p| p.at <= frame)
        .map(|p| p.at)
        .fold(f64::NEG_INFINITY, f64::max);
    let since = if clicked == f64::NEG_INFINITY {
        -1.0
    } else {
        frame - clicked
    };
    let (ripple_opacity, ripple_scale) = if (0.0..CLICK_FRAMES).contains(&since) {
        let p = (since / CLICK_FRAMES).clamp(0.0, 1.0);
        (0.5 * (1.0 - p), 2.5 * ease_out(p))
    } else {
        (0.0, 0.0)
    };
    let press = if (0.0..PRESS_FRAMES).contains(&since) {
        let half = PRESS_FRAMES / 2.0;
        let p = if since < half {
            since / half
        } else {
            1.0 - (since - half) / half
        };
        1.0 - p.clamp(0.0, 1.0)
    } else {
        1.0
    };
    CursorLook {
        x: from.x + (to.x - from.x) * t,
        y: from.y + (to.y - from.y) * t,
        ripple_opacity,
        ripple_scale,
        press,
    }
}

fn cursor(look: &CursorLook, variant: CursorVariant, size: f64, theme: &NeutralTheme) -> Node {
    let pressed = 0.1f64.mul_add(look.press, 0.9);
    let ripple = size * 1.8;
    div(Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .transform(vec![Transform::Translate(
            Length::px(look.x),
            Length::px(look.y),
            Length::ZERO,
        )]))
    .child(div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(-ripple / 2.0))
        .top(Length::px(-ripple / 2.0))
        .width(Length::px(ripple))
        .height(Length::px(ripple))
        .border_radius_all(Length::percent(50.0))
        .border(Edges::all(BorderSide {
            width: Length::px(2.0),
            style: BorderStyle::Solid,
            color: Some(css_color(theme.get("primary"))),
        }))
        .opacity(look.ripple_opacity)
        .transform(vec![Transform::Scale(
            Num(look.ripple_scale),
            Num(look.ripple_scale),
            Num(1.0),
        )])))
    .child(svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr(
                        "d",
                        if variant == CursorVariant::Pointer {
                            POINTER_PATH
                        } else {
                            ARROW_PATH
                        },
                    )
                    .attr("fill", theme.get("foreground"))
                    .attr("stroke", theme.get("background"))
                    .attr("stroke-width", 1.4)
                    .attr("stroke-linejoin", "round"),
            ),
        Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .top(Length::ZERO)
            .width(Length::px(size))
            .height(Length::px(size))
            .transform(vec![Transform::Scale(Num(pressed), Num(pressed), Num(1.0))])
            .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO))
            .filter(vec![Filter::DropShadow(Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(1.0),
                blur: Length::px(2.0),
                spread: Length::ZERO,
                color: Color::srgb_alpha(0.0, 0.0, 0.0, 0.3),
            })])
            .overflow(Overflow::Visible),
    ))
    .into()
}

fn column(gap: f64) -> Element {
    div(Style::default()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(gap))
}

fn label(text: &str, theme: &NeutralTheme) -> Node {
    div(Style::default()
        .font_size(13.0)
        .line_height(LineHeight::Length(Length::px(18.0)))
        .font_weight(500.0)
        .letter_spacing(Length::em(-0.01))
        .color(css_color(theme.get("foreground"))))
    .child(text.to_owned())
    .into()
}

fn help(text: &str, theme: &NeutralTheme) -> Node {
    div(Style::default()
        .font_size(12.0)
        .line_height(LineHeight::Length(Length::px(16.0)))
        .color(css_color(theme.get("mutedForeground")))
        .text_align(opsis_scene::style::TextAlign::Left))
    .child(text.to_owned())
    .into()
}

fn control(child: Node) -> Node {
    div(Style::default()
        .position(Position::Relative)
        .height(Length::px(40.0)))
    .child(child)
    .into()
}

struct FieldParts<'a> {
    label: &'a str,
    value: &'a str,
    help: Option<&'a str>,
    steps: [Step<InputState>; 3],
}

fn field(parts: &FieldParts<'_>, theme: &NeutralTheme, frame: f64, cx: &Cx<'_>) -> Node {
    let look = input_transition(&parts.steps, theme, frame);
    let mut out = column(6.0)
        .child(label(parts.label, theme))
        .child(control(input(
            &InputParts {
                placeholder: parts.value,
                value: parts.value,
                look: &look,
                theme,
                width: Length::px(320.0),
            },
            cx,
        )));
    if let Some(text) = parts.help {
        out = out.child(help(text, theme));
    }
    out.into()
}

const fn typed(start: f64, typing: f64, leave: f64) -> [Step<InputState>; 3] {
    [
        Step {
            at: start + DEMO,
            state: InputState::Active,
            duration: 6.0,
        },
        Step {
            at: start + 2.0 + DEMO,
            state: InputState::Typing,
            duration: typing,
        },
        Step {
            at: leave + DEMO,
            state: InputState::Blur,
            duration: 8.0,
        },
    ]
}

fn notice_place(corner: Corner) -> Style {
    let at = Length::px(NOTICE_INSET);
    let style = Style::default().position(Position::Absolute);
    match corner {
        Corner::BottomLeft => style.left(at.clone()).bottom(at),
        Corner::BottomRight => style.right(at.clone()).bottom(at),
        Corner::TopLeft => style.left(at.clone()).top(at),
        Corner::TopRight => style.right(at.clone()).top(at),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one pass over the form like the film's component"
)]
pub fn draw(props: &SignupFlow, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let theme = neutral_theme(&props.theme).with_radius(f64::from(props.radius));
    let authored = cx.authored_frame();
    let frame = authored * props.speed;
    let card = blur_in(0.0, 18.0, 0.0, frame);
    let header = blur_in(18.0, 16.0, RISE, frame);
    let enters: Vec<BlurIn> = [24.0, 30.0, 36.0, 42.0, 48.0]
        .iter()
        .map(|at| blur_in(*at, 16.0, RISE, frame))
        .collect();
    let success_at = 192.0 + DEMO + props.loading_seconds;
    let waypoint = |at: f64, x: f64, y: f64, duration: f64| Waypoint { at, x, y, duration };
    let pointer = cursor_at(
        &[
            waypoint(0.0, 160.0, 120.0, 24.0),
            waypoint(18.0 + DEMO, CENTER_X, NAME_Y, 18.0),
            waypoint(52.0 + DEMO, CENTER_X, EMAIL_Y, 30.0),
            waypoint(96.0 + DEMO, CENTER_X, PASS_Y, 40.0),
            waypoint(134.0 + DEMO, CENTER_X, CONFIRM_Y, 32.0),
            waypoint(176.0 + DEMO, CENTER_X, CREATE_Y, 38.0),
        ],
        frame,
    );
    let pressed = button_transition(
        &[
            Step {
                at: 176.0 + DEMO,
                state: ButtonState::Hover,
                duration: 8.0,
            },
            Step {
                at: 186.0 + DEMO,
                state: ButtonState::Press,
                duration: 6.0,
            },
            Step {
                at: 192.0 + DEMO,
                state: ButtonState::Loading,
                duration: 6.0,
            },
            Step {
                at: success_at,
                state: ButtonState::Success,
                duration: 16.0,
            },
        ],
        ButtonVariant::Default,
        &theme,
        frame,
    );
    let notice = toast_transition(
        &[
            Step {
                at: success_at,
                state: ToastState::Visible,
                duration: 14.0,
            },
            Step {
                at: success_at + props.notice_seconds,
                state: ToastState::Hidden,
                duration: 14.0,
            },
        ],
        frame,
    );
    let fields = [
        FieldParts {
            label: &props.name_label,
            value: &props.full_name,
            help: None,
            steps: typed(18.0, 20.0, 52.0),
        },
        FieldParts {
            label: &props.email_label,
            value: &props.email,
            help: Some(&props.email_help),
            steps: typed(52.0, 28.0, 96.0),
        },
        FieldParts {
            label: &props.password_label,
            value: &props.password,
            help: Some(&props.password_help),
            steps: typed(96.0, 22.0, 134.0),
        },
        FieldParts {
            label: &props.confirm_label,
            value: &props.password,
            help: None,
            steps: typed(134.0, 22.0, 176.0),
        },
    ];
    let mut group = column(16.0);
    for (parts, look) in fields.iter().zip(&enters) {
        group = group.child(revealed(look, field(parts, &theme, frame, cx)));
    }
    let heading = column(4.0)
        .child(
            div(Style::default()
                .font_size(22.0)
                .line_height(LineHeight::Length(Length::px(28.0)))
                .font_weight(600.0)
                .letter_spacing(Length::em(-0.02))
                .color(css_color(theme.get("cardForeground"))))
            .child(props.title.clone()),
        )
        .child(
            div(Style::default()
                .font_size(14.0)
                .line_height(LineHeight::Length(Length::px(20.0)))
                .color(css_color(theme.get("mutedForeground"))))
            .child(props.description.clone()),
        );
    let create = column(10.0).child(control(button(&ButtonParts {
        label: &props.create_label,
        variant: ButtonVariant::Default,
        look: &pressed,
        theme: &theme,
        spin: authored * 6.0,
    })));
    let shade = |y: f64, blur: f64, spread: f64, alpha: f64| Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(y),
        blur: Length::px(blur),
        spread: Length::px(spread),
        color: Color::srgb_alpha(0.0, 0.0, 0.0, alpha),
    };
    let form = div(blurred(
        Style::default()
            .position(Position::Absolute)
            .left(Length::px((STAGE_W - CARD_W) / 2.0))
            .top(Length::px(CARD_TOP))
            .width(Length::px(CARD_W))
            .height(Length::px(580.0))
            .padding(Edges::all(Length::px(28.0)))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(24.0)
            .background_color(css_color(theme.get("background")))
            .border(Edges::all(BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color(theme.get("border"))),
            }))
            .border_radius_all(f64::from(props.radius) + 4.0)
            .box_shadow(vec![
                shade(10.0, 30.0, -12.0, 0.22),
                shade(2.0, 8.0, -3.0, 0.10),
            ]),
        &card,
    ))
    .child(revealed(&header, heading))
    .child(group)
    .child(revealed(enters.last().unwrap_or(&card), create));
    let ground = if props.background_color.is_empty() {
        "transparent"
    } else {
        props.background_color.as_str()
    };
    Ok(div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .background_color(css_color(ground))
        .font_family(font_or(&props.font_family, FontRole::Main)))
    .child(form)
    .child(div(notice_place(props.notice_corner)).child(toast(
        &props.toast_title,
        ToastVariant::Success,
        &notice,
        &theme,
    )))
    .child(cursor(
        &pointer,
        props.cursor_shape,
        f64::from(props.cursor_size),
        &theme,
    ))
    .into())
}
