use opsis_macros::settings;
use opsis_scene::css::{font_family, with_background};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::{Filter, Shadow, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{ObjectFit, Overflow, Position, Style, TextAlign};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length, Num, Px};
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::oklch_mix::{mix_oklch, with_alpha};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{WarmTheme, resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct HeroLaunch {
    #[setting(
        "Left card (image/video)",
        default = "app:placeholders/screen-wide.svg",
        widget = "asset"
    )]
    pub image1: String,
    #[setting(
        "Right card (image/video)",
        default = "app:backdrops/aurora.jpg",
        widget = "asset"
    )]
    pub image2: String,
    #[setting("Headline", default = "Launch day, done right")]
    pub heading: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Card tilt",
        default = 17,
        range(min = 0, max = 60),
        step = 1,
        unit = "°",
        advanced
    )]
    pub card_tilt: u32,
    #[setting(
        "Card gap",
        default = 92,
        range(min = 0, max = 400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_gap: u32,
    #[setting(
        "Card shape",
        default = 1.613,
        range(min = 0.5, max = 3),
        step = 0.01,
        help = "Card width divided by its height.",
        advanced
    )]
    pub card_aspect: f64,
    #[setting(
        "Card corners",
        default = 30,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_radius: u32,
    #[setting(
        "Headline size",
        default = 42,
        range(min = 12, max = 140),
        step = 1,
        unit = "px",
        advanced
    )]
    pub headline_size: u32,
    #[setting(
        "Headline weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub headline_weight: u32,
    #[setting(
        "Fade the edges",
        default = true,
        help = "The cards fade into the background at the sides and bottom.",
        advanced
    )]
    pub edge_fade: bool,
}

pub fn definition() -> ElementMeta {
    meta::<HeroLaunch>(
            element(
                ElementKind::Component,
                "hero-launch",
                "Hero launch",
                "Product launch hero scene for a SaaS launch video intro. Cinematic product-launch hero — two cards slide into formation as the headline reveals above.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .kind_prop("image1Kind", "image1")
                .kind_prop("image2Kind", "image2")
                .look(NodeLook::of("lucide:presentation")),
        )
}

const SANS: &str = "\"Inter\"";
const PERSP: f64 = 1000.0;
const OUT: Easing = Easing::Bezier(0.22, 1.0, 0.36, 1.0);
const FALL: Easing = Easing::Bezier(0.5, 0.0, 0.9, 0.4);

const fn or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

fn fill() -> Style {
    Style::default()
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
}

fn tilted(deg: f64, origin: f64, s: f64) -> Style {
    fill()
        .transform(vec![
            Transform::Perspective(Some(Px(PERSP * s))),
            Transform::RotateY(Angle::deg(deg)),
        ])
        .transform_origin((Length::percent(origin), Length::percent(50.0), Length::ZERO))
}

fn image_card(src: &str, s: f64, radius: f64, theme: &WarmTheme) -> Node {
    let deep = mix_oklch(theme.get("background"), "#000", 0.75);
    let mut card = div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .overflow(Overflow::Hidden)
        .border_radius_all(radius * s)
        .background_color(css_color(&mix_oklch(
            theme.get("background"),
            theme.get("card"),
            0.35,
        )))
        .box_shadow(vec![
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(20.0 * s),
                blur: Length::px(52.0 * s),
                spread: Length::px(-28.0 * s),
                color: css_color(&with_alpha(&deep, 0.9)),
            },
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::ZERO,
                blur: Length::ZERO,
                spread: Length::px(1.0),
                color: css_color(&with_alpha(theme.get("foreground"), 0.22)),
            },
        ]));
    if !src.is_empty() {
        card = card.child(img(
            src.to_owned(),
            Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .width(Length::percent(100.0))
                .height(Length::percent(100.0))
                .object_fit(ObjectFit::Cover),
        ));
    }
    card.child(div(with_background(
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO),
        &format!(
            "linear-gradient(to top, {} 0%, transparent 32%)",
            with_alpha(&deep, 0.28)
        ),
    )))
    .child(div(with_background(
        Style::default()
            .position(Position::Absolute)
            .left(Length::px(26.0 * s))
            .right(Length::px(26.0 * s))
            .top(Length::ZERO)
            .height(Length::px(1.0)),
        &format!(
            "linear-gradient(90deg,transparent,{},transparent)",
            with_alpha(theme.get("foreground"), 0.5)
        ),
    )))
    .into()
}

#[expect(
    clippy::too_many_lines,
    reason = "the two cards, the fades and the headline in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &HeroLaunch, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let face = resolve_font(or(&props.font_family, theme.font_family().unwrap_or("")))
        .map_or_else(|| font_family(SANS), |f| font_family(&f));
    let s = (height / 720.0).min(width / 1280.0);
    let card_w = 1000.0 * s;
    let card_h = card_w / props.card_aspect;
    let center_left = (width - card_w) / 2.0;
    let card_top = 0.3 * height;
    let rest = f64::from(props.card_gap).mul_add(s, card_w) / 2.0;
    let right_start = 1.08 * width;
    let group_final = 0.16 * height;
    let seg = |start: f64, duration: f64, easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[start, start + duration],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let line = |t: f64, from: f64, to: f64| {
        interpolate(t, &[0.0, 1.0], &[from, to], &Interpolation::default())
    };
    let recede = seg(40.0, 30.0, OUT)?;
    let angle = seg(46.0, 36.0, OUT)?;
    let slide_left = seg(46.0, 36.0, OUT)?;
    let slide_right = seg(60.0, 40.0, OUT)?;
    let drop = seg(122.0, 34.0, FALL)?;
    let title = seg(126.0, 40.0, OUT)?;
    let left_scale = line(recede, 1.5, 1.0)?;
    let left_x = line(slide_left, 0.0, -rest)?;
    let left_deg = line(angle, 0.0, -f64::from(props.card_tilt))?;
    let left_opacity = clamped(frame, [2.0, 12.0], [0.0, 1.0])?;
    let right_x = line(slide_right, right_start, rest)?;
    let right_opacity = interpolate(
        slide_right,
        &[0.0, 0.35],
        &[0.0, 1.0],
        &Interpolation::clamp_right(),
    )?;
    let float_env = seg(94.0, 22.0, OUT)? * (1.0 - drop);
    let float_y = (frame * 0.09).sin() * 3.5 * s * float_env;
    let group_y = line(drop, 0.0, group_final)? + float_y;
    let title_y = line(title, 40.0 * s, 0.0)?;
    let title_scale = line(title, 0.82, 1.0)?;
    let title_blur = line(title, 10.0, 0.0)?;
    let vignette = seg(50.0, 40.0, Easing::Linear)?;
    let card_box = |x: f64, opacity: f64| {
        Style::default()
            .position(Position::Absolute)
            .left(Length::px(center_left))
            .top(Length::px(card_top))
            .width(Length::px(card_w))
            .height(Length::px(card_h))
            .opacity(opacity)
            .transform(vec![Transform::Translate(
                Length::px(x),
                Length::ZERO,
                Length::ZERO,
            )])
    };
    let radius = f64::from(props.card_radius);
    let tilt = f64::from(props.card_tilt);
    let left = div(card_box(left_x, left_opacity)).child(
        div(fill()
            .transform(vec![Transform::Scale(
                Num(left_scale),
                Num(left_scale),
                Num(1.0),
            )])
            .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO)))
        .child(div(tilted(left_deg, 100.0, s)).child(image_card(&props.image1, s, radius, &theme))),
    );
    let right = div(card_box(right_x, right_opacity))
        .child(div(tilted(tilt, 0.0, s)).child(image_card(&props.image2, s, radius, &theme)));
    let mut root = div(absolute_fill()
        .background_color(css_color(theme.get("background")))
        .font_family(face))
    .child(
        div(absolute_fill().transform(vec![Transform::Translate(
            Length::ZERO,
            Length::px(group_y),
            Length::ZERO,
        )]))
        .child(left)
        .child(right),
    );
    if props.edge_fade {
        let ground = theme.get("background");
        let edge_fade = |side: &str, stop: f64| -> Node {
            div(with_background(
                Style::default()
                    .position(Position::Absolute)
                    .inset(Length::ZERO),
                &format!(
                    "linear-gradient(to {side}, {ground} 0%, transparent {}%)",
                    js_number_text(stop)
                ),
            ))
            .into()
        };
        root = root.child(
            div(absolute_fill().opacity(vignette))
                .child(edge_fade("left", 30.0))
                .child(edge_fade("right", 22.0))
                .child(edge_fade("top", 30.0)),
        );
    }
    let headline_size = f64::from(props.headline_size);
    Ok(root
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .right(Length::ZERO)
                .top(Length::px(0.22 * height))
                .padding(Edges::axes(Length::ZERO, Length::px(40.0 * s)))
                .text_align(TextAlign::Center)
                .color(css_color(theme.get("foreground")))
                .font_weight(f64::from(props.headline_weight))
                .letter_spacing(Length::em(-0.02))
                .font_size(headline_size * s)
                .opacity(title)
                .filter(vec![Filter::Blur(Length::px(title_blur))])
                .transform(vec![
                    Transform::Translate(Length::ZERO, Length::px(title_y), Length::ZERO),
                    Transform::Scale(Num(title_scale), Num(title_scale), Num(1.0)),
                ]))
            .child(props.heading.clone()),
        )
        .into())
}
