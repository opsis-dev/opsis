use opsis_macros::settings;
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::{Style, TextAlign as SceneAlign, VerticalAlign, WhiteSpace};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::parts::caret::{Caret, caret};
use crate::elements::shared::{FontWeight, TextAlign};
use crate::elements::ui_kit::typewriter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Typewriter {
    #[setting("Text", default = "console.log('hello, world')", widget = "textarea")]
    pub text: String,
    #[setting("Show cursor", default = true)]
    pub cursor: bool,
    #[setting("Chars / sec", default = 22, range(min = 4, max = 60), step = 1)]
    pub chars_per_second: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Cursor color", default = "#171717", check = colour, advanced)]
    pub cursor_color: String,
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
        "Cursor width",
        default = 0.08,
        range(min = 0.02, max = 0.6),
        unit = "em",
        advanced
    )]
    pub cursor_width: f64,
    #[setting(
        "Blinks per second",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "/s",
        advanced
    )]
    pub blink_rate: f64,
    #[setting(
        "Align",
        default = TextAlign::Center,
        advanced
    )]
    pub align: TextAlign,
    #[setting("Letter spacing", default = -0.03, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Typewriter>(
        element(
            ElementKind::Component,
            "typewriter",
            "Typewriter",
            "Types the text out letter by letter with a blinking cursor.",
            ElementCategory::Text,
            120,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:text-cursor")),
    )
}

const CARET_GAP_EM: f64 = 0.04;
const TEXT_WIDTH_PERCENT: f64 = 90.0;

pub const fn text_align(align: &TextAlign) -> SceneAlign {
    match align {
        TextAlign::Center => SceneAlign::Center,
        TextAlign::Left => SceneAlign::Left,
        TextAlign::Right => SceneAlign::Right,
    }
}

pub fn draw(props: &Typewriter, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let typed = typewriter(
        &props.text,
        cx,
        Some(f64::from(props.chars_per_second)),
        props.speed,
        0.0,
    );
    let mut line = span(
        Style::default()
            .font_size(f64::from(props.font_size))
            .font_weight(props.font_weight.value())
            .color(cx.color(&props.color))
            .letter_spacing(Length::em(props.letter_spacing))
            .font_family(font_or(&props.font_family, FontRole::Main))
            .white_space(WhiteSpace::PreWrap)
            .max_width(Length::percent(TEXT_WIDTH_PERCENT))
            .text_align(text_align(&props.align)),
    )
    .child(typed.text.as_str());
    if props.cursor {
        line = line.child(caret(
            &Caret {
                color: cx.color(&props.cursor_color),
                blink: !typed.typing,
                speed: props.speed,
                blink_per_second: props.blink_rate,
                radius: 0.0,
                ..Caret::default()
            },
            cx,
            &Style::default()
                .width(Length::em(props.cursor_width))
                .height(Length::em(1.0))
                .margin_left(Length::em(CARET_GAP_EM))
                .vertical_align(VerticalAlign::TextBottom),
        ));
    }
    Ok(div(centered_stage(&props.background)).child(line).into())
}
