use opsis_macros::{choice, settings};
use opsis_scene::css::{font_family, shadows};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::paint::Transform;
use opsis_scene::style::{Align, BorderSide, BorderStyle, Display, Overflow, Position, Style};
use opsis_scene::timeline::ChildAt;
use opsis_scene::values::{Angle, Edges, Length, Px};
use opsis_types::looks::NodeLook;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::parts::device_screen::float_offset;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum PhoneVariant {
    #[label("Straight on")]
    Flat,
    #[label("Turned")]
    Tilt,
}

#[choice(export)]
pub enum PhoneEntrance {
    #[label("Slide in")]
    Rise,
    #[label("Turn in")]
    RotateIn,
    #[label("Float")]
    Float,
}

#[settings(export)]
pub struct PhoneFrame {
    #[setting("Variant", default = PhoneVariant::Flat)]
    pub variant: PhoneVariant,
    #[setting("Entrance", default = PhoneEntrance::Float)]
    pub entrance: PhoneEntrance,
    #[setting(
        "Bezel color",
        default = "#C7C9CE",
        widget = "color",
        zero_label = "Theme"
    )]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#0A0A0B",
        widget = "color",
        zero_label = "Theme",
        advanced
    )]
    pub screen_color: String,
    #[setting(
        "Body radius",
        default = 53,
        range(min = 24, max = 90),
        step = 1,
        advanced
    )]
    pub radius: u32,
    #[setting(
        "Screen radius",
        default = 41,
        range(min = 12, max = 78),
        step = 1,
        advanced
    )]
    pub screen_radius: u32,
    #[setting("Scale", default = 0.92, range(min = 0.4, max = 1.5), step = 0.05)]
    pub scale: f64,
    #[setting("Tilt angle", default = -12, range(min = -30, max = 30), step = 1, advanced)]
    pub tilt_angle: i32,
    #[setting("Dynamic island", default = true)]
    pub show_dynamic_island: bool,
    #[setting("Float loop")]
    pub float_loop: bool,
    #[setting(
        "Float amplitude",
        default = 6,
        range(min = 0, max = 20),
        step = 1,
        advanced
    )]
    pub float_amplitude: u32,
    #[setting("Shadow", default = "0 30px 70px rgba(16,24,40,0.28)", advanced)]
    pub shadow: String,
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
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Float period",
        default = 4.5,
        range(min = 1, max = 20),
        step = 0.1,
        unit = "s",
        show_if = { "floatLoop": ["true"] },
        advanced
    )]
    pub float_period: f64,
}

pub fn definition() -> ElementMeta {
    meta::<PhoneFrame>(
            element(
                ElementKind::Component,
                "phone-frame",
                "Phone frame",
                "A phone frame with a notch and a screen slot for your clip, flat or tilted, with rise, rotate-in or float entrances.",
                ElementCategory::Device,
                240,
            )
                .authored_clock()
                .design(1920, 1080)
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:smartphone")),
        )
}

const DEVICE_WIDTH: f64 = 320.0;
const DEVICE_HEIGHT: f64 = 656.0;
const BEZEL_WIDTH: f64 = 12.0;
const ENTRANCE_FRAMES: f64 = 30.0;
const SIDE_BUTTONS: [(bool, f64, f64); 4] = [
    (true, 118.0, 26.0),
    (true, 166.0, 46.0),
    (true, 222.0, 46.0),
    (false, 176.0, 68.0),
];

struct Pose {
    opacity: f64,
    rise: f64,
    turn: f64,
    scale: f64,
}

fn eased(frame: f64, to: f64, range: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &[0.0, to],
        &range,
        &Interpolation {
            easing: Some(Easing::ease_out(Easing::Cubic)),
            ..Interpolation::clamped()
        },
    )?)
}

fn pose(frame: f64, entrance: PhoneEntrance) -> Result<Pose, DrawFailure> {
    Ok(match entrance {
        PhoneEntrance::RotateIn => Pose {
            opacity: clamped(frame, [0.0, 12.0], [0.0, 1.0])?,
            rise: eased(frame, 28.0, [60.0, 0.0])?,
            turn: eased(frame, 28.0, [-8.0, 0.0])?,
            scale: eased(frame, 28.0, [0.94, 1.0])?,
        },
        PhoneEntrance::Float => Pose {
            opacity: clamped(frame, [0.0, 16.0], [0.0, 1.0])?,
            rise: 0.0,
            turn: 0.0,
            scale: eased(frame, 24.0, [0.96, 1.0])?,
        },
        PhoneEntrance::Rise => Pose {
            opacity: clamped(frame, [0.0, 12.0], [0.0, 1.0])?,
            rise: eased(frame, 26.0, [70.0, 0.0])?,
            turn: 0.0,
            scale: 1.0,
        },
    })
}

pub fn wrap(props: &PhoneFrame, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let child = child(cx.at.stage)?;
    let frame = cx.authored_frame() * props.speed;
    let theme = warm_theme(&props.theme);
    let island = warm_theme(&Theme {
        mode: ThemeMode::Dark,
        colors: props.theme.colors.clone(),
    });
    let face = resolve_font(if props.font_family.is_empty() {
        theme.font_family().unwrap_or("")
    } else {
        &props.font_family
    })
    .map_or_else(|| font_or("", FontRole::Main), |f| font_family(&f));
    let body = if props.bezel_color.is_empty() {
        theme.get("foreground")
    } else {
        &props.bezel_color
    };
    let glass = if props.screen_color.is_empty() {
        theme.get("card")
    } else {
        &props.screen_color
    };
    let pose = pose(frame, props.entrance)?;
    let start = if props.entrance == PhoneEntrance::Float {
        0.0
    } else {
        ENTRANCE_FRAMES
    };
    let bob = if props.float_loop {
        float_offset(
            frame,
            Cx::authored_fps(),
            f64::from(props.float_amplitude),
            start,
            props.float_period,
        )
    } else {
        0.0
    };
    let tilt = if props.variant == PhoneVariant::Tilt {
        f64::from(props.tilt_angle)
    } else {
        0.0
    };
    let mut body_style = Style::default()
        .position(Position::Relative)
        .width(Length::px(DEVICE_WIDTH))
        .height(Length::px(DEVICE_HEIGHT))
        .border_radius_all(f64::from(props.radius))
        .background_color(css_color(body))
        .padding(Edges::all(Length::px(BEZEL_WIDTH)));
    if !props.shadow.is_empty()
        && let Some(drop) = shadows(&props.shadow)
    {
        body_style = body_style.box_shadow(drop);
    }
    let mut device = div(body_style);
    for (left, top, length) in SIDE_BUTTONS {
        let style = Style::default()
            .position(Position::Absolute)
            .top(Length::px(top))
            .width(Length::px(3.0))
            .height(Length::px(length))
            .border_radius_all(2.0)
            .background_color(css_color(body));
        device = device.child(div(if left {
            style.left(Length::px(-3.0))
        } else {
            style.right(Length::px(-3.0))
        }));
    }
    let mut screen = div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .border_radius_all(f64::from(props.screen_radius))
        .background_color(css_color(glass))
        .overflow(Overflow::Hidden))
    .child(child);
    if props.show_dynamic_island {
        screen = screen.child(
            div(Style::default()
                .position(Position::Absolute)
                .top(Length::px(10.0))
                .left(Length::percent(50.0))
                .translate(Length::percent(-50.0), Length::ZERO)
                .width(Length::px(90.0))
                .height(Length::px(26.0))
                .border_radius_all(999.0)
                .background_color(css_color(island.get("background")))
                .z_index(10)
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::FlexEnd)
                .padding_right(Length::px(9.0)))
            .child(div(Style::default()
                .width(Length::px(9.0))
                .height(Length::px(9.0))
                .border_radius_all(999.0)
                .background_color(css_color(island.get("card")))
                .border(Edges::all(BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(css_color(island.get("border"))),
                })))),
        );
    }
    let device = device.child(screen);
    let grow = pose.scale * props.scale;
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(face))
    .child(
        div(Style::default()
            .opacity(pose.opacity)
            .translate(Length::ZERO, Length::px(pose.rise + bob))
            .rotate(Angle::deg(pose.turn))
            .scale(grow, grow))
        .child(
            div(Style::default()).child(
                div(Style::default().transform(vec![
                    Transform::Perspective(Some(Px(1400.0))),
                    Transform::RotateY(Angle::deg(tilt)),
                ]))
                .child(device),
            ),
        ),
    )
    .into())
}
