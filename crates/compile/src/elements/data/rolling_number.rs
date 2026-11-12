use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::{
    Align, Display, FlexDirection, LineHeight, NumericVariant, Overflow, Style, TextAlign,
    VerticalAlign,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::{float, float_wide};

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::GroupSeparator;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RollingNumber {
    #[setting("From", range(min = 0, max = 999_999_999_999_u64), step = 1)]
    #[specta(type = f64)]
    pub from: u64,
    #[setting(
        "To",
        default = 24813,
        range(min = 0, max = 999_999_999_999_u64),
        step = 1
    )]
    #[specta(type = f64)]
    pub to: u64,
    #[setting("Font size", default = 120, range(min = 24, max = 280), step = 4)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Finish counting at",
        default = 80.0,
        range(min = 10, max = 100),
        unit = "%",
        advanced
    )]
    pub finish_at: f64,
    #[setting(
        "Thousands separator",
        default = GroupSeparator::Space,
        advanced
    )]
    pub group_separator: GroupSeparator,
    #[setting(
        "Weight",
        default = 800,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting("Prefix")]
    pub prefix: String,
    #[setting("Suffix")]
    pub suffix: String,
    #[setting("Font", widget = "font", zero_label = "Code font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<RollingNumber>(
            element(
                ElementKind::Component,
                "rolling-number",
                "Rolling number",
                "Odometer where every place rolls at its own speed, new places rising into view, settling exactly on the target value.",
                ElementCategory::Data,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .fonts(&["JetBrains Mono"])
                .look(NodeLook::of("lucide:hash").moves(NodeMotion::Roll)),
        )
}

const FIT_WIDTH: f64 = 1800.0;
const REVEAL_BAND: f64 = 0.4;
const REVEAL_RISE_EM: f64 = 0.32;
const CELL: f64 = 1.1;
const DIGIT_WIDTH_EM: f64 = 0.62;
const SEPARATOR_WIDTH_EM: f64 = 0.34;

fn place_travel(start: f64, end: f64, place: i32) -> f64 {
    let pow = 10_f64.powi(place);
    let up = end >= start;
    let diff = (end - start).abs();
    let start_digit = (start / pow).floor() % 10.0;
    let final_digit = (end / pow).floor() % 10.0;
    let full_turns = (diff / (pow * 10.0)).floor();
    let step = if up {
        (final_digit - start_digit + 10.0) % 10.0
    } else {
        (start_digit - final_digit + 10.0) % 10.0
    };
    let magnitude = full_turns.mul_add(10.0, step);
    if up { magnitude } else { -magnitude }
}

fn place_reveal(current: f64, place: i32) -> Result<f64, DrawFailure> {
    if place == 0 {
        return Ok(1.0);
    }
    let threshold = 10_f64.powi(place);
    clamped_eased(
        current,
        [threshold * REVEAL_BAND, threshold],
        [0.0, 1.0],
        Easing::ease_in_out(Easing::Sin),
    )
}

fn wrap10(value: f64) -> f64 {
    ((value % 10.0) + 10.0) % 10.0
}

fn digit_column(value: f64, font_size: f64, reveal: f64) -> Node {
    let cell = font_size * CELL;
    let digits = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0].map(|d| {
        Node::from(
            span(Style::default()
                .height(cell)
                .line_height(LineHeight::Length(Length::px(cell))))
            .child(d.to_string()),
        )
    });
    span(Style::default()
        .display(Display::InlineBlock)
        .overflow(Overflow::Hidden)
        .height(cell)
        .width(Length::em(DIGIT_WIDTH_EM))
        .vertical_align(VerticalAlign::Top)
        .text_align(TextAlign::Center)
        .opacity(reveal)
        .translate(Length::ZERO, Length::px((1.0 - reveal) * font_size * REVEAL_RISE_EM)))
    .child(
        span(Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .translate(Length::ZERO, Length::px(-value * cell)))
        .children(digits),
    )
    .into()
}

const fn separator_mark(separator: GroupSeparator) -> &'static str {
    match separator {
        GroupSeparator::None | GroupSeparator::Space => "",
        GroupSeparator::Comma => ",",
        GroupSeparator::Dot => ".",
    }
}

pub fn draw(props: &RollingNumber, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame();
    let duration = cx.authored_duration() * props.speed;
    let start = float_wide(props.from);
    let end = float_wide(props.to);
    let eased = clamped_eased(
        frame * props.speed,
        [0.0, duration * props.finish_at / 100.0],
        [0.0, 1.0],
        Easing::ease_out(Easing::Poly(4.0)),
    )?;
    let current = eased.mul_add(end - start, start);
    let digits = props.from.max(props.to).to_string();
    let grouped = props.group_separator != GroupSeparator::None;
    let groups = if grouped { (digits.len() - 1) / 3 } else { 0 };
    let fitted = f64::from(props.font_size).min(
        FIT_WIDTH / (float(groups).mul_add(SEPARATOR_WIDTH_EM, float(digits.len())) * DIGIT_WIDTH_EM),
    );
    let cell = fitted * CELL;
    let mut cells: Vec<Node> = Vec::new();
    for place in 0..digits.len() {
        let power = i32::try_from(place).unwrap_or(i32::MAX);
        if grouped && place > 0 && place % 3 == 0 {
            cells.insert(
                0,
                span(Style::default()
                    .display(Display::InlineBlock)
                    .width(Length::em(SEPARATOR_WIDTH_EM))
                    .height(cell)
                    .line_height(LineHeight::Length(Length::px(cell)))
                    .text_align(TextAlign::Center))
                .child(separator_mark(props.group_separator))
                .into(),
            );
        }
        let start_digit = (start / 10_f64.powi(power)).floor() % 10.0;
        let position = eased.mul_add(place_travel(start, end, power), start_digit);
        cells.insert(0, digit_column(wrap10(position), fitted, place_reveal(current, power)?));
    }
    let line = || Style::default().line_height(LineHeight::Length(Length::px(cell)));
    let mut row = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::FlexStart)
        .font_size(fitted)
        .font_weight(f64::from(props.font_weight))
        .color(cx.color(&props.color))
        .font_family(font_or(&props.font_family, FontRole::Code))
        .font_variant_numeric(NumericVariant {
            tabular: true,
            ..NumericVariant::default()
        })
        .letter_spacing(Length::em(-0.03))
        .line_height(LineHeight::Number(Num(1.0))));
    if !props.prefix.is_empty() {
        row = row.child(span(line()).child(props.prefix.as_str()));
    }
    row = row.children(cells);
    if !props.suffix.is_empty() {
        row = row.child(span(line()).child(props.suffix.as_str()));
    }
    Ok(div(with_background(
        absolute_fill()
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center),
        &props.background,
    ))
    .child(row)
    .into())
}
