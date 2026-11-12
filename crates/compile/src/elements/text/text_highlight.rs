use opsis_macros::{choice, settings};
use opsis_scene::color::Color;
use opsis_scene::css::{font_family, with_background};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{ColorStop, Gradient, LayerSize, Transform};
use opsis_scene::spring::{DEFAULT_SPRING, Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, Display, GridLine, LineHeight, Position, Style, TextAlign, VerticalAlign, WhiteSpace,
};
use opsis_scene::values::{Angle, Length, Num, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::{interpolated_color, lightness, with_alpha};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum HighlightPreset {
    Marker,
    Color,
    Underline,
    Strikethrough,
    Shimmer,
}

#[settings(export)]
pub struct TextHighlight {
    #[setting("Preset", default = HighlightPreset::Marker)]
    pub preset: HighlightPreset,
    #[setting("Before", default = "Made for ")]
    pub before: String,
    #[setting("Highlight", default = "builders")]
    pub highlight: String,
    #[setting("After", default = ".")]
    pub after: String,
    #[setting(
        "Replace with",
        default = "Free",
        show_if = { "preset": ["strikethrough"] }
    )]
    pub replace_with: String,
    #[setting("Base color", widget = "color", zero_label = "Theme")]
    pub base_color: String,
    #[setting(
        "Accent color",
        default = "#266DF0",
        widget = "color",
        zero_label = "Theme"
    )]
    pub accent_color: String,
    #[setting(
        "Highlighted text colour",
        widget = "color",
        zero_label = "Auto",
        advanced
    )]
    pub highlighted_text_color: String,
    #[setting(
        "Start at",
        default = 0.2,
        range(min = 0, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub start_at: f64,
    #[setting(
        "Draw time",
        default = 0.4667,
        range(min = 0.1667, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_duration: f64,
    #[setting(
        "Swap time",
        default = 0.2333,
        range(min = 0.0333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        show_if = { "preset": ["strikethrough"] },
        advanced
    )]
    pub swap_length: f64,
    #[setting(
        "Line thickness",
        default = 4,
        range(min = 1, max = 20),
        step = 1,
        show_if = { "preset": ["underline", "strikethrough"] },
        advanced
    )]
    pub thickness: u32,
    #[setting(
        "Marker strength",
        default = 16.0,
        range(min = 5, max = 100),
        unit = "%",
        show_if = { "preset": ["marker"] },
        advanced
    )]
    pub marker_opacity: f64,
    #[setting(
        "Marker height",
        default = 1.0,
        range(min = 0.2, max = 1.4),
        step = 0.05,
        unit = "× line",
        show_if = { "preset": ["marker"] },
        advanced
    )]
    pub marker_height: f64,
    #[setting(
        "Shine color",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub shine_color: String,
    #[setting(
        "Shine width",
        default = 40.0,
        range(min = 10, max = 80),
        unit = "%",
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub shine_width: f64,
    #[setting(
        "Sweeps",
        default = 1,
        range(min = 1, max = 6),
        step = 1,
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub sweeps: u32,
    #[setting(
        "Shine angle",
        default = 110.0,
        range(min = 0, max = 180),
        step = 5,
        unit = "°",
        show_if = { "preset": ["shimmer"] },
        advanced
    )]
    pub band_angle: f64,
    #[setting("Font size", default = 56, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Set it to transparent to lay the text over other layers."
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Centre across",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub x: f64,
    #[setting(
        "Centre down",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub y: f64,
    #[setting("Letter spacing", default = -0.02, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 1.25, range(min = 0.9, max = 2), advanced)]
    pub line_height: f64,
    #[setting(
        "Text width",
        default = 80.0,
        range(min = 30, max = 100),
        unit = "%",
        advanced
    )]
    pub text_width: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TextHighlight>(
            element(
                ElementKind::Component,
                "text-highlight",
                "Text highlight",
                "Brings attention to one word in a sentence: a marker sweeps behind it, its colour changes, a line draws under it, it is struck out and replaced, or a shine passes over it.",
                ElementCategory::Text,
                56,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:baseline")),
        )
}

const SHADE_FROM: f64 = 0.5;
const SHADE_TO: f64 = 0.85;
const STRIKE_RISE: f64 = 8.0;
const SWEEP_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    ..DEFAULT_SPRING
};

fn lighter_of<'a>(a: &'a str, b: &'a str) -> &'a str {
    if lightness(a) >= lightness(b) { a } else { b }
}

fn shine_position(frame: f64, start: f64, duration: f64, sweeps: u32) -> Result<f64, DrawFailure> {
    let count = f64::from(sweeps.max(1));
    let progress = clamped(frame, [start, duration.mul_add(count, start)], [0.0, count])?;
    let pass = if progress >= count {
        1.0
    } else {
        progress - progress.floor()
    };
    clamped(pass, [0.0, 1.0], [200.0, -100.0])
}

const fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

fn shaded(amount: f64, base: &str, emphasis: &str) -> Result<Color, DrawFailure> {
    Ok(interpolated_color(amount, &[0.0, 1.0], &[base, emphasis])?)
}

#[expect(
    clippy::too_many_lines,
    reason = "each preset's markup follows the film's one view function"
)]
fn highlight_node(
    props: &TextHighlight,
    cx: &Cx<'_>,
    frame: f64,
    (base, accent, emphasis, shine): (&str, &str, &str, &str),
) -> Result<Node, DrawFailure> {
    let sweep = spring(&Spring {
        config: SWEEP_SPRING,
        duration_in_frames: Some(props.draw_duration),
        ..Spring::at(frame - props.start_at, Cx::authored_fps())
    })?;
    let thickness = f64::from(props.thickness);
    let swept = || clamped(sweep, [SHADE_FROM, SHADE_TO], [0.0, 1.0]);
    let wrapper = || {
        span(
            Style::default()
                .position(Position::Relative)
                .display(Display::InlineBlock),
        )
    };
    let left_center = (Length::ZERO, Length::percent(50.0), Length::ZERO);
    let scale_x = vec![Transform::Scale(Num(sweep), Num(1.0), Num(1.0))];
    Ok(match props.preset {
        HighlightPreset::Marker => wrapper()
            .child(span(
                Style::default()
                    .position(Position::Absolute)
                    .left(Length::em(-0.12))
                    .right(Length::em(-0.12))
                    .bottom(Length::em(-0.02))
                    .height(Length::percent(props.marker_height * 100.0))
                    .background_color(cx.color(&with_alpha(accent, props.marker_opacity / 100.0)))
                    .border_radius_all(Length::em(0.12))
                    .transform_origin(left_center)
                    .transform(scale_x)
                    .z_index(0),
            ))
            .child(
                span(
                    Style::default()
                        .position(Position::Relative)
                        .z_index(1)
                        .color(shaded(swept()?, base, emphasis)?),
                )
                .child(props.highlight.as_str()),
            )
            .into(),
        HighlightPreset::Color => {
            let amount = clamped(
                frame,
                [props.start_at, props.start_at + props.draw_duration],
                [0.0, 1.0],
            )?;
            span(Style::default().color(shaded(amount, base, emphasis)?))
                .child(props.highlight.as_str())
                .into()
        }
        HighlightPreset::Underline => wrapper()
            .child(
                span(Style::default().color(shaded(swept()?, base, emphasis)?))
                    .child(props.highlight.as_str()),
            )
            .child(span(
                Style::default()
                    .position(Position::Absolute)
                    .left(Length::ZERO)
                    .right(Length::ZERO)
                    .bottom(Length::em(-0.06))
                    .height(thickness)
                    .background_color(cx.color(accent))
                    .border_radius_all(thickness / 2.0)
                    .transform_origin(left_center)
                    .transform(scale_x),
            ))
            .into(),
        HighlightPreset::Strikethrough => {
            let draw_end = props.start_at + props.draw_duration;
            let fade_end = draw_end + props.swap_length;
            let cell = (GridLine::Line(1), GridLine::Auto);
            let replacement = non_empty(&props.replace_with, &props.highlight);
            span(
                Style::default()
                    .display(Display::InlineGrid)
                    .vertical_align(VerticalAlign::Baseline)
                    .justify_items(Align::Center),
            )
            .child(
                span(
                    Style::default()
                        .grid_row(cell)
                        .grid_column(cell)
                        .position(Position::Relative)
                        .white_space(WhiteSpace::Pre)
                        .opacity(clamped(frame, [draw_end, fade_end], [1.0, 0.0])?),
                )
                .child(props.highlight.as_str())
                .child(span(
                    Style::default()
                        .position(Position::Absolute)
                        .left(Length::ZERO)
                        .top(Length::percent(50.0))
                        .height(thickness)
                        .width(Length::percent(clamped(
                            frame,
                            [props.start_at, draw_end],
                            [0.0, 100.0],
                        )?))
                        .background_color(cx.color(accent))
                        .transform(vec![Transform::Translate(
                            Length::ZERO,
                            Length::percent(-50.0),
                            Length::ZERO,
                        )])
                        .border_radius_all(thickness / 2.0),
                )),
            )
            .child(
                span(
                    Style::default()
                        .grid_row(cell)
                        .grid_column(cell)
                        .white_space(WhiteSpace::Pre)
                        .color(cx.color(emphasis))
                        .opacity(clamped(frame, [draw_end, fade_end], [0.0, 1.0])?)
                        .transform(vec![Transform::Translate(
                            Length::ZERO,
                            Length::px(clamped(frame, [draw_end, fade_end], [STRIKE_RISE, 0.0])?),
                            Length::ZERO,
                        )]),
                )
                .child(replacement),
            )
            .into()
        }
        HighlightPreset::Shimmer => {
            let band = Gradient::linear(
                Angle::deg(props.band_angle),
                vec![
                    ColorStop::at(
                        Color::TRANSPARENT,
                        Length::percent(50.0 - props.shine_width / 2.0),
                    ),
                    ColorStop::at(cx.color(shine), Length::percent(50.0)),
                    ColorStop::at(
                        Color::TRANSPARENT,
                        Length::percent(50.0 + props.shine_width / 2.0),
                    ),
                ],
            );
            let mut layer = band.layer();
            layer.size = LayerSize::Explicit(Length::percent(200.0), Length::percent(100.0));
            layer.position = Position2 {
                x: Length::percent(shine_position(
                    frame,
                    props.start_at,
                    props.draw_duration,
                    props.sweeps,
                )?),
                y: Length::percent(50.0),
            };
            wrapper()
                .child(
                    span(Style::default().color(cx.color(emphasis)))
                        .child(props.highlight.as_str()),
                )
                .child(
                    span(
                        Style::default()
                            .position(Position::Absolute)
                            .inset(Length::ZERO)
                            .color(Color::TRANSPARENT)
                            .background(vec![layer])
                            .background_clip_text(true),
                    )
                    .child(props.highlight.as_str()),
                )
                .into()
        }
    })
}

pub fn draw(props: &TextHighlight, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let theme = warm_theme(&props.theme);
    let ground = non_empty(&props.background, theme.get("background"));
    let face = resolve_font(non_empty(
        &props.font_family,
        theme.font_family().unwrap_or(""),
    ))
    .map_or_else(|| font_or("", FontRole::Main), |f| font_family(&f));
    let base = non_empty(&props.base_color, theme.get("foreground"));
    let accent = non_empty(&props.accent_color, theme.get("primary"));
    let shine = if props.shine_color.is_empty() {
        lighter_of(theme.get("card"), theme.get("foreground"))
    } else {
        props.shine_color.as_str()
    };
    let emphasis = if props.highlighted_text_color.is_empty() {
        if matches!(
            props.preset,
            HighlightPreset::Color | HighlightPreset::Strikethrough
        ) {
            accent
        } else {
            base
        }
    } else {
        props.highlighted_text_color.as_str()
    };
    let highlighted = highlight_node(props, cx, frame, (base, accent, emphasis, shine))?;
    let text = span(
        Style::default()
            .font_size(f64::from(props.font_size))
            .font_weight(props.font_weight.value())
            .color(cx.color(base))
            .letter_spacing(Length::em(props.letter_spacing))
            .line_height(LineHeight::Number(Num(props.line_height)))
            .font_family(face)
            .white_space(WhiteSpace::PreWrap)
            .text_align(TextAlign::Center)
            .max_width(Length::percent(props.text_width))
            .position(Position::Absolute)
            .left(Length::percent(props.x * 100.0))
            .top(Length::percent(props.y * 100.0))
            .width(Length::percent(props.text_width))
            .transform(vec![Transform::Translate(
                Length::percent(-50.0),
                Length::percent(-50.0),
                Length::ZERO,
            )]),
    )
    .child(props.before.as_str())
    .child(highlighted)
    .child(props.after.as_str());
    Ok(div(with_background(
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO),
        ground,
    ))
    .child(text)
    .into())
}
