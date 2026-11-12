use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, Output, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{ColorStop, Gradient, RadialShape, RadialSize};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, NumericVariant, Overflow, Position,
    Style,
};
use opsis_scene::values::{Angle, Edges, Length, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped, clamped_eased, sprung};
use crate::elements::code_theme::code_palette;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::group_digits;
use crate::elements::text_split::easing_enter;
use crate::elements::shared::GroupSeparator;
use crate::elements::theme::{Theme, ThemeMode, stat_card_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct StatCard {
    #[setting("Value", default = 98.0, help = "Number the counter lands on.")]
    pub value: f64,
    #[setting("Suffix", default = "%", help = "Unit after the number.")]
    pub suffix: String,
    #[setting("Prefix", help = "Unit before the number.")]
    pub prefix: String,
    #[setting(
        "Decimals",
        range(min = 0, max = 6),
        step = 1,
        help = "Decimal places while counting and at rest.",
        advanced
    )]
    pub decimals: u32,
    #[setting("Label", default = "Satisfaction", help = "Metric label.")]
    pub label: String,
    #[setting(
        "Caption",
        default = "Customer survey, last 30 days",
        help = "Line under the label (source, window, cohort)."
    )]
    pub caption: String,
    #[setting(
        "Change",
        default = 4,
        range(min = -1_000_000, max = 1_000_000),
        step = 1,
        help = "Change against the previous period; lands after the number settles.",
        show_if = { "showDelta": ["true"] }
    )]
    pub delta: i32,
    #[setting(
        "Change suffix",
        default = "%",
        help = "Unit on the delta chip.",
        show_if = { "showDelta": ["true"] },
        advanced
    )]
    pub delta_suffix: String,
    #[setting(
        "Accent color",
        default = "#2DD4BF",
        help = "Ring, suffix, and delta colour.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = stat_card_themes,
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
    #[setting("Show change", default = true)]
    pub show_delta: bool,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        advanced
    )]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Ring full at",
        range(min = 0, max = 1_000_000_000),
        zero_label = "Full ring",
        step = 1,
        help = "The ring fills to the value out of this. Leave empty for a full ring.",
        "type" = "integer"
    )]
    pub max: Option<u32>,
    #[setting(
        "Glow",
        default = 10.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
    #[setting(
        "Ring thickness",
        default = 9,
        range(min = 2, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ring_thickness: u32,
    #[setting(
        "Thousands separator",
        default = GroupSeparator::None,
        advanced
    )]
    pub group_separator: GroupSeparator,
    #[setting("Font", widget = "font", zero_label = "Main font", advanced)]
    pub font_family: String,
    #[setting(
        "Falling colour",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub down_color: String,
    #[setting(
        "Ring glow",
        default = 10,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ring_glow: u32,
}

pub fn definition() -> ElementMeta {
    meta::<StatCard>(
            element(
                ElementKind::Component,
                "stat-card",
                "Stat card",
                "One big number counts up while a ring fills, then its label and the change since last period appear.",
                ElementCategory::Data,
                123,
            )
                .fonts(&["Inter"])
                .look(NodeLook::of("lucide:circle-gauge")),
        )
}

const RING_AT: f64 = 0.1;
const RING_FOR: f64 = 1.5;
const COUNT_AT: f64 = 0.16;
const COUNT_FOR: f64 = 2.2;
const LABEL_AT: f64 = 0.6;
const CAPTION_AT: f64 = 0.78;
const DELTA_AT: f64 = 2.7;
const FADE_FOR: f64 = 0.45;
const DELTA_SPRING: SpringConfig = SpringConfig {
    damping: 13.0,
    stiffness: 190.0,
    mass: 0.6,
    overshoot_clamping: false,
};

fn format_value(value: f64, decimals: usize, separator: GroupSeparator) -> String {
    let text = format!("{value:.decimals$}");
    match text.split_once('.') {
        Some((whole, fraction)) => format!("{}.{fraction}", group_digits(whole, separator)),
        None => group_digits(&text, separator),
    }
}

fn safe_padding(width: f64, height: f64) -> (f64, f64) {
    ((80.0 / 1080.0 * width).round(), (100.0 / 1080.0 * height).round())
}

fn tinted(color: &str, share: f64) -> Color {
    Color::mix(
        ColorInterpolation {
            space: ColorSpace::Srgb,
            ..ColorInterpolation::DEFAULT
        },
        (css_color(color), Some(share)),
        (Color::TRANSPARENT, None),
    )
}

fn perceptual(input: f64, to: [f64; 2], clamp: bool) -> Result<f64, DrawFailure> {
    let mut options = if clamp { Interpolation::clamped() } else { Interpolation::default() };
    options.output = Output::PerceptualScale;
    Ok(interpolate(input, &[0.0, 1.0], &to, &options)?)
}

#[expect(
    clippy::too_many_lines,
    reason = "the card's markup follows the film's one view function in order"
)]
pub fn draw(props: &StatCard, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let fps = cx.fps();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let delta = props.show_delta.then_some(props.delta);
    let delta_color = if delta.is_some_and(|d| d < 0) && !props.down_color.is_empty() {
        props.down_color.as_str()
    } else {
        props.accent_color.as_str()
    };
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64| clamped_eased(frame, [at(from), at(to)], [0.0, 1.0], easing_enter());
    let stage_w = width - side * 2.0;
    let stage_h = height - edge * 2.0;
    let u = if height > width {
        (stage_w / 620.0).min(stage_h / 1120.0)
    } else {
        (stage_w / 1120.0).min(stage_h / 620.0)
    };
    let share = props
        .max
        .filter(|m| *m > 0)
        .map_or(1.0, |m| (props.value / f64::from(m)).clamp(0.0, 1.0));
    let ring = ease(RING_AT, RING_AT + RING_FOR)? * share;
    let counted = ease(COUNT_AT, COUNT_AT + COUNT_FOR)? * props.value;
    let label_in = ease(LABEL_AT, LABEL_AT + FADE_FOR)?;
    let caption_in = if props.caption.is_empty() { 0.0 } else { ease(CAPTION_AT, CAPTION_AT + FADE_FOR)? };
    let delta_in = if delta.is_some() {
        sprung(frame - at(DELTA_AT), fps, DELTA_SPRING)?
    } else {
        0.0
    };
    let land = clamped(
        frame,
        [at(RING_AT + RING_FOR), at(RING_AT + RING_FOR + FADE_FOR)],
        [1.0, 0.0],
    )?;
    let size = (stage_w * 0.62).min(stage_h * 0.62).min(380.0 * u);
    let stroke = f64::from(props.ring_thickness) * u;
    let radius = size / 2.0 - stroke;
    let circumference = 2.0 * PI * radius;
    let accent = props.accent_color.as_str();
    let ring_svg = SvgElement::new("svg")
        .attr("width", size)
        .attr("height", size)
        .child(
            SvgElement::new("circle")
                .attr("cx", size / 2.0)
                .attr("cy", size / 2.0)
                .attr("r", radius)
                .attr("fill", "none")
                .attr("stroke", palette.get("band"))
                .attr("stroke-width", stroke),
        )
        .child(
            SvgElement::new("circle")
                .attr("cx", size / 2.0)
                .attr("cy", size / 2.0)
                .attr("r", radius)
                .attr("fill", "none")
                .attr("stroke", accent)
                .attr("stroke-width", stroke)
                .attr("stroke-linecap", "round")
                .attr("stroke-dasharray", circumference)
                .attr("stroke-dashoffset", circumference * (1.0 - ring))
                .attr(
                    "filter",
                    format!(
                        "drop-shadow(0 0 {}px {accent})",
                        js_number_text(land.mul_add(16.0, f64::from(props.ring_glow)) * u)
                    ),
                ),
        );
    let fg = css_color(palette.get("fg"));
    let digits_scale = perceptual(land, [1.0, 1.03], true)?;
    let mut figure = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .color(fg.clone())
        .font_size(92.0 * u)
        .font_weight(700.0)
        .letter_spacing(Length::em(-0.03))
        .font_variant_numeric(NumericVariant {
            tabular: true,
            ..NumericVariant::default()
        })
        .scale(digits_scale, digits_scale));
    if !props.prefix.is_empty() {
        figure = figure.child(
            span(Style::default()
                .font_size(54.0 * u)
                .align_self(Align::FlexStart)
                .margin_top(20.0 * u))
            .child(props.prefix.as_str()),
        );
    }
    let decimals = usize::try_from(props.decimals).unwrap_or(0);
    figure = figure.child(format_value(counted, decimals, props.group_separator));
    if !props.suffix.is_empty() {
        figure = figure.child(
            span(Style::default().font_size(54.0 * u).color(css_color(accent)))
                .child(props.suffix.as_str()),
        );
    }
    let mut column = div(Style::default()
        .position(Position::Relative)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(18.0 * u)
        .opacity(1.0)
        .translate(Length::ZERO, Length::ZERO))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(size)
            .height(size))
        .child(svg(ring_svg, Style::default().rotate(Angle::deg(-90.0))))
        .child(figure),
    );
    if !props.label.is_empty() {
        column = column.child(
            div(Style::default()
                .color(fg.clone())
                .font_size(34.0 * u)
                .font_weight(600.0)
                .letter_spacing(Length::em(-0.01))
                .opacity(label_in)
                .translate(Length::ZERO, Length::px((1.0 - label_in) * 12.0 * u)))
            .child(props.label.as_str()),
        );
    }
    if !props.caption.is_empty() {
        column = column.child(
            div(Style::default()
                .margin_top(-8.0 * u)
                .color(css_color(palette.get("faint")))
                .font_size(21.0 * u)
                .opacity(caption_in))
            .child(props.caption.as_str()),
        );
    }
    if let Some(delta) = delta {
        let shown = delta_in.min(1.0);
        let scale = perceptual(shown, [0.86, 1.0], false)?;
        let arrow = SvgElement::new("svg")
            .attr("width", 16.0 * u)
            .attr("height", 16.0 * u)
            .attr("viewBox", "0 0 16 16")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr(
                        "d",
                        if delta < 0 { "M8 3v10m0 0 4-4m-4 4-4-4" } else { "M8 13V3m0 0 4 4M8 3 4 7" },
                    )
                    .attr("stroke", delta_color)
                    .attr("stroke-width", 2)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round"),
            );
        column = column.child(
            div(Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(8.0 * u)
                .padding(Edges::axes(Length::px(8.0 * u), Length::px(15.0 * u)))
                .border_radius_all(999.0)
                .background_color(tinted(delta_color, 9.0))
                .border(Edges::all(BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(tinted(delta_color, 33.0)),
                }))
                .color(css_color(delta_color))
                .font_size(22.0 * u)
                .font_weight(600.0)
                .font_variant_numeric(NumericVariant {
                    tabular: true,
                    ..NumericVariant::default()
                })
                .opacity(shown)
                .scale(scale, scale))
            .child(svg(arrow, Style::default()))
            .child(format!(
                "{}{delta}{}",
                if delta > 0 { "+" } else { "" },
                props.delta_suffix
            )),
        );
    }
    let glow = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(55.0), Length::percent(50.0)),
        center: Position2 {
            x: Length::percent(50.0),
            y: Length::percent(45.0),
        },
        stops: vec![
            ColorStop::new(tinted(accent, props.glow)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(72.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        props.background_color.as_str()
    };
    Ok(div(with_background(
        Style::default()
            .width(width)
            .height(height)
            .font_family(font_or(&props.font_family, FontRole::Main))
            .position(Position::Relative)
            .overflow(Overflow::Hidden)
            .display(Display::Grid)
            .align_items(Align::Center)
            .justify_items(Align::Center),
        page,
    ))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![glow.layer()])))
    .child(column)
    .into())
}
