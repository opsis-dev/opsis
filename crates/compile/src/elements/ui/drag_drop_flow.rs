use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, Output, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{
    ColorStop, Gradient, LinearDirection, RadialShape, RadialSize, Shadow, Transform,
};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, NumericVariant, Overflow, Position,
    Style, TextAlign, TextTransform, WhiteSpace,
};
use opsis_scene::values::{Angle, Edges, Length, Num, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::{clamped, sprung};
use crate::elements::code_theme::code_palette;
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::path_utils::{sample_path, waypoints_to_path};
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct DragDropFlow {
    #[setting(
        "File name",
        default = "hero-take.mp4",
        help = "File the cursor picks up."
    )]
    pub file_name: String,
    #[setting(
        "File size",
        default = "48.2 MB",
        help = "Size shown once the upload completes."
    )]
    pub file_size: String,
    #[setting(
        "Other files",
        default = vec!["b-roll-rooftop.mov".into(), "interview-cam-b.mp4".into()],
        help = "Other rows in the source list."
    )]
    pub siblings: Vec<String>,
    #[setting(
        "Library title",
        default = "Media library",
        help = "Heading on the source panel."
    )]
    pub source_label: String,
    #[setting(
        "Label",
        default = "Drop your clip",
        help = "Idle prompt in the drop zone."
    )]
    pub label: String,
    #[setting(
        "Hint",
        default = "MP4 or MOV, up to 2 GB",
        help = "Second line under the prompt."
    )]
    pub hint: String,
    #[setting(
        "Label while dragging",
        default = "Release to upload",
        help = "Prompt while the file is held over the zone.",
        advanced
    )]
    pub active_label: String,
    #[setting(
        "Label when done",
        default = "Uploaded",
        help = "Label once the upload finishes.",
        advanced
    )]
    pub done_label: String,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Zone, cursor ring, and progress colour.",
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
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Label while uploading",
        default = "Uploading",
        help = "Label while the file uploads.",
        advanced
    )]
    pub uploading_label: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[setting(
        "Upload time",
        default = 1.26,
        range(min = 0.3, max = 8),
        unit = "s",
        advanced
    )]
    pub upload_seconds: f64,
    #[setting("Show percentage", default = true, advanced)]
    pub show_percent: bool,
    #[setting(
        "Row of the dragged file",
        default = 1,
        range(min = 0, max = 10),
        step = 1,
        advanced
    )]
    pub dragged_row: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting(
        "Cursor colour",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        advanced
    )]
    pub cursor_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<DragDropFlow>(
        element(
            ElementKind::Component,
            "drag-drop-flow",
            "File upload",
            "A pointer drags a file into an upload area, which catches it and uploads it.",
            ElementCategory::Ui,
            150,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:cloud-upload")),
    )
}

const ROWS_AT: f64 = 0.28;
const CURSOR_IN: f64 = 0.45;
const REACH: f64 = 1.0;
const PRESS: f64 = 1.02;
const LIFT_END: f64 = 1.22;
const DRAG_START: f64 = 1.12;
const DRAG_END: f64 = 2.12;
const OVER: f64 = 1.62;
const RELEASE: f64 = 2.12;
const SETTLE_END: f64 = 2.5;
const UPLOAD_START: f64 = 2.42;
const UPLOAD_END: f64 = 3.68;
const DONE_END: f64 = 4.05;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const SETTLE_SPRING: SpringConfig = SpringConfig {
    damping: 15.0,
    stiffness: 150.0,
    mass: 0.7,
    overshoot_clamping: false,
};
const PANEL_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 120.0,
    mass: 0.8,
    overshoot_clamping: false,
};

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn glyph(size: (f64, f64), view: &str, paths: Vec<SvgElement>) -> Node {
    svg(
        paths.into_iter().fold(
            SvgElement::new("svg")
                .attr("width", size.0)
                .attr("height", size.1)
                .attr("viewBox", view)
                .attr("fill", "none"),
            SvgElement::child,
        ),
        Style::default(),
    )
    .into()
}

fn stroked(d: &str, color: &str, width: f64) -> SvgElement {
    SvgElement::new("path")
        .attr("d", d)
        .attr("stroke", color)
        .attr("stroke-width", width)
}

fn file_glyph(size: f64, color: &str) -> Node {
    glyph(
        (size, size),
        "0 0 24 24",
        vec![
            stroked(
                "M13.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5L13.5 3Z",
                color,
                1.6,
            )
            .attr("stroke-linejoin", "round"),
            stroked("M13.3 3.2V8.7h5.5", color, 1.6),
            stroked("m10.4 12.6 4 2.4-4 2.4v-4.8Z", color, 1.4)
                .attr("fill", color)
                .attr("stroke-linejoin", "round"),
        ],
    )
}

fn upload_glyph(size: f64, color: &str) -> Node {
    glyph(
        (size, size),
        "0 0 24 24",
        vec![
            stroked("M12 15.5V4.2m0 0L7.6 8.6M12 4.2l4.4 4.4", color, 1.7)
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round"),
            stroked(
                "M4.5 15v3.2a1.8 1.8 0 0 0 1.8 1.8h11.4a1.8 1.8 0 0 0 1.8-1.8V15",
                color,
                1.7,
            )
            .attr("stroke-linecap", "round"),
        ],
    )
}

fn check_glyph(size: f64, color: &str, progress: f64) -> Node {
    glyph(
        (size, size),
        "0 0 24 24",
        vec![
            stroked("M4.8 12.6l4.9 4.9 9.5-10.4", color, 2.4)
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round")
                .attr("stroke-dasharray", "22")
                .attr("stroke-dashoffset", 22.0 * (1.0 - progress)),
        ],
    )
}

fn cursor_glyph(size: f64, fill: &str, stroke: &str) -> Node {
    glyph(
        (size, size * 1.35),
        "0 0 17 23",
        vec![
            stroked("M1.4 1.2 15.2 12.1l-6.1.6-3.2 6.3-4.5-17.8Z", stroke, 1.4)
                .attr("fill", fill)
                .attr("stroke-linejoin", "round"),
        ],
    )
}

fn placed(rect: Rect) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::px(rect.x))
        .top(Length::px(rect.y))
        .width(Length::px(rect.w))
        .height(Length::px(rect.h))
}

fn side(color: Color, width: f64, style: BorderStyle) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(width),
        style,
        color: Some(color),
    })
}

const fn shadow(y: f64, blur: f64, color: Color) -> Shadow {
    Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(y),
        blur: Length::px(blur),
        spread: Length::ZERO,
        color,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one scene laid out in the film's order, which keeps the port checkable line by line against it"
)]
pub fn draw(props: &DragDropFlow, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, frame, fps) = (cx.at.stage.width, cx.at.stage.height, cx.frame(), cx.fps());
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (pad_x, pad_y) = safe_padding(width, height);
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64, easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[at(from), at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let stage = Rect {
        x: pad_x,
        y: pad_y,
        w: f64::mul_add(pad_x, -2.0, width),
        h: f64::mul_add(pad_y, -2.0, height),
    };
    let portrait = height > width;
    let dragged = usize::try_from(props.dragged_row)
        .unwrap_or(usize::MAX)
        .min(props.siblings.len());
    let mut rows: Vec<&str> = props.siblings.iter().map(String::as_str).collect();
    rows.insert(dragged, &props.file_name);
    let count = float(rows.len());
    let tray_units = (count - 1.0).mul_add(12.0, count.mul_add(74.0, 26.0f64.mul_add(2.0, 46.0)));
    let u = (stage.w / 900.0)
        .min(stage.h / 500.0)
        .min(if portrait { stage.h * 0.45 } else { stage.h } / tray_units);
    let tray_pad = 26.0 * u;
    let row_h = 74.0 * u;
    let row_gap = 12.0 * u;
    let gap = 0.05 * stage.w.min(stage.h * 1.6);
    let tray_h = (count - 1.0).mul_add(
        row_gap,
        count.mul_add(row_h, 46.0f64.mul_add(u, tray_pad * 2.0)),
    );
    let zone_h = if portrait {
        stage.h - tray_h - gap
    } else {
        stage.h.min(tray_h.max(400.0 * u))
    };
    let tray = if portrait {
        Rect {
            x: 0.0,
            y: 0.0,
            w: stage.w,
            h: tray_h,
        }
    } else {
        Rect {
            x: 0.0,
            y: (stage.h - tray_h) / 2.0,
            w: stage.w * 0.34,
            h: tray_h,
        }
    };
    let zone = if portrait {
        Rect {
            x: 0.0,
            y: tray_h + gap,
            w: stage.w,
            h: zone_h,
        }
    } else {
        Rect {
            x: tray.w + gap,
            y: (stage.h - zone_h) / 2.0,
            w: stage.w - tray.w - gap,
            h: zone_h,
        }
    };
    let rows_top = 46.0f64.mul_add(u, tray.y + tray_pad);
    let row_y = |index: usize| float(index).mul_add(row_h + row_gap, rows_top);
    let card_w = tray_pad.mul_add(-2.0, tray.w).min(360.0 * u);
    let card_h = row_h;
    let home = (
        tray.x + tray_pad + card_w / 2.0,
        row_y(dragged) + card_h / 2.0,
    );
    let drop = (zone.x + zone.w / 2.0, zone.h.mul_add(0.34, zone.y));
    let grip = (card_w * 0.16, card_h * 0.18);
    let approach = waypoints_to_path(
        &[
            (tray.w.mul_add(0.12, tray.x), 90.0f64.mul_add(u, stage.h)),
            (card_w.mul_add(-0.1, home.0), row_h.mul_add(1.9, home.1)),
            (home.0 + grip.0, home.1 + grip.1),
        ],
        0.6,
    );
    let carry = waypoints_to_path(
        &[
            (home.0 + grip.0, home.1 + grip.1),
            (
                f64::midpoint(home.0, drop.0),
                stage.h.mul_add(-0.16, home.1.min(drop.1)),
            ),
            (drop.0 + grip.0, drop.1 + grip.1),
        ],
        0.9,
    );
    let carrying = frame >= at(DRAG_START);
    let cursor = if carrying {
        sample_path(&carry, ease(DRAG_START, DRAG_END, EDITORIAL)?)
    } else {
        sample_path(&approach, ease(CURSOR_IN, REACH, EDITORIAL)?)
    };
    let press = interpolate(
        frame,
        &[at(PRESS), at(PRESS + 0.09), at(LIFT_END)],
        &[0.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    let lift = interpolate(
        frame,
        &[at(PRESS), at(LIFT_END), at(RELEASE), at(SETTLE_END)],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation {
            easing: Some(EDITORIAL),
            ..Interpolation::clamped()
        },
    )?;
    let released = frame >= at(RELEASE);
    let settle = sprung(frame - at(RELEASE), fps, SETTLE_SPRING)?;
    let armed = interpolate(
        frame,
        &[at(OVER), at(OVER + 0.22), at(RELEASE), at(RELEASE + 0.3)],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    let upload_end = UPLOAD_START + props.upload_seconds;
    let upload = ease(UPLOAD_START, upload_end, EDITORIAL)?;
    let done = ease(
        upload_end,
        upload_end + (DONE_END - UPLOAD_END),
        easing_enter(),
    )?;
    let panel = sprung(frame, fps, PANEL_SPRING)?;
    let line = |t: f64, from: f64, to: f64| {
        interpolate(t, &[0.0, 1.0], &[from, to], &Interpolation::default())
    };
    let card_center = if released {
        (
            line(settle, cursor.x - grip.0, drop.0)?,
            line(settle, cursor.y - grip.1, drop.1)?,
        )
    } else if carrying {
        (cursor.x - grip.0, cursor.y - grip.1)
    } else {
        home
    };
    let zone_armed = armed > 0.0 || released;
    let zone_label = match (released, upload < 1.0, armed > 0.5) {
        (true, true, _) => &props.uploading_label,
        (true, false, _) => &props.done_label,
        (false, _, true) => &props.active_label,
        (false, _, false) => &props.label,
    };
    let accent = props.accent_color.as_str();
    let border = css_color(palette.get("border"));
    let n = js_number_text;
    let mut items: Vec<Node> = Vec::new();
    items.push(
        div(placed(tray)
            .border_radius_all(22.0 * u)
            .background_color(css_color(palette.get("window")))
            .border(side(border.clone(), 1.0, BorderStyle::Solid))
            .box_shadow(vec![
                Shadow {
                    inset: true,
                    x: Length::ZERO,
                    y: Length::px(1.0),
                    blur: Length::ZERO,
                    spread: Length::ZERO,
                    color: css_color(palette.get("highlight")),
                },
                shadow(26.0 * u, 70.0 * u, css_color(palette.get("shadow"))),
            ])
            .opacity(panel)
            .transform(vec![Transform::Translate(
                Length::px((1.0 - panel) * -22.0 * u),
                Length::ZERO,
                Length::ZERO,
            )]))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::px(tray_pad))
                .top(Length::px(tray_pad))
                .color(css_color(palette.get("dim")))
                .font_size(19.0 * u)
                .font_weight(600.0)
                .letter_spacing(Length::em(0.08))
                .text_transform(TextTransform::Uppercase))
            .child(props.source_label.clone()),
        )
        .into(),
    );
    for (index, name) in rows.iter().enumerate() {
        let start = float(index).mul_add(0.08, ROWS_AT);
        let row_in = interpolate(
            frame,
            &[at(start), at(start + 0.4)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing_enter()),
                ..Interpolation::clamped()
            },
        )?;
        let slot = Rect {
            x: tray.x + tray_pad,
            y: row_y(index),
            w: card_w,
            h: row_h,
        };
        if index == dragged {
            items.push(
                div(placed(slot)
                    .border_radius_all(14.0 * u)
                    .border(side(
                        css_color(palette.get("faint")),
                        1.0,
                        BorderStyle::Dashed,
                    ))
                    .opacity(row_in * (lift * 0.9).max(if released { 0.55 } else { 0.0 })))
                .into(),
            );
            continue;
        }
        items.push(
            div(placed(slot)
                .border_radius_all(14.0 * u)
                .border(side(border.clone(), 1.0, BorderStyle::Solid))
                .background_color(css_color(palette.get("band")))
                .opacity(row_in)
                .transform(vec![Transform::Translate(
                    Length::ZERO,
                    Length::px((1.0 - row_in) * 14.0 * u),
                    Length::ZERO,
                )])
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(14.0 * u)
                .padding_left(Length::px(16.0 * u))
                .color(css_color(palette.get("dim")))
                .font_size(21.0 * u)
                .font_weight(500.0))
            .child(file_glyph(26.0 * u, palette.get("faint")))
            .child(
                span(
                    Style::default()
                        .white_space(WhiteSpace::Nowrap)
                        .overflow(Overflow::Hidden),
                )
                .child((*name).to_owned()),
            )
            .into(),
        );
    }
    let glow_reach = 52.0 * u * armed.max(if released { 0.3 } else { 0.0 });
    let zone_style = placed(zone)
        .border_radius_all(26.0 * u)
        .border(side(
            if zone_armed {
                css_color(accent)
            } else {
                border.clone()
            },
            2.0 * u,
            BorderStyle::Dashed,
        ))
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::px(glow_reach),
            spread: Length::ZERO,
            color: tinted(accent, 33.0),
        }])
        .opacity(panel)
        .transform(vec![Transform::Scale(
            Num(armed.mul_add(0.012, 1.0)),
            Num(armed.mul_add(0.012, 1.0)),
            Num(1.0),
        )]);
    let zone_style = if zone_armed {
        zone_style.background(vec![
            Gradient::Linear {
                direction: LinearDirection::Angle(Angle::deg(180.0)),
                stops: vec![
                    ColorStop::new(tinted(accent, if released { 5.0 } else { 9.0 })),
                    ColorStop::at(Color::TRANSPARENT, Length::percent(72.0)),
                ],
                repeating: false,
                interpolation: ColorInterpolation::DEFAULT,
            }
            .layer(),
        ])
    } else {
        zone_style.background_color(css_color(palette.get("band")))
    };
    items.push(div(zone_style).into());
    let mut label_box = div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(zone.x))
        .top(Length::px(line(
            if released { settle } else { 0.0 },
            zone.h.mul_add(0.58, zone.y),
            82.0f64.mul_add(u, card_h.mul_add(0.5, drop.1)),
        )?))
        .width(Length::px(zone.w))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(10.0 * u)
        .text_align(TextAlign::Center)
        .transform(vec![Transform::Translate(
            Length::ZERO,
            Length::percent(-50.0),
            Length::ZERO,
        )]));
    if !released {
        label_box = label_box.child(
            div(Style::default().transform(vec![Transform::Translate(
                Length::ZERO,
                Length::px(armed * -6.0 * u),
                Length::ZERO,
            )]))
            .child(upload_glyph(
                44.0 * u,
                if armed > 0.4 {
                    accent
                } else {
                    palette.get("dim")
                },
            )),
        );
    }
    let mut headline = div(Style::default()
        .color(css_color(if done > 0.0 {
            accent
        } else {
            palette.get("fg")
        }))
        .font_size(30.0 * u)
        .font_weight(600.0)
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(10.0 * u));
    if done > 0.0 {
        headline = headline.child(check_glyph(28.0 * u, accent, done));
    }
    let detail = if released {
        format!("{} · {}", props.file_name, props.file_size)
    } else {
        props.hint.clone()
    };
    items.push(
        label_box
            .child(headline.child(zone_label.clone()))
            .child(
                div(Style::default()
                    .color(css_color(palette.get("faint")))
                    .font_size(20.0 * u))
                .child(detail),
            )
            .into(),
    );
    let lit_edge = lift > 0.2 || released;
    let grow = f64::mul_add(press, -0.04, lift.mul_add(0.06, 1.0));
    items.push(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(card_center.0 - card_w / 2.0))
            .top(Length::px(card_center.1 - card_h / 2.0))
            .width(Length::px(card_w))
            .height(Length::px(card_h))
            .border_radius_all(14.0 * u)
            .background_color(css_color(palette.get("window")))
            .border(side(
                if lit_edge {
                    tinted(accent, 53.0)
                } else {
                    border
                },
                1.0,
                BorderStyle::Solid,
            ))
            .box_shadow(vec![shadow(
                lift.mul_add(22.0, 6.0) * u,
                lift.mul_add(46.0, 16.0) * u,
                css_color(palette.get("shadow")),
            )])
            .transform(vec![
                Transform::Scale(Num(grow), Num(grow), Num(1.0)),
                Transform::Rotate(Angle::deg(lift * -2.2 * (1.0 - settle))),
            ])
            .opacity(clamped(
                frame,
                [at(ROWS_AT), at(ROWS_AT + 0.4)],
                [0.0, 1.0],
            )?)
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(14.0 * u)
            .padding_left(Length::px(16.0 * u))
            .padding_right(Length::px(16.0 * u))
            .color(css_color(palette.get("fg")))
            .font_size(21.0 * u)
            .font_weight(500.0)
            .overflow(Overflow::Hidden))
        .child(file_glyph(26.0 * u, accent))
        .child(
            span(Style::default().white_space(WhiteSpace::Nowrap)).child(props.file_name.clone()),
        )
        .child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .bottom(Length::ZERO)
            .height(Length::px(4.0 * u))
            .width(Length::percent(upload * 100.0))
            .background_color(css_color(accent))
            .opacity(if released { 1.0 } else { 0.0 })))
        .into(),
    );
    let mut percent = span(Style::default());
    if props.show_percent {
        percent = percent.child(format!("{}%", n(f64::mul_add(upload, 100.0, 0.5).floor())));
    }
    items.push(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(drop.0 - card_w / 2.0))
            .top(Length::px(14.0f64.mul_add(u, card_h.mul_add(0.5, drop.1))))
            .width(Length::px(card_w))
            .display(Display::Flex)
            .justify_content(Align::SpaceBetween)
            .color(css_color(palette.get("dim")))
            .font_size(17.0 * u)
            .font_variant_numeric(NumericVariant {
                tabular: true,
                ..NumericVariant::default()
            })
            .opacity(if released { settle } else { 0.0 }))
        .child(span(Style::default()))
        .child(percent)
        .into(),
    );
    let ripple = interpolate(
        frame,
        &[at(RELEASE), at(RELEASE + 0.34)],
        &[0.4, 1.5],
        &Interpolation {
            output: Output::PerceptualScale,
            ..Interpolation::clamped()
        },
    )?;
    items.push(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(30.0f64.mul_add(-u, cursor.x)))
            .top(Length::px(30.0f64.mul_add(-u, cursor.y)))
            .width(Length::px(60.0 * u))
            .height(Length::px(60.0 * u))
            .border_radius_all(Length::percent(50.0))
            .border(side(css_color(accent), 2.0 * u, BorderStyle::Solid))
            .opacity(if released {
                clamped(frame, [at(RELEASE), at(RELEASE + 0.34)], [0.75, 0.0])?
            } else {
                0.0
            })
            .transform(vec![Transform::Scale(Num(ripple), Num(ripple), Num(1.0))]))
        .into(),
    );
    let dark = props.theme.mode == ThemeMode::Dark;
    let fill = if props.cursor_color.is_empty() {
        if dark { "#F6F7FA" } else { "#1A1D24" }
    } else {
        &props.cursor_color
    };
    let pointer_opacity = interpolate(
        frame,
        &[
            at(CURSOR_IN),
            at(CURSOR_IN + 0.2),
            at(RELEASE + 0.25),
            at(RELEASE + 0.6),
        ],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    items.push(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(cursor.x))
            .top(Length::px(cursor.y))
            .opacity(pointer_opacity)
            .transform(vec![Transform::Scale(
                Num(press.mul_add(-0.14, 1.0)),
                Num(press.mul_add(-0.14, 1.0)),
                Num(1.0),
            )])
            .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO)))
        .child(cursor_glyph(
            19.0 * u,
            fill,
            if dark {
                "rgba(0,0,0,0.5)"
            } else {
                "rgba(255,255,255,0.7)"
            },
        ))
        .into(),
    );
    let halo = Gradient::Radial {
        shape: RadialShape::Ellipse,
        size: RadialSize::Explicit(Length::percent(60.0), Length::percent(55.0)),
        center: Position2 {
            x: Length::percent((stage.x + drop.0) / width * 100.0),
            y: Length::percent((stage.y + drop.1) / height * 100.0),
        },
        stops: vec![
            ColorStop::new(tinted(accent, if zone_armed { 13.0 } else { 6.0 })),
            ColorStop::at(Color::TRANSPARENT, Length::percent(70.0)),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    Ok(div(with_background(
        Style::default()
            .width(Length::px(width))
            .height(Length::px(height)),
        page,
    )
    .font_family(font_or(&props.font_family, FontRole::Main))
    .position(Position::Relative)
    .overflow(Overflow::Hidden))
    .child(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .background(vec![halo.layer()])))
    .child(div(placed(stage)).children(items))
    .into())
}
