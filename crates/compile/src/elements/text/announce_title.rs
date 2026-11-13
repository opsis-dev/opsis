use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{ColorStop, Gradient, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, Display, FlexDirection, LineHeight, Overflow, Position, Style, TextRendering,
    TransformStyle, Visibility, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float, truncate, whole};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AnnounceTitle {
    #[setting("Eyebrow", default = "Introducing")]
    pub eyebrow: String,
    #[setting("Title", default = "Acme 2.0")]
    pub title: String,
    #[setting(
        "Tagline",
        default = "Everything your team needs to ship, in one place."
    )]
    pub tagline: String,
    #[setting("Symbol path", advanced)]
    pub symbol_path: String,
    #[setting(
        "Symbol colours",
        length(min = 0, max = 8),
        help = "Leave empty for the built-in colours.",
        advanced
    )]
    pub symbol_colors: Vec<Colour>,
    #[setting(
        "Symbol size",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.05,
        advanced
    )]
    pub symbol_scale: f64,
    #[setting("Colour field", default = "#5600f5", check = colour)]
    pub field_color: String,
    #[setting("Title ink", default = "#4800c9", check = colour)]
    pub title_color: String,
    #[setting("Title paper", default = "#fcfcfa", check = colour)]
    pub paper_color: String,
    #[setting("Opening ground", default = "#100022", check = colour)]
    pub void_color: String,
    #[setting("Closing ground", default = "#000028", check = colour)]
    pub night_color: String,
    #[setting("Closing glow", default = "#08ff4b", check = colour)]
    pub glow_color: String,
    #[setting(
        "Glow strength",
        default = 0.139,
        range(min = 0, max = 0.4),
        step = 0.005,
        advanced
    )]
    pub glow_strength: f64,
    #[setting("Tagline ink", default = "#f2f8ff", check = colour)]
    pub tagline_color: String,
    #[setting("Shot ink", default = "#ffffff", check = colour)]
    pub ink_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Font weight",
        default = 400,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting("Text size", default = 1.0, range(min = 0.5, max = 2), advanced)]
    pub text_scale: f64,
    #[setting("Symbol glow", default = 1.0, range(min = 0, max = 3), advanced)]
    pub symbol_glow: f64,
    #[setting(
        "Tagline tint",
        default = vec!["#8a2447".into(), "#862547".into(), "#7b2a4f".into(), "#633864".into(), "#4f467d".into(), "#3c519f".into(), "#2f5ed2".into(), "#3266e5".into()],
        length(min = 1, max = 12),
        help = "The colours the tagline shows before it settles, from left to right.",
        advanced
    )]
    pub tagline_tint: Vec<Colour>,
}

pub fn definition() -> ElementMeta {
    meta::<AnnounceTitle>(
            element(
                ElementKind::Component,
                "announce-title",
                "Announce title",
                "Launch title animation, product announcement intro with kinetic typography. A four-shot launch title: the eyebrow rushes past the camera on a receding type plane and lands on a colour field, the name assembles word by word on paper, and a macro pan across the tagline cuts wide as the line builds itself last word first while a white sweep drains it out of colour.",
                ElementCategory::Text,
                170,
            )
                .authored_clock()
                .fonts(&["Geist", "Google Sans", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:megaphone")),
        )
}

const PLANE_OUT: f64 = 18.0;
const FIELD_OUT: f64 = 41.0;
const PAPER_OUT: f64 = 80.0;
const MACRO_OUT: f64 = 110.0;
const REF_W: f64 = 802.0;
const REF_H: f64 = 450.0;
const EYEBROW_SIZE: f64 = 41.6 / REF_W;
const EYEBROW_TRACK: f64 = -0.0666;
const TITLE_SIZE: f64 = 41.3 / REF_W;
const TITLE_TRACK: f64 = -0.0563;
const TAGLINE_SIZE: f64 = 35.0 / REF_W;
const TAGLINE_TRACK: f64 = -0.015;
const BASELINE_DROP: f64 = 0.335;
const EYEBROW_BASELINE: f64 = 236.87 / REF_H;
const TITLE_BASELINE: f64 = 240.4 / REF_H;
const TAGLINE_BASELINE: f64 = 233.5 / REF_H;
const SPARK_RAMP: [(f64, &str); 10] = [
    (0.0, "#0365f3"),
    (0.17, "#1c5cf1"),
    (0.31, "#3a54ec"),
    (0.41, "#554ce5"),
    (0.52, "#7041cb"),
    (0.62, "#9b3894"),
    (0.72, "#ca3068"),
    (0.83, "#e2284c"),
    (0.93, "#e71d32"),
    (1.0, "#e91927"),
];
const BLOOM_PASSES: [f64; 4] = [0.0825, 0.0315, 0.009, 0.009];
const PLANE_MAG: [(f64, f64); 16] = [
    (2.0, 52.6),
    (3.0, 43.7),
    (4.0, 36.3),
    (5.0, 30.2),
    (6.0, 25.1),
    (7.0, 19.8),
    (8.0, 16.1),
    (9.0, 13.6),
    (10.0, 11.75),
    (11.0, 10.4),
    (12.0, 9.3),
    (13.0, 8.45),
    (14.0, 7.5),
    (15.0, 6.35),
    (16.0, 5.2),
    (17.0, 4.0),
];
const PLANE_BASE: f64 = 6.0;
const PLANE_PERSPECTIVE: f64 = 0.36;
const PLANE_RAKE: [(f64, f64); 7] = [
    (5.0, 81.0),
    (7.0, 68.0),
    (9.0, 55.0),
    (11.0, 41.0),
    (13.0, 31.0),
    (15.0, 20.0),
    (17.0, 14.0),
];
const EYEBROW_ENTER: f64 = 0.53;
const EYEBROW_DECAY: f64 = 0.7605;
const EYEBROW_SPAN: f64 = 18.0;
const PIVOT_LIFT: f64 = 23.87 / 41.6;
const EXIT_FROM: f64 = 35.0;
const EXIT_BASE: f64 = 3.2;
const EXIT_STEP: f64 = 2.1 / REF_W;
const EXIT_LETTER_DELAY: f64 = 0.215;
const TITLE_DECAY: f64 = 0.82;
const TITLE_SPAN: f64 = 30.0;
const TITLE_ALPHA_DECAY: f64 = 0.79;
const TITLE_ENTRY: [(f64, f64, f64); 3] =
    [(41.0, 143.0, 0.39), (48.0, 87.0, 0.46), (54.0, 88.0, 0.29)];
const MACRO_SIZE: f64 = 487.0 / REF_W;
const MACRO_BASELINE: f64 = 347.0 / REF_H;
const MACRO_HOLD: f64 = 90.0;
const MACRO_ANCHOR: f64 = 0.505;
const MACRO_STAR_AT_HOLD: f64 = 565.0 / REF_W;
const MACRO_STAR_SIZE: f64 = 1.255;
const MACRO_STAR_SPIN: f64 = 2.9;
const MACRO_STAR_Y: f64 = 60.0 / REF_H;
const MACRO_STAR_SPREAD: f64 = 0.55;
const TAGLINE_TRAVEL: f64 = 417.0 / REF_W;
const TAGLINE_DECAY: f64 = 0.792;
const TAGLINE_SPAN: f64 = 30.0;
const TAGLINE_FIRST: f64 = 111.5;
const TAGLINE_STAGGER: f64 = 2.833;
const TAGLINE_INK: f64 = 693.0;
const WIPE_START: f64 = 124.6;
const WIPE_SPEED: f64 = 31.7 / TAGLINE_INK;
const WIPE_EDGE: f64 = 165.0 / TAGLINE_INK;
const STAR_REST_X: f64 = 781.4 / REF_W;
const STAR_REST_Y: f64 = 227.0 / REF_H;
const STAR_SIZE: f64 = 60.0 / REF_W;
const STAR_TURN: f64 = 30.0;
const STAR_EXIT: f64 = 134.0;
const STAR_GONE: f64 = 139.0;
const FILL_STOPS: u32 = 14;

fn settle(t: f64, span: f64, decay: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= span {
        return 1.0;
    }
    (1.0 - decay.powf(t)) / (1.0 - decay.powf(span))
}

fn sample(table: &[(f64, f64)], p: f64) -> f64 {
    let (Some(first), Some(last)) = (table.first(), table.last()) else {
        return 0.0;
    };
    if p <= first.0 {
        return first.1;
    }
    if p >= last.0 {
        return last.1;
    }
    for pair in table.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        if p <= x1 {
            return y0 + (y1 - y0) * (p - x0) / (x1 - x0);
        }
    }
    last.1
}

#[derive(Clone)]
struct Scene<'a> {
    face: &'a [String],
    width: f64,
    height: f64,
    cx: &'a Cx<'a>,
}

fn line_style(font_size: f64, tracking: f64, face: &[String]) -> Style {
    Style::default()
        .font_family(face.to_vec())
        .font_size(font_size)
        .line_height(LineHeight::Number(Num(1.0)))
        .letter_spacing(Length::em(tracking))
        .margin_right(Length::em(-tracking))
        .white_space(WhiteSpace::Pre)
        .text_rendering(TextRendering::GeometricPrecision)
}

fn stage(offset: f64, align: Align, child: Node) -> Node {
    div(absolute_fill()
        .align_items(align)
        .justify_content(Align::Center))
    .child(
        div(Style::default().transform(vec![Transform::Translate(
            Length::ZERO,
            Length::px(offset),
            Length::ZERO,
        )]))
        .child(child),
    )
    .into()
}

fn line(
    scene: &Scene<'_>,
    font_size: f64,
    tracking: f64,
    baseline: f64,
    style: &Style,
    children: Vec<Node>,
) -> Node {
    stage(
        BASELINE_DROP.mul_add(-font_size, (baseline - 0.5) * scene.height),
        Align::Center,
        div(line_style(font_size, tracking, scene.face).merged(style))
            .children(children)
            .into(),
    )
}

fn shutter(frame: f64, samples: f64, open: f64, draw: &dyn Fn(f64) -> Node) -> Vec<Node> {
    let n: u32 = truncate(samples.round().max(2.0));
    (0..n)
        .map(|i| {
            let i = f64::from(i);
            div(absolute_fill().opacity(1.0 / (i + 1.0)))
                .child(draw(frame - open + open * i / f64::from(n - 1)))
                .into()
        })
        .collect()
}

fn symbol_stops(colors: &[Colour]) -> Vec<(f64, String)> {
    match colors {
        [] => SPARK_RAMP
            .iter()
            .map(|(at, c)| (*at, (*c).to_owned()))
            .collect(),
        [only] => vec![(0.0, only.0.clone()), (1.0, only.0.clone())],
        many => many
            .iter()
            .enumerate()
            .map(|(i, c)| (float(i) / float(many.len() - 1), c.0.clone()))
            .collect(),
    }
}

struct MarkLook<'a> {
    path: &'a str,
    colors: &'a [Colour],
    size: f64,
    width: f64,
    rotation: f64,
    spread: f64,
    glow: f64,
    id: &'a str,
}

fn mark(look: &MarkLook<'_>) -> Option<Node> {
    if look.path.is_empty() {
        return None;
    }
    let edge = 0.5 + look.spread / 2.0;
    let gradient = symbol_stops(look.colors).into_iter().fold(
        SvgElement::new("linearGradient")
            .attr("id", look.id)
            .attr("x1", "0.5")
            .attr("y1", edge)
            .attr("x2", "0.5")
            .attr("y2", 1.0 - edge),
        |g, (at, color)| {
            g.child(
                SvgElement::new("stop")
                    .attr("offset", format!("{}%", js_number_text(at * 100.0)))
                    .attr("stop-color", color),
            )
        },
    );
    let mut defs = SvgElement::new("defs").child(gradient);
    let mut seen: Vec<f64> = Vec::new();
    for r in BLOOM_PASSES {
        if seen.iter().any(|s| exact_eq(*s, r)) {
            continue;
        }
        seen.push(r);
        defs = defs.child(
            SvgElement::new("filter")
                .attr("id", format!("{}-{}", look.id, js_number_text(r)))
                .attr("x", "-300%")
                .attr("y", "-300%")
                .attr("width", "700%")
                .attr("height", "700%")
                .child(SvgElement::new("feGaussianBlur").attr(
                    "stdDeviation",
                    r * look.glow * look.width * 100.0 / look.size,
                )),
        );
    }
    let fill = format!("url(#{})", look.id);
    let mut root = SvgElement::new("svg")
        .attr("viewBox", "0 0 100 100")
        .attr("width", look.size)
        .attr("height", look.size)
        .child(defs);
    for r in BLOOM_PASSES {
        root = root.child(
            SvgElement::new("path")
                .attr("d", look.path)
                .attr("fill", fill.as_str())
                .attr("filter", format!("url(#{}-{})", look.id, js_number_text(r)))
                .attr("style", "mix-blend-mode:screen"),
        );
    }
    root = root.child(
        SvgElement::new("path")
            .attr("d", look.path)
            .attr("fill", fill.as_str()),
    );
    Some(
        svg(
            root,
            Style::default()
                .display(Display::Block)
                .overflow(Overflow::Visible)
                .transform(vec![Transform::Rotate(Angle::deg(look.rotation))]),
        )
        .into(),
    )
}

fn receding_plane(scene: &Scene<'_>, text: &str, frame: f64, font_size: f64, ink: &str) -> Node {
    let mag = sample(&PLANE_MAG, frame);
    let rake = sample(&PLANE_RAKE, frame);
    let scale = mag / PLANE_BASE;
    stage(
        BASELINE_DROP.mul_add(-font_size, (EYEBROW_BASELINE - 0.5) * scene.height),
        Align::Center,
        div(Style::default().transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]))
            .child(
                div(Style::default()
                    .perspective(PLANE_PERSPECTIVE * scene.width)
                    .transform_style(TransformStyle::Preserve3d))
                .child(
                    div(
                        line_style(font_size * PLANE_BASE, EYEBROW_TRACK, scene.face)
                            .color(scene.cx.color(ink))
                            .transform(vec![Transform::RotateX(Angle::deg(rake))]),
                    )
                    .child(text),
                ),
            )
            .into(),
    )
}

fn eyebrow_exit(frame: f64, letter: usize, width: f64) -> f64 {
    let t = float(letter).mul_add(-EXIT_LETTER_DELAY, frame - EXIT_FROM);
    if t < 0.0 {
        return 0.0;
    }
    -EXIT_STEP * width * EXIT_BASE.powf(t)
}

fn eyebrow_line(scene: &Scene<'_>, text: &str, frame: f64, font_size: f64, ink: &str) -> Node {
    let scale = EYEBROW_ENTER.mul_add(
        1.0 - settle(frame - PLANE_OUT, EYEBROW_SPAN, EYEBROW_DECAY),
        1.0,
    );
    let letters = text
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            span(Style::default().transform(vec![Transform::Translate(
                Length::px(eyebrow_exit(frame, i, scene.width)),
                Length::ZERO,
                Length::ZERO,
            )]))
            .child(ch.to_string())
            .into()
        })
        .collect();
    line(
        scene,
        font_size,
        EYEBROW_TRACK,
        EYEBROW_BASELINE,
        &Style::default()
            .color(scene.cx.color(ink))
            .display(Display::Flex)
            .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO))
            .transform(vec![
                Transform::Translate(
                    Length::ZERO,
                    Length::px(PIVOT_LIFT * font_size * (scale - 1.0)),
                    Length::ZERO,
                ),
                Transform::Scale(Num(scale), Num(scale), Num(1.0)),
            ]),
        letters,
    )
}

fn title_line(scene: &Scene<'_>, words: &[&str], frame: f64, font_size: f64, color: &str) -> Node {
    let last = TITLE_ENTRY.len() - 1;
    let spans = words
        .iter()
        .enumerate()
        .filter_map(|(i, word)| {
            let extra = i.saturating_sub(last);
            let (at, travel, pop) = TITLE_ENTRY[i - extra];
            let t = float(extra).mul_add(-6.0, frame - at);
            if t < 0.0 {
                return None;
            }
            let x = travel * scene.width / REF_W * (1.0 - settle(t, TITLE_SPAN, TITLE_DECAY));
            Some(
                span(
                    Style::default()
                        .opacity(f64::mul_add(1.0 - pop, -TITLE_ALPHA_DECAY.powf(t), 1.0))
                        .transform(vec![Transform::Translate(
                            Length::px(x),
                            Length::ZERO,
                            Length::ZERO,
                        )]),
                )
                .child(if i == 0 {
                    (*word).to_owned()
                } else {
                    format!(" {word}")
                })
                .into(),
            )
        })
        .collect();
    line(
        scene,
        font_size,
        TITLE_TRACK,
        TITLE_BASELINE,
        &Style::default()
            .display(Display::Flex)
            .color(scene.cx.color(color)),
        spans,
    )
}

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let digits: String = hex
        .chars()
        .skip(1)
        .take_while(char::is_ascii_hexdigit)
        .collect();
    let n = u32::from_str_radix(&digits, 16).unwrap_or(0);
    [(n >> 16) & 255, (n >> 8) & 255, n & 255].map(|c| u8::try_from(c).unwrap_or(0))
}

fn macro_pan(frame: f64) -> f64 {
    let t = frame - MACRO_HOLD;
    -(78.8_f64.mul_add(1.0 - 0.81_f64.powf(t), 6.03 * (1.255_f64.powf(t) - 1.0)))
}

fn macro_shot(scene: &Scene<'_>, props: &AnnounceTitle, frame: f64) -> Vec<Node> {
    let pan = macro_pan(frame) * scene.width / REF_W;
    let font_size = MACRO_SIZE * props.text_scale * scene.width;
    let tagline = stage(
        BASELINE_DROP.mul_add(-font_size, (MACRO_BASELINE - 0.5) * scene.height),
        Align::FlexStart,
        div(line_style(font_size, TAGLINE_TRACK, scene.face)
            .color(scene.cx.color(&props.ink_color))
            .transform(vec![
                Transform::Translate(
                    Length::px(scene.width / 2.0 + pan),
                    Length::ZERO,
                    Length::ZERO,
                ),
                Transform::Translate(
                    Length::percent(-MACRO_ANCHOR * 100.0),
                    Length::ZERO,
                    Length::ZERO,
                ),
            ]))
        .child(props.tagline.as_str())
        .into(),
    );
    let star = mark(&MarkLook {
        path: &props.symbol_path,
        colors: &props.symbol_colors,
        size: MACRO_STAR_SIZE * props.symbol_scale * scene.width,
        width: scene.width,
        rotation: MACRO_STAR_SPIN.mul_add(frame - MACRO_HOLD, -50.0),
        spread: MACRO_STAR_SPREAD,
        glow: props.symbol_glow,
        id: "announce-title-macro-symbol",
    });
    vec![
        tagline,
        div(absolute_fill()
            .align_items(Align::Center)
            .justify_content(Align::Center))
        .child(
            div(Style::default().transform(vec![Transform::Translate(
                Length::px((MACRO_STAR_AT_HOLD - 0.5).mul_add(scene.width, -pan)),
                Length::px(MACRO_STAR_Y * scene.height),
                Length::ZERO,
            )]))
            .children(star),
        )
        .into(),
    ]
}

fn tagline_fill(front: f64, rest: &str, tint: &[Colour]) -> Gradient {
    let ramp = symbol_stops(tint);
    let rest = hex_to_rgb(rest);
    let mut stops = Vec::new();
    for i in 0..=FILL_STOPS {
        let p = f64::from(i) / f64::from(FILL_STOPS);
        let (mut below, mut above) = (ramp[0].clone(), ramp[ramp.len() - 1].clone());
        for j in 1..ramp.len() {
            if p <= ramp[j].0 {
                below = ramp[j - 1].clone();
                above = ramp[j].clone();
                break;
            }
        }
        let span = above.0 - below.0;
        let local = if span > 0.0 {
            (p - below.0) / span
        } else {
            0.0
        };
        let (from, to) = (hex_to_rgb(&below.1), hex_to_rgb(&above.1));
        let drained = ((front - p) / WIPE_EDGE + 0.5).clamp(0.0, 1.0);
        let mixed: [u8; 3] = std::array::from_fn(|k| {
            let (a, b) = (f64::from(from[k]), f64::from(to[k]));
            let value = f64::mul_add(
                f64::from(rest[k]),
                drained,
                f64::mul_add(b - a, local, a) * (1.0 - drained),
            );
            u8::try_from(whole(value)).unwrap_or(u8::MAX)
        });
        stops.push(ColorStop::at(
            Color::rgb8(mixed[0], mixed[1], mixed[2]),
            Length::percent((p * 100.0 * 10.0).round() / 10.0),
        ));
    }
    Gradient::linear(Angle::deg(90.0), stops)
}

fn closing_shot(
    scene: &Scene<'_>,
    props: &AnnounceTitle,
    words: &[&str],
    frame: f64,
    font_size: f64,
) -> Vec<Node> {
    let eased = 1.0 - settle(frame - MACRO_OUT, TAGLINE_SPAN, TAGLINE_DECAY);
    let slide = -TAGLINE_TRAVEL * scene.width * eased;
    let front = WIPE_SPEED * (frame - WIPE_START);
    let star_life = 1.0 - ((frame - STAR_EXIT) / (STAR_GONE - STAR_EXIT)).clamp(0.0, 1.0);
    let last = words.len().saturating_sub(1);
    let spans = words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let shown = frame >= float(last - i).mul_add(TAGLINE_STAGGER, TAGLINE_FIRST);
            span(Style::default().visibility(if shown {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }))
            .child(if i == 0 {
                (*word).to_owned()
            } else {
                format!(" {word}")
            })
            .into()
        })
        .collect();
    let mut out = vec![line(
        scene,
        font_size,
        TAGLINE_TRACK,
        TAGLINE_BASELINE,
        &Style::default()
            .display(Display::Flex)
            .transform(vec![Transform::Translate(
                Length::px(slide),
                Length::ZERO,
                Length::ZERO,
            )])
            .background(vec![
                tagline_fill(front, &props.tagline_color, &props.tagline_tint).layer(),
            ])
            .background_clip_text(true)
            .color(Color::TRANSPARENT),
        spans,
    )];
    if star_life > 0.0 {
        let star = mark(&MarkLook {
            path: &props.symbol_path,
            colors: &props.symbol_colors,
            size: STAR_SIZE * props.symbol_scale * scene.width,
            width: scene.width,
            rotation: -STAR_TURN * eased,
            spread: 1.0,
            glow: props.symbol_glow,
            id: "announce-title-symbol",
        });
        out.push(stage(
            (STAR_REST_Y - 0.5) * scene.height,
            Align::Center,
            div(Style::default()
                .transform(vec![
                    Transform::Translate(
                        Length::px((STAR_REST_X - 0.5).mul_add(scene.width, slide)),
                        Length::ZERO,
                        Length::ZERO,
                    ),
                    Transform::Scale(Num(star_life), Num(star_life), Num(1.0)),
                ])
                .opacity(star_life))
            .children(star)
            .into(),
        ));
    }
    out
}

fn fit_scales(scene: &Scene<'_>, weight: f64, lines: &[(&str, f64, f64, f64)]) -> Vec<f64> {
    let probes = lines.iter().map(|(text, size, tracking, _)| {
        span(line_style(*size, *tracking, scene.face))
            .child(*text)
            .into()
    });
    let root: Node = div(absolute_fill().font_weight(weight))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .top(Length::ZERO)
                .left(Length::ZERO)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(Align::FlexStart))
            .children(probes),
        )
        .into();
    let boxes = scene.cx.measure.boxes(&root);
    lines
        .iter()
        .enumerate()
        .map(|(i, (_, _, _, max))| {
            let width = boxes
                .iter()
                .find(|b| b.path == [0, i])
                .map_or(0.0, |b| b.width);
            if width > *max { max / width } else { 1.0 }
        })
        .collect()
}

pub fn draw(props: &AnnounceTitle, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let face = font_or(&props.font_family, FontRole::Main);
    let frame = cx.authored_frame() * props.speed;
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let scene = Scene {
        face: &face,
        width,
        height,
        cx,
    };
    let title_words: Vec<&str> = props.title.split_whitespace().collect();
    let tagline_words: Vec<&str> = props.tagline.split_whitespace().collect();
    let weight = f64::from(props.font_weight);
    let scale = props.text_scale * width;
    let scales = fit_scales(
        &scene,
        weight,
        &[
            (
                &props.eyebrow,
                EYEBROW_SIZE * scale,
                EYEBROW_TRACK,
                0.55 * width,
            ),
            (&props.title, TITLE_SIZE * scale, TITLE_TRACK, 0.8 * width),
            (
                &props.tagline,
                TAGLINE_SIZE * scale,
                TAGLINE_TRACK,
                0.88 * width,
            ),
        ],
    );
    let eyebrow_size = EYEBROW_SIZE * scale * scales[0];
    let title_size = TITLE_SIZE * scale * scales[1];
    let tagline_size = TAGLINE_SIZE * scale * scales[2];
    let shot: Node = if frame < PLANE_OUT {
        div(absolute_fill().background_color(cx.color(&props.void_color)))
            .children(shutter(frame, 20.0, 1.0, &|f| {
                receding_plane(&scene, &props.eyebrow, f, eyebrow_size, &props.ink_color)
            }))
            .into()
    } else if frame < FIELD_OUT {
        let travel = eyebrow_exit(frame, 0, width).abs();
        div(absolute_fill().background_color(cx.color(&props.field_color)))
            .children(shutter(
                frame,
                (travel / 4.0).clamp(6.0, 44.0),
                if travel > 0.0 { 1.0 } else { 0.5 },
                &|f| eyebrow_line(&scene, &props.eyebrow, f, eyebrow_size, &props.ink_color),
            ))
            .into()
    } else if frame < PAPER_OUT {
        div(absolute_fill().background_color(cx.color(&props.paper_color)))
            .child(title_line(
                &scene,
                &title_words,
                frame,
                title_size,
                &props.title_color,
            ))
            .into()
    } else {
        let content = if frame < MACRO_OUT {
            macro_shot(&scene, props, frame)
        } else {
            closing_shot(&scene, props, &tagline_words, frame, tagline_size)
        };
        div(absolute_fill().background_color(cx.color(&props.night_color)))
            .children(content)
            .into()
    };
    Ok(div(absolute_fill().font_weight(weight)).child(shot).into())
}
