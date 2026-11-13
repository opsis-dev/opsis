use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{ObjectFit, Overflow, Position, Style};
use opsis_scene::timeline::{DrawError, Stage, absolute_fill};
use opsis_scene::values::{Angle, Length, Position2, Px};
use opsis_types::recording::RecordingDevice;

use crate::elements::camera::click_log::design_scale_for;
use crate::elements::camera::window::{Chrome, Frame, window_frame};
use crate::elements::device::laptop_frame::{self, LaptopEntrance, LaptopFinale, LaptopFrame};
use crate::elements::device::phone_frame::{self, PhoneFrame};
use crate::elements::draw::{Cx, DrawFailure, tinted};
use crate::elements::fonts::{FontRole, font_or};
use crate::nodes::image_frame::{FramedImageProps, ImageTilt};
use crate::recording::DEFAULT_FRAME_THEME;

fn ground(props: &FramedImageProps) -> Option<Node> {
    if !props.background_image.is_empty() {
        return Some(
            div(absolute_fill())
                .child(img(
                    props.background_image.clone(),
                    Style::default()
                        .width(Length::percent(100.0))
                        .height(Length::percent(100.0))
                        .object_fit(ObjectFit::Cover),
                ))
                .into(),
        );
    }
    (!props.background.is_empty())
        .then(|| div(with_background(absolute_fill(), &props.background)).into())
}

fn tilt_transform(tilt: ImageTilt, long_side: f64) -> Vec<Transform> {
    vec![
        Transform::Perspective(Some(Px(tilt.perspective * long_side))),
        Transform::RotateY(Angle::deg(-tilt.turn)),
        Transform::RotateX(Angle::deg(tilt.tilt)),
        Transform::Rotate(Angle::deg(tilt.lean)),
    ]
}

fn shadow_of(props: &FramedImageProps) -> Option<Shadow> {
    if props.shadow_opacity <= 0.0 {
        return None;
    }
    let alpha = opsis_scene::clock::js_round(props.shadow_opacity.min(1.0) * 100.0);
    Some(Shadow {
        inset: false,
        x: Length::ZERO,
        y: Length::px(props.shadow_drop),
        blur: Length::px(props.shadow_softness),
        spread: Length::ZERO,
        color: tinted(&props.shadow_color, alpha / 100.0),
    })
}

fn picture(src: &str) -> Node {
    img(
        src.to_owned(),
        Style::default()
            .width(Length::percent(100.0))
            .height(Length::percent(100.0))
            .object_fit(ObjectFit::Cover)
            .object_position(Position2 {
                x: Length::percent(50.0),
                y: Length::ZERO,
            }),
    )
    .into()
}

fn on_device(
    props: &FramedImageProps,
    device: RecordingDevice,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let screen = |_: Stage| -> Result<Node, DrawError> { Ok(picture(&props.src)) };
    match device {
        RecordingDevice::Phone => phone_frame::wrap(
            &PhoneFrame {
                float_loop: false,
                ..PhoneFrame::default()
            },
            &screen,
            cx,
        ),
        RecordingDevice::Laptop => laptop_frame::wrap(
            &LaptopFrame {
                entrance: LaptopEntrance::None,
                finale: LaptopFinale::None,
                float_loop: false,
                show_notch: false,
                ..LaptopFrame::default()
            },
            &screen,
            cx,
        ),
    }
}

fn placed(props: &FramedImageProps, child: Node) -> Node {
    let mut style = Style::default()
        .position(Position::Absolute)
        .left(Length::px(props.x))
        .top(Length::px(props.y))
        .width(Length::px(props.width))
        .height(Length::px(props.height))
        .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO))
        .border_radius_all(props.radius);
    if let Some(tilt) = props.tilt {
        style = style.transform(tilt_transform(tilt, props.width.max(props.height)));
    }
    if let Some(shadow) = shadow_of(props) {
        style = style.box_shadow(vec![shadow]);
    }
    div(style).child(child).into()
}

pub fn draw(props: &FramedImageProps, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if props.src.is_empty() {
        return Ok(div(Style::default()).into());
    }
    let mut root = div(absolute_fill()).children(ground(props));
    if let Some(device) = props.device {
        return Ok(root.child(on_device(props, device, cx)?).into());
    }
    let inner: Node = match props.window {
        Some(style) => window_frame(
            &Frame {
                chrome: Some(Chrome {
                    style,
                    theme: props.theme.unwrap_or(DEFAULT_FRAME_THEME),
                    address_bar: false,
                    address: "",
                    font: font_or("", FontRole::Main),
                }),
                radius: None,
                style,
                lift: true,
            },
            vec![picture(&props.src)],
            design_scale_for(cx.at.stage.width),
        )
        .into(),
        None => div(Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .border_radius_all(props.radius)
            .overflow(Overflow::Hidden))
        .child(picture(&props.src))
        .into(),
    };
    root = root.child(placed(props, inner));
    Ok(root.into())
}
