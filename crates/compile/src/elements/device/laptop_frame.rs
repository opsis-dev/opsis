use opsis_macros::{choice, settings};
use opsis_scene::css::{font_family, shadows, with_background};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Element, Node, div, span};
use opsis_scene::paint::Transform;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style,
    TextRendering, WhiteSpace,
};
use opsis_scene::timeline::ChildAt;
use opsis_scene::values::{Angle, Corners, Edges, Length, Num, Px};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::oklch_mix::{mix_oklch, with_alpha};
use crate::elements::parts::device_screen::float_offset;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{WarmTheme, resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum LaptopEntrance {
    Rise,
    Open,
    None,
}

#[choice(export)]
pub enum LaptopFinale {
    #[label("None")]
    None,
    #[label("Zoom into the screen")]
    ZoomToScreen,
}

#[settings(export)]
pub struct LaptopFrame {
    #[setting("Entrance", default = LaptopEntrance::Open)]
    pub entrance: LaptopEntrance,
    #[setting("Finale", default = LaptopFinale::ZoomToScreen)]
    pub finale: LaptopFinale,
    #[setting("Notch label", default = "Recording started")]
    pub notch_label: String,
    #[setting("Notch", default = true)]
    pub show_notch: bool,
    #[setting(
        "Battery level",
        default = 85,
        range(min = 0, max = 100),
        step = 1,
        advanced
    )]
    pub battery_level: u32,
    #[setting("Bezel color", default = "#1D1D1F", check = colour)]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#FFFFFF",
        widget = "color",
        zero_label = "Theme",
        advanced
    )]
    pub screen_color: String,
    #[setting("Lid tilt", default = 18, range(min = 0, max = 45), step = 1, advanced)]
    pub rest_tilt: u32,
    #[setting(
        "Lid radius",
        default = 22,
        range(min = 8, max = 40),
        step = 1,
        advanced
    )]
    pub radius: u32,
    #[setting("Scale", default = 0.85, range(min = 0.4, max = 1.4), step = 0.05)]
    pub scale: f64,
    #[setting("Float loop", default = true, advanced)]
    pub float_loop: bool,
    #[setting(
        "Float amplitude",
        default = 4,
        range(min = 0, max = 16),
        step = 1,
        advanced
    )]
    pub float_amplitude: u32,
    #[setting("Shadow", default = "0 40px 80px rgba(10,12,20,0.45)", advanced)]
    pub shadow: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Recording light", default = "#30D158", check = colour, advanced)]
    pub indicator_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Notch at",
        default = 4.0,
        range(min = 1.6, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "When the notch shows its label. It starts loading 1.6 s before.",
        advanced
    )]
    pub notch_at: f64,
    #[setting(
        "Zoom at",
        default = 4.6667,
        range(min = 0, max = 30),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "When the finale zooms into the screen.",
        advanced
    )]
    pub zoom_at: f64,
    #[setting(
        "Zoom length",
        default = 1.5,
        range(min = 0.1, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub zoom_length: f64,
    #[setting(
        "Float period",
        default = 5.0,
        range(min = 1, max = 20),
        step = 0.1,
        unit = "s",
        show_if = { "floatLoop": ["true"] },
        advanced
    )]
    pub float_period: f64,
    #[setting(
        "Notch label size",
        default = 12,
        range(min = 8, max = 24),
        step = 1,
        unit = "px",
        advanced
    )]
    pub notch_label_size: u32,
}

pub fn definition() -> ElementMeta {
    meta::<LaptopFrame>(
            element(
                ElementKind::Component,
                "laptop-frame",
                "Laptop frame",
                "A laptop that opens, shows a notification in its notch, then moves the camera into the screen until your clip fills the frame.",
                ElementCategory::Device,
                240,
            )
                .authored_clock()
                .design(1920, 1080)
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:laptop")),
        )
}

const LID_WIDTH: f64 = 820.0;
const LID_HEIGHT: f64 = 500.0;
const BEZEL_WIDTH: f64 = 14.0;
const DECK_WIDTH: f64 = 900.0;
const DECK_HEIGHT: f64 = 22.0;
const PERSPECTIVE: f64 = 1600.0;
const ENTRANCE_FRAMES: f64 = 30.0;
const CAMERA_EASE: Easing = Easing::Bezier(0.2, 0.6, 0.35, 1.0);
const COLUMN_HEIGHT: f64 = LID_HEIGHT + DECK_HEIGHT;
const NOTCH_HEIGHT: f64 = 30.0;
const NOTCH_IDLE_WIDTH: f64 = 116.0;
const NOTCH_LOADING_WIDTH: f64 = 150.0;
const NOTCH_DONE_WIDTH: f64 = 320.0;
const NOTCH_LOADING_FOR: f64 = 48.0;

fn eased(frame: f64, from: &[f64], to: &[f64], easing: Easing) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        from,
        to,
        &Interpolation {
            easing: Some(easing),
            ..Interpolation::clamped()
        },
    )?)
}

fn corners(top: f64, bottom: f64) -> Corners<(Length, Length)> {
    let r = |v: f64| (Length::px(v), Length::px(v));
    Corners {
        top_left: r(top),
        top_right: r(top),
        bottom_right: r(bottom),
        bottom_left: r(bottom),
    }
}

fn dot(size: f64, color: &str) -> Style {
    Style::default()
        .width(Length::px(size))
        .height(Length::px(size))
        .border_radius_all(999.0)
        .background_color(css_color(color))
}

fn layer(gap: f64, opacity: f64) -> Element {
    div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .gap(gap)
        .opacity(opacity))
}

struct NotchParts<'a> {
    frame: f64,
    label: &'a str,
    label_size: f64,
    battery: f64,
    indicator: &'a str,
    done_at: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film pulses the loading dots with separate multiplies and adds"
)]
fn notch(parts: &NotchParts<'_>, shell: &WarmTheme) -> Result<Node, DrawFailure> {
    let frame = parts.frame;
    let done = parts.done_at;
    let loading_at = done - NOTCH_LOADING_FOR;
    let width = eased(
        frame,
        &[loading_at - 8.0, loading_at, done - 4.0, done + 12.0],
        &[
            NOTCH_IDLE_WIDTH,
            NOTCH_LOADING_WIDTH,
            NOTCH_LOADING_WIDTH,
            NOTCH_DONE_WIDTH,
        ],
        Easing::ease_in_out(Easing::Cubic),
    )?;
    let idle = clamped(frame, [loading_at - 6.0, loading_at], [1.0, 0.0])?;
    let loading = interpolate(
        frame,
        &[loading_at - 6.0, loading_at, done - 6.0, done],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    let finished = clamped(frame, [done - 4.0, done + 6.0], [0.0, 1.0])?;
    let (ground, ink) = (shell.get("background"), shell.get("foreground"));
    let idle_layer = layer(6.0, idle)
        .child(div(dot(9.0, &mix_oklch(ground, ink, 0.12)).border(
            Edges::all(BorderSide {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(css_color(&mix_oklch(ground, ink, 0.2))),
            }),
        )))
        .child(div(dot(5.0, &mix_oklch(ground, ink, 0.06))));
    let loading_layer = layer(4.0, loading).children((0..3).map(|i| {
        div(dot(5.0, ink).opacity(0.4 + 0.6 * (0.5 + 0.5 * (frame / 4.0 - float(i) * 0.9).sin())))
            .into()
    }));
    let done_layer = layer(8.0, finished)
        .child(
            span(
                Style::default()
                    .font_size(parts.label_size)
                    .font_weight(500.0)
                    .letter_spacing(Length::em(-0.01))
                    .color(css_color(ink))
                    .white_space(WhiteSpace::Nowrap),
            )
            .child(parts.label.to_owned()),
        )
        .child(
            div(Style::default()
                .width(Length::px(22.0))
                .height(Length::px(11.0))
                .border_radius_all(3.0)
                .border(Edges::all(BorderSide {
                    width: Length::px(1.5),
                    style: BorderStyle::Solid,
                    color: Some(css_color(parts.indicator)),
                }))
                .padding(Edges::all(Length::px(1.5)))
                .display(Display::Flex)
                .align_items(Align::Center))
            .child(div(Style::default()
                .width(Length::percent(parts.battery.clamp(0.0, 100.0)))
                .height(Length::percent(100.0))
                .border_radius_all(1.0)
                .background_color(css_color(parts.indicator)))),
        );
    Ok(div(Style::default()
        .position(Position::Absolute)
        .top(Length::px(9.0))
        .left(Length::percent(50.0))
        .translate(Length::percent(-50.0), Length::ZERO)
        .width(Length::px(width))
        .height(Length::px(NOTCH_HEIGHT))
        .border_radius_all(999.0)
        .background_color(css_color(ground))
        .overflow(Overflow::Hidden)
        .z_index(10))
    .child(idle_layer)
    .child(loading_layer)
    .child(done_layer)
    .into())
}

#[expect(
    clippy::too_many_lines,
    clippy::suboptimal_flops,
    reason = "one pass over the laptop like the film's component, with its separate multiplies and adds"
)]
pub fn wrap(props: &LaptopFrame, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let child = child(cx.at.stage)?;
    let frame = cx.authored_frame() * props.speed;
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let shell = warm_theme(&Theme {
        mode: ThemeMode::Dark,
        colors: props.theme.colors.clone(),
    });
    let face = resolve_font(if props.font_family.is_empty() {
        theme.font_family().unwrap_or("")
    } else {
        &props.font_family
    })
    .map_or_else(|| font_or("", FontRole::Main), |f| font_family(&f));
    let lid = if props.bezel_color.is_empty() {
        mix_oklch(shell.get("background"), shell.get("foreground"), 0.09)
    } else {
        props.bezel_color.clone()
    };
    let glass = if props.screen_color.is_empty() {
        theme.get("card")
    } else {
        &props.screen_color
    };
    let zoom = [props.zoom_at, props.zoom_at + props.zoom_length];
    let zooms = props.finale == LaptopFinale::ZoomToScreen;
    let rest = f64::from(props.rest_tilt);
    let cubic = Easing::ease_out(Easing::Cubic);
    let (opacity, rise) = match props.entrance {
        LaptopEntrance::None => (1.0, 0.0),
        LaptopEntrance::Open => (
            clamped(frame, [0.0, 10.0], [0.0, 1.0])?,
            eased(frame, &[0.0, 24.0], &[24.0, 0.0], cubic.clone())?,
        ),
        LaptopEntrance::Rise => (
            clamped(frame, [0.0, 12.0], [0.0, 1.0])?,
            eased(frame, &[0.0, 26.0], &[70.0, 0.0], cubic.clone())?,
        ),
    };
    let tilt = if zooms && frame >= zoom[0] {
        eased(frame, &zoom, &[rest, 0.0], CAMERA_EASE)?
    } else if props.entrance == LaptopEntrance::Open {
        eased(frame, &[0.0, ENTRANCE_FRAMES], &[82.0, rest], cubic)?
    } else {
        rest
    };
    let (camera_scale, camera_y) = if zooms {
        let p = eased(frame, &zoom, &[0.0, 1.0], CAMERA_EASE)?;
        let inner_w = (LID_WIDTH - 2.0 * BEZEL_WIDTH) * props.scale;
        let inner_h = (LID_HEIGHT - 2.0 * BEZEL_WIDTH) * props.scale;
        let cover = if width >= height {
            (width / inner_w).max(height / inner_h)
        } else {
            (width / inner_w).min(height / inner_h)
        };
        let d = (COLUMN_HEIGHT / 2.0 - LID_HEIGHT / 2.0) * props.scale;
        (1.0 + (cover - 1.0) * p, d * cover * p)
    } else {
        (1.0, 0.0)
    };
    let bob = if props.float_loop && !(zooms && frame >= zoom[0]) {
        float_offset(
            frame,
            Cx::authored_fps(),
            f64::from(props.float_amplitude),
            ENTRANCE_FRAMES,
            props.float_period,
        )
    } else {
        0.0
    };
    let notch_fade = if zooms {
        clamped(frame, [zoom[0], zoom[0] + 10.0], [1.0, 0.0])?
    } else {
        1.0
    };
    let screen_radius = 4.0f64.max(f64::from(props.radius) - BEZEL_WIDTH);
    let mut screen = div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .border_radius(corners(screen_radius, 4.0))
        .background_color(css_color(glass))
        .overflow(Overflow::Hidden))
    .child(child);
    if props.show_notch && notch_fade > 0.0 {
        screen = screen.child(div(Style::default().opacity(notch_fade)).child(notch(
            &NotchParts {
                frame,
                label: &props.notch_label,
                label_size: f64::from(props.notch_label_size),
                battery: f64::from(props.battery_level),
                indicator: &props.indicator_color,
                done_at: props.notch_at,
            },
            &shell,
        )?));
    }
    let lid_box = div(Style::default()
        .width(Length::px(LID_WIDTH))
        .height(Length::px(LID_HEIGHT))
        .border_radius(corners(f64::from(props.radius), 6.0))
        .background_color(css_color(&lid))
        .padding(Edges::all(Length::px(BEZEL_WIDTH)))
        .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO))
        .transform(vec![Transform::RotateX(Angle::deg(tilt))]))
    .child(screen);
    let mut deck_style = with_background(
        Style::default()
            .position(Position::Relative)
            .width(Length::px(DECK_WIDTH))
            .height(Length::px(DECK_HEIGHT))
            .border_radius(corners(6.0, 40.0)),
        &format!(
            "linear-gradient(to bottom, {}, {} 34%, {}), {lid}",
            with_alpha(shell.get("foreground"), 0.16),
            with_alpha(shell.get("background"), 0.0),
            with_alpha(shell.get("background"), 0.34),
        ),
    );
    if !props.shadow.is_empty()
        && let Some(drop) = shadows(&props.shadow)
    {
        deck_style = deck_style.box_shadow(drop);
    }
    let deck = div(deck_style).child(div(Style::default()
        .position(Position::Absolute)
        .top(Length::ZERO)
        .left(Length::percent(50.0))
        .translate(Length::percent(-50.0), Length::ZERO)
        .width(Length::px(132.0))
        .height(Length::px(8.0))
        .border_radius(corners(0.0, 6.0))
        .background_color(css_color(&with_alpha(shell.get("background"), 0.28)))));
    let grow = props.scale;
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(face))
    .child(
        div(Style::default()
            .transform(vec![
                Transform::Translate(Length::ZERO, Length::px(camera_y), Length::ZERO),
                Transform::Scale(Num(camera_scale), Num(camera_scale), Num(1.0)),
            ])
            .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO))
            .text_rendering(TextRendering::GeometricPrecision))
        .child(
            div(Style::default()
                .opacity(opacity)
                .translate(Length::ZERO, Length::px(rise + bob))
                .scale(grow, grow))
            .child(
                div(Style::default()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .align_items(Align::Center)
                    .perspective(Px(PERSPECTIVE)))
                .child(lid_box)
                .child(deck),
            ),
        ),
    )
    .into())
}
