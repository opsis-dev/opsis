use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::spring::{DEFAULT_SPRING, SpringConfig};
use opsis_scene::style::{Align, Display, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::anim::sprung;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::overlay_frame::{OverlayFrame, is_landscape, overlay_frame, overlay_padding};
use crate::elements::shared::Corner;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LowerThird {
    #[setting("Title", default = "Ada Lovelace")]
    pub title: String,
    #[setting("Subtitle", default = "Founder, Acme")]
    pub subtitle: String,
    #[setting("Position", default = Corner::BottomLeft)]
    pub position: Corner,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Panel colour", default = "rgba(10,10,12,0.72)", check = colour)]
    pub panel: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Exit time",
        default = 0.4667,
        range(min = 0, max = 2),
        unit = "s",
        advanced
    )]
    pub exit_time: f64,
    #[setting(
        "Slide distance",
        default = 120,
        range(min = 0, max = 600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub slide: u32,
    #[setting(
        "Margin",
        default = 80,
        range(min = 0, max = 300),
        step = 1,
        unit = "px",
        advanced
    )]
    pub margin: u32,
    #[setting(
        "Bar width",
        default = 8,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub bar_width: u32,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Background blur",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_blur: u32,
    #[setting(
        "Title size",
        default = 44,
        range(min = 16, max = 120),
        step = 1,
        unit = "px"
    )]
    pub title_size: u32,
    #[setting(
        "Title weight",
        default = 720,
        range(min = 100, max = 900),
        step = 10,
        advanced
    )]
    pub title_weight: u32,
    #[setting(
        "Subtitle size",
        default = 28,
        range(min = 10, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub subtitle_size: u32,
    #[setting("Subtitle opacity", default = 0.7, range(min = 0, max = 1), advanced)]
    pub subtitle_opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LowerThird>(
        element(
            ElementKind::Component,
            "lower-third",
            "Lower third",
            "A name or caption bar in the lower part of the screen.",
            ElementCategory::Overlay,
            120,
        )
        .look(NodeLook::of("lucide:panel-bottom")),
    )
}

const LOWER_THIRD_SPRING: SpringConfig = SpringConfig {
    damping: 200.0,
    ..DEFAULT_SPRING
};
const GAP_PX: f64 = 18.0;
const PANEL_PAD_Y: f64 = 18.0;
const PANEL_PAD_X: f64 = 28.0;
const TITLE_LETTER_SPACING_EM: f64 = -0.02;
const SUBTITLE_GAP_PX: f64 = 4.0;
const BAR_RADIUS: f64 = 99.0;

pub fn draw(props: &LowerThird, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let fps = cx.fps();
    let OverlayFrame { k, insets } = overlay_frame(cx.at.stage);
    let enter = sprung(frame, fps, LOWER_THIRD_SPRING)?;
    let exit = sprung(
        frame - (cx.duration() - (props.exit_time * fps).round()),
        fps,
        LOWER_THIRD_SPRING,
    )?;
    let top = matches!(props.position, Corner::TopLeft | Corner::TopRight)
        || !is_landscape(cx.at.stage.width, cx.at.stage.height);
    let right = matches!(props.position, Corner::BottomRight | Corner::TopRight);
    let x = f64::from(props.slide) * (if right { 1.0 } else { -1.0 }) * (1.0 - enter + exit);
    let color = cx.color(&props.color);
    let mut text = div(with_background(Style::default(), &props.panel)
        .border_radius_all(f64::from(props.corner_radius) * k)
        .padding(Edges::axes(
            Length::px(PANEL_PAD_Y * k),
            Length::px(PANEL_PAD_X * k),
        ))
        .backdrop_filter(vec![Filter::Blur(Length::px(f64::from(props.panel_blur)))]))
    .child(
        div(Style::default()
            .color(color.clone())
            .font_size(f64::from(props.title_size) * k)
            .font_weight(f64::from(props.title_weight))
            .letter_spacing(Length::em(TITLE_LETTER_SPACING_EM)))
        .child(props.title.as_str()),
    );
    if !props.subtitle.is_empty() {
        text = text.child(
            div(Style::default()
                .color(color)
                .opacity(props.subtitle_opacity)
                .font_size(f64::from(props.subtitle_size) * k)
                .margin_top(SUBTITLE_GAP_PX * k))
            .child(props.subtitle.as_str()),
        );
    }
    let bar = div(with_background(
        Style::default()
            .width(f64::from(props.bar_width) * k)
            .border_radius_all(BAR_RADIUS)
            .transform(vec![Transform::Scale(Num(1.0), Num(enter), Num(1.0))])
            .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO)),
        &props.accent,
    ));
    let row = div(Style::default()
        .display(Display::Flex)
        .gap(GAP_PX * k)
        .align_items(Align::Stretch)
        .transform(vec![Transform::Translate(
            Length::px(x * k),
            Length::ZERO,
            Length::ZERO,
        )])
        .opacity(enter.min(1.0 - exit))
        .font_family(font_or(&props.font_family, FontRole::Main)))
    .child(bar)
    .child(text);
    Ok(div(overlay_padding(
        absolute_fill()
            .justify_content(if top { Align::FlexStart } else { Align::FlexEnd })
            .align_items(if right { Align::FlexEnd } else { Align::FlexStart }),
        f64::from(props.margin) * k,
        insets,
    ))
    .child(row)
    .into())
}
