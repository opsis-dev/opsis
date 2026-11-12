use opsis_macros::settings;
use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{ColorStop, Gradient, RadialShape, RadialSize};
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, FlexWrap, LineHeight, NumericVariant,
    RepeatCount, Style, TextTransform, Track,
};
use opsis_scene::values::{Edges, Length, Num, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, whole};

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::code_theme::{Palette, palette_of};
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::{Direction, format_number, read_delta};
use crate::elements::parts::line_chart::{LineChart, line_chart};
use crate::elements::text_split::easing_enter;
use crate::elements::shared::NumberFormat;
use crate::elements::theme::{Theme, ThemeMode, metric_ticker_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MetricTickerMetric {
    #[setting("Label", default = "Support tickets")]
    pub label: String,
    #[setting("Value", default = 320.0)]
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Trend", "type" = "array")]
    pub trend: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Suffix", "type" = "string")]
    pub suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("From", "type" = "number")]
    pub from: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Prefix", "type" = "string")]
    pub prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct MetricTicker {
    #[setting(
        "Metrics",
        default = vec![
            MetricTickerMetric {
                label: "Sign-ups".into(),
                value: 12400.0,
                delta: Some("+18%".into()),
                trend: Some(vec![62.0, 71.0, 68.0, 84.0, 96.0, 112.0, 124.0]),
                ..Default::default()
            },
            MetricTickerMetric {
                label: "Avg. session".into(),
                value: 48.0,
                suffix: Some("min".into()),
                delta: Some("+6%".into()),
                trend: Some(vec![39.0, 41.0, 40.0, 43.0, 45.0, 44.0, 48.0]),
                ..Default::default()
            },
            MetricTickerMetric {
                delta: Some("-4%".into()),
                trend: Some(vec![36.0, 35.0, 37.0, 34.0, 33.0, 33.5, 32.0]),
                ..Default::default()
            },
        ],
        help = "Metric cards: label, value, and optional from, prefix, suffix, delta, trend, color. Set from above value for a metric that counts down."
    )]
    pub metrics: Vec<MetricTickerMetric>,
    #[setting("Title", default = "Growth this quarter", help = "Scene title.")]
    pub title: String,
    #[setting(
        "Small heading",
        default = "Quarter recap",
        help = "Short label above the title."
    )]
    pub eyebrow: String,
    #[setting(
        "Most cards",
        default = 4,
        help = "Cards beyond this count are dropped rather than squeezed.",
        advanced
    )]
    pub max_cards: u32,
    #[setting(
        "Accent color",
        widget = "color",
        zero_label = "Theme",
        help = "Value colour, overridable per metric. Leave empty to follow the theme."
    )]
    pub accent_color: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        advanced
    )]
    pub background_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = metric_ticker_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Show arrows", default = true, advanced)]
    pub show_arrows: bool,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting("Glow", default = 7.0, range(min = 0, max = 100), unit = "%", advanced)]
    pub glow: f64,
    #[setting("Capitalise small heading", default = true, advanced)]
    pub uppercase_eyebrow: bool,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Fill under trend", default = true, advanced)]
    pub trend_fill: bool,
    #[setting("Font", widget = "font", zero_label = "Main font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<MetricTicker>(
        element(
            ElementKind::Component,
            "metric-ticker",
            "Metric cards",
            "Key numbers on cards that count up, each with its change and a small trend line.",
            ElementCategory::Data,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:trending-up")),
    )
}

const DARK: [(&str, &str); 9] = [
    ("bg", "#080810"),
    ("ink", "#fafafa"),
    ("label", "rgba(250,250,250,0.58)"),
    ("eyebrow", "rgba(250,250,250,0.44)"),
    ("card", "rgba(250,250,250,0.035)"),
    ("border", "rgba(250,250,250,0.10)"),
    ("accent", "#e8b86d"),
    ("up", "#2dd4bf"),
    ("down", "#f87171"),
];
const LIGHT: [(&str, &str); 9] = [
    ("bg", "#F4F4F2"),
    ("ink", "#22252B"),
    ("label", "rgba(34,37,43,0.62)"),
    ("eyebrow", "rgba(34,37,43,0.48)"),
    ("card", "#FFFFFF"),
    ("border", "rgba(0,0,0,0.10)"),
    ("accent", "#b45309"),
    ("up", "#0f766e"),
    ("down", "#dc2626"),
];
const DURATION_NORMAL: f64 = 24.0;
const DURATION_SLOW: f64 = 36.0;
const STAGGER_TIGHT: f64 = 4.0;
const STAGGER_NORMAL: f64 = 8.0;
const CARD_SPRING: SpringConfig = SpringConfig {
    damping: 20.0,
    stiffness: 120.0,
    mass: 0.9,
    overshoot_clamping: false,
};

fn scale_font(size: f64, width: f64) -> f64 {
    f64::from(whole(size * (width / 1080.0)))
}

#[expect(
    clippy::too_many_lines,
    reason = "a card's markup follows the film's one view function in order"
)]
fn card(
    props: &MetricTicker,
    (index, metric): (usize, &MetricTickerMetric),
    palette: &Palette,
    (accent, width, portrait, card_width, padding): (&str, f64, bool, f64, f64),
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let delay = float(index).mul_add(STAGGER_NORMAL, STAGGER_NORMAL);
    let enter = spring(&Spring {
        config: CARD_SPRING,
        duration_in_frames: Some(DURATION_SLOW),
        ..Spring::at(frame - delay, cx.fps())
    })?;
    let (direction, delta_text) = read_delta(metric.delta.as_deref().unwrap_or(""));
    let delta_color = match direction {
        Direction::Down => palette.get("down"),
        Direction::Up => palette.get("up"),
        Direction::Flat => palette.get("label"),
    };
    let card_color = metric.color.as_deref().unwrap_or(accent);
    let detail = clamped(enter, [0.55, 1.0], [0.0, 1.0])?;
    let from = metric.from.unwrap_or(0.0);
    let figure = format!(
        "{}{}{}",
        metric.prefix.as_deref().unwrap_or(""),
        format_number((metric.value - from).mul_add(enter, from), props.number_format),
        metric.suffix.as_deref().unwrap_or("")
    );
    let mut value_row = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Baseline)
        .gap(scale_font(12.0, width))
        .flex_wrap(FlexWrap::Wrap))
    .child(
        span(Style::default()
            .color(css_color(card_color))
            .font_size(scale_font(if portrait { 76.0 } else { 64.0 }, width))
            .font_weight(700.0)
            .line_height(LineHeight::Number(Num(1.0)))
            .letter_spacing(Length::em(-0.03))
            .font_variant_numeric(NumericVariant {
                tabular: true,
                ..NumericVariant::default()
            }))
        .child(figure),
    );
    if !delta_text.is_empty() {
        let glyph = match direction {
            Direction::Up => "\u{2191}",
            Direction::Down => "\u{2193}",
            Direction::Flat => "",
        };
        let arrow = if props.show_arrows { format!("{glyph} ") } else { String::new() };
        value_row = value_row.child(
            span(Style::default()
                .color(css_color(delta_color))
                .font_size(scale_font(22.0, width))
                .font_weight(600.0)
                .line_height(LineHeight::Number(Num(1.0)))
                .opacity(detail))
            .child(format!("{arrow}{delta_text}")),
        );
    }
    let mut body = div(with_background(
        Style::default()
            .display(Display::Grid)
            .gap(scale_font(14.0, width))
            .align_content(Align::Start)
            .padding(Edges::all(Length::px(padding)))
            .border_radius_all(scale_font(f64::from(props.corner_radius), width))
            .border(Edges::all(BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color(palette.get("border"))),
            }))
            .opacity((enter * 1.6).min(1.0))
            .translate(Length::ZERO, Length::px((1.0 - enter) * scale_font(22.0, width))),
        palette.get("card"),
    ))
    .child(
        div(Style::default()
            .color(css_color(palette.get("label")))
            .font_size(scale_font(24.0, width))
            .font_weight(600.0)
            .line_height(LineHeight::Number(Num(1.0))))
        .child(metric.label.as_str()),
    )
    .child(value_row);
    if let Some(trend) = metric.trend.as_ref().filter(|t| t.len() > 1) {
        let mut chart = LineChart::new(
            trend.iter().enumerate().map(|(i, v)| (float(i), *v, None)).collect(),
            f64::from(whole(card_width - padding * 2.0)).max(1.0),
            scale_font(if portrait { 56.0 } else { 64.0 }, width),
            frame,
        );
        chart.color = (if delta_text.is_empty() { card_color } else { delta_color }).to_owned();
        chart.stroke_width = Some(scale_font(3.0, width).max(2.0));
        chart.show_axis = false;
        chart.show_x_labels = false;
        chart.include_zero = false;
        chart.show_dots = false;
        chart.show_head = false;
        chart.show_area = props.trend_fill;
        chart.duration = DURATION_SLOW;
        chart.delay = delay + STAGGER_TIGHT;
        chart.id = format!("metric-trend-{index}");
        body = body.child(div(Style::default().opacity(detail)).child(line_chart(&chart)?));
    }
    Ok(body.into())
}

pub fn draw(props: &MetricTicker, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let base = match props.theme.mode {
        ThemeMode::Dark => DARK,
        ThemeMode::Light => LIGHT,
    };
    let palette = palette_of(&base, &[], &props.theme.colors);
    let accent = if props.accent_color.is_empty() {
        palette.get("accent").to_owned()
    } else {
        props.accent_color.clone()
    };
    let frame = cx.frame();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let (side, edge) = (
        (80.0 / 1080.0 * width).round(),
        (100.0 / 1080.0 * height).round(),
    );
    let portrait = height > width;
    let cards: Vec<&MetricTickerMetric> = props
        .metrics
        .iter()
        .take(usize::try_from(props.max_cards).unwrap_or(usize::MAX))
        .collect();
    let columns = if portrait { 1 } else { cards.len().max(1) };
    let gap = scale_font(20.0, width);
    let padding = scale_font(28.0, width);
    let content = width - side * 2.0;
    let card_width = (content - gap * (float(columns) - 1.0)) / float(columns);
    let header = clamped_eased(frame, [0.0, DURATION_NORMAL], [0.0, 1.0], easing_enter())?;
    let glow = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(70.0), Length::percent(50.0)),
        center: Position2 {
            x: Length::percent(50.0),
            y: Length::percent(0.0),
        },
        stops: vec![
            ColorStop::new(Color::mix(
                ColorInterpolation {
                    space: ColorSpace::Srgb,
                    ..ColorInterpolation::DEFAULT
                },
                (css_color(&accent), Some(props.glow)),
                (Color::TRANSPARENT, None),
            )),
            ColorStop::at(Color::TRANSPARENT, Length::percent(70.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    let page = if props.background_color.is_empty() {
        palette.get("bg")
    } else {
        props.background_color.as_str()
    };
    let mut root = div(with_background(
        Style::default().width(width).height(height),
        page,
    )
    .background(vec![glow.layer()])
    .color(css_color(palette.get("ink")))
    .font_family(font_or(&props.font_family, FontRole::Main))
    .padding(Edges {
        top: Length::px(edge),
        right: Length::px(side),
        bottom: Length::px(edge),
        left: Length::px(side),
    })
    .display(Display::Flex)
    .flex_direction(FlexDirection::Column)
    .justify_content(Align::Center)
    .gap(scale_font(44.0, width)));
    if !props.title.is_empty() || !props.eyebrow.is_empty() {
        let mut head = div(Style::default()
            .display(Display::Grid)
            .gap(scale_font(10.0, width))
            .opacity(header)
            .translate(Length::ZERO, Length::px((1.0 - header) * scale_font(16.0, width))));
        if !props.eyebrow.is_empty() {
            let mut eyebrow = Style::default()
                .color(css_color(palette.get("eyebrow")))
                .font_size(scale_font(24.0, width))
                .font_weight(600.0);
            if props.uppercase_eyebrow {
                eyebrow = eyebrow
                    .letter_spacing(Length::em(0.08))
                    .text_transform(TextTransform::Uppercase);
            }
            head = head.child(span(eyebrow).child(props.eyebrow.as_str()));
        }
        if !props.title.is_empty() {
            head = head.child(
                div(Style::default()
                    .font_size(scale_font(if portrait { 72.0 } else { 58.0 }, width))
                    .font_weight(700.0)
                    .line_height(LineHeight::Number(Num(1.02)))
                    .letter_spacing(Length::em(-0.025)))
                .child(props.title.as_str()),
            );
        }
        root = root.child(head);
    }
    let mut grid = Vec::new();
    for (index, metric) in cards.iter().enumerate() {
        grid.push(card(
            props,
            (index, metric),
            &palette,
            (&accent, width, portrait, card_width, padding),
            cx,
        )?);
    }
    root = root.child(
        div(Style::default()
            .display(Display::Grid)
            .grid_template_columns(vec![Track::Repeat(
                RepeatCount::Times(u16::try_from(columns).unwrap_or(u16::MAX)),
                vec![Track::Fraction(Num(1.0))],
            )])
            .gap(gap)
            .opacity(1.0)
            .translate(Length::ZERO, Length::ZERO))
        .children(grid),
    );
    Ok(root.into())
}
