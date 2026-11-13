use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Align, Display, FlexDirection, LineHeight, Position, Style};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::{NeutralTheme, neutral_theme};
use crate::elements::oklch_mix::mix_oklch;
use crate::elements::parts::button::{ButtonParts, ButtonState, button, button_transition};
use crate::elements::parts::input::{InputParts, InputState, input, input_transition};
use crate::elements::parts::toast::{ToastState, ToastVariant, toast, toast_transition};
use crate::elements::parts::ui_steps::{Step, transition, tween};
use crate::elements::shared::{ButtonVariant, Corner};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AiPromptFlow {
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Prompt", default = "Summarize this thread")]
    pub prompt: String,
    #[setting("Button label", default = "Generate")]
    pub button_label: String,
    #[setting(
        "Answer",
        default = vec![
            "The thread debates the Q3 roadmap: ship the editor first,".into(),
            "defer billing to Q4, and pull the migration forward so".into(),
            "infra is unblocked before the team scales next quarter.".into(),
        ],
        help = "One line of the answer per item."
    )]
    pub answer_lines: Vec<String>,
    #[setting("Notice", default = "Response ready")]
    pub toast_title: String,
    #[setting(
        "Typing time",
        default = 1.67,
        range(min = 0.2, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub typing_seconds: f64,
    #[setting(
        "Thinking time",
        default = 2.87,
        range(min = 0.3, max = 8),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub loading_seconds: f64,
    #[setting(
        "Notice shown for",
        default = 2.0,
        range(min = 0.5, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub notice_seconds: f64,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Prompt field width",
        default = 320,
        range(min = 200, max = 900),
        step = 1,
        unit = "px",
        advanced
    )]
    pub field_width: u32,
    #[setting(
        "Answer width",
        default = 560,
        range(min = 280, max = 1100),
        step = 1,
        unit = "px",
        advanced
    )]
    pub answer_width: u32,
    #[setting(
        "Answer text size",
        default = 18,
        range(min = 12, max = 36),
        step = 1,
        unit = "px",
        advanced
    )]
    pub answer_size: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting("Button style", default = ButtonVariant::Default, advanced)]
    pub button_variant: ButtonVariant,
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
}

pub fn definition() -> ElementMeta {
    meta::<AiPromptFlow>(
            element(
                ElementKind::Component,
                "ai-prompt-flow",
                "AI prompt flow",
                "An AI prompt composition: a prompt types into the field, the Generate button runs hover → press → loading, a skeleton shimmer reveals, then crossfades into the generated answer, and a ready toast slides in. A pure orchestrator — the shimmer is owned by the skeleton-block motion atom; every other channel comes from a composed primitive's hook.",
                ElementCategory::Scene,
                240,
            )
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:wand-sparkles")),
        )
}

const STAGE_W: f64 = 1280.0;
const PROMPT_W: f64 = 520.0;
const PROMPT_TOP: f64 = 168.0;
const PROMPT_H: f64 = 92.0;
const BUTTON_W: f64 = 200.0;
const BUTTON_TOP: f64 = 278.0;
const BUTTON_H: f64 = 64.0;
const ANSWER_TOP: f64 = 380.0;
const NOTICE_INSET: f64 = 32.0;
const SWEEP_FRAMES: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loading {
    Loading,
    Loaded,
}

const fn opacities(state: Loading) -> (f64, f64) {
    match state {
        Loading::Loaded => (0.0, 1.0),
        Loading::Loading => (1.0, 0.0),
    }
}

fn box_at(left: f64, top: f64, width: f64, height: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::px(left))
        .top(Length::px(top))
        .width(Length::px(width))
        .height(Length::px(height))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film computes this with a separate multiply and add"
)]
fn shimmer(width: f64, height: f64, base: &str, theme: &NeutralTheme, frame: f64) -> Node {
    let highlight = mix_oklch(base, theme.get("foreground"), 0.13);
    let progress = (frame % SWEEP_FRAMES) / SWEEP_FRAMES;
    let x = 100.0 - progress * 200.0;
    div(with_background(
        Style::default()
            .width(Length::percent(width))
            .height(Length::px(height))
            .border_radius_all(6.0),
        &format!(
            "linear-gradient(90deg, {base} 20%, {highlight} 50%, {base} 80%) {}% 0 / 200% 100%",
            js_number_text(x)
        ),
    ))
    .into()
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

pub fn draw(props: &AiPromptFlow, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let theme = neutral_theme(&props.theme).with_radius(f64::from(props.radius));
    let authored = cx.authored_frame();
    let frame = authored * props.speed;
    let typing = props.typing_seconds;
    let press_at = typing + 2.0;
    let loaded_at = press_at + 12.0 + props.loading_seconds;
    let notice_at = loaded_at + 10.0;
    let field = input_transition(
        &[
            Step {
                at: 0.0,
                state: InputState::Active,
                duration: 1.0,
            },
            Step {
                at: 0.0,
                state: InputState::Typing,
                duration: typing,
            },
        ],
        &theme,
        frame,
    );
    let pressed = button_transition(
        &[
            Step {
                at: press_at,
                state: ButtonState::Hover,
                duration: 6.0,
            },
            Step {
                at: press_at + 6.0,
                state: ButtonState::Press,
                duration: 4.0,
            },
            Step {
                at: press_at + 10.0,
                state: ButtonState::Loading,
                duration: 4.0,
            },
        ],
        props.button_variant,
        &theme,
        frame,
    );
    let loading = transition(
        &[
            Step {
                at: press_at + 12.0,
                state: Loading::Loading,
                duration: 1.0,
            },
            Step {
                at: loaded_at,
                state: Loading::Loaded,
                duration: 16.0,
            },
        ],
        Loading::Loading,
        frame,
    );
    let (skeleton, content) = {
        let (a, b) = (opacities(loading.from), opacities(loading.to));
        (
            tween(a.0, b.0, loading.eased),
            tween(a.1, b.1, loading.eased),
        )
    };
    let notice = toast_transition(
        &[
            Step {
                at: notice_at,
                state: ToastState::Visible,
                duration: 14.0,
            },
            Step {
                at: notice_at + props.notice_seconds,
                state: ToastState::Hidden,
                duration: 14.0,
            },
        ],
        frame,
    );
    let answer_width = f64::from(props.answer_width);
    let size = f64::from(props.answer_size);
    let last = props.answer_lines.len().saturating_sub(1);
    let placeholder = div(Style::default()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(14.0)
        .width(Length::px(answer_width)))
    .children(props.answer_lines.iter().enumerate().map(|(i, _)| {
        shimmer(
            if i == last { 70.0 } else { 100.0 },
            size,
            theme.get("muted"),
            &theme,
            authored,
        )
    }));
    let lines = div(Style::default()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(8.0)
        .width(Length::px(answer_width))
        .font_family(font_or(&props.font_family, FontRole::Main))
        .font_size(size)
        .line_height(LineHeight::Number(Num(1.45)))
        .letter_spacing(Length::em(-0.01))
        .color(css_color(theme.get("foreground"))))
    .children(
        props
            .answer_lines
            .iter()
            .map(|line| span(Style::default()).child(line.clone()).into()),
    );
    let answer = div(Style::default()
        .position(Position::Relative)
        .display(Display::InlineBlock)
        .font_family(font_or("", FontRole::Main)))
    .child(div(Style::default().opacity(content)).child(lines))
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .opacity(skeleton))
        .child(placeholder),
    );
    let ground = if props.background_color.is_empty() {
        "transparent"
    } else {
        props.background_color.as_str()
    };
    Ok(div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .background_color(css_color(ground)))
    .child(
        div(box_at(
            (STAGE_W - PROMPT_W) / 2.0,
            PROMPT_TOP,
            PROMPT_W,
            PROMPT_H,
        ))
        .child(input(
            &InputParts {
                placeholder: "Ask anything…",
                value: &props.prompt,
                look: &field,
                theme: &theme,
                width: Length::px(f64::from(props.field_width)),
            },
            cx,
        )),
    )
    .child(
        div(box_at(
            (STAGE_W - BUTTON_W) / 2.0,
            BUTTON_TOP,
            BUTTON_W,
            BUTTON_H,
        ))
        .child(button(&ButtonParts {
            label: &props.button_label,
            variant: props.button_variant,
            look: &pressed,
            theme: &theme,
            spin: authored * 6.0,
        })),
    )
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px((STAGE_W - answer_width) / 2.0))
            .top(Length::px(ANSWER_TOP))
            .width(Length::px(answer_width))
            .display(Display::Flex)
            .justify_content(Align::Center)
            .opacity(pressed.spinner_opacity))
        .child(answer),
    )
    .child(div(notice_place(props.notice_corner)).child(toast(
        &props.toast_title,
        ToastVariant::Success,
        &notice,
        &theme,
    )))
    .into())
}
