use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::css::font_family;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{
    Align, Display, LineHeight, ObjectFit, Overflow, Position, Style, TextAlign, TextRendering,
    WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::{mix_oklch, with_alpha};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct LogoAssembleImage {
    #[setting(
        "Image",
        default = "app:backdrops/aurora.jpg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
}

#[settings(export)]
pub struct LogoAssemble {
    #[setting("Brand name", default = "Acme")]
    pub brand_name: String,
    #[setting("Center text", default = "Product demos, made simple")]
    pub middle_text: String,
    #[setting(
        "Logo (simple, monochrome)",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset"
    )]
    pub logo_src: String,
    #[setting(
        "Ring images",
        default = 10,
        range(min = 5, max = 16),
        step = 1,
        advanced
    )]
    pub count: u32,
    #[setting("Background", widget = "color", zero_label = "Theme")]
    pub background: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Images",
        default = vec![
            LogoAssembleImage::default(),
            LogoAssembleImage { image: "app:backdrops/bloom.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/canyon.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/chalk.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/cobalt.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/dusk.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/ember.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/flare.jpg".into() },
        ],
        length(min = 1)
    )]
    pub images: Vec<LogoAssembleImage>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Orbit time",
        default = 1.67,
        range(min = 0.3, max = 10),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "How long the cards circle before they gather into the logo.",
        advanced
    )]
    pub orbit_seconds: f64,
    #[setting(
        "Orbit speed",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.05,
        advanced
    )]
    pub orbit_speed: f64,
    #[setting(
        "Card shape",
        default = 1.22,
        range(min = 0.5, max = 2),
        step = 0.01,
        help = "Card height divided by its width.",
        advanced
    )]
    pub card_ratio: f64,
    #[setting(
        "Card shadow",
        default = 0.4,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub card_shadow: f64,
    #[setting(
        "Card corners",
        default = 4,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_radius: u32,
    #[setting(
        "Ring size",
        default = 36.0,
        range(min = 10, max = 48),
        step = 0.5,
        unit = "%",
        help = "As a share of the frame's short side.",
        advanced
    )]
    pub ring_size: f64,
    #[setting(
        "Card size",
        default = 15.0,
        range(min = 4, max = 30),
        step = 0.5,
        unit = "%",
        advanced
    )]
    pub card_size: f64,
    #[setting(
        "Logo size",
        default = 12.5,
        range(min = 4, max = 40),
        step = 0.5,
        unit = "%"
    )]
    pub logo_size: f64,
    #[setting(
        "Logo gap",
        default = 3.5,
        range(min = 0, max = 15),
        step = 0.5,
        unit = "%",
        advanced
    )]
    pub logo_gap: f64,
    #[setting(
        "Name size",
        default = 5.5,
        range(min = 2, max = 15),
        step = 0.1,
        unit = "%"
    )]
    pub name_size: f64,
    #[setting(
        "Center text size",
        default = 4.0,
        range(min = 1, max = 10),
        step = 0.1,
        unit = "%",
        advanced
    )]
    pub middle_text_size: f64,
    #[setting(
        "Font weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
}

pub fn definition() -> ElementMeta {
    meta::<LogoAssemble>(
            element(
                ElementKind::Component,
                "logo-assemble",
                "Logo assemble",
                "Logo reveal animation: image cards orbit and collapse into your logo and brand name. A single ring of image cards revolves around a centred title, then spins inward and drains to the middle — giving birth to a simple logo that slides left as the brand name reveals to its right.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:orbit")),
        )
}

struct Beats {
    contract_start: f64,
    contract_end: f64,
    middle_out: f64,
    logo_in: f64,
    logo_shift_start: f64,
    logo_shift_end: f64,
    name_in: f64,
}

const fn beats_from(contract_start: f64) -> Beats {
    Beats {
        contract_start,
        contract_end: contract_start + 16.0,
        middle_out: contract_start + 2.0,
        logo_in: contract_start + 8.0,
        logo_shift_start: contract_start + 24.0,
        logo_shift_end: contract_start + 40.0,
        name_in: contract_start + 24.0,
    }
}

const ORBIT_SPEED: f64 = 0.1;
const DRAIN_SPIN: f64 = std::f64::consts::PI * 1.1;
const EASE: Easing = Easing::Bezier(0.2, 0.6, 0.35, 1.0);
const EASE_IN: Easing = Easing::Bezier(0.5, 0.0, 0.75, 0.4);

fn eased(input: f64, from: &[f64], to: &[f64], easing: Easing) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        input,
        from,
        to,
        &Interpolation {
            easing: Some(easing),
            ..Interpolation::clamped()
        },
    )?)
}

fn contract_progress(frame: f64, b: &Beats) -> Result<f64, DrawFailure> {
    eased(
        frame,
        &[b.contract_start, b.contract_end],
        &[0.0, 1.0],
        EASE_IN,
    )
}

fn middle_text_opacity(frame: f64, b: &Beats) -> Result<f64, DrawFailure> {
    let fade_in = 16.0f64.min(b.middle_out - 6.0);
    Ok(interpolate(
        frame,
        &[
            6.0f64.min(fade_in - 1.0),
            fade_in,
            b.middle_out - 6.0,
            b.middle_out,
        ],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places the ring and the logo with separate multiplies and adds"
)]
pub fn draw(props: &LogoAssemble, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let images: Vec<&str> = props
        .images
        .iter()
        .map(|row| row.image.as_str())
        .filter(|src| !src.trim().is_empty())
        .collect();
    let frame = cx.authored_frame() * props.speed;
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let face = resolve_font(if props.font_family.is_empty() {
        theme.font_family().unwrap_or("")
    } else {
        &props.font_family
    })
    .map_or_else(|| font_or("", FontRole::Main), |f| font_family(&f));
    let stage = if props.background.is_empty() {
        theme.get("background")
    } else {
        &props.background
    };
    let card_shadow = with_alpha(&mix_oklch(stage, "#000", 0.75), props.card_shadow);
    let beats = beats_from(props.orbit_seconds);
    let (center_x, center_y) = (width / 2.0, height / 2.0);
    let short = width.min(height);
    let ring = short * props.ring_size / 100.0;
    let card_w = js_round(short * props.card_size / 100.0);
    let card_h = card_w * props.card_ratio;
    let contract = contract_progress(frame, &beats)?;
    let angle = frame * ORBIT_SPEED * props.orbit_speed + contract * DRAIN_SPIN;
    let ring_scale = 1.0 - contract;
    let card_scale = 1.0 - 0.66 * contract;
    let card_opacity = clamped(
        frame,
        [beats.contract_end - 8.0, beats.contract_end],
        [1.0, 0.0],
    )?;
    let middle_opacity = middle_text_opacity(frame, &beats)?;
    let logo_opacity = clamped(frame, [beats.logo_in, beats.logo_in + 8.0], [0.0, 1.0])?;
    let logo_scale = eased(
        frame,
        &[beats.logo_in, beats.logo_in + 10.0, beats.logo_in + 18.0],
        &[0.6, 1.05, 1.0],
        EASE,
    )?;
    let shift = eased(
        frame,
        &[beats.logo_shift_start, beats.logo_shift_end],
        &[0.0, 1.0],
        EASE,
    )?;
    let name_opacity = clamped(frame, [beats.name_in, beats.name_in + 10.0], [0.0, 1.0])?;
    let name_dx = eased(
        frame,
        &[beats.name_in, beats.name_in + 12.0],
        &[18.0, 0.0],
        EASE,
    )?;
    let count = props.count.max(1);
    let logo_size = js_round(short * props.logo_size / 100.0);
    let gap = js_round(short * props.logo_gap / 100.0);
    let rest_x = short * 0.13;
    let logo_x = center_x - rest_x * shift;
    let name_left = center_x - rest_x + logo_size / 2.0 + gap;
    let name_size = js_round(short * props.name_size / 100.0);
    let foreground = css_color(theme.get("foreground"));
    let weight = f64::from(props.font_weight);
    let mut root = div(absolute_fill()
        .overflow(Overflow::Hidden)
        .background_color(css_color(stage))
        .font_family(face)
        .text_rendering(TextRendering::GeometricPrecision));
    if card_opacity > 0.0 && !images.is_empty() {
        for i in 0..count {
            let theta = f64::from(i) / f64::from(count) * std::f64::consts::PI * 2.0 + angle;
            let x = center_x + theta.cos() * ring * ring_scale;
            let y = center_y + theta.sin() * ring * ring_scale;
            let source = images
                .get(usize::try_from(i).unwrap_or(0) % images.len())
                .copied()
                .unwrap_or_default();
            root = root.child(
                div(Style::default()
                    .position(Position::Absolute)
                    .left(Length::px(x - card_w / 2.0))
                    .top(Length::px(y - card_h / 2.0))
                    .width(Length::px(card_w))
                    .height(Length::px(card_h))
                    .opacity(card_opacity)
                    .scale(card_scale, card_scale)
                    .overflow(Overflow::Hidden)
                    .border_radius_all(f64::from(props.card_radius))
                    .box_shadow(vec![Shadow {
                        inset: false,
                        x: Length::ZERO,
                        y: Length::px(14.0),
                        blur: Length::px(34.0),
                        spread: Length::ZERO,
                        color: css_color(&card_shadow),
                    }]))
                .child(img(
                    source.to_owned(),
                    Style::default()
                        .position(Position::Absolute)
                        .inset(Length::ZERO)
                        .width(Length::percent(100.0))
                        .height(Length::percent(100.0))
                        .object_fit(ObjectFit::Cover),
                )),
            );
        }
    }
    if middle_opacity > 0.0 && !props.middle_text.is_empty() {
        root = root.child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::px(center_x - ring))
                .top(Length::px(center_y - card_h))
                .width(Length::px(ring * 2.0))
                .height(Length::px(card_h * 2.0))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::Center)
                .text_align(TextAlign::Center)
                .opacity(middle_opacity)
                .color(foreground.clone())
                .font_size(js_round(short * props.middle_text_size / 100.0))
                .font_weight(weight)
                .line_height(LineHeight::Number(Num(1.2)))
                .letter_spacing(Length::em(-0.02))
                .white_space(WhiteSpace::PreLine))
            .child(props.middle_text.clone()),
        );
    }
    if logo_opacity > 0.0 && !props.logo_src.trim().is_empty() {
        root = root.child(img(
            props.logo_src.clone(),
            Style::default()
                .position(Position::Absolute)
                .left(Length::px(logo_x - logo_size / 2.0))
                .top(Length::px(center_y - logo_size / 2.0))
                .width(Length::px(logo_size))
                .height(Length::px(logo_size))
                .object_fit(ObjectFit::Contain)
                .opacity(logo_opacity)
                .scale(logo_scale, logo_scale),
        ));
    }
    if name_opacity > 0.0 {
        root = root.child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::px(name_left))
                .top(Length::px(center_y - name_size))
                .height(Length::px(name_size * 2.0))
                .display(Display::Flex)
                .align_items(Align::Center)
                .opacity(name_opacity)
                .transform(vec![Transform::Translate(
                    Length::px(name_dx),
                    Length::ZERO,
                    Length::ZERO,
                )])
                .color(foreground)
                .font_size(name_size)
                .font_weight(weight)
                .letter_spacing(Length::em(-0.02))
                .white_space(WhiteSpace::Nowrap))
            .child(props.brand_name.clone()),
        );
    }
    Ok(root.into())
}
