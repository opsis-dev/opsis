use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{ColorStop, Gradient, RadialShape, RadialSize, Shadow, Transform};
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, LineHeight, NumericVariant, Overflow, Position, Style,
    TextTransform,
};
use opsis_scene::values::{Angle, Edges, Length, Num, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, steps_timeline_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct TimelineStep {
    #[setting("Title", default = "Next step")]
    pub title: String,
    #[setting("Description", default = "Close with one clear call to action.")]
    pub description: String,
}

#[settings(export)]
pub struct StepsTimeline {
    #[setting("Title", default = "How we read the quarter")]
    pub title: String,
    #[setting("Small heading")]
    pub eyebrow: String,
    #[setting(
        "Steps",
        default = vec![
            TimelineStep {
                title: "Hook".into(),
                description: "Lead with the sharpest insight.".into(),
            },
            TimelineStep {
                title: "Proof".into(),
                description: "Show the chart and the numbers.".into(),
            },
            TimelineStep::default(),
        ],
        length(min = 1, max = 5),
        help = "Up to five steps, checked off one after another."
    )]
    pub steps: Vec<TimelineStep>,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = steps_timeline_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Accent color", default = "#f59e0b", check = colour)]
    pub accent_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Set it to transparent to lay the timeline over other layers."
    )]
    pub background_color: String,
    #[setting("Speed", default = 1.5, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        advanced
    )]
    pub text_size: f64,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_caps: bool,
    #[setting("Glow", default = 1.0, range(min = 0, max = 3), step = 0.05, advanced)]
    pub glow: f64,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<StepsTimeline>(
        element(
            ElementKind::Component,
            "steps-timeline",
            "Steps timeline",
            "A row of steps that check off one after another along a line, each with its title and a short description.",
            ElementCategory::Data,
            150,
        )
        .look(NodeLook::of("lucide:route").moves(opsis_types::looks::NodeMotion::Draw)),
    )
}

const TITLE_AT: f64 = 0.0;
const START: f64 = 0.5;
const DWELL: f64 = 0.78;
const TRAVEL: f64 = 0.42;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const POP: SpringConfig = SpringConfig {
    mass: 0.6,
    damping: 12.0,
    stiffness: 200.0,
    overshoot_clamping: false,
};

fn palette<'a>(theme: &'a Theme, key: &str) -> &'a str {
    let dark = [
        ("page", "#0c0c14"),
        ("window", "#0B0C11"),
        ("border", "rgba(255,255,255,0.09)"),
        ("fg", "#D8DCE4"),
        ("dim", "#7A828F"),
        ("faint", "#4A5160"),
    ];
    let light = [
        ("page", "#F4F4F2"),
        ("window", "#FFFFFF"),
        ("border", "rgba(0,0,0,0.10)"),
        ("fg", "#22252B"),
        ("dim", "#6B7280"),
        ("faint", "#A0A6B0"),
    ];
    let base = if theme.mode == ThemeMode::Dark {
        dark
    } else {
        light
    };
    theme.colors.get(key).map_or_else(
        || {
            base.iter()
                .find(|(name, _)| *name == key)
                .map_or("", |(_, value)| value)
        },
        String::as_str,
    )
}

fn ramp(input: f64, from: [f64; 2], easing: Easing) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        input,
        &from,
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(easing),
            ..Interpolation::clamped()
        },
    )?)
}

fn arrival(index: usize) -> f64 {
    float(index).mul_add(DWELL + TRAVEL, START)
}

fn completion(index: usize) -> f64 {
    arrival(index) + DWELL
}

fn head_index(now: f64, count: usize) -> Result<f64, DrawFailure> {
    if now <= arrival(0) {
        return Ok(0.0);
    }
    for index in 0..count.saturating_sub(1) {
        if now < completion(index) {
            return Ok(float(index));
        }
        if now < arrival(index + 1) {
            return Ok(interpolate(
                now,
                &[completion(index), arrival(index + 1)],
                &[float(index), float(index + 1)],
                &Interpolation {
                    easing: Some(EDITORIAL),
                    ..Interpolation::clamped()
                },
            )?);
        }
    }
    Ok(float(count.saturating_sub(1)))
}

fn check_glyph(size: f64, color: &str, progress: f64) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr("d", "M4.8 12.6l4.9 4.9 9.5-10.4")
                    .attr("stroke", color)
                    .attr("stroke-width", 2.8)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round")
                    .attr("stroke-dasharray", 22.0)
                    .attr("stroke-dashoffset", 22.0 * (1.0 - progress)),
            ),
        Style::default()
            .width(Length::px(size))
            .height(Length::px(size)),
    )
    .into()
}

struct Rail {
    portrait: bool,
    across: f64,
    start: f64,
    span: f64,
    count: usize,
}

impl Rail {
    fn node_at(&self, index: f64) -> f64 {
        self.start
            + if self.count == 1 {
                self.span / 2.0
            } else {
                index / float(self.count - 1) * self.span
            }
    }

    const fn point(&self, along: f64) -> (f64, f64) {
        if self.portrait {
            (self.across, along)
        } else {
            (along, self.across)
        }
    }
}

#[expect(
    clippy::suboptimal_flops,
    clippy::too_many_lines,
    reason = "the film lays the steps out with separate multiplies and adds, in one pass like its component"
)]
pub fn draw(props: &StepsTimeline, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (frame, fps) = (cx.frame(), cx.fps());
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = &props.theme;
    let accent = props.accent_color.as_str();
    let speed = props.speed;
    let (side, edge) = safe_padding(width, height);
    let now = frame / fps * speed;
    let at = |seconds: f64| seconds * fps / speed;
    let ease = |from: f64, to: f64| ramp(frame, [at(from), at(to)], easing_enter());
    let stage = (side, edge, width - side * 2.0, height - edge * 2.0);
    let portrait = height > width;
    let u = (stage.2 / 1120.0).min(stage.3 / 620.0);
    let list: Vec<&TimelineStep> = props.steps.iter().take(5).collect();
    let count = list.len().max(1);
    let finished = completion(count - 1);
    let node = 60.0 * u;
    let has_title = !props.title.is_empty();
    let has_eyebrow = !props.eyebrow.is_empty();
    let header = if has_title || has_eyebrow {
        if has_title { 108.0 * u } else { 44.0 * u }
    } else {
        0.0
    };
    let inset = if portrait {
        node / 2.0 + 4.0 * u
    } else {
        stage.2 / float(count) / 2.0
    };
    let start = if portrait { header + node * 0.9 } else { inset };
    let end = if portrait {
        stage.3 - node * 0.6
    } else {
        stage.2 - inset
    };
    let span = (end - start).max(1.0);
    let has_description = list.iter().any(|step| !step.description.is_empty());
    let copy_drop = node * 0.78 + if has_description { 116.0 * u } else { 48.0 * u };
    let across = if portrait {
        node / 2.0
    } else {
        header + (node / 2.0).max((stage.3 - header - (copy_drop + node / 2.0)) / 2.0 + node / 2.0)
    };
    let rail = Rail {
        portrait,
        across,
        start,
        span,
        count,
    };
    let head = head_index(now, count)?;
    let head_pos = rail.node_at(head);
    let drawn = ease(START - 0.35, START + 0.15)?;
    let title_in = ease(TITLE_AT, TITLE_AT + 0.55)?;
    let page = palette(theme, "page");
    let ground = if props.background_color.is_empty() {
        page
    } else {
        props.background_color.as_str()
    };
    let glow = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(80.0), Length::percent(50.0)),
        center: Position2 {
            x: Length::percent(50.0),
            y: Length::percent(if portrait { 30.0 } else { 40.0 }),
        },
        stops: vec![
            ColorStop::new(tinted(accent, 8.0 * props.glow)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(70.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    let mut content = div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(stage.0))
        .top(Length::px(stage.1))
        .width(Length::px(stage.2))
        .height(Length::px(stage.3))
        .translate(Length::ZERO, Length::ZERO));
    if has_eyebrow {
        let mut style = Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .top(Length::ZERO)
            .color(css_color(accent))
            .font_size(21.0 * u)
            .font_weight(600.0)
            .text_transform(if props.eyebrow_caps {
                TextTransform::Uppercase
            } else {
                TextTransform::None
            })
            .opacity(title_in)
            .translate(Length::ZERO, Length::px((1.0 - title_in) * 10.0 * u));
        if props.eyebrow_caps {
            style = style.letter_spacing(Length::em(0.12));
        }
        content = content.child(div(style).child(props.eyebrow.clone()));
    }
    if has_title {
        content = content.child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::px(if has_eyebrow { 36.0 * u } else { 0.0 }))
                .margin(Edges::all(Length::ZERO))
                .color(css_color(palette(theme, "fg")))
                .font_size(62.0 * props.text_size * u)
                .font_weight(700.0)
                .line_height(LineHeight::Number(Num(1.05)))
                .letter_spacing(Length::em(-0.02))
                .opacity(title_in)
                .translate(Length::ZERO, Length::px((1.0 - title_in) * 16.0 * u)))
            .child(props.title.clone()),
        );
    }
    let (rail_left, rail_top) = if portrait {
        (across - 1.5 * u, start)
    } else {
        (start, across - 1.5 * u)
    };
    let filled = (head_pos - start).max(0.0);
    content = content
        .child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(rail_left))
            .top(Length::px(rail_top))
            .width(Length::px(if portrait { 3.0 * u } else { span }))
            .height(Length::px(if portrait { span } else { 3.0 * u }))
            .background_color(css_color(palette(theme, "border")))
            .border_radius_all(999.0)
            .transform_origin(if portrait {
                (Length::percent(50.0), Length::ZERO, Length::ZERO)
            } else {
                (Length::ZERO, Length::percent(50.0), Length::ZERO)
            })
            .transform(vec![if portrait {
                Transform::Scale(Num(1.0), Num(drawn), Num(1.0))
            } else {
                Transform::Scale(Num(drawn), Num(1.0), Num(1.0))
            }])))
        .child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(rail_left))
            .top(Length::px(rail_top))
            .width(Length::px(if portrait { 3.0 * u } else { filled }))
            .height(Length::px(if portrait { filled } else { 3.0 * u }))
            .background_color(css_color(accent))
            .border_radius_all(999.0)
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::px(14.0 * u),
                spread: Length::ZERO,
                color: tinted(accent, 53.0),
            }])
            .opacity(drawn)));
    for (index, step) in list.iter().enumerate() {
        let landed = ease(arrival(index), arrival(index) + 0.3)?;
        let work = ramp(now, [arrival(index), completion(index)], EDITORIAL)?;
        let check = ramp(
            now,
            [completion(index), completion(index) + 0.32],
            easing_enter(),
        )?;
        let active = landed * (1.0 - check * 0.55);
        let pop = spring(&Spring {
            config: POP,
            ..Spring::at(frame - at(arrival(index)), fps)
        })?;
        let (x, y) = rail.point(rail.node_at(float(index)));
        let copy_in = ease(arrival(index) + 0.08, arrival(index) + 0.5)?;
        let ring_radius = node / 2.0 - 2.0 * u;
        let ring_length = 2.0 * std::f64::consts::PI * ring_radius;
        let fill = if check > 0.4 {
            tinted(accent, 13.0)
        } else if landed > 0.1 {
            css_color(accent)
        } else {
            css_color(palette(theme, "window"))
        };
        let ink = if check > 0.4 {
            accent
        } else if landed > 0.1 {
            page
        } else {
            palette(theme, "dim")
        };
        let swell = 0.75 + 0.25 * drawn + pop * 0.06;
        let mark: Node = if check > 0.05 {
            check_glyph(30.0 * u, accent, check)
        } else {
            (index + 1).to_string().into()
        };
        let mut style = Style::default()
            .position(Position::Absolute)
            .left(Length::px(x - node / 2.0))
            .top(Length::px(y - node / 2.0))
            .width(Length::px(node))
            .height(Length::px(node))
            .border_radius_all(Length::percent(50.0))
            .background_color(fill)
            .border(Edges::all(BorderSide {
                width: Length::px(2.0 * u),
                style: BorderStyle::Solid,
                color: Some(css_color(if landed > 0.1 {
                    accent
                } else {
                    palette(theme, "border")
                })),
            }))
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::px(34.0 * u * active),
                spread: Length::ZERO,
                color: tinted(accent, 40.0),
            }])
            .display(Display::Grid)
            .align_items(Align::Center)
            .justify_items(Align::Center)
            .scale(swell, swell)
            .color(css_color(ink))
            .font_size(26.0 * u)
            .font_weight(700.0);
        style = style.font_variant_numeric(NumericVariant {
            tabular: true,
            ..NumericVariant::default()
        });
        content = content.child(div(style).child(mark));
        let ring = node + 16.0 * u;
        content = content.child(svg(
            SvgElement::new("svg")
                .attr("width", ring)
                .attr("height", ring)
                .child(
                    SvgElement::new("circle")
                        .attr("cx", ring / 2.0)
                        .attr("cy", ring / 2.0)
                        .attr("r", ring_radius + 6.0 * u)
                        .attr("fill", "none")
                        .attr("stroke", accent)
                        .attr("stroke-width", 2.4 * u)
                        .attr("stroke-linecap", "round")
                        .attr("stroke-dasharray", ring_length)
                        .attr("stroke-dashoffset", ring_length * (1.0 - work)),
                ),
            Style::default()
                .position(Position::Absolute)
                .left(Length::px(x - ring / 2.0))
                .top(Length::px(y - ring / 2.0))
                .width(Length::px(ring))
                .height(Length::px(ring))
                .opacity(landed * (1.0 - check))
                .transform(vec![Transform::Rotate(Angle::deg(-90.0))]),
        ));
        let (copy_left, copy_top, copy_width) = if portrait {
            (
                across + node * 0.85,
                rail.node_at(float(index)) - node * 0.34,
                stage.2 - across - node * 0.85,
            )
        } else {
            (
                x - stage.2 / float(count) / 2.0 + 10.0 * u,
                across + node * 0.78,
                stage.2 / float(count) - 20.0 * u,
            )
        };
        let mut copy = div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(copy_left))
            .top(Length::px(copy_top))
            .width(Length::px(copy_width))
            .opacity(copy_in)
            .translate(Length::ZERO, Length::px((1.0 - copy_in) * 14.0 * u)))
        .child(
            div(Style::default()
                .color(css_color(if check > 0.4 {
                    palette(theme, "dim")
                } else {
                    palette(theme, "fg")
                }))
                .font_size(30.0 * u)
                .font_weight(600.0)
                .line_height(LineHeight::Number(Num(1.15)))
                .letter_spacing(Length::em(-0.01)))
            .child(step.title.clone()),
        );
        if !step.description.is_empty() {
            copy = copy.child(
                div(Style::default()
                    .margin_top(Length::px(9.0 * u))
                    .color(css_color(palette(theme, "faint")))
                    .font_size(21.0 * u)
                    .line_height(LineHeight::Number(Num(1.35))))
                .child(step.description.clone()),
            );
        }
        content = content.child(copy);
    }
    let (head_x, head_y) = rail.point(head_pos);
    let moving = if now >= finished || (head % 1.0).abs() <= 0.02 {
        0.0
    } else {
        1.0
    };
    content = content.child(div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(head_x - 7.0 * u))
        .top(Length::px(head_y - 7.0 * u))
        .width(Length::px(14.0 * u))
        .height(Length::px(14.0 * u))
        .border_radius_all(Length::percent(50.0))
        .background_color(css_color(accent))
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::px(18.0 * u),
            spread: Length::ZERO,
            color: css_color(accent),
        }])
        .opacity(drawn * moving)));
    Ok(div(Style::default()
        .width(Length::px(width))
        .height(Length::px(height))
        .background_color(css_color(ground))
        .font_family(font_or(&props.font_family, FontRole::Main))
        .position(Position::Relative)
        .overflow(Overflow::Hidden))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![glow.layer()])))
    .child(content)
    .into())
}
