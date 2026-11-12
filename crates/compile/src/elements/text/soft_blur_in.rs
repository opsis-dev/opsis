use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Filter;
use opsis_scene::style::{Display, Style, Visibility, WhiteSpace};
use opsis_scene::values::Length;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::float;

use crate::elements::anim::clamped_eased;
use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SoftBlurIn {
    #[setting("Text", default = "Made for every take.")]
    pub text: String,
    #[setting("Blur", default = 12, range(min = 1, max = 30), step = 1)]
    pub blur: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
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
    #[setting(
        "Stagger",
        default = 0.0333,
        range(min = 0, max = 0.5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub stagger: f64,
    #[setting(
        "Distance",
        default = 16,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub distance: u32,
    #[setting("Letter spacing", default = -0.05, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SoftBlurIn>(
        element(
            ElementKind::Component,
            "soft-blur-in",
            "Soft blur in",
            "Per-character fade-in with a gentle blur and upward motion.",
            ElementCategory::Text,
            60,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::BlurSwap)),
    )
}

const CHAR_DURATION_FRAMES: f64 = 27.0;
const CHAR_TRAVEL_FRAMES: f64 = 9.0;
const ORIGIN_DOWN_PERCENT: f64 = 55.0;

const fn soft_easing() -> Easing {
    Easing::Bezier(0.22, 1.0, 0.36, 1.0)
}

pub fn draw(props: &SoftBlurIn, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let chars = props
        .text
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let local = float(i).mul_add(-props.stagger, frame);
            let opacity = clamped_eased(
                local,
                [0.0, CHAR_DURATION_FRAMES],
                [0.0, 1.0],
                soft_easing(),
            )?;
            let y = clamped_eased(
                local,
                [0.0, CHAR_TRAVEL_FRAMES],
                [f64::from(props.distance), 0.0],
                soft_easing(),
            )?;
            let blur = clamped_eased(
                local,
                [0.0, CHAR_DURATION_FRAMES],
                [f64::from(props.blur), 0.0],
                soft_easing(),
            )?;
            Ok(span(
                Style::default()
                    .display(Display::InlineBlock)
                    .white_space(WhiteSpace::Pre)
                    .backface_visibility(Visibility::Hidden)
                    .transform_origin((
                        Length::percent(50.0),
                        Length::percent(ORIGIN_DOWN_PERCENT),
                        Length::ZERO,
                    ))
                    .opacity(opacity)
                    .translate(Length::ZERO, Length::px(y))
                    .filter(vec![Filter::Blur(Length::px(blur))]),
            )
            .child(c.to_string())
            .into())
        })
        .collect::<Result<Vec<Node>, DrawFailure>>()?;
    Ok(div(centered_stage(&props.background))
        .child(
            span(
                Style::default()
                    .font_size(f64::from(props.font_size))
                    .font_weight(props.font_weight.value())
                    .color(cx.color(&props.color))
                    .letter_spacing(Length::em(props.letter_spacing))
                    .font_family(font_or(&props.font_family, FontRole::Main)),
            )
            .children(chars),
        )
        .into())
}
