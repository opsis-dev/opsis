use opsis_scene::css::with_background;
use opsis_scene::node::{Element, Node, SvgElement, div, svg};
use opsis_scene::paint::{Filter, Shadow};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style,
    TextOverflow, WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::recording::{FrameTheme, WindowStyle};

use crate::elements::draw::css_color;

pub const CHROME_H: f64 = 38.0;
pub const WINDOW_RADIUS: f64 = 14.0;
const PILL_RADIUS: f64 = 9999.0;
const INK: &str = "15, 23, 42";
const LIGHT_SIZE: f64 = 11.0;
const LIGHT_GAP: f64 = 7.0;
const TRAFFIC_LIGHTS: [(&str, &str); 3] = [
    ("#ff5f57", "#e0443e"),
    ("#febc2e", "#dea123"),
    ("#28c840", "#1aab29"),
];
const ICONS: [&str; 3] = ["M3 8h10", "M3.5 3.5h9v9h-9z", "M4 4l8 8M12 4l-8 8"];
const BAR_GAP: f64 = 10.0;
const BAR_PAD_X: f64 = 14.0;
const FIELD_HEIGHT: f64 = 22.0;
const FIELD_FONT_SIZE: f64 = 11.5;
const FIELD_PAD_X: f64 = 12.0;
const FIELD_WEIGHT: f64 = 450.0;
const FIELD_TRACKING: f64 = 0.01;

pub fn ink(alpha: f64) -> String {
    format!(
        "rgba({INK}, {})",
        opsis_scene::random::js_number_text(alpha)
    )
}

struct Colors {
    bar: &'static str,
    edge: String,
    field: String,
    field_edge: String,
    text: String,
    icon: String,
    button: String,
}

fn colors(theme: FrameTheme) -> Colors {
    match theme {
        FrameTheme::Light => Colors {
            bar: "linear-gradient(180deg, #fafbfc 0%, #f0f2f5 100%)",
            edge: ink(0.07),
            field: ink(0.045),
            field_edge: ink(0.05),
            text: ink(0.5),
            icon: ink(0.62),
            button: ink(0.07),
        },
        FrameTheme::Dark => Colors {
            bar: "linear-gradient(180deg, #2c2c31 0%, #232328 100%)",
            edge: "rgba(0, 0, 0, 0.45)".to_owned(),
            field: "rgba(255, 255, 255, 0.07)".to_owned(),
            field_edge: "rgba(255, 255, 255, 0.06)".to_owned(),
            text: "rgba(255, 255, 255, 0.62)".to_owned(),
            icon: "rgba(255, 255, 255, 0.78)".to_owned(),
            button: "rgba(255, 255, 255, 0.1)".to_owned(),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Buttons {
    size: f64,
    gap: f64,
    glyph: f64,
    full_height: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Slot {
    Lights,
    Buttons(Buttons),
    Spacer,
    Empty,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film adds the slot widths with separate multiplies and adds"
)]
fn slot_width(slot: Slot) -> f64 {
    let count = 3.0;
    match slot {
        Slot::Lights => count * LIGHT_SIZE + (count - 1.0) * LIGHT_GAP,
        Slot::Buttons(look) => count * look.size + (count - 1.0) * look.gap,
        Slot::Spacer | Slot::Empty => 0.0,
    }
}

struct Look {
    window_radius: f64,
    field_radius: f64,
    field_align: Align,
    lead: Slot,
    trail: Slot,
}

const fn look_of(style: WindowStyle) -> Look {
    match style {
        WindowStyle::Macos => Look {
            window_radius: WINDOW_RADIUS,
            field_radius: 7.0,
            field_align: Align::Center,
            lead: Slot::Lights,
            trail: Slot::Spacer,
        },
        WindowStyle::Linux => Look {
            window_radius: WINDOW_RADIUS,
            field_radius: 7.0,
            field_align: Align::Center,
            lead: Slot::Spacer,
            trail: Slot::Buttons(Buttons {
                size: 20.0,
                gap: 9.0,
                glyph: 11.0,
                full_height: false,
            }),
        },
        WindowStyle::Windows => Look {
            window_radius: 8.0,
            field_radius: PILL_RADIUS,
            field_align: Align::FlexStart,
            lead: Slot::Empty,
            trail: Slot::Buttons(Buttons {
                size: 44.0,
                gap: 0.0,
                glyph: 12.0,
                full_height: true,
            }),
        },
    }
}

pub const fn window_radius(style: WindowStyle) -> f64 {
    look_of(style).window_radius
}

fn traffic_lights(k: f64) -> Node {
    div(Style::default()
        .display(Display::Flex)
        .gap(LIGHT_GAP * k)
        .flex_shrink(0.0))
    .children(TRAFFIC_LIGHTS.iter().map(|(fill, rim)| {
        div(with_background(
            Style::default()
                .width(Length::px(LIGHT_SIZE * k))
                .height(Length::px(LIGHT_SIZE * k))
                .border_radius_all(PILL_RADIUS)
                .box_shadow(vec![Shadow {
                    inset: true,
                    x: Length::ZERO,
                    y: Length::ZERO,
                    blur: Length::ZERO,
                    spread: Length::px(0.5 * k),
                    color: css_color("rgba(0,0,0,0.12)"),
                }]),
            &format!("radial-gradient(circle at 35% 30%, {fill} 0%, {rim} 100%)"),
        ))
        .into()
    }))
    .into()
}

fn glyph(d: &str, size: f64, color: &str) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 16 16")
            .child(
                SvgElement::new("path")
                    .attr("d", d)
                    .attr("stroke", color)
                    .attr("stroke-width", 1.2)
                    .attr("fill", "none")
                    .attr("stroke-linecap", "round"),
            ),
        Style::default()
            .width(Length::px(size))
            .height(Length::px(size)),
    )
    .into()
}

fn window_buttons(look: Buttons, theme: FrameTheme, k: f64) -> Node {
    let c = colors(theme);
    let row = Style::default().display(Display::Flex).flex_shrink(0.0);
    let row = if look.full_height {
        row.align_self(Align::Stretch)
            .margin_right(Length::px(-BAR_PAD_X * k))
    } else {
        row.gap(look.gap * k)
    };
    div(row)
        .children(ICONS.iter().map(|d| {
            let button = Style::default()
                .display(Display::Grid)
                .align_items(Align::Center)
                .justify_items(Align::Center)
                .width(Length::px(look.size * k));
            let button = if look.full_height {
                button.align_self(Align::Stretch)
            } else {
                button
                    .height(Length::px(look.size * k))
                    .border_radius_all(PILL_RADIUS)
                    .background_color(css_color(&c.button))
            };
            div(button).child(glyph(d, look.glyph * k, &c.icon)).into()
        }))
        .into()
}

fn slot_view(slot: Slot, facing: Slot, theme: FrameTheme, k: f64) -> Option<Node> {
    match slot {
        Slot::Lights => Some(traffic_lights(k)),
        Slot::Buttons(look) => Some(window_buttons(look, theme, k)),
        Slot::Spacer => Some(
            div(Style::default()
                .width(Length::px(slot_width(facing) * k))
                .flex_shrink(0.0))
            .into(),
        ),
        Slot::Empty => None,
    }
}

fn flexed() -> Style {
    Style::default()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .flex_basis(Length::percent(0.0))
}

#[derive(Debug)]
pub struct Chrome<'a> {
    pub style: WindowStyle,
    pub theme: FrameTheme,
    pub address_bar: bool,
    pub address: &'a str,
    pub font: Vec<String>,
}

fn title_bar(chrome: &Chrome<'_>, k: f64) -> Node {
    let c = colors(chrome.theme);
    let look = look_of(chrome.style);
    let mut bar = div(with_background(
        Style::default()
            .flex_grow(0.0)
            .flex_shrink(0.0)
            .flex_basis(Length::px(CHROME_H * k))
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(BAR_GAP * k)
            .padding(Edges::axes(Length::ZERO, Length::px(BAR_PAD_X * k)))
            .border(Edges {
                bottom: BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(css_color(&c.edge)),
                },
                ..Edges::all(BorderSide {
                    width: Length::ZERO,
                    style: BorderStyle::None,
                    color: None,
                })
            }),
        c.bar,
    ));
    if let Some(lead) = slot_view(look.lead, look.trail, chrome.theme, k) {
        bar = bar.child(lead);
    }
    bar = if chrome.address_bar {
        bar.child(
            div(flexed()
                .height(Length::px(FIELD_HEIGHT * k))
                .border_radius_all(look.field_radius * k)
                .background_color(css_color(&c.field))
                .border(Edges::all(BorderSide {
                    width: Length::px(k),
                    style: BorderStyle::Solid,
                    color: Some(css_color(&c.field_edge)),
                }))
                .color(css_color(&c.text))
                .font_size(FIELD_FONT_SIZE * k)
                .font_family(chrome.font.clone())
                .font_weight(FIELD_WEIGHT)
                .letter_spacing(Length::em(FIELD_TRACKING))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(look.field_align)
                .padding(Edges::axes(Length::ZERO, Length::px(FIELD_PAD_X * k)))
                .overflow(Overflow::Hidden)
                .white_space(WhiteSpace::Nowrap)
                .text_overflow(TextOverflow::Ellipsis))
            .child(chrome.address.to_owned()),
        )
    } else {
        bar.child(div(flexed()))
    };
    if let Some(trail) = slot_view(look.trail, look.lead, chrome.theme, k) {
        bar = bar.child(trail);
    }
    bar.into()
}

#[derive(Debug)]
pub struct Frame<'a> {
    pub chrome: Option<Chrome<'a>>,
    pub radius: Option<f64>,
    pub style: WindowStyle,
    pub lift: bool,
}

pub fn lift_filters() -> Vec<Filter> {
    vec![
        Filter::Saturate(Num(1.05)),
        Filter::Contrast(Num(1.025)),
        Filter::Brightness(Num(1.01)),
    ]
}

pub fn lifted(node: Node) -> Node {
    match node {
        Node::Element(mut element) => {
            let mut filters = opsis_scene::prop!(&element.style, Filter)
                .cloned()
                .unwrap_or_default();
            filters.extend(lift_filters());
            element.style = element.style.filter(filters);
            Node::Element(element)
        }
        text @ Node::Text(_) => text,
    }
}

pub fn window_frame(frame: &Frame<'_>, children: Vec<Node>, k: f64) -> Element {
    let mut style = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .border_radius_all(frame.radius.unwrap_or_else(|| window_radius(frame.style)) * k)
        .overflow(Overflow::Hidden)
        .background_color(css_color("#fff"))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .box_shadow(vec![Shadow {
            inset: true,
            x: Length::ZERO,
            y: Length::px(k),
            blur: Length::ZERO,
            spread: Length::ZERO,
            color: css_color("rgba(255,255,255,0.9)"),
        }]);
    if frame.lift {
        style = style.filter(lift_filters());
    }
    let mut out = div(style);
    if let Some(chrome) = &frame.chrome {
        out = out.child(title_bar(chrome, k));
    }
    out.child(
        div(flexed()
            .position(Position::Relative)
            .overflow(Overflow::Hidden)
            .min_height(Length::ZERO))
        .children(children),
    )
}
