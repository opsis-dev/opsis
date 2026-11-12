use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Transform;
use opsis_scene::spring::{DEFAULT_SPRING, SpringConfig};
use opsis_scene::style::{Align, Display, FlexWrap, LineHeight, Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::{clamped, clamped_eased, exit_opacity, sprung};
use crate::elements::backgrounds::backgrounded;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct KineticHeadline {
    #[setting(
        "Text — wrap words in *asterisks* to emphasise",
        default = "Ship *faster* with demos that *move*"
    )]
    pub text: String,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Emphasis colour", default = "#facc15", check = colour)]
    pub emphasis_color: String,
    #[setting("Background", default = "transparent")]
    pub background: String,
    #[setting("Font size", default = 104, range(min = 24, max = 240), step = 1)]
    pub font_size: u32,
    #[setting(
        "Time between words",
        default = 4,
        range(min = 1, max = 20),
        step = 1,
        unit = "frames"
    )]
    pub word_stagger: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 780, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting("Letter spacing", default = -0.04, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 1.05, range(min = 0.7, max = 1.8), advanced)]
    pub line_height: f64,
    #[setting(
        "Side margin",
        default = 8.0,
        range(min = 0, max = 30),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Word gap",
        default = 0.28,
        range(min = 0, max = 1),
        unit = "em",
        advanced
    )]
    pub word_gap: f64,
    #[setting("Rise", default = 0.5, range(min = 0, max = 2), unit = "em", advanced)]
    pub rise: f64,
    #[setting("Tilt", default = 4.0, range(min = -30, max = 30), unit = "°", advanced)]
    pub tilt: f64,
    #[setting("Underline emphasised words", default = true, advanced)]
    pub underline_emphasis: bool,
    #[setting(
        "Fade out",
        default = 10,
        range(min = 1, max = 60),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub fade_out_in_frames: u32,
}

pub fn definition() -> ElementMeta {
    meta::<KineticHeadline>(
        element(
            ElementKind::Component,
            "kinetic-headline",
            "Animated headline",
            "A headline that animates in word by word.",
            ElementCategory::Text,
            90,
        )
        .look(NodeLook::of("lucide:whole-word")),
    )
}

const WORD_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 160.0,
    ..DEFAULT_SPRING
};
const EMPHASIS_SWEEP_FRAMES: [f64; 2] = [8.0, 22.0];
const WORD_FADE_FRAMES: f64 = 6.0;
const ROW_GAP_EM: f64 = 0.08;
const UNDERLINE_BOTTOM_EM: f64 = 0.02;
const UNDERLINE_HEIGHT_EM: f64 = 0.09;
const UNDERLINE_RADIUS: f64 = 99.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmphasisedWord {
    pub word: String,
    pub em: bool,
}

pub fn emphasised_words(text: &str) -> Vec<EmphasisedWord> {
    text.split_whitespace()
        .map(|w| {
            let trimmed = w.strip_prefix('*').unwrap_or(w);
            let trimmed = trimmed.strip_suffix('*').unwrap_or(trimmed);
            EmphasisedWord {
                word: trimmed.to_owned(),
                em: w.starts_with('*') || w.ends_with('*'),
            }
        })
        .collect()
}

fn kinetic_word(
    props: &KineticHeadline,
    word: &EmphasisedWord,
    frame: f64,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let s = sprung(frame, cx.fps(), WORD_SPRING)?;
    let sweep = clamped_eased(
        frame,
        EMPHASIS_SWEEP_FRAMES,
        [0.0, 1.0],
        Easing::ease_out(Easing::Cubic),
    )?;
    let emphasis = cx.color(&props.emphasis_color);
    let mut node = span(Style::default()
        .position(Position::Relative)
        .display(Display::InlineBlock)
        .color(if word.em {
            emphasis.clone()
        } else {
            cx.color(&props.color)
        })
        .opacity(clamped(frame, [0.0, WORD_FADE_FRAMES], [0.0, 1.0])?)
        .transform(vec![
            Transform::Translate(Length::ZERO, Length::em((1.0 - s) * props.rise), Length::ZERO),
            Transform::Rotate(Angle::deg((1.0 - s) * props.tilt)),
        ]))
    .child(word.word.as_str());
    if word.em && props.underline_emphasis {
        node = node.child(span(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .bottom(Length::em(UNDERLINE_BOTTOM_EM))
            .height(Length::em(UNDERLINE_HEIGHT_EM))
            .background_color(emphasis)
            .transform_origin((Length::ZERO, Length::percent(50.0), Length::ZERO))
            .transform(vec![Transform::Scale(Num(sweep), Num(1.0), Num(1.0))])
            .border_radius_all(UNDERLINE_RADIUS)));
    }
    Ok(node.into())
}

pub fn draw(props: &KineticHeadline, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let size = f64::from(props.font_size);
    let words = emphasised_words(&props.text)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            kinetic_word(
                props,
                word,
                float(i).mul_add(-f64::from(props.word_stagger), frame),
                cx,
            )
        })
        .collect::<Result<Vec<Node>, DrawFailure>>()?;
    let body = div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .padding(Edges::axes(Length::ZERO, Length::percent(props.side_margin)))
        .opacity(exit_opacity(
            frame,
            cx.duration(),
            Some(f64::from(props.fade_out_in_frames)),
        )?))
    .child(
        div(Style::default()
            .display(Display::Flex)
            .flex_wrap(FlexWrap::Wrap)
            .justify_content(Align::Center)
            .column_gap(size * props.word_gap)
            .row_gap(size * ROW_GAP_EM)
            .font_family(font_or(&props.font_family, FontRole::Heading))
            .font_size(size)
            .font_weight(f64::from(props.font_weight))
            .letter_spacing(Length::em(props.letter_spacing))
            .line_height(LineHeight::Number(Num(props.line_height))))
        .children(words),
    );
    Ok(div(absolute_fill())
        .child(backgrounded(&props.background, Vec::new()))
        .child(body)
        .into())
}
