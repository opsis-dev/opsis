use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::color::Color;
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, Output, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style, TextAlign,
    TextTransform,
};
use opsis_scene::timeline::{ChildAt, Stage, absolute_fill};
use opsis_scene::values::{Corners, Edges, Length};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MultiDeviceLineup {
    #[setting(
        "Scale",
        default = 1.6,
        range(min = 0.4, max = 2),
        step = 0.1,
        help = "Overall size of the lineup."
    )]
    pub scale: f64,
    #[setting(
        "Stagger",
        default = 9,
        unit = "frames",
        help = "Frames between devices.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Bezel color", default = "#15171D", help = "Device body.", check = colour)]
    pub bezel_color: String,
    #[setting(
        "Screen color",
        default = "#0B0C11",
        help = "Screen behind your content.",
        check = colour,
        advanced
    )]
    pub screen_color: String,
    #[setting(
        "Label color",
        default = "#7A828F",
        help = "Captions under the devices.",
        check = colour
    )]
    pub label_color: String,
    #[setting("Phone label", default = "Phone")]
    pub phone_label: String,
    #[setting("Tablet label", default = "Tablet")]
    pub tablet_label: String,
    #[setting("Laptop label", default = "Laptop")]
    pub laptop_label: String,
    #[setting(
        "Laptop base",
        default = "linear-gradient(180deg, #23262E 0%, #14161B 100%)",
        widget = "gradient",
        advanced
    )]
    pub base_gradient: String,
    #[setting("Phone island color", default = "#05060A", check = colour, advanced)]
    pub island_color: String,
    #[setting("Tablet camera color", default = "rgba(255,255,255,0.28)", check = colour, advanced)]
    pub camera_color: String,
    #[setting("Edge color", default = "rgba(255,255,255,0.10)", check = colour, advanced)]
    pub edge_color: String,
    #[setting(
        "Shadow",
        default = 0.55,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Gap",
        default = 26,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub gap: u32,
    #[setting(
        "Label size",
        default = 12,
        range(min = 8, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_size: u32,
    #[setting("Uppercase labels", default = true, advanced)]
    pub label_uppercase: bool,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<MultiDeviceLineup>(
        element(
            ElementKind::Wrapper,
            "multi-device-lineup",
            "Device lineup",
            "Shows the clip on a phone, a tablet and a laptop side by side.",
            ElementCategory::Device,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("tabler:devices")),
    )
}

const DELAY: f64 = 4.0;
const RISE: SpringConfig = SpringConfig {
    mass: 0.75,
    damping: 17.0,
    stiffness: 150.0,
    overshoot_clamping: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Phone,
    Tablet,
    Laptop,
}

struct Hardware {
    screen_w: f64,
    screen_h: f64,
    bezel: f64,
    radius: f64,
}

const fn hardware(kind: Kind) -> Hardware {
    match kind {
        Kind::Phone => Hardware {
            screen_w: 148.0,
            screen_h: 320.0,
            bezel: 8.0,
            radius: 26.0,
        },
        Kind::Tablet => Hardware {
            screen_w: 236.0,
            screen_h: 316.0,
            bezel: 11.0,
            radius: 20.0,
        },
        Kind::Laptop => Hardware {
            screen_w: 396.0,
            screen_h: 248.0,
            bezel: 9.0,
            radius: 12.0,
        },
    }
}

fn bottom_round(radius: f64) -> Corners<(Length, Length)> {
    let r = |v: f64| (Length::px(v), Length::px(v));
    Corners {
        top_left: r(0.0),
        top_right: r(0.0),
        bottom_right: r(radius),
        bottom_left: r(radius),
    }
}

fn centred(style: Style) -> Style {
    style
        .position(Position::Absolute)
        .left(Length::percent(50.0))
        .transform(vec![Transform::Translate(
            Length::percent(-50.0),
            Length::ZERO,
            Length::ZERO,
        )])
}

#[expect(
    clippy::too_many_lines,
    clippy::suboptimal_flops,
    reason = "one pass over the three devices like the film's component, with its separate multiplies and adds"
)]
pub fn wrap(
    props: &MultiDeviceLineup,
    child: ChildAt<'_>,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let (frame, fps) = (cx.frame(), cx.fps());
    let scale = props.scale;
    let devices = [
        (Kind::Phone, &props.phone_label),
        (Kind::Tablet, &props.tablet_label),
        (Kind::Laptop, &props.laptop_label),
    ];
    let mut row = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::FlexEnd)
        .justify_content(Align::Center)
        .gap(f64::from(props.gap) * scale)
        .opacity(1.0)
        .translate(Length::ZERO, Length::px(0.0)));
    for (index, (kind, label)) in devices.into_iter().enumerate() {
        let spec = hardware(kind);
        let width = spec.bezel.mul_add(2.0, spec.screen_w);
        let height = spec.bezel.mul_add(2.0, spec.screen_h);
        let start = DELAY + opsis_types::numbers::float(index) * f64::from(props.stagger_in_frames);
        let rise = spring(&Spring {
            config: RISE,
            ..Spring::at(frame - start, fps)
        })?;
        let fade = interpolate(
            frame,
            &[start, start + 10.0],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing_enter()),
                ..Interpolation::clamped()
            },
        )?;
        let grow = interpolate(
            rise,
            &[0.0, 1.0],
            &[0.94, 1.0],
            &Interpolation {
                output: Output::PerceptualScale,
                ..Interpolation::clamped()
            },
        )?;
        let (screen_w, screen_h) = (spec.screen_w * scale, spec.screen_h * scale);
        let content = child(Stage {
            width: js_round(screen_w),
            height: js_round(screen_h),
        })?;
        let screen = div(Style::default()
            .width(Length::px(screen_w))
            .height(Length::px(screen_h))
            .border_radius_all(4.0f64.max(spec.radius - spec.bezel) * scale)
            .background_color(css_color(&props.screen_color))
            .overflow(Overflow::Hidden)
            .position(Position::Relative))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column))
            .child(
                div(Style::default()
                    .position(Position::Absolute)
                    .inset(Length::ZERO)
                    .overflow(Overflow::Hidden))
                .child(content),
            ),
        );
        let edge = css_color(&props.edge_color);
        let mut body = div(Style::default()
            .position(Position::Relative)
            .width(Length::px(width * scale))
            .height(Length::px(height * scale))
            .border_radius_all(spec.radius * scale)
            .background_color(css_color(&props.bezel_color))
            .border(Edges::all(BorderSide {
                width: Length::px(scale),
                style: BorderStyle::Solid,
                color: Some(edge.clone()),
            }))
            .box_shadow(vec![
                Shadow {
                    inset: true,
                    x: Length::ZERO,
                    y: Length::px(scale),
                    blur: Length::ZERO,
                    spread: Length::ZERO,
                    color: edge,
                },
                Shadow {
                    inset: false,
                    x: Length::ZERO,
                    y: Length::px(18.0 * scale),
                    blur: Length::px(44.0 * scale),
                    spread: Length::ZERO,
                    color: Color::srgb_alpha(0.0, 0.0, 0.0, props.shadow),
                },
            ])
            .padding(Edges::all(Length::px(spec.bezel * scale))))
        .child(screen);
        match kind {
            Kind::Phone => {
                body = body.child(div(centred(Style::default())
                    .top(Length::px((spec.bezel + 5.0) * scale))
                    .width(Length::px(44.0 * scale))
                    .height(Length::px(9.0 * scale))
                    .border_radius_all(999.0)
                    .background_color(css_color(&props.island_color))));
            }
            Kind::Tablet => {
                body = body.child(div(centred(Style::default())
                    .top(Length::px(spec.bezel / 2.0 * scale))
                    .width(Length::px(4.0 * scale))
                    .height(Length::px(4.0 * scale))
                    .border_radius_all(999.0)
                    .background_color(css_color(&props.camera_color))));
            }
            Kind::Laptop => {}
        }
        let mut device = div(Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .gap(10.0 * scale)
            .opacity(fade)
            .translate(Length::ZERO, Length::px((1.0 - rise) * 26.0 * scale))
            .scale(grow, grow))
        .child(body);
        if kind == Kind::Laptop {
            device = device.child(
                div(with_background(
                    Style::default()
                        .width(Length::px((width + 52.0) * scale))
                        .height(Length::px(9.0 * scale))
                        .margin_top(Length::px(-8.0 * scale))
                        .border_radius(bottom_round(7.0 * scale))
                        .box_shadow(vec![Shadow {
                            inset: false,
                            x: Length::ZERO,
                            y: Length::px(8.0 * scale),
                            blur: Length::px(18.0 * scale),
                            spread: Length::ZERO,
                            color: Color::srgb_alpha(0.0, 0.0, 0.0, props.shadow * 0.5 / 0.55),
                        }])
                        .position(Position::Relative),
                    &props.base_gradient,
                ))
                .child(div(centred(Style::default())
                    .top(Length::ZERO)
                    .width(Length::px(74.0 * scale))
                    .height(Length::px(3.0 * scale))
                    .border_radius(bottom_round(3.0 * scale))
                    .background_color(Color::srgb_alpha(0.0, 0.0, 0.0, 0.55)))),
            );
        }
        if !label.is_empty() {
            device = device.child(
                div(Style::default()
                    .color(css_color(&props.label_color))
                    .font_size(f64::from(props.label_size) * scale)
                    .font_weight(600.0)
                    .letter_spacing(Length::em(0.08))
                    .text_transform(if props.label_uppercase {
                        TextTransform::Uppercase
                    } else {
                        TextTransform::None
                    })
                    .font_family(font_or(&props.font_family, FontRole::Main)))
                .child(label.clone()),
            );
        }
        row = row.child(device);
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(row)
    .into())
}
