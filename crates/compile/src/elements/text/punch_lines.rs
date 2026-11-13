use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::css::{font_family, length as css_length};
use opsis_scene::drawing::Font;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, Display, FlexDirection, LineHeight, Position, Style, TextRendering, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PunchLines {
    #[setting(
        "Script (| card, / line)",
        default = "Record it. / Edit it. | One take. | Ship."
    )]
    pub script: String,
    #[setting("Card looks", default = "slide,punch")]
    pub styles: String,
    #[setting("Card sizes", default = "1,0.945,2.7")]
    pub sizes: String,
    #[setting("Card lengths", default = "1.6", frames_at = 30)]
    pub holds: String,
    #[setting("Accent card", default = 3, range(min = 0, max = 8), step = 1)]
    pub accent_beat: u32,
    #[setting("Font size", default = 93, range(min = 24, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Line height (em)",
        default = 0.89,
        range(min = 0.7, max = 1.6),
        step = 0.005,
        advanced
    )]
    pub line_height: f64,
    #[setting("Letter spacing", default = "-0.022em", advanced)]
    pub letter_spacing: String,
    #[setting("Word spacing", default = "-0.02em", advanced)]
    pub word_spacing: String,
    #[setting("Ground", widget = "color", zero_label = "Theme")]
    pub ground: String,
    #[setting("Type", widget = "color", zero_label = "Theme")]
    pub ink: String,
    #[setting("Accent", default = "#ff1c8e", check = colour)]
    pub accent_color: String,
    #[setting(
        "Entrance lead",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub lead: f64,
    #[setting("Slide from (em)", default = -0.737, range(min = -3, max = 0), step = 0.01, advanced)]
    pub slide_from: f64,
    #[setting(
        "Slide kick (em/s)",
        default = 7.34,
        range(min = 0, max = 20),
        step = 0.1,
        advanced
    )]
    pub slide_kick: f64,
    #[setting(
        "Slide settle",
        default = 2.846,
        range(min = 0.5, max = 8),
        step = 0.05,
        advanced
    )]
    pub slide_settle: f64,
    #[setting(
        "Line stagger",
        default = 0.1167,
        range(min = 0, max = 0.6667),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub line_stagger: f64,
    #[setting("Slide drop (em)", default = 0.313, range(min = -1, max = 1), step = 0.01, advanced)]
    pub slide_drop: f64,
    #[setting(
        "Motion blur",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub motion_blur: f64,
    #[setting(
        "Punch from",
        default = 0.32,
        range(min = 0.05, max = 1),
        step = 0.01,
        advanced
    )]
    pub punch_from: f64,
    #[setting(
        "Punch",
        default = 0.5,
        range(min = 0.0667, max = 1.3333),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub punch_frames: f64,
    #[setting(
        "Word stagger",
        default = 0.0933,
        range(min = 0, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub word_stagger: f64,
    #[setting(
        "Push from",
        default = 0.778,
        range(min = 0.3, max = 1),
        step = 0.005,
        advanced
    )]
    pub push_from: f64,
    #[setting(
        "Push per second",
        default = 0.205,
        range(min = 0, max = 1),
        step = 0.005,
        advanced
    )]
    pub push_rate: f64,
    #[setting(
        "Rush lead",
        default = 0.4067,
        range(min = 0, max = 1.3333),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub rush_lead: f64,
    #[setting(
        "Rush",
        default = 0.6,
        range(min = 0.1333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub rush_frames: f64,
    #[setting(
        "Rush travel",
        default = 0.85,
        range(min = 0.2, max = 0.95),
        step = 0.01,
        advanced
    )]
    pub rush_travel: f64,
    #[setting(
        "Fit width",
        default = 0.92,
        range(min = 0.4, max = 1),
        step = 0.01,
        advanced
    )]
    pub fit: f64,
    #[setting("Theme", transform = warm_themes, widget = "theme")]
    pub theme: Theme,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<PunchLines>(
            element(
                ElementKind::Component,
                "punch-lines",
                "Punch lines",
                "Kinetic typography title cards: bold statements that cut hard between beats. A run of full-frame title cards that cut hard between beats.",
                ElementCategory::Text,
                144,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:type").moves(NodeMotion::HardCut)),
        )
}

const SANS: &str = "\"Inter\"";
const DEFAULT_HOLD: f64 = 48.0;
const DEFAULT_BASELINE: f64 = 88.0;
const DEFAULT_ASCENT: f64 = 70.0;
const PROBE_SIZE: f64 = 100.0;
const MIN_SIGMA: f64 = 0.2;

fn punch_ease(t: f64) -> f64 {
    Easing::Bezier(0.65, 0.0, 0.35, 1.0).at(t)
}

fn rush_ease(t: f64) -> f64 {
    Easing::Bezier(1.0, 0.0, 0.48, 1.0).at(t)
}

fn slide_swing(from: f64, kick: f64, settle: f64) -> f64 {
    let peak_at = if kick > 0.0 {
        (1.0 / settle - from / kick).max(0.0)
    } else {
        0.0
    };
    let peak = kick.mul_add(peak_at, from) * (-settle * peak_at).exp();
    from.abs().max(peak.abs())
}

fn list(text: &str) -> Vec<&str> {
    text.split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect()
}

fn numbers(text: &str) -> Vec<f64> {
    list(text)
        .into_iter()
        .filter_map(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .collect()
}

fn pick<T: Copy>(values: &[T], i: usize, fallback: T) -> T {
    values
        .get(i.min(values.len().saturating_sub(1)))
        .copied()
        .unwrap_or(fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Beat {
    Slide,
    Punch,
}

struct Planned {
    lines: Vec<Vec<String>>,
    style: Beat,
    size: f64,
    accent: bool,
    start: f64,
    end: f64,
    reveal_at: f64,
}

fn plan(props: &PunchLines) -> Vec<Planned> {
    let styles = list(&props.styles);
    let sizes = numbers(&props.sizes);
    let holds = numbers(&props.holds);
    let mut reveal_at = 0.0;
    let mut beats = Vec::new();
    for (i, text) in props
        .script
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let hold = pick(&holds, i, DEFAULT_HOLD).max(1.0);
        beats.push(Planned {
            lines: text
                .split('/')
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(|l| l.split_whitespace().map(str::to_owned).collect())
                .collect(),
            style: if pick(&styles, i, "punch") == "slide" {
                Beat::Slide
            } else {
                Beat::Punch
            },
            size: pick(&sizes, i, 1.0),
            accent: u32::try_from(i + 1).is_ok_and(|n| n == props.accent_beat),
            start: if i == 0 { 0.0 } else { reveal_at - props.lead },
            reveal_at,
            end: reveal_at + hold,
        });
        reveal_at += hold;
    }
    beats
}

struct Faces {
    baseline: f64,
    ascents: Vec<f64>,
    descents: Vec<f64>,
    widths: Vec<f64>,
}

fn spacing(text: &str) -> Length {
    css_length(text).unwrap_or(Length::ZERO)
}

fn measure_faces(props: &PunchLines, beats: &[Planned], face: &[String], cx: &Cx<'_>) -> Faces {
    let weight = props.font_weight.value();
    let hidden = || {
        div(Style::default()
            .position(Position::Absolute)
            .top(Length::ZERO)
            .left(Length::ZERO)
            .white_space(WhiteSpace::Nowrap))
    };
    let probe = hidden().child(
        span(
            Style::default()
                .font_size(PROBE_SIZE)
                .line_height(LineHeight::Length(Length::px(
                    props.line_height * PROBE_SIZE,
                ))),
        )
        .child("Hxg")
        .child(span(
            Style::default()
                .display(Display::InlineBlock)
                .width(Length::ZERO)
                .height(Length::ZERO),
        )),
    );
    let mut lines = hidden();
    for beat in beats {
        for words in &beat.lines {
            lines = lines.child(
                span(
                    Style::default()
                        .display(Display::InlineBlock)
                        .font_size(f64::from(props.font_size) * beat.size)
                        .line_height(LineHeight::Length(Length::em(props.line_height)))
                        .letter_spacing(spacing(&props.letter_spacing))
                        .word_spacing(spacing(&props.word_spacing)),
                )
                .child(words.join(" ")),
            );
        }
    }
    let root: Node = div(absolute_fill()
        .font_family(face.to_vec())
        .font_weight(weight))
    .child(probe)
    .child(lines)
    .into();
    let boxes = cx.measure.boxes(&root);
    let find = |path: &[usize]| boxes.iter().find(|b| b.path == path);
    let count: usize = beats.iter().map(|b| b.lines.len()).sum();
    let font = Font {
        families: face.to_vec(),
        size: Num(PROBE_SIZE),
        weight: Num(weight),
        italic: false,
    };
    let joined = |words: Option<&Vec<String>>| words.map_or_default(|w| w.join(" "));
    Faces {
        baseline: find(&[0, 0, 1]).map_or(DEFAULT_BASELINE, |b| b.y),
        ascents: beats
            .iter()
            .map(|b| {
                let ascent = cx.measure.text_ink(&joined(b.lines.first()), &font).0;
                if ascent > 0.0 { ascent } else { DEFAULT_ASCENT }
            })
            .collect(),
        descents: beats
            .iter()
            .map(|b| {
                cx.measure
                    .text_ink(&joined(b.lines.last()), &font)
                    .1
                    .max(0.0)
            })
            .collect(),
        widths: (0..count)
            .map(|i| find(&[1, i]).map_or(0.0, |b| b.width))
            .collect(),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "a card's motion follows the film's one card function in order"
)]
fn card(
    props: &PunchLines,
    beat: &Planned,
    index: usize,
    widths: &[f64],
    faces: &Faces,
    (frame, color): (f64, Color),
    cx: &Cx<'_>,
) -> Option<Node> {
    if frame < beat.reveal_at || frame >= beat.end {
        return None;
    }
    let fps = Cx::authored_fps();
    let width = cx.at.stage.width;
    let font_size = f64::from(props.font_size);
    let t = (frame - beat.start) / fps;
    let natural = widths.iter().copied().fold(0.0, f64::max);
    let max_push = if beat.style == Beat::Punch {
        props
            .push_rate
            .mul_add((beat.end - beat.start) / fps, props.push_from)
    } else {
        1.0
    };
    let swing = if beat.style == Beat::Slide {
        slide_swing(props.slide_from, props.slide_kick, props.slide_settle) * font_size * beat.size
    } else {
        0.0
    };
    let need = natural.mul_add(max_push, 2.0 * swing);
    let shrink = if need > width * props.fit && need > 0.0 {
        width * props.fit / need
    } else {
        1.0
    };
    let em = font_size * beat.size * shrink;
    let line_box = props.line_height * em;
    let baseline = faces.baseline / PROBE_SIZE * em;
    let rise = faces.ascents.get(index).copied().unwrap_or(DEFAULT_ASCENT) / PROBE_SIZE * em;
    let fall = faces.descents.get(index).copied().unwrap_or(0.0) / PROBE_SIZE * em;
    let drop = if beat.style == Beat::Slide {
        props.slide_drop * em
    } else {
        0.0
    };
    let dy = line_box / 2.0 - baseline + (rise - fall) / 2.0 + drop;
    let push = if beat.style == Beat::Punch {
        props.push_rate.mul_add(t.max(0.0), props.push_from)
    } else {
        1.0
    };
    let rush_t = (frame - (beat.end - props.rush_lead)) / fps;
    let rush = if beat.style == Beat::Slide && rush_t > 0.0 {
        1.0 / props.rush_travel.mul_add(
            -rush_ease((rush_t / (props.rush_frames / fps)).min(1.0)),
            1.0,
        )
    } else {
        1.0
    };
    let mut word_count = 0;
    let mut rows = Vec::new();
    for (li, words) in beat.lines.iter().enumerate() {
        let lt = t - float(li) * props.line_stagger / fps;
        let slide = |u: f64| {
            if beat.style != Beat::Slide {
                0.0
            } else if u <= 0.0 {
                props.slide_from * em
            } else {
                (props.slide_kick * em).mul_add(u, props.slide_from * em)
                    * (-props.slide_settle * u).exp()
            }
        };
        let x = slide(lt);
        let sigma = props.motion_blur * (x - slide(lt - 1.0 / fps)).abs() / 12.0_f64.sqrt();
        let blur_id = format!("punch-lines-{index}-{li}");
        let mut row_style = Style::default()
            .white_space(WhiteSpace::Nowrap)
            .font_size(em)
            .line_height(LineHeight::Length(Length::px(line_box)))
            .letter_spacing(spacing(&props.letter_spacing))
            .word_spacing(spacing(&props.word_spacing))
            .transform(vec![Transform::Translate(
                Length::px(x),
                Length::ZERO,
                Length::ZERO,
            )]);
        let mut row: Vec<Node> = Vec::new();
        if sigma > MIN_SIGMA {
            row_style = row_style.filter(vec![Filter::Url(blur_id.clone())]);
            let filter = SvgElement::new("svg")
                .attr("width", 0)
                .attr("height", 0)
                .child(
                    SvgElement::new("filter")
                        .attr("id", blur_id.as_str())
                        .attr("x", "-25%")
                        .attr("y", "-25%")
                        .attr("width", "150%")
                        .attr("height", "150%")
                        .attr("color-interpolation-filters", "sRGB")
                        .child(
                            SvgElement::new("feGaussianBlur")
                                .attr("stdDeviation", format!("{} 0", js_number_text(sigma))),
                        ),
                );
            row.push(svg(filter, Style::default().position(Position::Absolute)).into());
        }
        for (wi, word) in words.iter().enumerate() {
            let wt = t - float(word_count) * props.word_stagger / fps;
            word_count += 1;
            let scale = if beat.style == Beat::Punch {
                let p = punch_ease((wt / (props.punch_frames / fps)).clamp(0.0, 1.0));
                (1.0 - props.punch_from).mul_add(p, props.punch_from)
            } else {
                1.0
            };
            if wi > 0 {
                row.push(Node::from(" "));
            }
            row.push(
                span(
                    Style::default()
                        .display(Display::InlineBlock)
                        .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))])
                        .transform_origin((
                            Length::percent(50.0),
                            Length::px(line_box / 2.0),
                            Length::ZERO,
                        )),
                )
                .child(word.as_str())
                .into(),
            );
        }
        rows.push(div(row_style).children(row).into());
    }
    let zoom = push * rush;
    Some(
        div(absolute_fill()
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .color(color)
            .transform(vec![Transform::Scale(Num(zoom), Num(zoom), Num(1.0))])
            .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO)))
        .child(
            div(Style::default()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(Align::Center)
                .transform(vec![Transform::Translate(
                    Length::ZERO,
                    Length::px(dy),
                    Length::ZERO,
                )]))
            .children(rows),
        )
        .into(),
    )
}

pub fn draw(props: &PunchLines, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let theme = warm_theme(&props.theme);
    let face_name = if props.font_family.is_empty() {
        theme.font_family().unwrap_or("")
    } else {
        &props.font_family
    };
    let face = font_family(&resolve_font(face_name).unwrap_or_else(|| SANS.to_owned()));
    let paper = if props.ground.is_empty() {
        theme.get("background")
    } else {
        &props.ground
    };
    let ink = if props.ink.is_empty() {
        theme.get("foreground")
    } else {
        &props.ink
    };
    let beats = plan(props);
    let faces = measure_faces(props, &beats, &face, cx);
    let mut cards = Vec::new();
    let mut first_line = 0;
    for (i, beat) in beats.iter().enumerate() {
        let lines = beat.lines.len();
        let widths = faces
            .widths
            .get(first_line..first_line + lines)
            .unwrap_or_default();
        first_line += lines;
        let color = cx.color(if beat.accent {
            &props.accent_color
        } else {
            ink
        });
        cards.extend(card(props, beat, i, widths, &faces, (frame, color), cx));
    }
    Ok(div(absolute_fill()
        .background_color(cx.color(paper))
        .font_family(face)
        .font_weight(props.font_weight.value())
        .text_rendering(TextRendering::GeometricPrecision))
    .children(cards)
    .into())
}
