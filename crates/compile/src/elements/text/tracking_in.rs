use opsis_macros::settings;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Filter;
use opsis_scene::spring::{DEFAULT_SPRING, SpringConfig};
use opsis_scene::style::{Style, WhiteSpace};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped, sprung};
use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct TrackingIn {
    #[setting("Text", default = "tracking in")]
    pub text: String,
    #[setting(
        "Start tracking (em)",
        default = 0.5,
        range(min = 0, max = 2),
        step = 0.05
    )]
    pub start_tracking: f64,
    #[setting(
        "Start blur",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        advanced
    )]
    pub start_blur: u32,
    #[setting("Font size", default = 96, range(min = 12, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("End letter spacing", default = -0.03, range(min = -0.2, max = 1), unit = "em", advanced)]
    pub end_tracking: f64,
    #[setting(
        "Fade in",
        default = 0.5,
        range(min = 0, max = 2),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub fade_in: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TrackingIn>(
        element(
            ElementKind::Component,
            "tracking-in",
            "Tracking in",
            "Letter-spacing collapses and blur clears with a springy motion.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .design(1920, 1080)
        .look(NodeLook::of("lucide:type")),
    )
}

const TRACKING_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 90.0,
    ..DEFAULT_SPRING
};

pub fn draw(props: &TrackingIn, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let t = sprung(frame, Cx::authored_fps(), TRACKING_SPRING)?;
    let tracking = interpolate(
        t,
        &[0.0, 1.0],
        &[props.start_tracking, props.end_tracking],
        &Interpolation::default(),
    )?;
    let blur = interpolate(
        t,
        &[0.0, 1.0],
        &[f64::from(props.start_blur), 0.0],
        &Interpolation::default(),
    )?;
    let opacity = clamped(frame, [0.0, props.fade_in], [0.0, 1.0])?;
    Ok(div(centered_stage(&props.background))
        .child(
            span(
                Style::default()
                    .font_size(f64::from(props.font_size))
                    .font_weight(props.font_weight.value())
                    .color(cx.color(&props.color))
                    .letter_spacing(Length::em(tracking))
                    .opacity(opacity)
                    .filter(vec![Filter::Blur(Length::px(blur))])
                    .font_family(font_or(&props.font_family, FontRole::Main))
                    .white_space(WhiteSpace::Nowrap),
            )
            .child(props.text.as_str()),
        )
        .into())
}
