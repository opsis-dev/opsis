use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::Shadow;
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, DecorationLine, Display, LineHeight, NumericVariant, Overflow, Position, Style,
    TextAlign, TextDecoration,
};
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::anim::clamped;
use crate::elements::code_syntax::{CodeTokenKind, code_line, tokenize_code};
use crate::elements::code_theme::themed_code;
use crate::elements::code_window::{
    backdrop_glow, backdrop_vignette, bottom_line, editor_enter, editor_stage, editor_window,
    traffic_lights,
};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::units::utf16_len;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct CodeDiffWipe {
    #[setting(
        "Before",
        default = "export function render(scene) {\n  const frames = scene.frames;\n  return frames.map(draw);\n}",
        widget = "code",
        help = "Source before the patch."
    )]
    pub before: String,
    #[setting(
        "After",
        default = "export function render(scene, options) {\n  const frames = scene.frames;\n  const scale = options.scale ?? 1;\n  return frames.map((frame) => draw(frame, scale));\n}",
        widget = "code",
        help = "Source after the patch; the scene diffs the two."
    )]
    pub after: String,
    #[setting(
        "Title",
        default = "render.ts",
        help = "Filename on the window header."
    )]
    pub title: String,
    #[setting(
        "Wipe (seconds)",
        default = 1.7,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "Seconds the apply front takes to travel the file.",
        advanced
    )]
    pub wipe_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Apply front and glow color.",
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
        default = 20,
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
    #[setting("Line height", default = 1.7, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting(
        "Start at",
        default = 0.7,
        range(min = 0, max = 10),
        unit = "s",
        advanced
    )]
    pub start_at: f64,
    #[setting("Collapse removed lines", default = true, advanced)]
    pub collapse_removed: bool,
    #[setting("Change count", default = true, advanced)]
    pub show_counts: bool,
    #[setting("Dim unchanged lines", default = true, advanced)]
    pub dim_unchanged: bool,
    #[setting("Strike out removed lines", default = true, advanced)]
    pub strike_removed: bool,
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
    meta::<CodeDiffWipe>(
            element(
                ElementKind::Component,
                "code-diff-wipe",
                "Code diff",
                "A code editor that applies a change line by line: removed lines collapse and new lines type in.",
                ElementCategory::Ui,
                135,
            )
                .fonts(&["JetBrains Mono"])
                .look(NodeLook::of("lucide:file-diff")),
        )
}

const REF: (f64, f64) = (1280.0, 720.0);
const HEADER_HEIGHT: f64 = 46.0;
const BODY_PADDING: f64 = 22.0;
const GUTTER_WIDTH: f64 = 46.0;
const CODE_INSET: f64 = 18.0;
const MONO_ADVANCE: f64 = 0.6;
const PLUS: &str = "M12 5.5v13M5.5 12h13";
const MINUS: &str = "M5.5 12h13";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffKind {
    Context,
    Remove,
    Add,
}

fn diff_lines<'a>(before: &[&'a str], after: &[&'a str]) -> Vec<(DiffKind, &'a str)> {
    let mut table = vec![vec![0_usize; after.len() + 1]; before.len() + 1];
    for i in (0..before.len()).rev() {
        for j in (0..after.len()).rev() {
            table[i][j] = if before[i] == after[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut rows) = (0, 0, Vec::new());
    while i < before.len() && j < after.len() {
        if before[i] == after[j] {
            rows.push((DiffKind::Context, before[i]));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            rows.push((DiffKind::Remove, before[i]));
            i += 1;
        } else {
            rows.push((DiffKind::Add, after[j]));
            j += 1;
        }
    }
    rows.extend(before[i..].iter().map(|t| (DiffKind::Remove, *t)));
    rows.extend(after[j..].iter().map(|t| (DiffKind::Add, *t)));
    rows
}

fn lines_of(code: &str) -> Vec<&str> {
    code.trim_end()
        .trim_start_matches(['\r', '\n'])
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect()
}

fn tint_alpha(value: f64) -> String {
    let alpha: u32 = truncate(value.clamp(0.0, 1.0).mul_add(30.0, 0.5).floor());
    format!("{alpha:02x}")
}

fn sign(size: f64, d: &str, color: &str) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr("d", d)
                    .attr("stroke", color)
                    .attr("stroke-width", 2.6)
                    .attr("stroke-linecap", "round"),
            ),
        Style::default(),
    )
    .into()
}

fn eased(frame: f64, from: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &from,
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(easing_enter()),
            ..Interpolation::clamped()
        },
    )?)
}

pub fn draw(props: &CodeDiffWipe, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, fps) = (cx.at.stage.width, cx.at.stage.height, cx.fps());
    let font_size = f64::from(props.font_size);
    let row_height = font_size * props.line_height;
    let palette = themed_code(&props.theme);
    let dark = props.theme.mode == ThemeMode::Dark;
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    let scale = (width / REF.0).min(height / REF.1);
    let frame = cx.frame() * props.speed;
    let seconds = |value: f64| value * fps;
    let (before, after) = (lines_of(&props.before), lines_of(&props.after));
    let rows = diff_lines(&before, &after);
    let texts: Vec<&str> = rows.iter().map(|(_, t)| *t).collect();
    let tokens = tokenize_code(&texts);
    let count = float(rows.len());
    let longest = float(texts.iter().map(|t| utf16_len(t)).max().unwrap_or(0));
    let window_width = f64::from(props.window_width).max(
        BODY_PADDING
            .mul_add(
                2.0,
                longest.mul_add(font_size * MONO_ADVANCE, GUTTER_WIDTH + CODE_INSET),
            )
            .ceil(),
    );
    let tallest = count.mul_add(row_height, BODY_PADDING.mul_add(2.0, HEADER_HEIGHT));
    let fit = (REF.0 * 0.94 / window_width)
        .min(REF.1 * 0.92 / tallest)
        .min(1.0);
    let added = float(rows.iter().filter(|r| r.0 == DiffKind::Add).count());
    let removed = float(rows.iter().filter(|r| r.0 == DiffKind::Remove).count());
    let (add_color, remove_color) = (palette.get("added"), palette.get("removed"));
    let wipe_start = seconds(props.start_at);
    let wipe_frames = seconds(props.wipe_seconds);
    let wipe_end = wipe_start + wipe_frames;
    let trigger = |index: usize| {
        if rows.len() <= 1 {
            wipe_start
        } else {
            (float(index) / count).mul_add(wipe_frames, wipe_start)
        }
    };
    let progress_of = |index: usize| eased(frame, [trigger(index), trigger(index) + seconds(0.3)]);
    let enter = editor_enter(frame, fps)?;
    let chrome_in = eased(frame, [seconds(0.1), seconds(0.4)])?;
    let settle = eased(frame, [wipe_end + seconds(0.7), wipe_end + seconds(1.4)])?;
    let drop_old = eased(frame, [wipe_end + seconds(0.3), wipe_end + seconds(0.75)])?;
    let mut heights = Vec::with_capacity(rows.len());
    for (index, (kind, _)) in rows.iter().enumerate() {
        heights.push(match kind {
            DiffKind::Remove if props.collapse_removed => row_height * (1.0 - drop_old),
            DiffKind::Add => row_height * progress_of(index)?,
            _ => row_height,
        });
    }
    let front_index = clamped(frame, [wipe_start, wipe_end], [0.0, count])?;
    let front_y: f64 = heights
        .iter()
        .take(truncate::<usize>(front_index.floor()))
        .sum();
    let front_visible = interpolate(
        frame,
        &[
            wipe_start - seconds(0.15),
            wipe_start,
            wipe_end,
            wipe_end + seconds(0.3),
        ],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    let body_height = BODY_PADDING.mul_add(2.0, heights.iter().sum::<f64>());
    let glyph = font_size * 0.8;
    let faint = palette.get("faint").to_owned();
    let mut list = Vec::new();
    for (index, (kind, text)) in rows.iter().enumerate() {
        let progress = progress_of(index)?;
        let (is_add, is_remove) = (*kind == DiffKind::Add, *kind == DiffKind::Remove);
        let tint = if is_remove {
            progress
        } else {
            (1.0 - settle) * progress
        };
        let color = if is_add {
            add_color
        } else if is_remove {
            remove_color
        } else {
            palette.get("faint")
        };
        let length = utf16_len(text);
        let revealed: usize = if is_add {
            truncate(f64::mul_add(progress, float(length), 0.5).floor())
        } else {
            length
        };
        let mut style = Style::default()
            .height(Length::px(heights[index]))
            .overflow(Overflow::Hidden)
            .display(Display::Flex)
            .align_items(Align::FlexStart)
            .font_size(font_size)
            .line_height(LineHeight::Length(Length::px(row_height)))
            .opacity(if is_remove {
                drop_old.mul_add(-0.5, progress.mul_add(-0.3, 1.0))
            } else {
                1.0
            });
        if (is_add || is_remove)
            && let Some(fill) = Color::css(&format!("{color}{}", tint_alpha(tint)))
        {
            style = style.background_color(fill);
        }
        let mark = match kind {
            DiffKind::Add => Some(sign(glyph, PLUS, add_color)),
            DiffKind::Remove => Some(sign(glyph, MINUS, remove_color)),
            DiffKind::Context => None,
        };
        let muted = props.dim_unchanged && *kind == DiffKind::Context;
        let token_color = |k: CodeTokenKind| {
            if muted {
                faint.clone()
            } else {
                palette.get(k.id()).to_owned()
            }
        };
        let mut code = Style::default()
            .padding_left(Length::px(CODE_INSET))
            .height(Length::px(row_height))
            .display(Display::Flex)
            .align_items(Align::Center);
        if props.strike_removed && is_remove && progress > 0.15 {
            code = code.text_decoration(TextDecoration {
                lines: vec![DecorationLine::LineThrough],
                color: Some(css_color(remove_color)),
                ..TextDecoration::default()
            });
        }
        list.push(
            div(style)
                .child(
                    span(
                        Style::default()
                            .width(Length::px(GUTTER_WIDTH))
                            .height(Length::px(row_height))
                            .flex_shrink(0.0)
                            .display(Display::Flex)
                            .align_items(Align::Center)
                            .justify_content(Align::Center),
                    )
                    .children(mark),
                )
                .child(span(code).child(code_line(
                    &tokens[index],
                    &token_color,
                    (revealed < length).then_some(revealed),
                )))
                .into(),
        );
    }
    let accent = props.accent_color.as_str();
    list.push(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .top(Length::px(front_y))
            .height(Length::px(2.0))
            .background_color(css_color(accent))
            .opacity(front_visible * 0.9)
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::px(18.0),
                spread: Length::px(2.0),
                color: tinted(accent, 40.0),
            }]))
        .into(),
    );
    let tally = |total: f64| -> Result<String, DrawFailure> {
        Ok(js_number_text(
            (clamped(frame, [wipe_start, wipe_end], [0.0, total])? + 0.5).floor(),
        ))
    };
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
        span(
            Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .right(Length::ZERO)
                .text_align(TextAlign::Center)
                .color(css_color(palette.get("dim")))
                .font_size(15.0)
                .font_weight(500.0)
                .letter_spacing(Length::px(0.2)),
        )
        .child(props.title.clone()),
    )
    .child(
        span(
            Style::default()
                .margin_left(Length::Auto)
                .display(if props.show_counts {
                    Display::Flex
                } else {
                    Display::None
                })
                .gap(12.0)
                .font_size(14.0)
                .font_variant_numeric(NumericVariant {
                    tabular: true,
                    ..NumericVariant::default()
                })
                .opacity(clamped(
                    frame,
                    [wipe_start - seconds(0.2), wipe_start + seconds(0.2)],
                    [0.0, 1.0],
                )?),
        )
        .child(
            span(Style::default().color(css_color(add_color))).child(format!("+{}", tally(added)?)),
        )
        .child(
            span(Style::default().color(css_color(remove_color)))
                .child(format!("\u{2212}{}", tally(removed)?)),
        ),
    );
    let body = div(Style::default()
        .height(Length::px(body_height))
        .padding_top(Length::px(BODY_PADDING))
        .padding_bottom(Length::px(BODY_PADDING))
        .position(Position::Relative)
        .overflow(Overflow::Hidden))
    .child(div(Style::default().position(Position::Relative)).children(list));
    let window = div(editor_window(
        &palette,
        window_width,
        scale * fit,
        enter,
        f64::from(props.corner_radius),
    )?)
    .child(header)
    .child(body);
    Ok(div(editor_stage(
        width,
        height,
        page,
        font_or(&props.font, FontRole::Code),
    ))
    .child(backdrop_glow(accent, props.glow, chrome_in))
    .child(backdrop_vignette(props.vignette, dark))
    .child(window)
    .into())
}
