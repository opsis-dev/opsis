use opsis_macros::{choice, settings};
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, Overflow, Position, Style, TextOverflow, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::{clamped, clamped_eased, sprung};
use crate::elements::code_theme::{Palette, code_palette};
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::text_split::{easing_enter, easing_exit};
use crate::elements::theme::{Theme, ThemeMode, notification_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ToastTone {
    Info,
    Success,
    Warn,
    Error,
}

#[choice(export)]
pub enum ToastCorner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[settings(export)]
pub struct NotificationStackToast {
    #[setting("Title", default = "Upload failed")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Body", "type" = "string")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Tone", "type" = "string", "enum" = ["info", "success", "warn", "error"])]
    pub tone: Option<ToastTone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Meta", "type" = "string")]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("At seconds", "type" = "number")]
    pub at_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Hold for", "type" = "number")]
    pub hold_for: Option<f64>,
}

#[settings(export)]
pub struct NotificationStack {
    #[setting(
        "Notifications",
        default = vec![
            NotificationStackToast {
                title: "Render finished".into(),
                body: Some("launch-teaser.mp4 · 1080p".into()),
                tone: Some(ToastTone::Success),
                meta: Some("now".into()),
                ..Default::default()
            },
            NotificationStackToast {
                title: "3 new comments".into(),
                body: Some("Mina left notes on scene 4".into()),
                tone: Some(ToastTone::Info),
                meta: Some("1m".into()),
                ..Default::default()
            },
            NotificationStackToast {
                title: "Storage at 86%".into(),
                body: Some("Archive old renders to free space".into()),
                tone: Some(ToastTone::Warn),
                meta: Some("2m".into()),
                ..Default::default()
            },
            NotificationStackToast {
                body: Some("b-roll-rooftop.mov · retrying".into()),
                tone: Some(ToastTone::Error),
                meta: Some("2m".into()),
                ..Default::default()
            },
        ],
        help = "Title, optional body and meta, tone, and optional per-toast arrival and dwell."
    )]
    pub toasts: Vec<NotificationStackToast>,
    #[setting(
        "Position",
        default = ToastCorner::TopRight,
        help = "Corner the stack anchors to. Bottom anchors grow upward."
    )]
    pub align: ToastCorner,
    #[setting(
        "Start at (seconds)",
        default = 0.3,
        step = 0.01,
        help = "Second the first toast arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting("Stagger (seconds)", default = 0.62, step = 0.01, advanced)]
    pub stagger_seconds: f64,
    #[setting(
        "Time on screen (seconds)",
        default = 2.3,
        step = 0.1,
        help = "Default seconds a toast stays up before dismissing itself.",
        advanced
    )]
    pub life_seconds: f64,
    #[setting(
        "Show timer bar",
        default = true,
        help = "Draining line along each toast's bottom edge.",
        advanced
    )]
    pub show_progress: bool,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Colour used by the warn tone.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = notification_themes,
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
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting("Size", default = 1.0, range(min = 0.5, max = 2), unit = "×")]
    pub size: f64,
    #[setting(
        "Toast width",
        default = 370,
        range(min = 200, max = 700),
        step = 1,
        unit = "px",
        advanced
    )]
    pub toast_width: u32,
    #[setting(
        "Corner radius",
        default = 14,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Margin",
        default = 80,
        range(min = 0, max = 300),
        step = 1,
        unit = "px",
        advanced
    )]
    pub margin: u32,
}

pub fn definition() -> ElementMeta {
    meta::<NotificationStack>(
        element(
            ElementKind::Component,
            "notification-stack",
            "Notification stack",
            "Notifications slide in, stack up and dismiss themselves.",
            ElementCategory::Overlay,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:bell")),
    )
}

const TONES: [(&str, &str); 3] = [
    ("success", "#7FD1A0"),
    ("error", "#F97362"),
    ("info", "#7DD3E8"),
];
const VERTICAL_SHARE: f64 = 1.25;
const ENTER_FOR: f64 = 0.34;
const LEAVE_FOR: f64 = 0.3;
const REFERENCE: f64 = 1080.0;
const SLIDE_SPRING: SpringConfig = SpringConfig {
    damping: 19.0,
    stiffness: 150.0,
    mass: 0.7,
    overshoot_clamping: false,
};

fn tone_color<'a>(tone: ToastTone, accent: &'a str, palette: &'a Palette) -> &'a str {
    match tone {
        ToastTone::Success => palette.get("success"),
        ToastTone::Error => palette.get("error"),
        ToastTone::Warn => accent,
        ToastTone::Info => palette.get("info"),
    }
}

fn tone_glyph(tone: ToastTone, size: f64, color: &str) -> Node {
    let root = SvgElement::new("svg")
        .attr("width", size)
        .attr("height", size)
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none");
    let stroke = |d: &str, width: f64| {
        SvgElement::new("path")
            .attr("d", d)
            .attr("stroke", color)
            .attr("stroke-width", width)
            .attr("stroke-linecap", "round")
    };
    let dot = |cy: &str, r: &str| {
        SvgElement::new("circle")
            .attr("cx", "12")
            .attr("cy", cy)
            .attr("r", r)
            .attr("fill", color)
    };
    let root = match tone {
        ToastTone::Success => root.child(
            stroke("M5.5 12.5l4.3 4.3L18.5 7.6", 2.4).attr("stroke-linejoin", "round"),
        ),
        ToastTone::Error => root.child(stroke("M7.4 7.4l9.2 9.2M16.6 7.4l-9.2 9.2", 2.4)),
        ToastTone::Warn => root
            .child(
                SvgElement::new("path")
                    .attr("d", "M12 4.6L21 19.4H3L12 4.6Z")
                    .attr("stroke", color)
                    .attr("stroke-width", 2)
                    .attr("stroke-linejoin", "round"),
            )
            .child(stroke("M12 10.2v4.1", 2.0))
            .child(dot("16.7", "1.05")),
        ToastTone::Info => root
            .child(
                SvgElement::new("circle")
                    .attr("cx", "12")
                    .attr("cy", "12")
                    .attr("r", "8.4")
                    .attr("stroke", color)
                    .attr("stroke-width", 2),
            )
            .child(stroke("M12 11v5.4", 2.0))
            .child(dot("7.9", "1.15")),
    };
    svg(root, Style::default()).into()
}

fn present(text: Option<&String>) -> Option<&str> {
    text.map(String::as_str).filter(|t| !t.is_empty())
}

struct Beat {
    spawn: f64,
    life: f64,
    enter: f64,
    leave: f64,
    occupancy: f64,
    box_height: f64,
}

fn tinted(color: &str, share: f64) -> Color {
    Color::mix(
        ColorInterpolation::DEFAULT,
        (css_color(color), Some(share)),
        (Color::TRANSPARENT, None),
    )
}

fn border(color: Color) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(color),
    })
}

fn ellipsis_line(style: Style) -> Style {
    style
        .white_space(WhiteSpace::Nowrap)
        .text_overflow(TextOverflow::Ellipsis)
        .overflow(Overflow::Hidden)
}

#[expect(
    clippy::too_many_lines,
    reason = "one toast's boxes follow the film's markup in order"
)]
fn toast_node(
    props: &NotificationStack,
    toast: &NotificationStackToast,
    beat: &Beat,
    (offset, u): (f64, f64),
    palette: &Palette,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let fps = cx.fps();
    let at = |seconds: f64| seconds * fps / props.speed;
    let is_right = matches!(props.align, ToastCorner::TopRight | ToastCorner::BottomRight);
    let is_bottom = matches!(props.align, ToastCorner::BottomRight | ToastCorner::BottomLeft);
    let tone = toast.tone.unwrap_or(ToastTone::Info);
    let color = tone_color(tone, &props.accent_color, palette);
    let remaining = if props.show_progress {
        1.0 - clamped(frame, [at(beat.spawn), at(beat.spawn + beat.life)], [0.0, 1.0])?
    } else {
        1.0
    };
    let slide = sprung(frame - at(beat.spawn), fps, SLIDE_SPRING)?;
    let away = if is_right { 1.0 } else { -1.0 };
    let mut title_row = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Baseline)
        .gap(10.0 * u))
    .child(
        span(ellipsis_line(
            Style::default()
                .color(css_color(palette.get("fg")))
                .font_size(18.0 * u)
                .font_weight(600.0),
        ))
        .child(toast.title.as_str()),
    );
    if let Some(meta) = present(toast.meta.as_ref()) {
        title_row = title_row.child(
            span(Style::default()
                .margin_left(Length::Auto)
                .flex_shrink(0.0)
                .color(css_color(palette.get("faint")))
                .font_size(14.5 * u)
                .font_weight(500.0))
            .child(meta),
        );
    }
    let mut text = div(Style::default().min_width(Length::ZERO).flex_grow(1.0).flex_shrink(1.0).flex_basis(Length::percent(0.0)))
        .child(title_row);
    if let Some(body) = present(toast.body.as_ref()) {
        text = text.child(
            div(ellipsis_line(
                Style::default()
                    .margin_top(3.0 * u)
                    .color(css_color(palette.get("dim")))
                    .font_size(15.0 * u)
                    .font_weight(500.0),
            ))
            .child(body),
        );
    }
    let badge = div(Style::default()
        .flex_shrink(0.0)
        .width(32.0 * u)
        .height(32.0 * u)
        .border_radius_all(10.0 * u)
        .background_color(tinted(color, 12.0))
        .border(border(tinted(color, 35.0)))
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center))
    .child(tone_glyph(tone, 19.0 * u, color));
    let mut card = div(Style::default()
        .height(beat.box_height)
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(12.0 * u)
        .padding(Edges::axes(Length::ZERO, Length::px(15.0 * u)))
        .border_radius_all(f64::from(props.corner_radius) * u)
        .background_color(css_color(&format!("{}F2", palette.get("window"))))
        .border(border(css_color(palette.get("border"))))
        .box_shadow(vec![
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::px(1.0),
                blur: Length::ZERO,
                spread: Length::ZERO,
                color: css_color(palette.get("highlight")),
            },
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(16.0 * u),
                blur: Length::px(40.0 * u),
                spread: Length::ZERO,
                color: css_color(palette.get("shadow")),
            },
        ])
        .overflow(Overflow::Hidden)
        .position(Position::Relative))
    .child(badge)
    .child(text);
    if props.show_progress {
        card = card.child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .bottom(Length::ZERO)
            .height(2.5 * u)
            .width(Length::percent(remaining * 100.0))
            .background_color(css_color(color))
            .opacity(0.75)));
    }
    let mut holder = Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .right(Length::ZERO)
        .transform(vec![
            Transform::Translate(
                Length::ZERO,
                Length::px(if is_bottom { -offset } else { offset }),
                Length::ZERO,
            ),
            Transform::Translate(
                Length::px((beat.leave * 64.0).mul_add(u, (1.0 - slide) * 44.0 * u) * away),
                Length::ZERO,
                Length::ZERO,
            ),
        ])
        .opacity(beat.enter * (1.0 - beat.leave));
    holder = if is_bottom {
        holder.bottom(Length::ZERO)
    } else {
        holder.top(Length::ZERO)
    };
    Ok(div(holder).child(card).into())
}

pub fn draw(props: &NotificationStack, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let fps = cx.fps();
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let palette = code_palette(props.theme.mode, &TONES, &props.theme.colors);
    let side = (f64::from(props.margin) / REFERENCE * width).round();
    let edge = (f64::from(props.margin) * VERTICAL_SHARE / REFERENCE * height).round();
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64, easing: Easing| clamped_eased(frame, [at(from), at(to)], [0.0, 1.0], easing);
    let portrait = height > width;
    let u = props.size
        * if portrait {
            (width / 496.0).min(height / 896.0)
        } else {
            (width / 1024.0).min(height / 576.0)
        };
    let is_right = matches!(props.align, ToastCorner::TopRight | ToastCorner::BottomRight);
    let is_bottom = matches!(props.align, ToastCorner::BottomRight | ToastCorner::BottomLeft);
    let gap = 11.0 * u;
    let toast_width = side.mul_add(-2.0, width).min(f64::from(props.toast_width) * u);
    let mut plan = Vec::new();
    for (index, toast) in props.toasts.iter().enumerate() {
        let spawn = toast
            .at_seconds
            .unwrap_or_else(|| props.stagger_seconds.mul_add(float(index), props.start_at_seconds));
        let life = toast.hold_for.unwrap_or(props.life_seconds);
        let enter = ease(spawn, spawn + ENTER_FOR, easing_enter())?;
        let leave = ease(spawn + life, spawn + life + LEAVE_FOR, easing_exit())?;
        plan.push(Beat {
            spawn,
            life,
            enter,
            leave,
            occupancy: enter * (1.0 - leave),
            box_height: if present(toast.body.as_ref()).is_some() { 82.0 * u } else { 60.0 * u },
        });
    }
    let mut toasts = Vec::new();
    let mut offset = 0.0;
    for (toast, beat) in props.toasts.iter().zip(&plan) {
        toasts.push(toast_node(props, toast, beat, (offset, u), &palette, cx)?);
        offset = f64::mul_add(beat.occupancy, beat.box_height + gap, offset);
    }
    let mut stack = Style::default().position(Position::Absolute).width(toast_width);
    stack = if is_right { stack.right(side) } else { stack.left(side) };
    stack = if is_bottom { stack.bottom(edge) } else { stack.top(edge) };
    Ok(div(absolute_fill().font_family(font_or(&props.font_family, FontRole::Main)))
        .child(div(stack).children(toasts))
        .into())
}
