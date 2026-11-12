use opsis_macros::settings;
use opsis_scene::drawing::Font;
use opsis_scene::node::{Node, div, span};
use opsis_scene::spring::{DEFAULT_SPRING, SpringConfig};
use opsis_scene::style::{
    Display, FlexDirection, LineHeight, Overflow, Style, TextAlign, VerticalAlign, WhiteSpace,
};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::float;

use crate::elements::anim::sprung;
use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::{FontWeight, VerticalDirection};
use crate::elements::units::{utf16_len, utf16_units};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SlotMachineRoll {
    #[setting("From", default = "$99")]
    pub from: String,
    #[setting("To", default = "$199")]
    pub to: String,
    #[setting("Font size", default = 120, range(min = 12, max = 240), step = 1)]
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
    #[setting(
        "Column stagger",
        default = 0.1333,
        range(min = 0, max = 1),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub column_stagger: f64,
    #[setting(
        "Direction",
        default = VerticalDirection::Up,
        advanced
    )]
    pub direction: VerticalDirection,
    #[setting("Letter spacing", default = -0.03, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SlotMachineRoll>(
        element(
            ElementKind::Component,
            "slot-machine-roll",
            "Slot machine roll",
            "Vertical reel scrolls characters from one value to another.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type").moves(NodeMotion::Roll)),
    )
}

const ROW_EM: f64 = 1.1;
const ROLL_SPRING: SpringConfig = SpringConfig {
    damping: 14.0,
    ..DEFAULT_SPRING
};
const SPACE: u16 = 0x20;

fn padded(text: &str, len: usize) -> Vec<u16> {
    let units = utf16_units(text);
    let mut out = vec![SPACE; len.saturating_sub(units.len())];
    out.extend(units);
    out
}

fn unit_text(unit: Option<u16>) -> String {
    unit.map_or_default(|u| String::from_utf16_lossy(&[u]))
}

fn row(text: String) -> Node {
    span(
        Style::default()
            .height(Length::em(ROW_EM))
            .white_space(WhiteSpace::Pre),
    )
    .child(text)
    .into()
}

pub fn draw(props: &SlotMachineRoll, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let face = font_or(&props.font_family, FontRole::Main);
    let frame = cx.authored_frame() * props.speed;
    let size = f64::from(props.font_size);
    let len = utf16_len(&props.from).max(utf16_len(&props.to));
    let (from, to) = (padded(&props.from, len), padded(&props.to, len));
    let font = Font {
        families: face.clone(),
        size: Num(size),
        weight: Num(props.font_weight.value()),
        italic: false,
    };
    let spacing = props.letter_spacing * size;
    let up = props.direction == VerticalDirection::Up;
    let columns = (0..len)
        .map(|i| {
            let (a, b) = (
                unit_text(from.get(i).copied()),
                unit_text(to.get(i).copied()),
            );
            let width = cx
                .measure
                .text_width(&a, &font, spacing)
                .max(cx.measure.text_width(&b, &font, spacing));
            let raw = sprung(
                float(i).mul_add(-props.column_stagger, frame),
                Cx::authored_fps(),
                ROLL_SPRING,
            )?;
            let t = raw.clamp(0.0, 1.0);
            let (first, second) = if up { (a, b) } else { (b, a) };
            let shift = -(if up { t } else { 1.0 - t }) * size * ROW_EM;
            Ok(span(
                Style::default()
                    .display(Display::InlineBlock)
                    .overflow(Overflow::Hidden)
                    .height(Length::em(ROW_EM))
                    .line_height(LineHeight::Length(Length::em(ROW_EM)))
                    .vertical_align(VerticalAlign::Top)
                    .width(width)
                    .text_align(TextAlign::Center),
            )
            .child(
                span(
                    Style::default()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .translate(Length::ZERO, Length::px(shift)),
                )
                .child(row(first))
                .child(row(second)),
            )
            .into())
        })
        .collect::<Result<Vec<Node>, DrawFailure>>()?;
    Ok(div(centered_stage(&props.background))
        .child(
            span(
                Style::default()
                    .font_size(size)
                    .font_weight(props.font_weight.value())
                    .color(cx.color(&props.color))
                    .letter_spacing(Length::em(props.letter_spacing))
                    .font_family(face)
                    .line_height(LineHeight::Number(Num(ROW_EM))),
            )
            .children(columns),
        )
        .into())
}
