use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{ColorStop, Filter, Gradient, RadialShape, RadialSize, Shadow};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, DecorationLine, Display, FlexDirection, LineHeight,
    NumericVariant, Position, Style, TextDecoration,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::{clamped, sprung};
use crate::elements::code_theme::code_palette;
use crate::elements::code_window::{CodeWindow, code_window};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::oklch_mix::{parsed_or_black, with_alpha};
use crate::elements::path_utils::path_length;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::units::js_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct PricingCard {
    #[setting(
        "Plan name",
        default = "Studio",
        help = "Tier name across the top of the card."
    )]
    pub tier: String,
    #[setting("Price", default = 32.0, help = "The number the price rolls up to.")]
    pub price: f64,
    #[setting("Currency", default = "$", help = "Symbol set ahead of the number.")]
    pub currency: String,
    #[setting(
        "Billing period",
        default = "/mo",
        help = "Cadence after the price: \"/mo\", \"per seat\", anything short."
    )]
    pub period: String,
    #[setting(
        "Badge",
        default = "Most picked",
        help = "Chip beside the tier name. Omit to drop it."
    )]
    pub badge: String,
    #[setting(
        "Small print",
        default = "Billed annually. Cancel whenever.",
        help = "Line under the price."
    )]
    pub note: String,
    #[setting(
        "Features",
        default = vec![
            "2,000 render minutes a month".into(),
            "4K exports, no watermark".into(),
            "9 team seats".into(),
            "Render API and webhooks".into(),
            "Priority render queue".into(),
        ],
        help = "Ticked lines under the price, revealed in order."
    )]
    pub features: Vec<String>,
    #[setting(
        "Button label",
        default = "Start rendering",
        help = "Button text. Omit to drop the button."
    )]
    pub cta_label: String,
    #[setting(
        "Price at (seconds)",
        default = 0.42,
        step = 0.01,
        help = "Second the price starts rolling.",
        advanced
    )]
    pub price_at_seconds: f64,
    #[setting(
        "Roll (seconds)",
        default = 0.9,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long the roll takes.",
        advanced
    )]
    pub roll_seconds: f64,
    #[setting(
        "Features at (seconds)",
        default = 1.05,
        step = 0.1,
        help = "Second the first feature ticks in.",
        advanced
    )]
    pub features_at_seconds: f64,
    #[setting(
        "Feature stagger (seconds)",
        default = 0.22,
        step = 0.01,
        help = "Seconds between features. The CTA lands 0.16s after the last one.",
        advanced
    )]
    pub feature_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Badge, ticks, card border wash and the CTA fill.",
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
        "Button text color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme.",
        check = colour_or_empty,
        advanced
    )]
    pub cta_text_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Old price",
        range(min = 0, max = 1_000_000),
        zero_label = "None",
        step = 1,
        help = "Shown struck out next to the price.",
        "type" = "integer"
    )]
    pub was_price: Option<u32>,
    #[setting(
        "Roll from",
        default = 0,
        range(min = 0, max = 1_000_000),
        step = 1,
        advanced
    )]
    pub roll_from: u32,
    #[setting(
        "Card width",
        default = 460,
        range(min = 300, max = 1000),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<PricingCard>(
        element(
            ElementKind::Component,
            "pricing-card",
            "Pricing card",
            "A pricing plan card: the price rolls in, then each feature is ticked off.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:badge-dollar-sign")),
    )
}

const CARD_FOR: f64 = 0.48;
const HEAD_AT: f64 = 0.16;
const HEAD_FOR: f64 = 0.4;
const CTA_AFTER_FEATURES: f64 = 0.16;
const CTA_FOR: f64 = 0.42;
const CHECK: &str = "M6.5 12.5 L10.5 16.5 L17.5 8";
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const POP: Easing = Easing::Bezier(0.34, 1.56, 0.64, 1.0);
const CARD_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.85,
    overshoot_clamping: false,
};

fn outline(color: &str, share: f64) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(tinted(color, share)),
    })
}

fn feature_row(props: &PricingCard, feature: &str, feature_in: f64, u: f64, fg: &str) -> Node {
    let accent = props.accent_color.as_str();
    let length = path_length(CHECK);
    let tick = svg(
        SvgElement::new("svg")
            .attr("width", 18.0 * u)
            .attr("height", 18.0 * u)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("circle")
                    .attr("cx", "12")
                    .attr("cy", "12")
                    .attr("r", "11")
                    .attr("fill", with_alpha(accent, 0.13 * alpha_of(accent))),
            )
            .child(
                SvgElement::new("path")
                    .attr("d", CHECK)
                    .attr("stroke", accent)
                    .attr("stroke-width", 2.6)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round")
                    .attr("stroke-dasharray", length)
                    .attr("stroke-dashoffset", (1.0 - feature_in) * length),
            ),
        Style::default(),
    );
    div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(11.0 * u)
        .opacity(feature_in)
        .translate(Length::px((1.0 - feature_in) * -10.0 * u), Length::ZERO))
    .child(tick)
    .child(
        span(
            Style::default()
                .color(css_color(fg))
                .font_size(15.5 * u)
                .font_weight(500.0),
        )
        .child(feature.to_owned()),
    )
    .into()
}

fn alpha_of(css: &str) -> f64 {
    parsed_or_black(css).alpha.unwrap_or(1.0)
}

#[expect(
    clippy::too_many_lines,
    reason = "the card's parts in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &PricingCard, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, frame, fps) = (cx.at.stage.width, cx.at.stage.height, cx.frame(), cx.fps());
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64, easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[at(from), at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let u = if height > width {
        (width / 496.0).min(height / 896.0)
    } else {
        (width / 1024.0).min(height / 576.0)
    };
    let card_in = sprung(frame - at(0.0), fps, CARD_SPRING)?;
    let open = ease(0.0, CARD_FOR, EDITORIAL)?;
    let head_in = ease(HEAD_AT, HEAD_AT + HEAD_FOR, easing_enter())?;
    let roll = ease(
        props.price_at_seconds,
        props.price_at_seconds + props.roll_seconds,
        EDITORIAL,
    )?;
    let price_text = js_number_text(props.price);
    let decimals = price_text
        .split_once('.')
        .map_or(0, |(_, f)| f.len().min(2));
    let value = interpolate(
        roll,
        &[0.0, 1.0],
        &[f64::from(props.roll_from), props.price],
        &Interpolation::default(),
    )?;
    let shown = js_fixed(value, i32::try_from(decimals).unwrap_or(0));
    let roll_blur = (PI * roll).sin() * 2.2;
    let last_feature_at = float(props.features.len().saturating_sub(1))
        .mul_add(props.feature_stagger_seconds, props.features_at_seconds);
    let cta_at = last_feature_at + CTA_AFTER_FEATURES;
    let cta_in = ease(cta_at, cta_at + CTA_FOR, POP)?;
    let card_w = f64::mul_add(side, -2.0, width).min(f64::from(props.card_width) * u);
    let accent = props.accent_color.as_str();
    let shown_now = if roll > 0.0 { 1.0 } else { 0.0 };
    let mut head = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(10.0 * u)
        .opacity(head_in)
        .translate(Length::ZERO, Length::px((1.0 - head_in) * 10.0 * u)))
    .child(
        span(
            Style::default()
                .color(css_color(palette.get("fg")))
                .font_size(19.0 * u)
                .font_weight(700.0)
                .letter_spacing(Length::em(0.02)),
        )
        .child(props.tier.clone()),
    );
    if !props.badge.is_empty() {
        head = head.child(
            span(
                Style::default()
                    .padding(Edges::axes(Length::px(3.5 * u), Length::px(9.0 * u)))
                    .border_radius_all(999.0)
                    .background_color(tinted(accent, 13.0))
                    .border(outline(accent, 40.0))
                    .color(css_color(accent))
                    .font_size(14.5 * u)
                    .font_weight(600.0),
            )
            .child(props.badge.clone()),
        );
    }
    let mut price_style = Style::default()
        .margin_top(Length::px(16.0 * u))
        .display(Display::Flex)
        .align_items(Align::Baseline)
        .gap(8.0 * u);
    if roll_blur > 0.05 {
        price_style = price_style.filter(vec![Filter::Blur(Length::px(roll_blur))]);
    }
    let tabular = NumericVariant {
        tabular: true,
        ..NumericVariant::default()
    };
    let mut price = div(price_style)
        .child(
            span(
                Style::default()
                    .color(css_color(palette.get("dim")))
                    .font_size(30.0 * u)
                    .font_weight(600.0)
                    .opacity(shown_now),
            )
            .child(props.currency.clone()),
        )
        .child(
            span(
                Style::default()
                    .color(css_color(palette.get("fg")))
                    .font_size(68.0 * u)
                    .font_weight(800.0)
                    .letter_spacing(Length::em(-0.04))
                    .line_height(LineHeight::Number(Num(1.0)))
                    .font_variant_numeric(tabular)
                    .translate(Length::ZERO, Length::px((1.0 - roll) * 10.0 * u))
                    .opacity(shown_now),
            )
            .child(shown),
        );
    if !props.period.is_empty() {
        price = price.child(
            span(
                Style::default()
                    .color(css_color(palette.get("dim")))
                    .font_size(19.0 * u)
                    .font_weight(600.0)
                    .opacity(clamped(roll, [0.5, 1.0], [0.0, 1.0])?),
            )
            .child(props.period.clone()),
        );
    }
    if let Some(was) = props.was_price {
        price = price.child(
            span(
                Style::default()
                    .margin_left(Length::px(4.0 * u))
                    .color(css_color(palette.get("faint")))
                    .font_size(20.0 * u)
                    .font_weight(600.0)
                    .text_decoration(TextDecoration {
                        lines: vec![DecorationLine::LineThrough],
                        ..TextDecoration::default()
                    })
                    .font_variant_numeric(tabular)
                    .opacity(clamped(roll, [0.7, 1.0], [0.0, 1.0])?),
            )
            .child(format!("{}{was}", props.currency)),
        );
    }
    let mut body = div(Style::default().position(Position::Relative))
        .child(head)
        .child(price);
    if !props.note.is_empty() {
        body = body.child(
            div(Style::default()
                .margin_top(Length::px(6.0 * u))
                .color(css_color(palette.get("dim")))
                .font_size(14.5 * u)
                .font_weight(500.0)
                .opacity(clamped(roll, [0.6, 1.0], [0.0, 1.0])?))
            .child(props.note.clone()),
        );
    }
    let mut features = Vec::new();
    for (index, feature) in props.features.iter().enumerate() {
        let feature_at =
            float(index).mul_add(props.feature_stagger_seconds, props.features_at_seconds);
        let feature_in = ease(feature_at, feature_at + 0.38, easing_enter())?;
        features.push(feature_row(
            props,
            feature,
            feature_in,
            u,
            palette.get("fg"),
        ));
    }
    body = body.child(
        div(Style::default()
            .margin_top(Length::px(20.0 * u))
            .padding_top(Length::px(18.0 * u))
            .border(Edges {
                top: BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(css_color(palette.get("border"))),
                },
                right: BorderSide::default(),
                bottom: BorderSide::default(),
                left: BorderSide::default(),
            })
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(12.0 * u))
        .children(features),
    );
    if !props.cta_label.is_empty() {
        let dark = props.theme.mode == ThemeMode::Dark;
        let ink = if !props.cta_text_color.is_empty() {
            props.cta_text_color.as_str()
        } else if dark {
            "#141013"
        } else {
            "#1B1509"
        };
        let grow = cta_in.mul_add(0.06, 0.94);
        body = body.child(
            div(Style::default()
                .margin_top(Length::px(22.0 * u))
                .height(Length::px(48.0 * u))
                .border_radius_all(12.0 * u)
                .background_color(css_color(accent))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::Center)
                .color(css_color(ink))
                .font_size(16.5 * u)
                .font_weight(700.0)
                .letter_spacing(Length::em(0.01))
                .opacity((cta_in * 1.4).min(1.0))
                .scale(grow, grow)
                .box_shadow(vec![Shadow {
                    inset: false,
                    x: Length::ZERO,
                    y: Length::px(10.0 * u),
                    blur: Length::px(26.0 * u),
                    spread: Length::ZERO,
                    color: tinted(accent, 20.0),
                }]))
            .child(props.cta_label.clone()),
        );
    }
    let glow = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(120.0), Length::percent(80.0)),
        center: Position2 {
            x: Length::percent(8.0),
            y: Length::percent(0.0),
        },
        stops: vec![
            ColorStop::new(tinted(accent, 12.0)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(62.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    let card = div(code_window(
        &palette,
        CodeWindow {
            width: card_w,
            radius: 24.0 * u,
            open,
            rise: (1.0 - card_in) * 24.0 * u,
            clip_round: 24.0 * u,
            shadow: (28.0 * u, 70.0 * u),
        },
    )
    .border(outline(accent, 23.0))
    .padding(Edges::all(Length::px(30.0 * u)))
    .position(Position::Relative))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![glow.layer()])))
    .child(body);
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    Ok(div(with_background(
        absolute_fill()
            .font_family(font_or(&props.font_family, FontRole::Main))
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .padding(Edges::axes(Length::px(edge), Length::px(side))),
        page,
    ))
    .child(card)
    .into())
}
