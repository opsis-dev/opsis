use opsis_macros::settings;
use opsis_scene::node::{Node, div, span};
use opsis_scene::random::random;
use opsis_scene::style::{Style, WhiteSpace};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, truncate};

use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::units::utf16_units;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MatrixDecode {
    #[setting("Text", default = "DECRYPTED")]
    pub text: String,
    #[setting(
        "Scramble characters",
        default = "!@#$%^&*()_+-=<>?/\\|",
        zero_label = "Built-in",
        help = "Characters the text flickers through before it settles.",
        advanced
    )]
    pub charset: String,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#22c55e", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Reveal time",
        default = 2.0,
        range(min = 0.3333, max = 8),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub reveal_duration: f64,
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
        "Scramble every",
        default = 0.0667,
        range(min = 0.0333, max = 0.5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub scramble_every: f64,
    #[setting("Variation", step = 1, advanced)]
    pub seed: u32,
    #[setting("Letter spacing", default = 0.05, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<MatrixDecode>(
        element(
            ElementKind::Component,
            "matrix-decode",
            "Matrix decode",
            "Random scramble resolves left-to-right into the target text.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:binary").morphs("lucide:type")),
    )
}

const SPACE: u16 = 0x20;

fn scrambled(props: &MatrixDecode, i: usize, frame: f64, charset: &[u16]) -> Option<u16> {
    let tick: i64 = truncate((frame / props.scramble_every.max(1.0)).floor());
    let key = if props.seed == 0 {
        format!("{i}-{tick}")
    } else {
        format!("{}-{i}-{tick}", props.seed)
    };
    let pick: usize = truncate((random(key) * float(charset.len())).floor());
    charset.get(pick).copied()
}

pub fn draw(props: &MatrixDecode, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let text = utf16_units(&props.text);
    let charset = utf16_units(&props.charset);
    let length = float(text.len().max(1));
    let mut output: Vec<u16> = Vec::with_capacity(text.len());
    for (i, &unit) in text.iter().enumerate() {
        let reveal = float(i) / length * props.reveal_duration;
        if unit == SPACE {
            output.push(SPACE);
        } else if frame >= reveal {
            output.push(unit);
        } else {
            output.extend(scrambled(props, i, frame, &charset));
        }
    }
    Ok(div(centered_stage(&props.background))
        .child(
            span(
                Style::default()
                    .font_size(f64::from(props.font_size))
                    .font_weight(props.font_weight.value())
                    .color(cx.color(&props.color))
                    .letter_spacing(Length::em(props.letter_spacing))
                    .white_space(WhiteSpace::Pre)
                    .font_family(font_or(&props.font_family, FontRole::Code)),
            )
            .child(String::from_utf16_lossy(&output)),
        )
        .into())
}
