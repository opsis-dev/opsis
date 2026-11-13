use opsis_macros::{choice, settings};
use opsis_scene::clock::js_round;
use opsis_scene::color::Color;
use opsis_scene::css::{gradient, shadows, with_background};
use opsis_scene::interpolate::{Interpolation, Output, interpolate};
use opsis_scene::node::{Element, Node, div, span};
use opsis_scene::paint::{Filter, MaskLayer, MaskMode, Transform};
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style,
};
use opsis_scene::timeline::{ChildAt, Stage, absolute_fill};
use opsis_scene::values::{Corners, Edges, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::geometry::safe_padding;
use crate::elements::parts::split_chars::scale_font;
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MockupDevice {
    #[label("Phone")]
    Phone,
    #[label("Browser")]
    Browser,
    #[label("Laptop")]
    Laptop,
}

#[settings(export)]
pub struct DeviceMockupZoom {
    #[setting(
        "Device",
        default = MockupDevice::Laptop,
        help = "A laptop has a base, a browser is only its window, and a phone is held upright."
    )]
    pub device: MockupDevice,
    #[setting("Background color", default = "#080810", check = colour)]
    pub background_color: String,
    #[setting("Accent color", default = "#e8b86d", check = colour)]
    pub accent_color: String,
    #[setting("Device color", default = "#171b26", check = colour, advanced)]
    pub device_color: String,
    #[setting("Device edge color", default = "#242a3a", check = colour, advanced)]
    pub device_edge_color: String,
    #[setting("Laptop bezel color", default = "#0d111b", check = colour, advanced)]
    pub bezel_color: String,
    #[setting("Button color", default = "#0c0e15", check = colour, advanced)]
    pub button_color: String,
    #[setting("Camera color", default = "#05060a", check = colour, advanced)]
    pub camera_color: String,
    #[setting("Phone island color", default = "#000000", check = colour, advanced)]
    pub island_color: String,
    #[setting("Lens color", default = "#1e2230", check = colour, advanced)]
    pub lens_color: String,
    #[setting(
        "Laptop base",
        default = "linear-gradient(180deg, #30384b 0%, #171b26 60%, #0b0d14 100%)",
        widget = "gradient",
        advanced
    )]
    pub base_gradient: String,
    #[setting("Second accent", default = "#2dd4bf", check = colour, advanced)]
    pub second_accent: String,
    #[setting("Third accent", default = "#f472b6", check = colour, advanced)]
    pub third_accent: String,
    #[setting("Background tint", default = "#5b6b9a", check = colour, advanced)]
    pub background_tint: String,
    #[setting(
        "Shadow",
        default = 0.42,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Accent glow",
        default = 0.2,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub glow: f64,
    #[setting("Grid", default = true, advanced)]
    pub show_grid: bool,
    #[setting(
        "Starting zoom",
        default = 1.18,
        range(min = 1, max = 2),
        step = 0.01,
        help = "How close the camera starts before it pulls back.",
        advanced
    )]
    pub pull_back_from: f64,
    #[setting(
        "Device size",
        default = 1.0,
        range(min = 0.5, max = 1.5),
        step = 0.05,
        advanced
    )]
    pub scale: f64,
}

pub fn definition() -> ElementMeta {
    meta::<DeviceMockupZoom>(
            element(
                ElementKind::Wrapper,
                "device-mockup-zoom",
                "Device mockup",
                "Shows the clip on a laptop, phone or browser screen while the camera slowly pulls back.",
                ElementCategory::Device,
                90,
            )
                .optional_clip()
                .look(NodeLook::of("lucide:laptop-minimal")),
        )
}

const SMOOTH: SpringConfig = SpringConfig {
    damping: 200.0,
    stiffness: 100.0,
    mass: 1.0,
    overshoot_clamping: true,
};
const STAGGER_NORMAL: f64 = 8.0;

fn entering(frame: f64, from: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &from,
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(easing_enter()),
            ..Interpolation::clamped()
        },
    )?)
}

fn corners(
    top_left: f64,
    top_right: f64,
    bottom_right: f64,
    bottom_left: f64,
) -> Corners<(Length, Length)> {
    let r = |v: f64| (Length::px(v), Length::px(v));
    Corners {
        top_left: r(top_left),
        top_right: r(top_right),
        bottom_right: r(bottom_right),
        bottom_left: r(bottom_left),
    }
}

fn styled(style: Style, css: &str) -> Style {
    with_background(style, css)
}

fn shadowed(style: Style, css: &str) -> Style {
    match shadows(css) {
        Some(list) => style.box_shadow(list),
        None => style,
    }
}

struct Shell {
    radius: f64,
    pad: f64,
    screen_radius: f64,
    button: f64,
    ratio: f64,
    header: f64,
    gap: f64,
}

fn shell(device: MockupDevice, width: f64) -> Shell {
    let s = |n: f64| scale_font(n, width);
    let radius = if device == MockupDevice::Phone {
        s(56.0)
    } else {
        s(18.0)
    };
    let pad = match device {
        MockupDevice::Phone => s(9.0),
        MockupDevice::Browser => s(10.0),
        MockupDevice::Laptop => s(7.0),
    };
    Shell {
        radius,
        pad,
        screen_radius: match device {
            MockupDevice::Phone => pad.mul_add(-0.7, radius),
            MockupDevice::Browser => s(12.0),
            MockupDevice::Laptop => s(3.0),
        },
        button: s(3.0),
        ratio: if device == MockupDevice::Phone {
            0.4615
        } else {
            1.6
        },
        header: match device {
            MockupDevice::Phone => 0.0,
            MockupDevice::Browser => s(32.0),
            MockupDevice::Laptop => s(15.0),
        },
        gap: if device == MockupDevice::Browser {
            s(8.0)
        } else {
            0.0
        },
    }
}

fn button(style: Style, top: f64, height: f64, look: &DeviceMockupZoom) -> Node {
    div(style
        .position(Position::Absolute)
        .top(Length::percent(top))
        .height(Length::px(height))
        .background_color(css_color(&look.button_color)))
    .into()
}

fn header(look: &DeviceMockupZoom, width: f64) -> Option<Node> {
    let s = |n: f64| scale_font(n, width);
    match look.device {
        MockupDevice::Phone => None,
        MockupDevice::Browser => {
            let mut bar = div(Style::default()
                .height(Length::px(s(32.0)))
                .border_radius_all(s(14.0))
                .background_color(Color::srgb_alpha(1.0, 1.0, 1.0, 0.06))
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(s(8.0))
                .padding(Edges::axes(Length::ZERO, Length::px(s(12.0)))));
            for color in [&look.third_accent, &look.accent_color, &look.second_accent] {
                bar = bar.child(span(
                    Style::default()
                        .width(Length::px(s(10.0)))
                        .height(Length::px(s(10.0)))
                        .border_radius_all(99.0)
                        .background_color(css_color(color)),
                ));
            }
            Some(
                bar.child(div(Style::default()
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
                    .flex_basis(Length::percent(0.0))
                    .height(Length::px(s(16.0)))
                    .border_radius_all(999.0)
                    .background_color(Color::srgb_alpha(1.0, 1.0, 1.0, 0.08))
                    .margin_left(Length::px(s(8.0)))))
                    .into(),
            )
        }
        MockupDevice::Laptop => Some(
            div(Style::default()
                .height(Length::px(s(15.0)))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::Center))
            .child(div(shadowed(
                Style::default()
                    .width(Length::px(s(5.0)))
                    .height(Length::px(s(5.0)))
                    .border_radius_all(Length::percent(50.0))
                    .background_color(css_color(&look.camera_color)),
                "inset 0 0 0 1px rgba(255,255,255,0.18)",
            )))
            .into(),
        ),
    }
}

fn laptop_base(look: &DeviceMockupZoom, width: f64) -> Vec<Node> {
    let s = |n: f64| scale_font(n, width);
    let n = opsis_scene::random::js_number_text;
    vec![
        div(styled(
            Style::default()
                .width(Length::percent(104.0))
                .height(Length::px(2.0)),
            "linear-gradient(90deg, transparent, rgba(0,0,0,0.55) 12%, rgba(0,0,0,0.55) 88%, transparent)",
        ))
        .into(),
        div(shadowed(
            styled(
                Style::default()
                    .width(Length::percent(104.0))
                    .height(Length::px(s(30.0)))
                    .border_radius(corners(0.0, 0.0, s(16.0), s(16.0)))
                    .display(Display::Flex)
                    .justify_content(Align::Center),
                &look.base_gradient,
            ),
            &format!(
                "0 {}px {}px rgba(0,0,0,{}), inset 0 1px 0 rgba(255,255,255,0.16)",
                n(s(24.0)),
                n(s(50.0)),
                n(look.shadow * 0.38 / 0.42)
            ),
        ))
        .child(div(Style::default()
            .margin_top(Length::px(s(6.0)))
            .width(Length::percent(16.0))
            .height(Length::px(s(4.0)))
            .border_radius_all(3.0)
            .background_color(Color::srgb_alpha(1.0, 1.0, 1.0, 0.1))))
        .into(),
        div(Style::default()
            .width(Length::percent(82.0))
            .height(Length::px(s(18.0)))
            .border_radius_all(Length::percent(50.0))
            .background_color(Color::srgb_alpha(0.0, 0.0, 0.0, look.shadow * 0.28 / 0.42))
            .filter(vec![Filter::Blur(Length::px(s(12.0)))])
            .margin_top(Length::px(s(10.0))))
        .into(),
    ]
}

fn device_shell(
    look: &DeviceMockupZoom,
    geometry: &Shell,
    progress: f64,
    width: f64,
    screen: Node,
) -> Element {
    let s = |n: f64| scale_font(n, width);
    let n = opsis_scene::random::js_number_text;
    let phone = look.device == MockupDevice::Phone;
    let laptop = look.device == MockupDevice::Laptop;
    let b = geometry.button;
    let mut body = div(shadowed(
        styled(
            Style::default()
                .position(Position::Relative)
                .width(Length::percent(if laptop { 92.0 } else { 100.0 }))
                .aspect_ratio(Num(geometry.ratio))
                .border_radius_all(geometry.radius)
                .padding(Edges::all(Length::px(geometry.pad)))
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(geometry.gap),
            &format!(
                "linear-gradient(145deg, {}, {})",
                look.device_edge_color, look.device_color
            ),
        ),
        &format!(
            "0 {}px {}px rgba(0,0,0,{}), 0 0 {}px color-mix(in srgb, {} {}%, transparent), inset 0 0 0 1px rgba(255,255,255,0.1)",
            n(s(34.0)),
            n(s(90.0)),
            n(look.shadow),
            n(s(96.0) * progress),
            look.accent_color,
            n(look.glow * 100.0)
        ),
    ));
    if phone {
        let left = Style::default()
            .left(Length::px(-b))
            .width(Length::px(b))
            .border_radius(corners(b, 0.0, 0.0, b));
        body = body
            .child(button(left.clone(), 16.0, s(9.0), look))
            .child(button(left.clone(), 26.0, s(16.0), look))
            .child(button(left, 37.0, s(16.0), look))
            .child(button(
                Style::default()
                    .right(Length::px(-b))
                    .width(Length::px(b))
                    .border_radius(corners(0.0, b, b, 0.0)),
                22.0,
                s(22.0),
                look,
            ));
    }
    if let Some(bar) = header(look, width) {
        body = body.child(bar);
    }
    let mut glass = div(Style::default()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .flex_basis(Length::percent(0.0))
        .min_height(Length::ZERO)
        .position(Position::Relative)
        .border_radius_all(geometry.screen_radius)
        .overflow(Overflow::Hidden)
        .background_color(css_color(&look.bezel_color))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(Color::srgb_alpha(1.0, 1.0, 1.0, 0.08)),
        })))
    .child(screen);
    if phone {
        glass = glass.child(
            div(Style::default()
                .position(Position::Absolute)
                .top(Length::percent(2.6))
                .left(Length::percent(50.0))
                .transform(vec![Transform::Translate(
                    Length::percent(-50.0),
                    Length::ZERO,
                    Length::ZERO,
                )])
                .width(Length::percent(34.0))
                .height(Length::px(s(9.0)))
                .border_radius_all(999.0)
                .background_color(css_color(&look.island_color))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::FlexEnd)
                .padding(Edges::axes(Length::ZERO, Length::px(s(6.0)))))
            .child(div(shadowed(
                Style::default()
                    .width(Length::px(s(4.0)))
                    .height(Length::px(s(4.0)))
                    .border_radius_all(Length::percent(50.0))
                    .background_color(css_color(&look.lens_color)),
                "inset 0 0 0 1px rgba(255,255,255,0.06)",
            ))),
        );
    }
    let mut out = div(Style::default()
        .width(Length::percent(100.0))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center))
    .child(body.child(glass));
    if laptop {
        out = out.children(laptop_base(look, width));
    }
    out
}

#[expect(
    clippy::too_many_lines,
    clippy::suboptimal_flops,
    reason = "one pass over the mockup like the film's component, sizing the screen with separate multiplies and adds"
)]
pub fn wrap(look: &DeviceMockupZoom, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (frame, fps) = (cx.frame(), cx.fps());
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let (side, edge) = safe_padding(width, height);
    let portrait = height > width;
    let camera = spring(&Spring {
        config: SMOOTH,
        delay: 8.0,
        duration_in_frames: Some(58.0),
        ..Spring::at(frame, fps)
    })?;
    let device_in = entering(frame, [STAGGER_NORMAL, 64.0])?;
    let accent_in = entering(frame, [20.0, 72.0])?;
    let camera_scale = interpolate(
        camera,
        &[0.0, 1.0],
        &[look.pull_back_from, 0.95],
        &Interpolation {
            output: Output::PerceptualScale,
            ..Interpolation::clamped()
        },
    )?;
    let camera_y = interpolate(
        camera,
        &[0.0, 1.0],
        &[scale_font(20.0, width), 0.0],
        &Interpolation::clamped(),
    )?;
    let share = match look.device {
        MockupDevice::Phone => {
            if portrait {
                58.0
            } else {
                32.0
            }
        }
        _ if portrait => 96.0,
        MockupDevice::Laptop => 86.0,
        MockupDevice::Browser => 82.0,
    } * look.scale;
    let geometry = shell(look.device, width);
    let area = width - side * 2.0;
    let main = area.min(area * share / 100.0);
    let box_w = if look.device == MockupDevice::Laptop {
        main * 0.92
    } else {
        main
    };
    let box_h = box_w / geometry.ratio;
    let screen_w = box_w - geometry.pad * 2.0 - 2.0;
    let screen_h = box_h - geometry.pad * 2.0 - geometry.header - geometry.gap - 2.0;
    let content = child(Stage {
        width: js_round(screen_w),
        height: js_round(screen_h),
    })?;
    let screen: Node = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .overflow(Overflow::Hidden))
    .child(content)
    .into();
    let n = opsis_scene::random::js_number_text;
    let ground = &look.background_color;
    let mut root = div(absolute_fill()
        .background_color(css_color(ground))
        .overflow(Overflow::Hidden))
    .child(div(styled(
        absolute_fill(),
        &format!(
            "radial-gradient(circle at 72% 42%, color-mix(in srgb, {} 19%, transparent) 0, transparent 32%), radial-gradient(circle at 16% 82%, color-mix(in srgb, {} 12.5%, transparent) 0, transparent 26%), linear-gradient(135deg, {ground} 0%, color-mix(in srgb, {ground} 93%, {}) 54%, color-mix(in srgb, {ground} 97%, black) 100%)",
            look.accent_color, look.second_accent, look.background_tint
        ),
    )));
    if look.show_grid {
        let cell = n(scale_font(56.0, width));
        let mut grid = styled(
            absolute_fill().opacity(0.16),
            &format!(
                "linear-gradient(rgba(255,255,255,0.2) 1px, transparent 1px) 0 0 / {cell}px {cell}px, linear-gradient(90deg, rgba(255,255,255,0.2) 1px, transparent 1px) 0 0 / {cell}px {cell}px"
            ),
        );
        if let Some(fade) = gradient("radial-gradient(circle at 68% 44%, black, transparent 72%)") {
            grid = grid.mask(vec![MaskLayer {
                layer: fade.layer(),
                mode: MaskMode::Alpha,
            }]);
        }
        root = root.child(div(grid));
    }
    Ok(root
        .child(
            div(Style::default()
                .position(Position::Relative)
                .z_index(1)
                .width(Length::px(width))
                .height(Length::px(height))
                .padding(Edges::axes(Length::px(edge), Length::px(side)))
                .display(Display::Grid)
                .align_items(Align::Center))
            .child(
                div(Style::default()
                    .width(Length::percent(100.0))
                    .max_width(Length::percent(share))
                    .justify_self(Align::Center)
                    .opacity(device_in)
                    .scale(camera_scale, camera_scale)
                    .translate(Length::ZERO, Length::px(camera_y)))
                .child(device_shell(look, &geometry, accent_in, width, screen)),
            ),
        )
        .into())
}
