use std::f64::consts::PI;

use opsis_macros::{choice, settings};
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Shadow;
use opsis_scene::random::{js_number_text, random};
use opsis_scene::style::{Align, Display, Style, TextAlign, WhiteSpace};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::parts::split_chars::{SplitChars, SplitEffect, split_chars};
use crate::elements::shared::FontWeight;
use crate::elements::text_split::{SplitOrder, SplitTiming, Unit};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum FlickerOrder {
    Start,
    End,
    Center,
    Edges,
    Random,
}

#[settings(export)]
pub struct NeonFlickerText {
    #[setting("Text", default = "Open all night", help = "The sign.")]
    pub text: String,
    #[setting(
        "Glow color",
        default = "#f472b6",
        help = "Colour of the halo around the tube.",
        check = colour
    )]
    pub glow_color: String,
    #[setting(
        "Glow size",
        default = 26.0,
        help = "Four stacked shadows, so it stays soft."
    )]
    pub glow_size: f64,
    #[setting(
        "Unlit brightness",
        default = 0.14,
        step = 0.01,
        help = "Brightness of an unlit tube, 0–1.",
        advanced
    )]
    pub off_level: f64,
    #[setting(
        "Hum",
        default = 0.12,
        step = 0.01,
        help = "Depth of the mains hum once the sign has settled.",
        advanced
    )]
    pub hum: f64,
    #[setting(
        "Buzz",
        default = 0.06,
        step = 0.01,
        help = "Chance per beat that a settled tube stutters. 0 disables it.",
        advanced
    )]
    pub buzz: f64,
    #[setting(
        "Flicker on exit",
        default = true,
        help = "Stutter back off instead of fading cleanly.",
        advanced
    )]
    pub flicker_on_exit: bool,
    #[setting("Order", default = FlickerOrder::Start, help = "Which tube strikes first.", advanced)]
    pub order: FlickerOrder,
    #[setting(
        "Stagger",
        default = 2,
        unit = "frames",
        help = "Frames between one tube striking and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Variation", default = 1, advanced)]
    pub seed: u32,
    #[setting("Color", default = "#fdf2f8", check = colour)]
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
        "Unlit color",
        widget = "color",
        zero_label = "From the glow",
        help = "The tube with no gas lit: cold glass, not invisible. Leave empty for a faint glow colour.",
        advanced
    )]
    pub off_color: String,
    #[setting(
        "Strike time",
        default = 20,
        range(min = 2, max = 60),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Hum rate",
        default = 2.0,
        range(min = 0.2, max = 10),
        unit = "Hz",
        advanced
    )]
    pub hum_rate: f64,
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
    meta::<NeonFlickerText>(
        element(
            ElementKind::Component,
            "neon-flicker-text",
            "Neon sign",
            "Text that flickers on like a neon sign, then hums gently.",
            ElementCategory::Text,
            120,
        )
        .look(NodeLook::of("lucide:type")),
    )
}

const STRIKE_FLOOR: f64 = 0.22;
const STRIKE_RISE: f64 = 0.78;
const HUM_PHASE_PER_UNIT: f64 = 1.3;
const STUTTER_LIFT: f64 = 0.2;
const LIT_MARGIN: f64 = 0.001;
const OFF_TINT_PERCENT: f64 = 16.0;
const GLOW_SHARES: [f64; 4] = [0.16, 0.5, 1.0, 2.2];

const fn split_order(order: FlickerOrder) -> SplitOrder {
    match order {
        FlickerOrder::Start => SplitOrder::Start,
        FlickerOrder::End => SplitOrder::End,
        FlickerOrder::Center => SplitOrder::Center,
        FlickerOrder::Edges => SplitOrder::Edges,
        FlickerOrder::Random => SplitOrder::Random,
    }
}

fn beat(frame: f64, every: f64) -> i64 {
    truncate((frame / every).floor())
}

fn one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn lit_level(props: &NeonFlickerText, unit: &Unit, frame: f64, hum_step: f64) -> f64 {
    let seed = f64::from(props.seed);
    let index = float(unit.index);
    let mut lit = if unit.enter <= 0.0 {
        0.0
    } else if unit.enter < 1.0 {
        let roll = random(format!(
            "{}:{}:{}",
            props.seed,
            unit.index,
            beat(frame, 2.0)
        ));
        if roll < STRIKE_RISE.mul_add(unit.enter, STRIKE_FLOOR) {
            1.0
        } else {
            props.off_level
        }
    } else {
        let wave = 0.5_f64.mul_add(index.mul_add(HUM_PHASE_PER_UNIT, frame * hum_step).sin(), 0.5);
        let stutter = props.buzz > 0.0
            && random(format!(
                "{}:{}:{}",
                js_number_text(seed + 7.0),
                unit.index,
                beat(frame, 3.0)
            )) < props.buzz;
        if stutter {
            props.off_level + STUTTER_LIFT
        } else {
            props.hum.mul_add(-wave, 1.0)
        }
    };
    if props.flicker_on_exit && unit.exit > 0.0 {
        let dying = random(format!(
            "{}:{}:{}",
            js_number_text(seed + 3.0),
            unit.index,
            beat(frame, 2.0)
        ));
        if dying < unit.exit {
            lit = props.off_level;
        }
    }
    lit
}

pub fn draw(props: &NeonFlickerText, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let glow_color = cx.color(&props.glow_color);
    let off_color = if props.off_color.is_empty() {
        Color::mix(
            ColorInterpolation::DEFAULT,
            (glow_color.clone(), Some(OFF_TINT_PERCENT)),
            (Color::TRANSPARENT, None),
        )
    } else {
        cx.color(&props.off_color)
    };
    let on_color = cx.color(&props.color);
    let hum_step = props.hum_rate * PI * 2.0 / cx.fps();
    let render = |unit: &Unit| -> Result<Vec<Node>, DrawFailure> {
        let lit = lit_level(props, unit, frame, hum_step);
        let on = lit > props.off_level + LIT_MARGIN;
        let glow = lit.max(0.0);
        let mut style = Style::default()
            .display(Display::InlineBlock)
            .white_space(WhiteSpace::Pre)
            .opacity(if unit.enter > 0.0 {
                1.0 - unit.exit
            } else {
                0.0
            })
            .color(if on {
                on_color.clone()
            } else {
                off_color.clone()
            });
        if on {
            style = style.text_shadow(
                GLOW_SHARES
                    .iter()
                    .map(|share| Shadow {
                        inset: false,
                        x: Length::ZERO,
                        y: Length::ZERO,
                        blur: Length::px(one_decimal(props.glow_size * share * glow)),
                        spread: Length::ZERO,
                        color: glow_color.clone(),
                    })
                    .collect::<Vec<_>>(),
            );
        }
        Ok(vec![span(style).child(unit.text.as_str()).into()])
    };
    let chars = split_chars(
        &SplitChars {
            text: props.text.clone(),
            timing: SplitTiming {
                stagger: Some(f64::from(props.stagger_in_frames)),
                duration: Some(f64::from(props.duration_in_frames)),
                order: split_order(props.order),
                seed: f64::from(props.seed),
                exit: true,
                ..SplitTiming::default()
            },
            effect: SplitEffect::None,
            font_size: props.font_size.map(f64::from),
            font_weight: props.font_weight.value(),
            word_spacing: props.word_spacing,
            color: on_color.clone(),
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
