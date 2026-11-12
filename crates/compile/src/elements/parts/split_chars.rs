use opsis_scene::color::Color;
use opsis_scene::node::{Element, Node, span};
use opsis_scene::paint::Filter;
use opsis_scene::style::{Align, Display, FlexDirection, LineHeight, Style, TextAlign, WhiteSpace};
use opsis_scene::values::{Length, Num};
use opsis_types::numbers::whole;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::text_split::{SplitTiming, Unit, split_text};
use crate::elements::units::to_fixed;

const REFERENCE_WIDTH: f64 = 1080.0;
const DEFAULT_FONT_AT_1080: f64 = 84.0;
const BLUR_PER_DISPLACE: f64 = 8.0;
const MIN_BLUR: f64 = 0.1;
const SCALE_FROM: f64 = 0.72;
const SCALE_GROWTH: f64 = 0.28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitEffect {
    FadeUp,
    Fade,
    Scale,
    Blur,
    None,
}

#[derive(Debug, Clone)]
pub struct SplitChars {
    pub text: String,
    pub timing: SplitTiming,
    pub effect: SplitEffect,
    pub travel: f64,
    pub font_size: Option<f64>,
    pub color: Color,
    pub font_weight: f64,
    pub font_family: Option<Vec<String>>,
    pub line_height: f64,
    pub letter_spacing: Option<Length>,
    pub text_align: TextAlign,
    pub word_spacing: f64,
    pub style: Style,
}

impl Default for SplitChars {
    fn default() -> Self {
        Self {
            text: String::new(),
            timing: SplitTiming::default(),
            effect: SplitEffect::FadeUp,
            travel: 0.42,
            font_size: None,
            color: Color::rgb8(0xf4, 0xf4, 0xf5),
            font_weight: 600.0,
            font_family: None,
            line_height: 1.15,
            letter_spacing: None,
            text_align: TextAlign::Center,
            word_spacing: 0.26,
            style: Style::default(),
        }
    }
}

pub fn scale_font(size_at_1080: f64, width: f64) -> f64 {
    f64::from(whole(size_at_1080 * (width / REFERENCE_WIDTH)))
}

fn unit_style(effect: SplitEffect, unit: &Unit, travel: f64) -> Style {
    let base = Style::default()
        .display(Display::InlineBlock)
        .white_space(WhiteSpace::Pre);
    match effect {
        SplitEffect::None => base,
        SplitEffect::Fade => base.opacity(unit.opacity),
        SplitEffect::Scale => {
            let scale = unit.value.mul_add(SCALE_GROWTH, SCALE_FROM);
            base.opacity(unit.opacity)
                .scale(scale, scale)
                .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO))
        }
        SplitEffect::Blur => {
            let blur = unit.displace * BLUR_PER_DISPLACE;
            let base = base.opacity(unit.opacity);
            if blur > MIN_BLUR {
                base.filter(vec![Filter::Blur(Length::px(to_fixed(blur, 2)))])
            } else {
                base
            }
        }
        SplitEffect::FadeUp => base.opacity(unit.opacity).translate(
            Length::ZERO,
            Length::em(to_fixed(unit.displace * travel, 4)),
        ),
    }
}

const fn cross_align(align: TextAlign) -> Align {
    match align {
        TextAlign::Left => Align::FlexStart,
        TextAlign::Right => Align::FlexEnd,
        _ => Align::Center,
    }
}

pub type RenderUnit<'r> = &'r dyn Fn(&Unit) -> Result<Vec<Node>, DrawFailure>;

pub fn split_chars(
    props: &SplitChars,
    cx: &Cx<'_>,
    render: Option<RenderUnit<'_>>,
) -> Result<Element, DrawFailure> {
    let font_size = props
        .font_size
        .unwrap_or_else(|| scale_font(DEFAULT_FONT_AT_1080, cx.at.stage.width));
    let lines = split_text(&props.text, &props.timing, cx.frame(), cx.duration())?;
    let mut style = Style::default()
        .display(Display::InlineFlex)
        .flex_direction(FlexDirection::Column)
        .align_items(cross_align(props.text_align))
        .font_size(font_size)
        .font_weight(props.font_weight)
        .color(props.color.clone())
        .line_height(LineHeight::Number(Num(props.line_height)))
        .text_align(props.text_align);
    if let Some(spacing) = &props.letter_spacing {
        style = style.letter_spacing(spacing.clone());
    }
    if let Some(family) = &props.font_family {
        style = style.font_family(family.clone());
    }
    let mut rows = Vec::new();
    for line in lines {
        let last = line.words.len().saturating_sub(1);
        let mut words = Vec::new();
        for (i, word) in line.words.iter().enumerate() {
            let gap = if i == last {
                Length::ZERO
            } else {
                Length::em(props.word_spacing)
            };
            let mut units = Vec::new();
            for unit in &word.units {
                let inner = match render {
                    Some(render) => render(unit)?,
                    None => vec![Node::from(unit.text.as_str())],
                };
                units.push(
                    span(unit_style(props.effect, unit, props.travel))
                        .children(inner)
                        .into(),
                );
            }
            words.push(
                span(
                    Style::default()
                        .display(Display::InlineBlock)
                        .margin_right(gap),
                )
                .children(units)
                .into(),
            );
        }
        rows.push(
            span(
                Style::default()
                    .display(Display::Block)
                    .white_space(WhiteSpace::Normal),
            )
            .children(words)
            .into(),
        );
    }
    Ok(span(style.merged(&props.style)).children(rows))
}
