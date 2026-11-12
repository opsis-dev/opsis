use opsis_macros::settings;
use opsis_scene::drawing::Font;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::{Display, Style, WhiteSpace};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float};

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct WordPush {
    #[setting("Text", default = "move at your own pace")]
    pub text: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Frames / word", default = 10, range(min = 3, max = 30), step = 1)]
    pub word_gap: u32,
    #[setting(
        "Gap shrink / word",
        default = 0.8,
        range(min = 0.5, max = 1),
        step = 0.05,
        advanced
    )]
    pub accel: f64,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W400)]
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
        "Push time",
        default = 0.4,
        range(min = 0.1, max = 1.5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub push_time: f64,
    #[setting(
        "Drive-in distance",
        default = 0.5,
        range(min = 0, max = 3),
        unit = "em",
        advanced
    )]
    pub drive_distance: f64,
    #[setting("Letter spacing", default = -0.02, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<WordPush>(
            element(
                ElementKind::Component,
                "word-push",
                "Word push",
                "A phrase builds from its first word centered: each next word drives in from the right and pushes the line left, easing into the new center.",
                ElementCategory::Text,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type")),
        )
}

const REVEAL_FRAMES: f64 = 2.0;
const SPACE_WIDTH_EM: f64 = 0.22;

fn reveal_frame(index: usize, word_gap: f64, accel: f64) -> f64 {
    if exact_eq(accel, 1.0) {
        return float(index) * word_gap;
    }
    word_gap * (1.0 - accel.powf(float(index))) / (1.0 - accel)
}

fn push_easing() -> Easing {
    Easing::ease_out(Easing::Cubic)
}

const fn drive_easing() -> Easing {
    Easing::Bezier(0.15, 0.7, 0.25, 1.0)
}

pub fn draw(props: &WordPush, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let face = font_or(&props.font_family, FontRole::Main);
    let frame = cx.authored_frame() * props.speed;
    let size = f64::from(props.font_size);
    let gap = f64::from(props.word_gap);
    let words: Vec<&str> = props.text.split_whitespace().collect();
    let font = Font {
        families: face.clone(),
        size: Num(size),
        weight: Num(props.font_weight.value()),
        italic: false,
    };
    let mut line_x = clamped_eased(
        frame,
        [0.0, props.push_time],
        [-props.drive_distance * size, 0.0],
        drive_easing(),
    )?;
    for (j, word) in words.iter().enumerate().skip(1) {
        let width = SPACE_WIDTH_EM.mul_add(size, cx
            .measure
            .text_width(word, &font, props.letter_spacing * size));
        let reveal = reveal_frame(j, gap, props.accel);
        let pushed = clamped_eased(
            frame,
            [reveal + 1.0, reveal + 1.0 + props.push_time],
            [0.0, 1.0],
            push_easing(),
        )?;
        line_x = (width / 2.0).mul_add(1.0 - pushed, line_x);
    }
    let last = words.len().saturating_sub(1);
    let spans = words
        .iter()
        .enumerate()
        .map(|(j, word)| {
            let reveal = reveal_frame(j, gap, props.accel);
            let opacity = clamped(frame, [reveal, reveal + REVEAL_FRAMES], [0.0, 1.0])?;
            let driven = if j == 0 {
                1.0
            } else {
                clamped_eased(
                    frame,
                    [reveal, reveal + props.push_time],
                    [0.0, 1.0],
                    drive_easing(),
                )?
            };
            let mut style = Style::default()
                .display(Display::InlineBlock)
                .white_space(WhiteSpace::Pre)
                .opacity(opacity)
                .translate(
                    Length::px(props.drive_distance * size * (1.0 - driven)),
                    Length::ZERO,
                );
            if j < last {
                style = style.margin_right(Length::em(SPACE_WIDTH_EM));
            }
            Ok(span(style).child(*word).into())
        })
        .collect::<Result<Vec<Node>, DrawFailure>>()?;
    Ok(div(centered_stage(&props.background)
        .white_space(WhiteSpace::Nowrap)
        .translate(Length::px(line_x), Length::ZERO))
    .child(
        span(
            Style::default()
                .font_size(size)
                .font_weight(props.font_weight.value())
                .color(cx.color(&props.color))
                .letter_spacing(Length::em(props.letter_spacing))
                .font_family(face),
        )
        .children(spans),
    )
    .into())
}
