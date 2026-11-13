use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::{font_family, with_background};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, img, span, svg};
use opsis_scene::paint::{ClipPath, ColorStop, Gradient, LinearDirection};
use opsis_scene::style::{
    Align, Display, ObjectFit, Overflow, Position, Style, TextAlign, WhiteSpace,
};
use opsis_scene::values::{Angle, Edges, Length};
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoWipe {
    #[setting("Name", default = "Northwind")]
    pub wordmark: String,
    #[setting(
        "Logo",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*",
        help = "The mark that spins in front of the name."
    )]
    pub logo_src: String,
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
        "Name end colour",
        widget = "color",
        zero_label = "Base color",
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
        "Wipe time",
        default = 0.4667,
        range(min = 0.1667, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub draw_duration: f64,
    #[setting(
        "Mark size (× font)",
        default = 1.05,
        range(min = 0.4, max = 3),
        step = 0.05,
        advanced
    )]
    pub logo_scale: f64,
    #[setting(
        "Mark spin (turns)",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.25,
        advanced
    )]
    pub spin_turns: f64,
    #[setting(
        "Lockup hold",
        default = 0.4,
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub hold_duration: f64,
    #[setting(
        "Rush time",
        default = 0.8,
        range(min = 0.2, max = 2.6667),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub rush_duration: f64,
    #[setting(
        "Rush scale (× mark)",
        default = 42,
        range(min = 2, max = 80),
        step = 1,
        advanced
    )]
    pub rush_scale: u32,
    #[setting("Swing across (× width)", default = 0.55, range(min = -0.8, max = 0.8), step = 0.05, advanced)]
    pub rush_drift: f64,
    #[setting(
        "Swing arc (× width)",
        default = 0.25,
        range(min = 0, max = 0.33),
        step = 0.01,
        advanced
    )]
    pub swing_out: f64,
    #[setting(
        "Ink color (end of rush)",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub cover_color: String,
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
        help = "Set it to transparent to lay the logo over other layers."
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
    #[setting("Letter spacing", default = -0.02, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting(
        "Gap after mark",
        default = 0.3,
        range(min = 0, max = 1.5),
        unit = "× font size",
        advanced
    )]
    pub mark_gap: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoWipe>(
        element(
            ElementKind::Component,
            "logo-wipe",
            "Logo wipe",
            "Your name wipes in behind a spinning mark, then the mark rushes at the camera and floods the frame.",
            ElementCategory::Scene,
            56,
        )
        .authored_clock()
        .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
        .look(NodeLook::of("lucide:pen-tool")),
    )
}

const INK_LAG: f64 = 0.7;
const WIPE_EASE: Easing = Easing::Bezier(0.2, 0.6, 0.35, 1.0);
const RUSH_EASE: Easing = Easing::Bezier(0.45, 0.2, 0.45, 1.0);
const MARK: &str = "M589 3805 c-167 -41 -305 -149 -380 -297 -60 -120 -59 -84 -59 -1239 0 -1004 1 -1059 19 -1112 48 -147 148 -226 331 -262 100 -20 127 -33 186 -87 53 -49 74 -109 74 -212 1 -121 42 -196 133 -244 l52 -27 980 -3 c883 -3 986 -1 1042 13 75 21 133 67 171 139 l27 51 3 315 c2 173 2 342 0 374 l-3 60 -31 -50 c-36 -59 -101 -115 -170 -146 l-49 -23 -810 -3 c-591 -2 -825 0 -865 8 -128 28 -258 121 -325 234 -65 110 -66 128 -63 826 l3 625 24 58 c44 111 147 210 270 259 l56 23 830 3 c809 2 831 2 895 -18 80 -25 149 -74 198 -142 l37 -52 3 334 c3 367 -3 418 -56 494 -30 44 -98 91 -152 106 -22 6 -457 10 -1175 9 -923 -1 -1151 -3 -1196 -14z M3800 2983 c-327 -196 -598 -361 -602 -367 -4 -6 -7 -254 -5 -550 l2 -540 145 -91 c80 -50 208 -130 285 -177 77 -47 283 -174 457 -282 174 -108 326 -196 337 -196 11 0 27 7 35 16 14 14 16 141 16 1254 0 781 -4 1248 -10 1264 -7 20 -16 26 -37 26 -18 -1 -232 -123 -623 -357z";

fn perspective_scale(travel: f64, max_scale: f64) -> f64 {
    if max_scale <= 1.0 {
        return 1.0;
    }
    let p = 1.0 - 1.0 / max_scale;
    1.0 / p.mul_add(-travel.clamp(0.0, 0.9999), 1.0)
}

fn swing_world_x(travel: f64, drift: f64, out: f64) -> f64 {
    drift.mul_add(travel, -(out * (PI * travel).sin()))
}

fn swing_gain(max_scale: f64) -> f64 {
    (1..128)
        .map(|i| {
            let t = f64::from(i) / 128.0;
            (PI * t).sin() * perspective_scale(t, max_scale)
        })
        .fold(1.0, f64::max)
}

fn ink_reach(width: f64, height: f64, drift: f64) -> f64 {
    let furthest = width / 2.0 + drift;
    let horizontal = drift
        .abs()
        .max(furthest.abs())
        .max((width - drift).abs())
        .max((width - furthest).abs());
    horizontal.hypot(height / 2.0) * 1.05
}

const fn or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

#[expect(
    clippy::too_many_lines,
    reason = "the wipe, the rush and the flood in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &LogoWipe, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let ground = or(&props.background, theme.get("background"));
    let face = resolve_font(or(&props.font_family, theme.font_family().unwrap_or("")))
        .map_or_else(|| font_or("", FontRole::Main), |f| font_family(&f));
    let base = or(&props.base_color, theme.get("foreground"));
    let accent = or(&props.accent_color, theme.get("primary"));
    let emphasis = or(&props.highlighted_text_color, base);
    let font_size = f64::from(props.font_size);
    let eased = |range: [f64; 2], easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &range,
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let wipe = eased(
        [props.start_at, props.start_at + props.draw_duration],
        WIPE_EASE,
    )?;
    let edge = (1.0 - wipe) * 100.0;
    let rush_start = props.start_at + props.draw_duration + props.hold_duration;
    let travel = eased([rush_start, rush_start + props.rush_duration], RUSH_EASE)?;
    let rush_scale = f64::from(props.rush_scale);
    let mark_size = font_size * props.logo_scale;
    let rushed = perspective_scale(travel, rush_scale);
    let mark_box = mark_size * rushed;
    let diagonal = width.hypot(height);
    let world_drift = props.rush_drift * width / rush_scale;
    let world_bow = props.swing_out * width / swing_gain(rush_scale);
    let swing = rushed * swing_world_x(travel, world_drift, world_bow);
    let flood = clamped(mark_box, [diagonal * 0.55, diagonal * 1.45], [0.0, 1.0])?;
    let ink = swing * (1.0 - INK_LAG);
    let flood_radius = if flood <= 0.0 {
        0.0
    } else {
        interpolate(
            flood,
            &[0.0, 1.0],
            &[
                mark_box * 0.03,
                ink_reach(width, height, props.rush_drift * width * (1.0 - INK_LAG)),
            ],
            &Interpolation::default(),
        )?
    };
    let mark: Node = if props.logo_src.is_empty() {
        svg(
            SvgElement::new("svg")
                .attr("width", mark_size)
                .attr("height", mark_size)
                .attr("viewBox", "0 0 464 409")
                .attr("fill", "none")
                .child(
                    SvgElement::new("g")
                        .attr("transform", "translate(0,409) scale(0.1,-0.1)")
                        .child(SvgElement::new("path").attr("d", MARK).attr("fill", accent)),
                ),
            Style::default().display(Display::Block),
        )
        .into()
    } else {
        img(
            props.logo_src.clone(),
            Style::default()
                .display(Display::Block)
                .width(Length::px(mark_size))
                .height(Length::px(mark_size))
                .max_width(Length::Auto)
                .object_fit(ObjectFit::Contain),
        )
        .into()
    };
    let spin_scale = clamped(wipe, [0.0, 1.0], [0.55, 1.0])? * rushed;
    let cover = or(&props.cover_color, accent);
    let traveller = div(Style::default()
        .position(Position::Absolute)
        .top(Length::percent(50.0))
        .left(Length::percent(edge))
        .translate(
            Length::sum([
                Length::percent(-100.0),
                Length::px(font_size.mul_add(-0.16, swing)),
            ]),
            Length::percent(-50.0),
        ))
    .child(div(Style::default()
        .position(Position::Absolute)
        .left(Length::percent(50.0))
        .top(Length::percent(50.0))
        .translate(
            Length::sum([Length::percent(-50.0), Length::px(-(swing - ink))]),
            Length::percent(-50.0),
        )
        .border_radius_all(Length::percent(50.0))
        .background_color(css_color(cover))
        .width(Length::px(2.0 * flood_radius))
        .height(Length::px(2.0 * flood_radius))))
    .child(
        div(Style::default()
            .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO))
            .rotate(Angle::deg(
                (props.spin_turns * 360.0).mul_add(wipe, 30.0 * travel),
            ))
            .scale(spin_scale, spin_scale))
        .child(mark),
    );
    let highlight = span(
        Style::default()
            .display(Display::Block)
            .clip_path(ClipPath::Inset {
                edges: Edges {
                    top: Length::ZERO,
                    right: Length::ZERO,
                    bottom: Length::ZERO,
                    left: Length::percent(edge),
                },
                round: None,
            })
            .background(vec![
                Gradient::Linear {
                    direction: LinearDirection::Angle(Angle::deg(90.0)),
                    stops: vec![
                        ColorStop::new(css_color(accent)),
                        ColorStop::new(css_color(emphasis)),
                    ],
                    repeating: false,
                    interpolation: ColorInterpolation::DEFAULT,
                }
                .layer(),
            ])
            .background_clip_text(true)
            .color(Color::TRANSPARENT),
    )
    .child(props.wordmark.clone());
    let text = span(
        Style::default()
            .font_size(font_size)
            .font_weight(props.font_weight.value())
            .color(css_color(base))
            .letter_spacing(Length::em(props.letter_spacing))
            .font_family(face)
            .text_align(TextAlign::Center)
            .position(Position::Relative)
            .display(Display::InlineBlock)
            .white_space(WhiteSpace::Nowrap),
    )
    .child(highlight)
    .child(traveller);
    Ok(div(with_background(
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO),
        ground,
    )
    .display(Display::Flex)
    .align_items(Align::Center)
    .justify_content(Align::Center)
    .overflow(Overflow::Visible))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .padding_left(Length::px(font_size.mul_add(props.mark_gap, mark_size))))
        .child(text),
    )
    .into())
}
