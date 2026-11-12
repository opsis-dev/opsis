use opsis_scene::color::Color;
use opsis_scene::node::{Element, span};
use opsis_scene::style::{Display, Style};
use opsis_scene::values::{Corners, Length};
use opsis_types::numbers::truncate;

use crate::elements::draw::Cx;

#[derive(Debug, Clone)]
pub struct Caret {
    pub color: Color,
    pub width: Length,
    pub height: Length,
    pub radius: f64,
    pub opacity: Option<f64>,
    pub blink: bool,
    pub blink_per_second: f64,
    pub speed: f64,
    pub margin_left: f64,
}

impl Default for Caret {
    fn default() -> Self {
        Self {
            color: Color::CurrentColor,
            width: Length::px(2.0),
            height: Length::px(18.0),
            radius: 1.0,
            opacity: None,
            blink: false,
            blink_per_second: 1.0,
            speed: 1.0,
            margin_left: 0.0,
        }
    }
}

pub fn blink_opacity(frame: f64, fps: f64, blink_per_second: f64, speed: f64) -> f64 {
    let cycles = if blink_per_second <= 0.0 {
        1.0
    } else {
        blink_per_second
    };
    let half_period = fps / cycles / 2.0;
    if half_period <= 0.0 {
        return 1.0;
    }
    let step: i64 = truncate((frame * speed / half_period).floor());
    if step.rem_euclid(2) == 0 { 1.0 } else { 0.0 }
}

pub fn caret(props: &Caret, cx: &Cx<'_>, extra: &Style) -> Element {
    let opacity = props.opacity.unwrap_or_else(|| {
        if props.blink {
            blink_opacity(
                cx.authored_frame(),
                Cx::authored_fps(),
                props.blink_per_second,
                props.speed,
            )
        } else {
            1.0
        }
    });
    span(
        Style::default()
            .display(Display::InlineBlock)
            .flex_shrink(0.0)
            .width(props.width.clone())
            .height(props.height.clone())
            .border_radius(Corners::all((
                Length::px(props.radius),
                Length::px(props.radius),
            )))
            .background_color(props.color.clone())
            .opacity(opacity)
            .margin_left(props.margin_left)
            .merged(extra),
    )
}
