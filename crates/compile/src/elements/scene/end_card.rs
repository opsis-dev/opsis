use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, img, span, svg};
use opsis_scene::paint::{ColorStop, Gradient, RadialShape, RadialSize, Shadow};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, FlexWrap, LineHeight, ObjectFit,
    Overflow, Position, Style, TextAlign, TextOverflow, TextTransform, WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::anim::{clamped, sprung};
use crate::elements::code_theme::code_palette;
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::units::{utf16_len, utf16_slice};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct EndCard {
    #[setting("Title", default = "Ready to try it?", help = "Closing headline.")]
    pub title: String,
    #[setting(
        "Subtitle",
        default = "Start free. No credit card needed.",
        help = "Line under the title."
    )]
    pub subtitle: String,
    #[setting("Small heading", help = "Chip above the title.")]
    pub eyebrow: String,
    #[setting(
        "Button label",
        default = "Start free trial",
        help = "Button label. Omit to end on the title alone."
    )]
    pub cta: String,
    #[setting(
        "Address",
        default = "example.com/signup",
        help = "Address typed under the button."
    )]
    pub url: String,
    #[setting(
        "Handles",
        default = vec!["@acme".into(), "acme.example/blog".into()],
        help = "Handles or channels listed along the foot."
    )]
    pub handles: Vec<String>,
    #[setting("Logo", widget = "asset", accept = "image/*")]
    pub logo_src: String,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Button, pulse, and eyebrow colour.",
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
        help = "Animation speed multiplier.",
        advanced
    )]
    pub speed: f64,
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
        "Logo size",
        range(min = 10, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        "type" = "integer",
        advanced
    )]
    pub logo_size: Option<u32>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Accent glow",
        default = 12,
        range(min = 0, max = 60),
        step = 1,
        unit = "%",
        advanced
    )]
    pub glow: u32,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        help = "Scales every line of text together.",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Title weight",
        default = 700,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub title_weight: u32,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_caps: bool,
    #[setting(
        "Button corners",
        default = 999,
        range(min = 0, max = 999),
        step = 1,
        unit = "px",
        help = "999 makes a pill.",
        advanced
    )]
    pub button_radius: u32,
    #[setting("Button arrow", default = true, advanced)]
    pub button_arrow: bool,
}

pub fn definition() -> ElementMeta {
    meta::<EndCard>(
        element(
            ElementKind::Component,
            "end-card",
            "End card",
            "A closing card with a title, a button, your web address and social handles.",
            ElementCategory::Scene,
            150,
        )
        .look(NodeLook::of("lucide:flag")),
    )
}

const LOGO_AT: f64 = 0.0;
const EYEBROW_AT: f64 = 0.14;
const TITLE_AT: f64 = 0.26;
const SUBTITLE_AT: f64 = 0.6;
const BUTTON_AT: f64 = 0.78;
const BUTTON_LABEL_AT: f64 = 0.84;
const URL_AT: f64 = 1.24;
const HANDLES_AT: f64 = 1.6;
const PULSE_AT: f64 = 2.05;
const URL_CPS: f64 = 26.0;
const ARROW: &str = "M5 12h13m0 0-5.4-5.4M18 12l-5.4 5.4";
const BUTTON_SPRING: SpringConfig = SpringConfig {
    damping: 15.0,
    stiffness: 160.0,
    mass: 0.7,
    overshoot_clamping: false,
};

fn rise(by: f64) -> Style {
    Style::default().translate(Length::ZERO, Length::px(by))
}

#[expect(
    clippy::too_many_lines,
    reason = "the card's parts in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &EndCard, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, frame, fps) = (cx.at.stage.width, cx.at.stage.height, cx.frame(), cx.fps());
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let now = frame / fps * props.speed;
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[at(from), at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing_enter()),
                ..Interpolation::clamped()
            },
        )?)
    };
    let (stage_w, stage_h) = (
        f64::mul_add(side, -2.0, width),
        f64::mul_add(edge, -2.0, height),
    );
    let u = if height > width {
        (stage_w / 620.0).min(stage_h / 1120.0)
    } else {
        (stage_w / 1120.0).min(stage_h / 620.0)
    };
    let tu = u * props.text_size;
    let accent = props.accent_color.as_str();
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    let mut column = div(Style::default()
        .position(Position::Relative)
        .width(Length::px(stage_w.min(860.0 * u)))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(16.0 * u)
        .text_align(TextAlign::Center));
    if !props.logo_src.is_empty() {
        let logo_in = ease(LOGO_AT, LOGO_AT + 0.5)?;
        let mark = props.logo_size.map_or(76.0 * u, f64::from);
        column = column.child(img(
            props.logo_src.clone(),
            rise((1.0 - logo_in) * 12.0 * u)
                .width(Length::px(mark))
                .height(Length::px(mark))
                .object_fit(ObjectFit::Contain)
                .opacity(logo_in),
        ));
    }
    if !props.eyebrow.is_empty() {
        let eyebrow_in = ease(EYEBROW_AT, EYEBROW_AT + 0.45)?;
        let mut style = rise((1.0 - eyebrow_in) * 10.0 * u)
            .color(css_color(accent))
            .font_size(20.0 * tu)
            .font_weight(600.0)
            .text_transform(if props.eyebrow_caps {
                TextTransform::Uppercase
            } else {
                TextTransform::None
            })
            .opacity(eyebrow_in);
        if props.eyebrow_caps {
            style = style.letter_spacing(Length::em(0.14));
        }
        column = column.child(div(style).child(props.eyebrow.clone()));
    }
    let title_in = ease(TITLE_AT, TITLE_AT + 0.55)?;
    column = column.child(
        div(Style::default()
            .overflow(Overflow::Hidden)
            .padding_bottom(Length::em(0.04)))
        .child(
            div(Style::default()
                .margin(Edges::all(Length::ZERO))
                .color(css_color(palette.get("fg")))
                .font_size(68.0 * tu)
                .font_weight(f64::from(props.title_weight))
                .line_height(LineHeight::Number(Num(1.06)))
                .letter_spacing(Length::em(-0.03))
                .translate(Length::ZERO, Length::percent((1.0 - title_in) * 100.0)))
            .child(props.title.clone()),
        ),
    );
    if !props.subtitle.is_empty() {
        let subtitle_in = ease(SUBTITLE_AT, SUBTITLE_AT + 0.5)?;
        column = column.child(
            div(rise((1.0 - subtitle_in) * 12.0 * u)
                .max_width(Length::px(stage_w * 0.7))
                .color(css_color(palette.get("dim")))
                .font_size(28.0 * tu)
                .line_height(LineHeight::Number(Num(1.35)))
                .opacity(subtitle_in))
            .child(props.subtitle.clone()),
        );
    }
    if !props.cta.is_empty() {
        let button = sprung(frame - at(BUTTON_AT), fps, BUTTON_SPRING)?.min(1.0);
        let label = ease(BUTTON_LABEL_AT, BUTTON_LABEL_AT + 0.4)?;
        let pulse = clamped(frame, [at(PULSE_AT), at(PULSE_AT + 0.7)], [0.0, 1.0])?;
        let ring = interpolate(pulse, &[0.0, 1.0], &[1.0, 1.28], &Interpolation::default())?;
        let grow = interpolate(button, &[0.0, 1.0], &[0.9, 1.0], &Interpolation::default())?;
        let radius = f64::from(props.button_radius);
        let mut face = div(Style::default()
            .position(Position::Relative)
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(12.0 * u)
            .padding(Edges::axes(Length::px(17.0 * u), Length::px(34.0 * u)))
            .border_radius_all(radius)
            .background_color(css_color(accent))
            .color(css_color(palette.get("page")))
            .font_size(27.0 * tu)
            .font_weight(700.0)
            .letter_spacing(Length::em(-0.01))
            .box_shadow(vec![Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(14.0 * u),
                blur: Length::px(40.0 * u),
                spread: Length::ZERO,
                color: tinted(accent, 27.0),
            }])
            .max_width(Length::px(stage_w))
            .white_space(WhiteSpace::Nowrap)
            .opacity(button)
            .scale(grow, grow)
            .overflow(Overflow::Hidden))
        .child(
            span(
                Style::default()
                    .display(Display::Block)
                    .min_width(Length::ZERO)
                    .overflow(Overflow::Hidden)
                    .text_overflow(TextOverflow::Ellipsis)
                    .opacity(label)
                    .translate(Length::ZERO, Length::percent((1.0 - label) * 60.0)),
            )
            .child(props.cta.clone()),
        );
        if props.button_arrow {
            face = face.child(svg(
                SvgElement::new("svg")
                    .attr("width", 20.0 * u)
                    .attr("height", 20.0 * u)
                    .attr("viewBox", "0 0 24 24")
                    .attr("fill", "none")
                    .child(
                        SvgElement::new("path")
                            .attr("d", ARROW)
                            .attr("stroke", palette.get("page"))
                            .attr("stroke-width", 2.2)
                            .attr("stroke-linecap", "round")
                            .attr("stroke-linejoin", "round"),
                    ),
                Style::default().opacity(label),
            ));
        }
        column = column.child(
            div(Style::default()
                .position(Position::Relative)
                .margin_top(Length::px(10.0 * u)))
            .child(div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .border_radius_all(radius)
                .border(Edges::all(BorderSide {
                    width: Length::px(2.0 * u),
                    style: BorderStyle::Solid,
                    color: Some(css_color(accent)),
                }))
                .opacity(if pulse > 0.0 && pulse < 1.0 {
                    (1.0 - pulse) * 0.8
                } else {
                    0.0
                })
                .scale(ring, ring)))
            .child(face),
        );
    }
    if !props.url.is_empty() {
        let typed = ((now - URL_AT).max(0.0) * URL_CPS).floor();
        let length = float(utf16_len(&props.url));
        let shown: usize = truncate(typed.min(length));
        column = column.child(
            div(Style::default()
                .margin_top(Length::px(4.0 * u))
                .color(css_color(palette.get("dim")))
                .font_size(24.0 * tu)
                .font_weight(500.0)
                .display(Display::Flex)
                .align_items(Align::Center)
                .opacity(if typed > 0.0 { 1.0 } else { 0.0 }))
            .child(utf16_slice(&props.url, 0, shown))
            .child(span(
                Style::default()
                    .display(Display::InlineBlock)
                    .width(Length::px(2.0 * u))
                    .height(Length::px(24.0 * tu))
                    .margin_left(Length::px(3.0 * u))
                    .background_color(css_color(accent))
                    .opacity(if typed >= length { 0.0 } else { 1.0 }),
            )),
        );
    }
    if !props.handles.is_empty() {
        let handles_in = ease(HANDLES_AT, HANDLES_AT + 0.5)?;
        column = column.child(
            div(rise((1.0 - handles_in) * 10.0 * u)
                .margin_top(Length::px(10.0 * u))
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .justify_content(Align::Center)
                .row_gap(Length::px(8.0 * u))
                .column_gap(Length::px(20.0 * u))
                .color(css_color(palette.get("faint")))
                .font_size(21.0 * tu)
                .opacity(handles_in))
            .children(
                props
                    .handles
                    .iter()
                    .map(|h| span(Style::default()).child(h.clone()).into())
                    .collect::<Vec<Node>>(),
            ),
        );
    }
    let glow = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(60.0), Length::percent(46.0)),
        center: Position2 {
            x: Length::percent(50.0),
            y: Length::percent(44.0),
        },
        stops: vec![
            ColorStop::new(tinted(accent, f64::from(props.glow))),
            ColorStop::at(Color::TRANSPARENT, Length::percent(72.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    Ok(div(with_background(
        Style::default()
            .width(Length::px(width))
            .height(Length::px(height)),
        page,
    )
    .font_family(font_or(&props.font_family, FontRole::Main))
    .position(Position::Relative)
    .overflow(Overflow::Hidden)
    .display(Display::Grid)
    .align_items(Align::Center)
    .justify_items(Align::Center))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![glow.layer()])))
    .child(column)
    .into())
}
