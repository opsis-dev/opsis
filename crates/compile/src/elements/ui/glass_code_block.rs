use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{ClipPath, Filter, Shadow};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, LineHeight, Overflow, Position, Style,
    TextAlign, WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped;
use crate::elements::code_syntax::{CodeToken, CodeTokenKind, tokenize_code};
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::Colour;
use crate::elements::units::utf16_len;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GlassCodeBlock {
    #[setting("Title", default = "hero.tsx")]
    pub title: String,
    #[setting("Width", default = 760, range(min = 300, max = 1100), step = 10)]
    pub width: u32,
    #[setting("Height", default = 460, range(min = 200, max = 700), step = 10)]
    pub height: u32,
    #[setting("Font size", default = 16, range(min = 10, max = 28), step = 1)]
    pub font_size: u32,
    #[setting("Glass color", default = "rgba(10, 10, 10, 0.6)", check = colour)]
    pub glass_color: String,
    #[setting(
        "Line stagger",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub stagger_frames: f64,
    #[setting("Traffic lights", default = true, advanced)]
    pub show_traffic_lights: bool,
    #[setting(
        "Code",
        default = "export async function publish(draft: Draft) {\n  const page = await render(draft);\n  await upload(page, { public: true });\n  return page.url;\n}",
        widget = "code"
    )]
    pub code: String,
    #[setting("Glow behind the window")]
    pub aura: bool,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Frost",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub frost: u32,
    #[setting(
        "Corner radius",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Line height", default = 1.55, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting("Code colour", default = "#e4e4e7", widget = "color", check = colour, advanced)]
    pub code_color: String,
    #[setting("Comment colour", default = "#52525b", widget = "color", check = colour, advanced)]
    pub comment_color: String,
    #[setting("Text in quotes colour", default = "#86efac", widget = "color", check = colour, advanced)]
    pub string_color: String,
    #[setting("Keyword colour", default = "#c4b5fd", widget = "color", check = colour, advanced)]
    pub keyword_color: String,
    #[setting("Number colour", default = "#fcd34d", widget = "color", check = colour, advanced)]
    pub number_color: String,
    #[setting("Title colour", default = "#a1a1aa", widget = "color", check = colour, advanced)]
    pub title_color: String,
    #[setting("Line number colour", default = "#3f3f46", widget = "color", check = colour, advanced)]
    pub line_number_color: String,
    #[setting(
        "Glow colours",
        default = vec!["rgba(56,189,248,0.22)".into(), "rgba(168,85,247,0.18)".into()],
        length(min = 2, max = 2),
        advanced
    )]
    pub aura_colors: Vec<Colour>,
}

pub fn definition() -> ElementMeta {
    meta::<GlassCodeBlock>(
        element(
            ElementKind::Component,
            "glass-code-block",
            "Glass code block",
            "A frosted-glass code editor that reveals the code line by line.",
            ElementCategory::Ui,
            180,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:square-code")),
    )
}

const MIN_FONT_SIZE: f64 = 9.0;
const HEADER: f64 = 40.0;
const BODY_PADDING: (f64, f64) = (20.0, 24.0);
const LINE_GAP: f64 = 4.0;
const MONO_SHARE: f64 = 0.62;
const LINE_FADE: f64 = 8.0;
const LIGHTS: [&str; 3] = ["#ff5f57", "#febc2e", "#28c840"];
const RIM: &str = "linear-gradient(180deg, rgba(255,255,255,0.18) 0%, rgba(255,255,255,0) 100%)";

fn token_color(props: &GlassCodeBlock, kind: CodeTokenKind) -> &str {
    match kind {
        CodeTokenKind::Comment => &props.comment_color,
        CodeTokenKind::String => &props.string_color,
        CodeTokenKind::Keyword => &props.keyword_color,
        CodeTokenKind::Number => &props.number_color,
        _ => &props.code_color,
    }
}

fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn aura(colors: &[Colour], frame: f64) -> Option<Node> {
    let [first, second, ..] = colors else {
        return None;
    };
    let t = frame / 60.0;
    let x = t.sin().mul_add(20.0, 50.0);
    let y = (t * 0.7).cos().mul_add(15.0, 50.0);
    let n = js_number_text;
    Some(
        div(with_background(
            Style::default().position(Position::Absolute).inset(Length::ZERO),
            &format!(
                "radial-gradient(circle at {}% {}%, {}, transparent 50%), radial-gradient(circle at {}% {}%, {}, transparent 55%)",
                n(x),
                n(y),
                first.0,
                n(100.0 - x),
                n(100.0 - y),
                second.0
            ),
        ))
        .into(),
    )
}

fn light(color: &str) -> Node {
    div(Style::default()
        .width(Length::px(12.0))
        .height(Length::px(12.0))
        .border_radius_all(Length::percent(50.0))
        .background_color(css_color(color))
        .opacity(0.6))
    .into()
}

fn header(props: &GlassCodeBlock) -> Node {
    let mut bar = div(Style::default()
        .height(Length::px(HEADER))
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(8.0)
        .padding(Edges::axes(Length::ZERO, Length::px(16.0)))
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
    if props.show_traffic_lights {
        bar = bar.children(LIGHTS.iter().map(|c| light(c)));
    }
    bar.child(
        div(Style::default()
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .flex_basis(Length::percent(0.0))
            .text_align(TextAlign::Center)
            .color(css_color(&props.title_color))
            .font_size(12.0)
            .letter_spacing(Length::em(0.02)))
        .child(props.title.clone()),
    )
    .into()
}

struct Line<'a> {
    tokens: &'a [CodeToken],
    index: usize,
    digits: usize,
    font_size: f64,
}

fn code_line(props: &GlassCodeBlock, line: &Line<'_>, frame: f64) -> Result<Node, DrawFailure> {
    let opacity = clamped(frame, [0.0, LINE_FADE], [0.0, 1.0])?;
    let rise = clamped(frame, [0.0, LINE_FADE], [4.0, 0.0])?;
    if line.tokens.is_empty() {
        return Ok(div(Style::default()
            .height(Length::px(line.font_size * 0.8))
            .opacity(opacity))
        .into());
    }
    let number = format!("{:>width$}", line.index + 1, width = line.digits);
    let spans: Vec<Node> = line
        .tokens
        .iter()
        .map(|t| {
            span(Style::default().color(css_color(token_color(props, t.kind))))
                .child(t.text.clone())
                .into()
        })
        .collect();
    Ok(div(Style::default()
        .opacity(opacity)
        .translate(Length::ZERO, Length::px(rise))
        .white_space(WhiteSpace::Pre)
        .display(Display::Flex)
        .gap(0.0))
    .child(
        span(
            Style::default()
                .width(Length::Ch(Num(float(line.digits + 1))))
                .flex_shrink(0.0)
                .color(css_color(&props.line_number_color)),
        )
        .child(number),
    )
    .child(span(Style::default()).children(spans))
    .into())
}

pub fn draw(props: &GlassCodeBlock, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let lines: Vec<&str> = props.code.split('\n').collect();
    let tokenized = tokenize_code(&lines);
    let count = float(lines.len());
    let digits = lines.len().to_string().len();
    let longest = float(lines.iter().map(|l| utf16_len(l)).max().unwrap_or(0).max(1));
    let (width, height) = (f64::from(props.width), f64::from(props.height));
    let body_height = height - 2.0 - HEADER - HEADER;
    let fitted = MIN_FONT_SIZE.max(
        f64::from(props.font_size)
            .min((width - 2.0 - 48.0) / ((longest + float(digits) + 1.0) * MONO_SHARE))
            .min((body_height / count - LINE_GAP) / props.line_height),
    );
    let frame = cx.authored_frame();
    let shown = if props.stagger_frames > 0.0 {
        count.min((frame * props.speed / props.stagger_frames).floor() + 1.0)
    } else {
        count
    };
    let scroll = shown
        .mul_add(fitted.mul_add(props.line_height, LINE_GAP), -body_height)
        .max(0.0);
    let mut rows = Vec::new();
    for (i, tokens) in tokenized.iter().enumerate() {
        let from = js_round(cx.real_frames(float(i) * props.stagger_frames / props.speed));
        if cx.frame() < from {
            continue;
        }
        let local = (cx.frame() - from) * Cx::authored_fps() / cx.fps();
        rows.push(code_line(
            props,
            &Line {
                tokens,
                index: i,
                digits,
                font_size: fitted,
            },
            local,
        )?);
    }
    let body = div(Style::default()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .flex_basis(Length::percent(0.0))
        .padding(Edges::axes(
            Length::px(BODY_PADDING.0),
            Length::px(BODY_PADDING.1),
        ))
        .overflow(Overflow::Hidden)
        .clip_path(ClipPath::Inset {
            edges: Edges::axes(Length::px(BODY_PADDING.0), Length::ZERO),
            round: None,
        })
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(LINE_GAP)
        .font_size(fitted)
        .line_height(LineHeight::Number(Num(props.line_height))))
    .child(
        div(Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(LINE_GAP)
            .translate(Length::ZERO, Length::px(-scroll)))
        .children(rows),
    );
    let radius = f64::from(props.corner_radius);
    let glass = div(with_background(
        Style::default()
            .width(Length::percent(100.0))
            .height(Length::percent(100.0))
            .border_radius_all((radius - 1.0).max(0.0)),
        &props.glass_color,
    )
    .backdrop_filter(vec![Filter::Blur(Length::px(f64::from(props.frost)))])
    .box_shadow(vec![Shadow {
        inset: true,
        x: Length::ZERO,
        y: Length::px(1.0),
        blur: Length::ZERO,
        spread: Length::ZERO,
        color: css_color("rgba(255,255,255,0.06)"),
    }])
    .display(Display::Flex)
    .flex_direction(FlexDirection::Column)
    .overflow(Overflow::Hidden)
    .font_family(font_or(&props.font, FontRole::Code)))
    .child(header(props))
    .child(body);
    let card = div(with_background(
        Style::default()
            .position(Position::Relative)
            .padding(Edges::all(Length::px(1.0)))
            .border_radius_all(radius)
            .width(Length::px(width))
            .height(Length::px(height)),
        RIM,
    )
    .box_shadow(vec![Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(50.0),
        blur: Length::px(120.0),
        spread: Length::ZERO,
        color: css_color("rgba(0,0,0,0.55)"),
    }]))
    .child(glass);
    let mut stage = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center));
    if props.aura
        && let Some(glow) = aura(&props.aura_colors, frame)
    {
        stage = stage.child(glow);
    }
    Ok(stage.child(card).into())
}
