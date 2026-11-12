use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Transform;
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, LineHeight, NumericVariant, Overflow, Position, Style,
    TextAlign, TextTransform,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float, truncate};

use crate::elements::anim::clamped;
use crate::elements::code_syntax::{
    CodeToken, CodeTokenKind, code_line, line_length, tokenize_code,
};
use crate::elements::code_theme::{code_palette, code_tokens};
use crate::elements::code_window::{
    backdrop_glow, backdrop_vignette, bottom_line, editor_enter, editor_stage, editor_window,
    traffic_lights,
};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::parts::caret::{Caret, caret};
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::units::utf16_len;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct CodeReveal {
    #[setting(
        "Code",
        default = "export async function checkout(cart: Cart) {\n  const total = cart.items.reduce((sum, item) => sum + item.price, 0);\n  const order = await api.orders.create({ items: cart.items, total });\n\n  return order.id;\n}",
        widget = "code",
        help = "Source shown in the editor; surrounding blank lines are trimmed."
    )]
    pub code: String,
    #[setting(
        "Highlighted lines",
        default = vec![3.0],
        help = "Line numbers as shown in the gutter, focused once the listing finishes writing."
    )]
    pub highlighted_lines: Vec<f64>,
    #[setting("Title", default = "checkout.ts", help = "Filename on the editor tab.")]
    pub title: String,
    #[setting(
        "Language",
        default = "tsx",
        help = "Language badge on the right of the header.",
        advanced
    )]
    pub language: String,
    #[setting(
        "Start line",
        default = 1,
        help = "First line number in the gutter, for excerpts.",
        advanced
    )]
    pub start_line: u32,
    #[setting("Line numbers", default = true, help = "Shows the gutter.")]
    pub show_line_numbers: bool,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Caret, focus band and glow color.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[setting(
        "Font size",
        default = 21,
        range(min = 12, max = 32),
        step = 1,
        unit = "px"
    )]
    pub font_size: u32,
    #[setting(
        "Window width",
        default = 940,
        range(min = 400, max = 1200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub window_width: u32,
    #[setting("Line height", default = 1.62, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting(
        "Visible lines",
        default = 10,
        range(min = 3, max = 30),
        step = 1,
        advanced
    )]
    pub visible_lines: u32,
    #[setting(
        "Typing speed",
        default = 55.0,
        range(min = 5, max = 200),
        unit = "characters a second",
        advanced
    )]
    pub typing_speed: f64,
    #[setting(
        "Pause at line ends",
        default = 0.07,
        range(min = 0, max = 1),
        unit = "s",
        advanced
    )]
    pub line_pause: f64,
    #[setting(
        "Start typing at",
        default = 0.4,
        range(min = 0, max = 10),
        unit = "s",
        advanced
    )]
    pub start_at: f64,
    #[setting(
        "Highlight after",
        default = 0.34,
        range(min = 0, max = 5),
        unit = "s",
        advanced
    )]
    pub highlight_after: f64,
    #[setting("Other lines", default = 0.34, range(min = 0, max = 1), advanced)]
    pub dim_others: f64,
    #[setting("Glow behind the window", default = true, advanced)]
    pub glow: bool,
    #[setting("Vignette", default = true, advanced)]
    pub vignette: bool,
    #[setting(
        "Corner radius",
        default = 16,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
}

pub fn definition() -> ElementMeta {
    meta::<CodeReveal>(
        element(
            ElementKind::Component,
            "code-reveal",
            "Code reveal",
            "A code editor that types out a file, then highlights the lines that matter.",
            ElementCategory::Ui,
            165,
        )
        .fonts(&["JetBrains Mono"])
        .look(NodeLook::of("lucide:code")),
    )
}

const REF: (f64, f64) = (1280.0, 720.0);
const HEADER_HEIGHT: f64 = 46.0;
const BODY_PADDING: f64 = 24.0;
const GUTTER_WIDTH: f64 = 62.0;
const CODE_INSET: f64 = 20.0;
const MONO_ADVANCE: f64 = 0.6;

#[derive(Debug, Clone)]
struct Row {
    tokens: Vec<CodeToken>,
    indent: f64,
    cost: f64,
    starts_at: f64,
    ends_at: f64,
}

fn eased(frame: f64, from: [f64; 2], to: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &from,
        &to,
        &Interpolation {
            easing: Some(easing_enter()),
            ..Interpolation::clamped()
        },
    )?)
}

fn lines_of(code: &str) -> Vec<&str> {
    let trimmed = code.trim_end().trim_start_matches(['\r', '\n']);
    trimmed
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect()
}

fn written(row: &Row, frame: f64) -> Result<f64, DrawFailure> {
    let typed = clamped(
        frame,
        [row.starts_at, row.ends_at.max(row.starts_at + 1.0)],
        [0.0, row.cost],
    )?;
    Ok(row.indent + (typed + 0.5).floor())
}

fn fade(top: bool, window: &str, opacity: f64) -> Node {
    let gradient = if top {
        format!("linear-gradient({window}, {window}00)")
    } else {
        format!("linear-gradient({window}00, {window})")
    };
    let place = Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .right(Length::ZERO)
        .height(Length::px(BODY_PADDING));
    let place = if top {
        place.top(Length::ZERO)
    } else {
        place.bottom(Length::ZERO)
    };
    div(with_background(place, &gradient).opacity(opacity)).into()
}

pub fn draw(props: &CodeReveal, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, fps) = (cx.at.stage.width, cx.at.stage.height, cx.fps());
    let font_size = f64::from(props.font_size);
    let row_height = font_size * props.line_height;
    let palette = code_palette(
        props.theme.mode,
        code_tokens(props.theme.mode),
        &props.theme.colors,
    );
    let dark = props.theme.mode == ThemeMode::Dark;
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    let scale = (width / REF.0).min(height / REF.1);
    let frame = cx.frame() * props.speed;
    let seconds = |value: f64| value * fps;
    let lines = lines_of(&props.code);
    let longest = float(lines.iter().map(|l| utf16_len(l)).max().unwrap_or(0));
    let inset = if props.show_line_numbers {
        GUTTER_WIDTH + CODE_INSET
    } else {
        BODY_PADDING
    };
    let window_width = f64::from(props.window_width).max(
        BODY_PADDING
            .mul_add(2.0, longest.mul_add(font_size * MONO_ADVANCE, inset))
            .ceil(),
    );
    let fit = (REF.0 * 0.94 / window_width).min(1.0);
    let has_focus = !props.highlighted_lines.is_empty();
    let write_start = seconds(props.start_at);
    let mut cursor = write_start;
    let rows: Vec<Row> = tokenize_code(&lines)
        .into_iter()
        .zip(&lines)
        .map(|(tokens, raw)| {
            let indent = float(utf16_len(raw) - utf16_len(raw.trim_start()));
            let cost = (float(line_length(&tokens)) - indent).max(0.0);
            let starts_at = cursor;
            let ends_at = (cost / props.typing_speed).mul_add(fps, starts_at);
            cursor = ends_at + seconds(props.line_pause);
            Row {
                tokens,
                indent,
                cost,
                starts_at,
                ends_at,
            }
        })
        .collect();
    let write_end = rows.last().map_or(write_start, |r| r.ends_at);
    let focus_at = write_end + seconds(props.highlight_after);
    let enter = editor_enter(frame, fps)?;
    let chrome_in = eased(frame, [seconds(0.1), seconds(0.4)], [0.0, 1.0])?;
    let focus = if has_focus {
        eased(frame, [focus_at, focus_at + seconds(0.5)], [0.0, 1.0])?
    } else {
        0.0
    };
    let char_width = font_size * MONO_ADVANCE;
    let caret_row = rows
        .iter()
        .position(|r| frame < r.ends_at)
        .unwrap_or_else(|| rows.len().saturating_sub(1));
    let caret_chars = rows.get(caret_row).map_or(Ok(0.0), |r| written(r, frame))?;
    let resting = frame >= write_end;
    let visible = float(rows.len()).min(f64::from(props.visible_lines));
    let body_height = visible.mul_add(row_height, BODY_PADDING * 2.0);
    let mut open_rows = 0.0;
    for row in &rows {
        open_rows += eased(frame, [row.starts_at, row.starts_at + 6.0], [0.0, 1.0])?;
    }
    let scroll = (open_rows - visible).max(0.0) * row_height;
    let accent = props.accent_color.as_str();
    let token_color = |kind: CodeTokenKind| palette.get(kind.id()).to_owned();
    let mut lines_out = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let started = frame >= row.starts_at;
        let shown = if started { written(row, frame)? } else { 0.0 };
        let complete = shown >= row.indent + row.cost;
        let number = float(index) + f64::from(props.start_line);
        let focused = props.highlighted_lines.iter().any(|l| exact_eq(*l, number));
        let recede = if has_focus && !focused { focus } else { 0.0 };
        let opacity = if started {
            interpolate(
                recede,
                &[0.0, 1.0],
                &[1.0, props.dim_others],
                &Interpolation::default(),
            )?
        } else {
            0.0
        };
        let mut line = div(Style::default()
            .height(Length::px(row_height))
            .display(Display::Flex)
            .align_items(Align::Center)
            .position(Position::Relative)
            .opacity(opacity));
        if focused {
            line = line
                .child(div(Style::default()
                    .position(Position::Absolute)
                    .inset(Length::ZERO)
                    .background_color(tinted(accent, 8.0))
                    .opacity(focus)))
                .child(div(Style::default()
                    .position(Position::Absolute)
                    .left(Length::ZERO)
                    .top(Length::px(3.0))
                    .bottom(Length::px(3.0))
                    .width(Length::px(3.0))
                    .border_radius_all(3.0)
                    .background_color(css_color(accent))
                    .opacity(focus)
                    .transform(vec![Transform::Scale(
                        Num(1.0),
                        Num(focus.mul_add(0.65, 0.35)),
                        Num(1.0),
                    )])));
        }
        if props.show_line_numbers {
            let label = js_number_text(number);
            let mut gutter = span(
                Style::default()
                    .width(Length::px(GUTTER_WIDTH))
                    .flex_shrink(0.0)
                    .padding_right(Length::px(16.0))
                    .text_align(TextAlign::Right)
                    .color(css_color(palette.get("faint")))
                    .font_variant_numeric(NumericVariant {
                        tabular: true,
                        ..NumericVariant::default()
                    })
                    .font_size(font_size * 0.82)
                    .position(Position::Relative),
            )
            .child(label.clone());
            if focused {
                gutter = gutter.child(
                    span(
                        Style::default()
                            .position(Position::Absolute)
                            .inset(Length::ZERO)
                            .padding_right(Length::px(16.0))
                            .color(css_color(accent))
                            .opacity(focus),
                    )
                    .child(label),
                );
            }
            line = line.child(gutter);
        }
        let reveal = (!complete).then(|| truncate::<usize>(shown));
        line = line.child(
            span(
                Style::default()
                    .padding_left(Length::px(if props.show_line_numbers {
                        CODE_INSET
                    } else {
                        BODY_PADDING
                    }))
                    .position(Position::Relative),
            )
            .child(code_line(&row.tokens, &token_color, reveal)),
        );
        lines_out.push(line.into());
    }
    if frame >= write_start && !rows.is_empty() {
        lines_out.push(
            div(Style::default()
                .position(Position::Absolute)
                .top(Length::px(float(caret_row) * row_height))
                .left(Length::px(caret_chars.mul_add(char_width, inset)))
                .height(Length::px(row_height))
                .display(Display::Flex)
                .align_items(Align::Center))
            .child(caret(
                &Caret {
                    color: css_color(accent),
                    width: Length::px(char_width * 0.9),
                    height: Length::px(font_size * 1.15),
                    radius: 2.0,
                    blink: resting,
                    blink_per_second: 1.6,
                    speed: props.speed,
                    ..Caret::default()
                },
                cx,
                &Style::default(),
            ))
            .into(),
        );
    }
    let mut body = div(Style::default()
        .height(Length::px(body_height))
        .overflow(Overflow::Hidden)
        .position(Position::Relative));
    if props.show_line_numbers {
        body = body.child(div(Style::default()
            .position(Position::Absolute)
            .top(Length::ZERO)
            .bottom(Length::ZERO)
            .left(Length::ZERO)
            .width(Length::px(GUTTER_WIDTH))
            .background_color(css_color(palette.get("gutter")))
            .border(Edges {
                top: BorderSide::default(),
                right: BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(css_color(palette.get("border"))),
                },
                bottom: BorderSide::default(),
                left: BorderSide::default(),
            })));
    }
    let window_color = palette.get("window");
    let body = body
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .padding_top(Length::px(BODY_PADDING))
                .padding_bottom(Length::px(BODY_PADDING)))
            .child(
                div(Style::default()
                    .transform(vec![Transform::Translate(
                        Length::ZERO,
                        Length::px(-scroll),
                        Length::ZERO,
                    )])
                    .font_size(font_size)
                    .line_height(LineHeight::Length(Length::px(row_height)))
                    .position(Position::Relative))
                .children(lines_out),
            ),
        )
        .child(fade(true, window_color, (scroll / 10.0).min(1.0)))
        .child(fade(false, window_color, 1.0));
    let header = div(Style::default()
        .height(Length::px(HEADER_HEIGHT))
        .display(Display::Flex)
        .align_items(Align::Center)
        .padding(Edges::axes(Length::ZERO, Length::px(18.0)))
        .border(bottom_line(palette.get("border")))
        .background_color(css_color(palette.get("header")))
        .opacity(chrome_in)
        .position(Position::Relative))
    .child(traffic_lights(props.window_dots))
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .gap(9.0))
        .child(div(Style::default()
            .width(Length::px(7.0))
            .height(Length::px(7.0))
            .border_radius_all(Length::percent(50.0))
            .background_color(css_color(accent))
            .opacity(clamped(
                frame,
                [write_end, write_end + seconds(0.4)],
                [1.0, 0.2],
            )?)))
        .child(
            span(
                Style::default()
                    .color(css_color(palette.get("dim")))
                    .font_size(15.0)
                    .font_weight(500.0)
                    .letter_spacing(Length::px(0.2)),
            )
            .child(props.title.clone()),
        ),
    )
    .child(
        span(
            Style::default()
                .margin_left(Length::Auto)
                .color(css_color(palette.get("faint")))
                .font_size(13.0)
                .letter_spacing(Length::px(0.6))
                .text_transform(TextTransform::Uppercase),
        )
        .child(props.language.clone()),
    );
    let window = div(editor_window(
        &palette,
        window_width,
        scale * fit,
        enter,
        f64::from(props.corner_radius),
    )?
    .position(Position::Relative))
    .child(header)
    .child(body);
    Ok(div(editor_stage(
        width,
        height,
        page,
        font_or(&props.font, FontRole::Code),
    ))
    .child(backdrop_glow(
        accent,
        props.glow,
        chrome_in * focus.mul_add(0.45, 0.55),
    ))
    .child(backdrop_vignette(props.vignette, dark))
    .child(window)
    .into())
}
