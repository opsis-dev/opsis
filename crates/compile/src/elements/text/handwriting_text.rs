use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::css::font_family;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{ColorStop, Gradient, LinearDirection, MaskLayer, MaskMode, Side};
use opsis_scene::random::random;
use opsis_scene::style::{Align, Display, Position, Style, TextAlign, WhiteSpace};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Length};
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::split_chars::{SplitChars, SplitEffect, split_chars};
use crate::elements::text_split::{SplitMode, SplitOrder, SplitTiming, Unit};
use crate::elements::units::to_fixed;
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct HandwritingText {
    #[setting(
        "Text",
        default = "Signed by hand",
        help = "The line that gets written."
    )]
    pub text: String,
    #[setting(
        "Pen size",
        default = 0.14,
        step = 0.01,
        help = "Diameter of the nib, in em. 0 hides it.",
        advanced
    )]
    pub pen_size: f64,
    #[setting("Pen color", default = "#e8b86d", help = "Nib colour.", check = colour, advanced)]
    pub pen_color: String,
    #[setting(
        "Ink softness",
        default = 0.18,
        step = 0.01,
        help = "Softness of the ink edge, as a share of one glyph.",
        advanced
    )]
    pub ink_softness: f64,
    #[setting(
        "Wobble",
        default = 1.6,
        step = 0.1,
        help = "Per-character tilt and baseline drift, in degrees. Fixed, not animated.",
        advanced
    )]
    pub wobble: f64,
    #[setting(
        "Stagger",
        default = 3,
        unit = "frames",
        help = "Frames between one character and the next. This is the writing speed."
    )]
    pub stagger_in_frames: u32,
    #[setting(
        "Font",
        widget = "font",
        zero_label = "Handwriting",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Variation", default = "handwriting", advanced)]
    pub seed: String,
    #[setting("Color", default = "#f4f4f5", check = colour)]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Font size",
        range(min = 12, max = 600),
        zero_label = "Auto",
        step = 1,
        unit = "px",
        help = "Leave empty to size the text to the frame.",
        "type" = "integer"
    )]
    pub font_size: Option<u32>,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Stroke time",
        default = 10,
        range(min = 1, max = 60),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Word spacing",
        default = 0.26,
        range(min = 0, max = 1),
        unit = "em",
        advanced
    )]
    pub word_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<HandwritingText>(
        element(
            ElementKind::Component,
            "handwriting-text",
            "Handwriting",
            "Writes the text out by hand, stroke by stroke.",
            ElementCategory::Text,
            120,
        )
        .fonts(&["Caveat"])
        .look(NodeLook::of("lucide:signature").moves(NodeMotion::Draw)),
    )
}

const HAND_FAMILY: &str = "\"Caveat\"";
const WRITING_FROM: f64 = 0.02;
const WRITING_UNTIL: f64 = 0.98;
const DRIFT_EM: f64 = 0.012;
const PEN_LIFT_EM: f64 = 0.08;

fn jitter(seed: &str, index: usize, salt: u32) -> f64 {
    random(format!("{seed}-{index}-{salt}")).mul_add(2.0, -1.0)
}

fn ink_mask(stop: f64, softness: f64) -> MaskLayer {
    let ink = Gradient::Linear {
        direction: LinearDirection::ToSide {
            horizontal: Some(Side::Right),
            vertical: None,
        },
        stops: vec![
            ColorStop::at(Color::BLACK, Length::percent(to_fixed(stop, 2))),
            ColorStop::at(
                Color::TRANSPARENT,
                Length::percent(to_fixed((stop + softness).min(100.0), 2)),
            ),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    MaskLayer {
        layer: ink.layer(),
        mode: MaskMode::Alpha,
    }
}

fn unit_node(props: &HandwritingText, nib: &Color, softness: f64, unit: &Unit) -> Node {
    let written = unit.enter.clamp(0.0, 1.0);
    let stop = written * 100.0;
    let tilt = jitter(&props.seed, unit.index, 1) * props.wobble;
    let drift = jitter(&props.seed, unit.index, 2) * props.wobble * DRIFT_EM;
    let writing = written > WRITING_FROM && written < WRITING_UNTIL;
    let letter = span(Style::default()
        .display(Display::InlineBlock)
        .mask(vec![ink_mask(stop, softness)]))
    .child(unit.text.as_str());
    let mut wrapper = span(Style::default()
        .position(Position::Relative)
        .display(Display::InlineBlock)
        .white_space(WhiteSpace::Pre)
        .opacity(unit.opacity)
        .rotate(Angle::deg(to_fixed(tilt, 3)))
        .translate(Length::ZERO, Length::em(to_fixed(drift, 4))))
    .child(letter);
    if props.pen_size > 0.0 {
        let pen = Length::em(props.pen_size);
        wrapper = wrapper.child(span(Style::default()
            .position(Position::Absolute)
            .left(Length::percent(to_fixed(stop, 2)))
            .bottom(Length::em(PEN_LIFT_EM))
            .width(pen.clone())
            .height(pen)
            .margin_left(Length::em(to_fixed(-props.pen_size / 2.0, 4)))
            .border_radius_all(Length::percent(50.0))
            .background_color(nib.clone())
            .opacity(if writing { unit.opacity } else { 0.0 })));
    }
    wrapper.into()
}

pub fn draw(props: &HandwritingText, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let nib = cx.color(&props.pen_color);
    let softness = props.ink_softness.max(0.0) * 100.0;
    let render = |unit: &Unit| -> Result<Vec<Node>, DrawFailure> {
        Ok(vec![unit_node(props, &nib, softness, unit)])
    };
    let family = if props.font_family.is_empty() {
        HAND_FAMILY
    } else {
        &props.font_family
    };
    let chars = split_chars(
        &SplitChars {
            text: props.text.clone(),
            timing: SplitTiming {
                mode: SplitMode::Chars,
                order: SplitOrder::Start,
                stagger: Some(f64::from(props.stagger_in_frames)),
                duration: Some(f64::from(props.duration_in_frames)),
                ..SplitTiming::default()
            },
            effect: SplitEffect::None,
            color: cx.color(&props.color),
            font_family: Some(font_family(family)),
            font_size: props.font_size.map(f64::from),
            font_weight: props.font_weight.value(),
            word_spacing: props.word_spacing,
            ..SplitChars::default()
        },
        cx,
        Some(&render),
    )?;
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(chars)
    .into())
}
