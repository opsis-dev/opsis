use opsis_macros::{choice, settings};
use opsis_scene::color::Color;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::Shadow;
use opsis_scene::style::{Align, Overflow, Position, Style, TextAlign};
use opsis_scene::timeline::{ChildAt, absolute_fill};
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::parts::handwrite::{HAND_FAMILY, Handwrite, handwrite};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum PhotoShape {
    #[serde(rename = "16:9")]
    #[label("16:9")]
    Wide,
    #[serde(rename = "4:3")]
    #[label("4:3")]
    Classic,
    #[serde(rename = "1:1")]
    #[label("Square")]
    Square,
    #[serde(rename = "3:4")]
    #[label("3:4")]
    Tall,
}

#[settings(export)]
pub struct Polaroid {
    #[setting("Caption", default = "first light")]
    pub caption: String,
    #[setting("Caption starts", range(min = 0, max = 60), step = 1, advanced)]
    pub caption_at: u32,
    #[setting("Card width", default = 652, range(min = 320, max = 900), step = 4)]
    pub width: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Caption size",
        range(min = 12, max = 56),
        zero_label = "Fit",
        step = 1,
        unit = "px",
        help = "Leave empty to size the caption with the card.",
        "type" = "integer",
        advanced
    )]
    pub caption_size: Option<u32>,
    #[setting(
        "Time per pose",
        default = 0.1,
        range(min = 0.0333, max = 0.2),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub step: f64,
    #[setting("Frame colour", default = "#fdfcf8", check = colour)]
    pub frame_color: String,
    #[setting("Caption colour", default = "#26242c", check = colour)]
    pub caption_color: String,
    #[setting(
        "Photo shape",
        default = PhotoShape::Wide,
        advanced
    )]
    pub photo_shape: PhotoShape,
    #[setting(
        "Caption strip height",
        default = 62,
        range(min = 30, max = 200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub caption_strip: u32,
    #[setting(
        "Rounded corners",
        default = 4,
        range(min = 0, max = 24),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Caption weight",
        default = 600,
        range(min = 400, max = 700),
        step = 100,
        advanced
    )]
    pub caption_weight: u32,
    #[setting("Letters per pose", default = 1.4, range(min = 0.5, max = 4), advanced)]
    pub write_speed: f64,
    #[setting("Caption font", widget = "font", zero_label = "Handwriting", advanced)]
    pub caption_font: String,
    #[setting("Photo background", default = "#100f14", widget = "color", check = colour, advanced)]
    pub photo_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Polaroid>(
            element(
                ElementKind::Wrapper,
                "polaroid",
                "Polaroid",
                "An instant-photo frame with a handwritten caption for video, images, or live compositions.",
                ElementCategory::Effect,
                150,
            )
                .fonts(&["Caveat"])
                .look(NodeLook::of("tabler:polaroid")),
        )
}

const BASE_WIDTH: f64 = 652.0;
const BASE_PAD: f64 = 16.0;
const BASE_MEDIA_WIDTH: f64 = 620.0;
const BASE_CAPTION_SIZE: f64 = 36.0;

const fn photo_height(shape: PhotoShape) -> f64 {
    match shape {
        PhotoShape::Wide => 349.0,
        PhotoShape::Classic => 465.0,
        PhotoShape::Square => 620.0,
        PhotoShape::Tall => 827.0,
    }
}

pub fn wrap(props: &Polaroid, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let child = child(cx.at.stage)?;
    let width = f64::from(props.width);
    let scale = width / BASE_WIDTH;
    let pad = BASE_PAD * scale;
    let media_width = BASE_MEDIA_WIDTH * scale;
    let media_height = photo_height(props.photo_shape) * scale;
    let caption_height = f64::from(props.caption_strip) * scale;
    let height = pad + media_height + caption_height;
    let caption_size = props
        .caption_size
        .map_or(BASE_CAPTION_SIZE * scale, f64::from)
        .min(caption_height * 0.62)
        .min(media_width / (float(props.caption.chars().count()) * 0.52).max(1.0));
    let shadow = |y: f64, blur: f64, alpha: f64| Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(y),
        blur: Length::px(blur),
        spread: Length::ZERO,
        color: Color::srgb_alpha(38.0 / 255.0, 36.0 / 255.0, 44.0 / 255.0, alpha),
    };
    let mut strip = div(Style::default()
        .position(Position::Relative)
        .height(Length::px(caption_height)));
    if !props.caption.is_empty() {
        strip = strip.child(handwrite(
            &Handwrite {
                text: &props.caption,
                font_size: caption_size,
                color: &props.caption_color,
                delay: f64::from(props.caption_at),
                per_step: props.write_speed,
                weight: f64::from(props.caption_weight),
                centered: true,
                font_family: if props.caption_font.is_empty() {
                    HAND_FAMILY
                } else {
                    &props.caption_font
                },
                step: props.step,
                wobble: 1.0,
            },
            cx.authored_frame(),
        ));
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(
        div(Style::default()
            .width(Length::px(width))
            .height(Length::px(height))
            .background_color(css_color(&props.frame_color))
            .border_radius_all(f64::from(props.corner_radius))
            .box_shadow(vec![shadow(8.0, 20.0, 0.18), shadow(2.0, 5.0, 0.12)])
            .padding_top(Length::px(pad)))
        .child(
            div(Style::default()
                .position(Position::Relative)
                .width(Length::px(media_width))
                .height(Length::px(media_height))
                .margin(Edges::axes(Length::ZERO, Length::Auto))
                .background_color(css_color(&props.photo_color))
                .overflow(Overflow::Hidden))
            .child(child),
        )
        .child(strip),
    )
    .into())
}
