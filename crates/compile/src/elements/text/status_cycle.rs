use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::css::font_family;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Transform;
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, LineHeight, Overflow, Position, Style,
    TextRendering, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::anim::{clamped, sprung};
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::oklch_mix::mix_oklch;
use crate::elements::shared::{Colour, FontWeight};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct StatusCycle {
    #[setting("Prefix", default = "Acme is")]
    pub prefix: String,
    #[setting(
        "Statuses (comma-separated)",
        default = "recording, editing, exporting, live"
    )]
    pub statuses: String,
    #[setting(
        "Chips (comma-separated)",
        default = "Record, Auto zoom, Captions, Export, Share"
    )]
    pub chips: String,
    #[setting("Field (act 1)", widget = "color", zero_label = "Theme", advanced)]
    pub field_color: String,
    #[setting("Page (act 2)", widget = "color", zero_label = "Theme", advanced)]
    pub page_color: String,
    #[setting("Pill", widget = "color", zero_label = "Theme", advanced)]
    pub pill_color: String,
    #[setting("Pill label", widget = "color", zero_label = "Default", advanced)]
    pub pill_label_color: String,
    #[setting("Prefix ink", widget = "color", zero_label = "Theme", advanced)]
    pub prefix_color: String,
    #[setting(
        "Chip fills",
        length(min = 0, max = 8),
        help = "Leave empty for the theme's chip colours.",
        advanced
    )]
    pub chip_fills: Vec<Colour>,
    #[setting("Chip ink", widget = "color", zero_label = "Theme", advanced)]
    pub chip_text_color: String,
    #[setting(
        "Chip hairline",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        advanced
    )]
    pub chip_border_color: String,
    #[setting("Font weight", default = FontWeight::W400)]
    pub font_weight: FontWeight,
    #[setting("Chip font weight", default = FontWeight::W400, advanced)]
    pub chip_font_weight: FontWeight,
    #[setting("Status size", default = 63, range(min = 24, max = 140), step = 1)]
    pub font_size: u32,
    #[setting(
        "Chip size",
        default = 160,
        range(min = 48, max = 260),
        step = 2,
        advanced
    )]
    pub chip_font_size: u32,
    #[setting(
        "Intro length",
        default = 0.8,
        range(min = 0.2667, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub intro_frames: f64,
    #[setting(
        "Status hold",
        default = 1.0,
        range(min = 0.3333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub status_hold: f64,
    #[setting(
        "Chip stagger",
        default = 0.2667,
        range(min = 0.1333, max = 1),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub chip_stagger: f64,
    #[setting(
        "Lockup height",
        default = 0.516,
        range(min = 0.2, max = 0.8),
        step = 0.004,
        advanced
    )]
    pub pill_center_y: f64,
    #[setting(
        "Start at",
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub start_at: f64,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Status corner radius",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub pill_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Chip corner radius",
        range(min = 0, max = 200),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub chip_radius: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Chip border width",
        range(min = 0, max = 20),
        zero_label = "Auto",
        step = 0.5,
        unit = "px",
        advanced,
        "type" = "number"
    )]
    pub chip_border_width: Option<f64>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Pill height",
        default = 1.128,
        range(min = 0.9, max = 2),
        unit = "× text size",
        advanced
    )]
    pub pill_height: f64,
    #[setting(
        "Pill padding",
        default = 0.19,
        range(min = 0, max = 0.8),
        unit = "× pill height",
        advanced
    )]
    pub pill_padding: f64,
    #[setting(
        "Gap before pill",
        default = 0.147,
        range(min = 0, max = 0.8),
        unit = "× pill height",
        advanced
    )]
    pub pill_gap: f64,
    #[setting(
        "Chip padding",
        default = 0.42,
        range(min = 0, max = 1.5),
        unit = "× chip size",
        advanced
    )]
    pub chip_padding: f64,
    #[setting(
        "Chip side margin",
        default = 0.04,
        range(min = 0, max = 0.25),
        unit = "× width",
        advanced
    )]
    pub chip_inset: f64,
    #[setting(
        "Chip spacing",
        default = 0.057,
        range(min = 0, max = 0.2),
        unit = "× height",
        advanced
    )]
    pub chip_gap: f64,
    #[setting(
        "Pill resize time",
        default = 0.6,
        range(min = 0.1, max = 2),
        unit = "s",
        advanced
    )]
    pub pill_resize_time: f64,
    #[setting(
        "Label roll time",
        default = 0.5,
        range(min = 0.1, max = 2),
        unit = "s",
        advanced
    )]
    pub roll_time: f64,
    #[setting(
        "Field fade time",
        default = 0.3,
        range(min = 0, max = 2),
        unit = "s",
        advanced
    )]
    pub field_fade_time: f64,
    #[setting(
        "Letter stagger",
        default = 0.0333,
        range(min = 0, max = 0.2),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub letter_stagger: f64,
    #[setting(
        "Intro zoom",
        default = 1.16,
        range(min = 1, max = 1.6),
        unit = "×",
        advanced
    )]
    pub intro_zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<StatusCycle>(
            element(
                ElementKind::Component,
                "status-cycle",
                "Status cycle",
                "Status badge animation: a pill whose label cycles through states. A centred line whose status pill cycles: the label rolls up behind a hard clip while the pill's width springs past its target and back, and the line re-centres around it.",
                ElementCategory::Text,
                198,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:badge")),
        )
}

const SANS: &str = "\"Inter\"";
const PILL_R: f64 = 0.229;
const CHIP_PAD_Y: f64 = 0.165;
const CHIP_R: f64 = 0.162;
const CHIP_BORDER: f64 = 0.017;
const ACT1_FIT: f64 = 0.92;
const ROLL_TRAVEL: f64 = 1.02;
const EXIT_SECONDS: f64 = 0.2;
const SCROLL_TAU: f64 = 0.06;
const NBSP: &str = "\u{a0}";
const LETTER_SPRING: SpringConfig = SpringConfig {
    damping: 5.0,
    stiffness: 100.0,
    mass: 1.0,
    overshoot_clamping: false,
};
const GROUP_SPRING: SpringConfig = SpringConfig {
    damping: 26.0,
    stiffness: 90.0,
    mass: 1.4,
    overshoot_clamping: false,
};

fn width_ease(t: f64) -> f64 {
    Easing::Bezier(0.65, 0.2, 0.3, 1.5).at(t)
}

fn roll_ease(t: f64) -> f64 {
    Easing::Bezier(0.45, 0.0, 0.55, 1.0).at(t)
}

fn to_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn settle(frames: f64, tau: f64) -> f64 {
    if frames <= 0.0 {
        return 0.0;
    }
    1.0 - (-frames / tau).exp()
}

fn status_at(frame: f64, intro: f64, hold: f64, count: usize) -> (usize, f64) {
    if count == 0 {
        return (0, 0.0);
    }
    let mut index = 0;
    for i in 1..count {
        if frame >= float(i - 1).mul_add(hold, intro) {
            index = i;
        }
    }
    let since = frame - (float(index) - 1.0).mul_add(hold, intro);
    (index, since)
}

fn chip_at(frame: f64, beat: f64, stagger: f64, count: usize) -> (usize, f64) {
    if count == 0 || frame < beat {
        return (0, 0.0);
    }
    let steps: usize = truncate(((frame - beat) / stagger).floor().max(0.0));
    let arrived = count.min(steps + 1);
    (
        arrived,
        (float(arrived) - 1.0).mul_add(-stagger, frame - beat),
    )
}

fn fit_scale(measured: f64, budget: f64) -> f64 {
    if measured.is_nan() || budget.is_nan() || measured <= 0.0 || budget <= 0.0 {
        return 1.0;
    }
    (budget / measured).min(1.0)
}

const fn or_theme<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

#[derive(Debug, Default)]
struct Metrics {
    letters: Vec<(f64, f64)>,
    prefix_width: f64,
    baseline: f64,
    status_widths: Vec<f64>,
    chip_widths: Vec<f64>,
}

fn type_style(face: &[String], weight: f64) -> Style {
    Style::default()
        .font_family(face.to_vec())
        .font_weight(weight)
        .text_rendering(TextRendering::GeometricPrecision)
        .white_space(WhiteSpace::Nowrap)
}

fn glyph_text(glyph: &str) -> &str {
    if glyph == " " { NBSP } else { glyph }
}

fn measure(
    cx: &Cx<'_>,
    glyphs: &[String],
    labels: &[String],
    chip_labels: &[String],
    (em, chip_em): (f64, f64),
    (typed, chip_typed): (&Style, &Style),
) -> Metrics {
    let letters: Vec<Node> = glyphs
        .iter()
        .map(|g| {
            span(Style::default().display(Display::InlineBlock))
                .child(glyph_text(g))
                .into()
        })
        .collect();
    let prefix = span(
        Style::default()
            .font_size(em)
            .line_height(LineHeight::Number(Num(2.0)))
            .merged(typed),
    )
    .children(letters)
    .child(span(
        Style::default()
            .display(Display::InlineBlock)
            .width(Length::ZERO)
            .height(Length::ZERO),
    ));
    let mut column = div(Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::FlexStart))
    .child(prefix);
    for text in labels {
        column = column.child(
            span(
                Style::default()
                    .display(Display::InlineBlock)
                    .font_size(em)
                    .merged(typed),
            )
            .child(text.as_str()),
        );
    }
    for text in chip_labels {
        column = column.child(
            span(
                Style::default()
                    .display(Display::InlineBlock)
                    .font_size(chip_em)
                    .merged(chip_typed),
            )
            .child(text.as_str()),
        );
    }
    let root: Node = div(absolute_fill()).child(column).into();
    let boxes = cx.measure.boxes(&root);
    let find = |path: &[usize]| boxes.iter().find(|b| b.path == path);
    let letters_end = glyphs.len();
    Metrics {
        letters: (0..glyphs.len())
            .map(|i| find(&[0, 0, i]).map_or((0.0, 0.0), |b| (b.x, b.width)))
            .collect(),
        prefix_width: find(&[0, 0]).map_or(0.0, |b| b.width),
        baseline: find(&[0, 0, letters_end]).map_or(0.0, |b| b.y),
        status_widths: (0..labels.len())
            .map(|i| find(&[0, 1 + i]).map_or(0.0, |b| b.width))
            .collect(),
        chip_widths: (0..chip_labels.len())
            .map(|i| find(&[0, 1 + labels.len() + i]).map_or(0.0, |b| b.width))
            .collect(),
    }
}

struct Look<'a> {
    em: f64,
    typed: &'a Style,
    cy: f64,
}

fn prefix_node(
    glyphs: &[String],
    metrics: &Metrics,
    (frame, fps): (f64, f64),
    (fit, stagger, x, color): (f64, f64, f64, Color),
    look: &Look<'_>,
) -> Result<Node, DrawFailure> {
    let mut spans = Vec::new();
    for (i, glyph) in glyphs.iter().enumerate() {
        let p = sprung(float(i).mul_add(-stagger, frame), fps, LETTER_SPRING)?;
        let scale = interpolate(p, &[0.0, 1.0], &[0.1, 1.0], &Interpolation::default())?;
        let left = metrics.letters.get(i).map_or(0.0, |l| l.0) * fit;
        spans.push(
            span(
                Style::default()
                    .position(Position::Absolute)
                    .left(left)
                    .top(Length::ZERO)
                    .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))])
                    .transform_origin((
                        Length::percent(50.0),
                        Length::px(metrics.baseline * fit),
                        Length::ZERO,
                    ))
                    .merged(look.typed),
            )
            .child(glyph_text(glyph))
            .into(),
        );
    }
    Ok(div(Style::default()
        .position(Position::Absolute)
        .left(x)
        .top(look.cy - look.em)
        .height(2.0 * look.em)
        .color(color)
        .font_size(look.em)
        .line_height(LineHeight::Number(Num(2.0)))
        .merged(look.typed))
    .children(spans)
    .into())
}

struct PillLook {
    x: f64,
    width: f64,
    height: f64,
    fill: Color,
    ink: Color,
    radius: f64,
    travel: f64,
}

fn pill_node(labels: &[String], index: usize, roll: f64, pill: &PillLook, look: &Look<'_>) -> Node {
    let label = |text: &str, offset: f64| -> Node {
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::percent(50.0))
            .top(Length::ZERO)
            .height(pill.height)
            .display(Display::Flex)
            .align_items(Align::Center)
            .transform(vec![Transform::Translate(
                Length::percent(-50.0),
                Length::px(offset),
                Length::ZERO,
            )])
            .color(pill.ink.clone())
            .font_size(look.em)
            .merged(look.typed))
        .child(text)
        .into()
    };
    let mut node = div(Style::default()
        .position(Position::Absolute)
        .left(pill.x)
        .top(look.cy - pill.height / 2.0)
        .width(pill.width)
        .height(pill.height)
        .border_radius_all(pill.radius)
        .background_color(pill.fill.clone())
        .overflow(Overflow::Hidden));
    if roll < 1.0 && index > 0 {
        node = node.child(label(
            labels.get(index - 1).map_or("", String::as_str),
            -roll * pill.travel,
        ));
    }
    node.child(label(
        labels.get(index).map_or("", String::as_str),
        (1.0 - roll) * pill.travel,
    ))
    .into()
}

#[expect(
    clippy::too_many_lines,
    reason = "the element follows the film's one view function in order"
)]
pub fn draw(props: &StatusCycle, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame().mul_add(props.speed, -props.start_at);
    let fps = Cx::authored_fps();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let face_name = if props.font_family.is_empty() {
        theme.font_family().unwrap_or("").to_owned()
    } else {
        props.font_family.clone()
    };
    let face = font_family(&resolve_font(&face_name).unwrap_or_else(|| SANS.to_owned()));
    let labels = to_list(&props.statuses);
    let chip_labels = to_list(&props.chips);
    let glyphs: Vec<String> = props.prefix.chars().map(String::from).collect();
    let field = or_theme(&props.field_color, theme.get("primary")).to_owned();
    let page = or_theme(&props.page_color, theme.get("background"));
    let pill_fill = or_theme(&props.pill_color, theme.get("foreground"));
    let prefix_ink = or_theme(&props.prefix_color, theme.get("foreground"));
    let label_ink = if props.pill_label_color.is_empty() {
        mix_oklch(&field, theme.get("card"), 0.62)
    } else {
        props.pill_label_color.clone()
    };
    let chip_ink = or_theme(&props.chip_text_color, theme.get("foreground"));
    let chip_edge = if props.chip_border_color.is_empty() {
        mix_oklch(theme.get("border"), theme.get("foreground"), 0.28)
    } else {
        props.chip_border_color.clone()
    };
    let fills: Vec<String> = if props.chip_fills.is_empty() {
        vec![
            theme.get("card").to_owned(),
            mix_oklch(theme.get("card"), &field, 0.14),
            field.clone(),
        ]
    } else {
        props.chip_fills.iter().map(|c| c.0.clone()).collect()
    };
    let nominal_em = f64::from(props.font_size);
    let nominal_chip_em = f64::from(props.chip_font_size);
    let typed = type_style(&face, props.font_weight.value());
    let chip_typed = type_style(&face, props.chip_font_weight.value());
    let metrics = measure(
        cx,
        &glyphs,
        &labels,
        &chip_labels,
        (nominal_em, nominal_chip_em),
        (&typed, &chip_typed),
    );
    let swap_at = |i: f64| (i - 1.0).mul_add(props.status_hold, props.intro_frames);
    let act1_end = swap_at(float(labels.len())) + props.status_hold;
    let fade_frames = props.field_fade_time * fps;
    let beat = act1_end + fade_frames / 2.0;
    let faded = clamped(frame, [act1_end, act1_end + fade_frames], [0.0, 1.0])?;
    let raw_pad = props.pill_padding * props.pill_height * nominal_em;
    let raw_gap = props.pill_gap * props.pill_height * nominal_em;
    let widest = metrics.status_widths.iter().copied().fold(0.0, f64::max);
    let act1_fit = fit_scale(
        2.0f64.mul_add(raw_pad, metrics.prefix_width + raw_gap + widest),
        ACT1_FIT * width,
    );
    let widest_chip = metrics.chip_widths.iter().copied().fold(0.0, f64::max);
    let chip_fit = fit_scale(
        (2.0 * props.chip_padding).mul_add(nominal_chip_em, widest_chip),
        2.0f64.mul_add(-props.chip_inset, 1.0) * width,
    );
    let em = nominal_em * act1_fit;
    let chip_em = nominal_chip_em * chip_fit;
    let pill_h = props.pill_height * em;
    let pill_pad = props.pill_padding * pill_h;
    let gap = props.pill_gap * pill_h;
    let prefix_width = metrics.prefix_width * act1_fit;
    let pill_widths: Vec<f64> = metrics
        .status_widths
        .iter()
        .map(|w| 2.0f64.mul_add(pill_pad, w * act1_fit))
        .collect();
    let width_frames = props.pill_resize_time * fps;
    let roll_frames = props.roll_time * fps;
    let (index, since) = status_at(frame, props.intro_frames, props.status_hold, labels.len());
    let width_p = if index == 0 {
        width_ease((frame / props.intro_frames).clamp(0.0, 1.0))
    } else {
        width_ease((since / width_frames).clamp(0.0, 1.0))
    };
    let roll = if index == 0 {
        1.0
    } else {
        roll_ease((since / roll_frames).clamp(0.0, 1.0))
    };
    let from_width = if index == 0 {
        0.0
    } else {
        pill_widths.get(index - 1).copied().unwrap_or(0.0)
    };
    let pill_w = interpolate(
        width_p,
        &[0.0, 1.0],
        &[from_width, pill_widths.get(index).copied().unwrap_or(0.0)],
        &Interpolation::default(),
    )?;
    let group_scale = spring(&Spring {
        config: GROUP_SPRING,
        from: props.intro_zoom,
        to: 1.0,
        ..Spring::at(frame, fps)
    })?;
    let exit = settle((frame - beat) * (1.0 / EXIT_SECONDS / fps), 0.28);
    let exit_y = if frame < beat {
        0.0
    } else {
        -exit * 0.62 * height
    };
    let total = prefix_width + gap + pill_w;
    let origin_x = (width - total) / 2.0;
    let cy = props.pill_center_y * height;
    let look = Look {
        em,
        typed: &typed,
        cy,
    };
    let mut layers: Vec<Node> = vec![
        div(absolute_fill()
            .background_color(cx.color(&field))
            .opacity(1.0 - faded))
        .into(),
    ];
    if frame < EXIT_SECONDS.mul_add(fps, beat) {
        layers.push(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .transform(vec![
                    Transform::Translate(Length::ZERO, Length::px(exit_y), Length::ZERO),
                    Transform::Scale(Num(group_scale), Num(group_scale), Num(1.0)),
                ])
                .transform_origin((Length::px(width / 2.0), Length::px(cy), Length::ZERO))
                .width(width)
                .height(height))
            .child(prefix_node(
                &glyphs,
                &metrics,
                (frame, fps),
                (
                    act1_fit,
                    props.letter_stagger,
                    origin_x,
                    cx.color(prefix_ink),
                ),
                &look,
            )?)
            .child(pill_node(
                &labels,
                index,
                roll,
                &PillLook {
                    x: origin_x + prefix_width + gap,
                    width: pill_w,
                    height: pill_h,
                    fill: cx.color(pill_fill),
                    ink: cx.color(&label_ink),
                    radius: props.pill_radius.map_or(PILL_R * pill_h, f64::from),
                    travel: ROLL_TRAVEL * pill_h,
                },
                &look,
            ))
            .into(),
        );
    }
    if frame >= beat {
        let chip_h = chip_em * 2.0_f64.mul_add(CHIP_PAD_Y, 1.0);
        let step = props.chip_gap.mul_add(height, chip_h);
        let (arrived, since) = chip_at(frame, beat, props.chip_stagger, chip_labels.len());
        let p = settle(since, SCROLL_TAU * fps);
        let field_y = (float(arrived) - 2.0 + p)
            .mul_add(-step, props.chip_gap.mul_add(-height, height) - chip_h);
        let mut chips = Vec::new();
        for (i, text) in chip_labels.iter().take(arrived).enumerate() {
            let w = (2.0 * props.chip_padding).mul_add(
                chip_em,
                metrics.chip_widths.get(i).copied().unwrap_or(0.0) * chip_fit,
            );
            let fill = fills.get(i % fills.len().max(1)).map_or("", String::as_str);
            let mut style = Style::default()
                .position(Position::Absolute)
                .top(Length::ZERO);
            style = if i % 2 == 0 {
                style.left(props.chip_inset * width)
            } else {
                style.right(props.chip_inset * width)
            };
            chips.push(
                div(style
                    .transform(vec![Transform::Translate(
                        Length::ZERO,
                        Length::px(float(i).mul_add(step, field_y)),
                        Length::ZERO,
                    )])
                    .width(w)
                    .height(chip_h)
                    .border_radius_all(props.chip_radius.map_or(CHIP_R * chip_h, f64::from))
                    .background_color(cx.color(fill))
                    .border(Edges::all(BorderSide {
                        width: Length::px(props.chip_border_width.unwrap_or(CHIP_BORDER * chip_h)),
                        style: BorderStyle::Solid,
                        color: Some(cx.color(&chip_edge)),
                    }))
                    .display(Display::Flex)
                    .align_items(Align::Center)
                    .justify_content(Align::Center)
                    .color(cx.color(chip_ink))
                    .font_size(chip_em)
                    .merged(&chip_typed))
                .child(text.as_str())
                .into(),
            );
        }
        layers.push(
            div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .overflow(Overflow::Hidden))
            .children(chips)
            .into(),
        );
    }
    Ok(div(absolute_fill()
        .background_color(cx.color(page))
        .overflow(Overflow::Hidden))
    .children(layers)
    .into())
}
