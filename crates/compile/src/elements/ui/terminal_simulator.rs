use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{ClipPath, Shadow};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style, TextAlign,
    WhiteSpace,
};
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float, truncate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::parsed_or_black;
use crate::elements::units::{utf16_len, utf16_slice};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalLineKind {
    Command,
    Log,
    Success,
    Error,
}

#[settings(export)]
pub struct TerminalSimulatorLine {
    #[setting("Text", default = "Done")]
    pub text: String,
    #[setting("Kind", default = TerminalLineKind::Log)]
    pub r#type: TerminalLineKind,
    #[setting(
        "Wait before",
        default = 0.2667,
        range(min = 0, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub delay: f64,
}

#[settings(export)]
pub struct TerminalSimulator {
    #[setting("Prompt", default = "$")]
    pub prompt: String,
    #[setting("Title", default = "~/projects/acme")]
    pub title: String,
    #[setting("Background", default = "#0a0a0a", check = colour)]
    pub background: String,
    #[setting("Title bar colour", default = "#1a1a1a", check = colour, advanced)]
    pub chrome_color: String,
    #[setting("Font size", default = 18, range(min = 10, max = 32), step = 1)]
    pub font_size: u32,
    #[setting(
        "Chars / frame",
        default = 1.0,
        range(min = 0.25, max = 6),
        step = 0.25
    )]
    pub chars_per_frame: f64,
    #[setting(
        "Chunk size",
        default = 1,
        range(min = 1, max = 20),
        step = 1,
        advanced
    )]
    pub chunk_size: u32,
    #[setting(
        "Lines",
        default = vec![
            TerminalSimulatorLine {
                text: "deploy --production".into(),
                r#type: TerminalLineKind::Command,
                delay: 0.0,
            },
            TerminalSimulatorLine { text: "Running tests...".into(), delay: 0.2, ..Default::default() },
            TerminalSimulatorLine {
                text: "128 tests passed".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.3333,
            },
            TerminalSimulatorLine { text: "Building the image...".into(), ..Default::default() },
            TerminalSimulatorLine {
                text: "Image ready in 41s".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.3333,
            },
            TerminalSimulatorLine { text: "Rolling out to everyone...".into(), ..Default::default() },
            TerminalSimulatorLine {
                text: "Live in production".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.4,
            },
        ],
        help = "Lines ending in ... pause before the next one."
    )]
    pub lines: Vec<TerminalSimulatorLine>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Width",
        default = 900,
        range(min = 300, max = 1600),
        step = 1,
        unit = "px"
    )]
    pub width: u32,
    #[setting(
        "Height",
        default = 480,
        range(min = 200, max = 1000),
        step = 1,
        unit = "px"
    )]
    pub height: u32,
    #[setting(
        "Pause after ...",
        default = 0.6,
        range(min = 0, max = 5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub ellipsis_pause: f64,
    #[setting(
        "Start at",
        default = 0.3333,
        range(min = 0, max = 10),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub start_at: f64,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
    #[setting(
        "Corner radius",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Line height", default = 1.6, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting(
        "Command colour",
        widget = "color",
        zero_label = "Follows the background",
        check = colour_or_empty,
        advanced
    )]
    pub command_color: String,
    #[setting(
        "Output colour",
        widget = "color",
        zero_label = "Follows the background",
        check = colour_or_empty,
        advanced
    )]
    pub log_color: String,
    #[setting(
        "Success colour",
        widget = "color",
        zero_label = "Follows the background",
        check = colour_or_empty,
        advanced
    )]
    pub success_color: String,
    #[setting(
        "Error colour",
        widget = "color",
        zero_label = "Follows the background",
        check = colour_or_empty,
        advanced
    )]
    pub error_color: String,
    #[setting(
        "Prompt colour",
        widget = "color",
        zero_label = "Success colour",
        check = colour_or_empty,
        advanced
    )]
    pub prompt_color: String,
    #[setting(
        "Cursor colour",
        widget = "color",
        zero_label = "Line colour",
        check = colour_or_empty,
        advanced
    )]
    pub cursor_color: String,
    #[setting("Title colour", default = "#71717a", widget = "color", check = colour, advanced)]
    pub title_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<TerminalSimulator>(
        element(
            ElementKind::Component,
            "terminal-simulator",
            "Terminal simulator",
            "Terminal window that types out commands and rolls old lines off the top.",
            ElementCategory::Ui,
            240,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:square-terminal")),
    )
}

const CHROME_HEIGHT: f64 = 40.0;
const BODY_PADDING: f64 = 20.0;
const LIGHTS: [&str; 3] = ["#ff5f57", "#febc2e", "#28c840"];
const DARK_KINDS: [&str; 4] = ["#fafafa", "#a1a1aa", "#22c55e", "#ef4444"];
const LIGHT_KINDS: [&str; 4] = ["#18181b", "#52525b", "#15803d", "#b91c1c"];
const ELLIPSIS: &str = "...";

const fn kind_index(kind: TerminalLineKind) -> usize {
    match kind {
        TerminalLineKind::Command => 0,
        TerminalLineKind::Log => 1,
        TerminalLineKind::Success => 2,
        TerminalLineKind::Error => 3,
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film weighs the channels with separate multiplies and adds"
)]
fn is_light(color: &str) -> bool {
    let rgb = parsed_or_black(color);
    rgb.alpha.unwrap_or(1.0) > 0.5 && 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b > 0.5
}

fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn kind_colors(props: &TerminalSimulator) -> [String; 4] {
    let auto = if is_light(&props.background) {
        LIGHT_KINDS
    } else {
        DARK_KINDS
    };
    let chosen = [
        &props.command_color,
        &props.log_color,
        &props.success_color,
        &props.error_color,
    ];
    std::array::from_fn(|i| {
        if chosen[i].is_empty() {
            auto[i].to_owned()
        } else {
            chosen[i].clone()
        }
    })
}

fn starts(props: &TerminalSimulator) -> Vec<f64> {
    let mut at = props.start_at;
    props
        .lines
        .iter()
        .map(|line| {
            at += line.delay;
            let start = at;
            let typing = (float(utf16_len(&line.text))
                / (f64::from(props.chunk_size) * props.chars_per_frame))
                .ceil();
            let pause = if line.text.trim_end().ends_with(ELLIPSIS) {
                props.ellipsis_pause
            } else {
                0.0
            };
            at += typing + pause;
            start
        })
        .collect()
}

fn light(color: &str) -> Node {
    div(Style::default()
        .width(Length::px(12.0))
        .height(Length::px(12.0))
        .border_radius_all(Length::percent(50.0))
        .background_color(css_color(color))
        .opacity(0.85))
    .into()
}

struct Row<'a> {
    line: &'a TerminalSimulatorLine,
    color: &'a str,
    prompt_color: &'a str,
    line_height: f64,
}

fn row(props: &TerminalSimulator, row: &Row<'_>, local: f64) -> Result<Node, DrawFailure> {
    let font_size = f64::from(props.font_size);
    let total = float(utf16_len(&row.line.text));
    let linear = clamped(local, [0.0, total / props.chars_per_frame], [0.0, total])?.floor();
    let chunk = f64::from(props.chunk_size);
    let revealed = total.min((linear / chunk).ceil() * chunk);
    let shown: usize = truncate(revealed);
    let cursor_on = exact_eq((local / Cx::authored_fps() * 2.0).floor() % 2.0, 0.0);
    let mut out = div(Style::default()
        .height(Length::px(row.line_height))
        .font_size(font_size)
        .color(css_color(row.color))
        .display(Display::Flex)
        .align_items(Align::Center)
        .white_space(WhiteSpace::Pre));
    if row.line.r#type == TerminalLineKind::Command {
        out = out.child(
            span(
                Style::default()
                    .color(css_color(row.prompt_color))
                    .margin_right(Length::px(8.0)),
            )
            .child(props.prompt.clone()),
        );
    }
    out = out.child(span(Style::default()).child(utf16_slice(&row.line.text, 0, shown)));
    if revealed < total && cursor_on {
        let cursor = if props.cursor_color.is_empty() {
            row.color
        } else {
            &props.cursor_color
        };
        out = out.child(span(
            Style::default()
                .display(Display::InlineBlock)
                .width(Length::px(font_size * 0.55))
                .height(Length::px(font_size))
                .background_color(css_color(cursor))
                .margin_left(Length::px(2.0)),
        ));
    }
    Ok(out.into())
}

pub fn draw(props: &TerminalSimulator, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let line_height = js_round(f64::from(props.font_size) * props.line_height);
    let height = f64::from(props.height);
    let visible = (BODY_PADDING.mul_add(-2.0, height - CHROME_HEIGHT) / line_height)
        .floor()
        .max(1.0);
    let colors = kind_colors(props);
    let prompt_color = if props.prompt_color.is_empty() {
        &colors[2]
    } else {
        &props.prompt_color
    };
    let begins = starts(props);
    let scrolled = begins
        .iter()
        .skip(truncate::<usize>(visible))
        .filter(|start| frame >= **start)
        .count();
    let shift = -float(scrolled) * line_height;
    let mut rows = Vec::new();
    for (line, start) in props.lines.iter().zip(&begins) {
        let from = js_round(cx.real_frames(start / props.speed));
        if cx.frame() < from {
            continue;
        }
        let local = (cx.frame() - from) * Cx::authored_fps() / cx.fps() * props.speed;
        rows.push(row(
            props,
            &Row {
                line,
                color: &colors[kind_index(line.r#type)],
                prompt_color,
                line_height,
            },
            local,
        )?);
    }
    let mut bar = div(Style::default()
        .height(Length::px(CHROME_HEIGHT))
        .background_color(css_color(&props.chrome_color))
        .display(Display::Flex)
        .align_items(Align::Center)
        .padding(Edges::axes(Length::ZERO, Length::px(16.0)))
        .gap(8.0)
        .border(Edges {
            top: BorderSide::default(),
            right: BorderSide::default(),
            bottom: BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color("rgba(255,255,255,0.06)")),
            },
            left: BorderSide::default(),
        }));
    if props.window_dots {
        bar = bar.children(LIGHTS.iter().map(|c| light(c)));
    }
    let bar = bar.child(
        div(Style::default()
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .flex_basis(Length::percent(0.0))
            .text_align(TextAlign::Center)
            .color(css_color(&props.title_color))
            .font_size(13.0))
        .child(props.title.clone()),
    );
    let body = div(Style::default()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .flex_basis(Length::percent(0.0))
        .padding(Edges::all(Length::px(BODY_PADDING)))
        .overflow(Overflow::Hidden)
        .clip_path(ClipPath::Inset {
            edges: Edges::axes(Length::px(BODY_PADDING), Length::ZERO),
            round: None,
        })
        .position(Position::Relative))
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(BODY_PADDING))
            .right(Length::px(BODY_PADDING))
            .top(Length::px(BODY_PADDING))
            .translate(Length::ZERO, Length::px(shift)))
        .children(rows),
    );
    let window = div(with_background(
        Style::default()
            .width(Length::px(f64::from(props.width)))
            .height(Length::px(height)),
        &props.background,
    )
    .border_radius_all(f64::from(props.corner_radius))
    .overflow(Overflow::Hidden)
    .box_shadow(vec![
        Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(30.0),
            blur: Length::px(80.0),
            spread: Length::ZERO,
            color: css_color("rgba(0,0,0,0.6)"),
        },
        Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::ZERO,
            spread: Length::px(1.0),
            color: css_color("rgba(255,255,255,0.06)"),
        },
    ])
    .display(Display::Flex)
    .flex_direction(FlexDirection::Column)
    .font_family(font_or(&props.font, FontRole::Code)))
    .child(bar)
    .child(body);
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center))
    .child(window)
    .into())
}
