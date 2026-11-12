use opsis_scene::color::{Color, ColorSpace, HueMethod, Interpolation};
use opsis_scene::node::Node;
use opsis_scene::paint::Shadow;
use opsis_scene::style::{
    BorderStyle, DecorationLine, DecorationStyle, Display, FontStyle, LineHeight, NumericVariant,
    Outline, Overflow, PaintOrder, Position, Style, Tag, TextAlign, TextDecoration, TextOverflow,
    TextStroke, TextTransform, TextWrap, Track, VerticalAlign, WhiteSpace, WordBreak,
};
use opsis_scene::values::{Angle, Edges, Length, Num};

use crate::paints::{colour, hex, rgba_of};
use crate::{cell, colored, cool, d, slate, stage, styled, warm};

const SAMPLE: &str = "Sphinx of black quartz";
const PARAGRAPH: &str = "Opsis turns takes into finished films, one frame at a time.";

fn bg() -> Style {
    Style::default().background_color(slate())
}

fn family(name: &str) -> Vec<String> {
    vec![name.to_owned()]
}

pub fn font_family() -> Node {
    let families = [
        "Inter",
        "JetBrains Mono",
        "Fraunces",
        "Instrument Serif",
        "Caveat",
        "Geist",
        "Space Grotesk",
        "Playfair Display",
    ];
    stage(
        families
            .iter()
            .enumerate()
            .map(|(i, name)| {
                d(cell(
                    i,
                    &Style::default()
                        .font_family(family(name))
                        .font_size(22.0)
                        .line_height(LineHeight::Number(Num(1.2))),
                ))
                .child(SAMPLE)
                .child(" 0123")
                .into()
            })
            .collect(),
    )
}

pub fn font_size() -> Node {
    let sizes = [8.0, 11.0, 13.5, 16.0, 24.0, 36.0, 64.0, 120.0];
    stage(vec![
        d(Style::default()
            .position(opsis_scene::style::Position::Absolute)
            .left(24.0)
            .top(8.0)
            .right(24.0)
            .line_height(LineHeight::Number(Num(1.0))))
        .children(sizes.iter().map(|size| {
            d(Style::default()
                .font_size(*size)
                .white_space(WhiteSpace::Nowrap))
            .child("Ag ")
            .child(opsis_scene::random::js_number_text(*size))
            .child("px")
            .into()
        }))
        .into(),
    ])
}

pub fn letter_spacing() -> Node {
    let spacings = [
        ("-0.05em", Length::em(-0.05)),
        ("-0.02em", Length::em(-0.02)),
        ("0", Length::px(0.0)),
        ("0.02em", Length::em(0.02)),
        ("0.1em", Length::em(0.1)),
        ("0.3em", Length::em(0.3)),
        ("4px", Length::px(4.0)),
        ("-1.5px", Length::px(-1.5)),
    ];
    stage(vec![
        d(Style::default()
            .position(opsis_scene::style::Position::Absolute)
            .left(24.0)
            .top(12.0)
            .right(24.0))
        .children(spacings.into_iter().map(|(label, spacing)| {
            d(Style::default()
                .letter_spacing(spacing)
                .font_size(26.0)
                .line_height(LineHeight::Number(Num(1.25)))
                .font_weight(600.0)
                .white_space(WhiteSpace::Nowrap))
            .child("LETTERS spaced ")
            .child(label)
            .into()
        }))
        .into(),
    ])
}

pub fn line_height() -> Node {
    let heights = [
        LineHeight::Number(Num(0.9)),
        LineHeight::Number(Num(1.0)),
        LineHeight::Number(Num(1.15)),
        LineHeight::Number(Num(1.5)),
        LineHeight::Number(Num(2.0)),
        LineHeight::Length(Length::px(20.0)),
        LineHeight::Normal,
        LineHeight::Length(Length::percent(150.0)),
    ];
    stage(
        heights
            .into_iter()
            .enumerate()
            .map(|(i, height)| {
                d(cell(i, &bg().line_height(height).font_size(15.0)))
                    .child(PARAGRAPH)
                    .into()
            })
            .collect(),
    )
}

fn text_align_shape() -> Node {
    let aligns = [
        TextAlign::Left,
        TextAlign::Center,
        TextAlign::Right,
        TextAlign::Justify,
        TextAlign::Start,
        TextAlign::End,
    ];
    let mut cells: Vec<Node> = aligns
        .iter()
        .enumerate()
        .map(|(i, align)| {
            d(cell(
                i,
                &bg()
                    .text_align(*align)
                    .font_size(15.0)
                    .line_height(LineHeight::Number(Num(1.3))),
            ))
            .child(PARAGRAPH)
            .into()
        })
        .collect();
    cells.push(
        d(cell(6, &bg().text_align(TextAlign::Center).font_size(15.0)))
            .child(
                crate::s(
                    Style::default()
                        .display(opsis_scene::style::Display::InlineBlock)
                        .background_color(slate())
                        .padding(opsis_scene::values::Edges::axes(
                            Length::px(2.0),
                            Length::px(6.0),
                        )),
                )
                .child("chip"),
            )
            .child(" ")
            .child("centred")
            .into(),
    );
    cells.push(
        d(cell(
            7,
            &bg()
                .text_align(TextAlign::Center)
                .font_size(15.0)
                .direction(opsis_scene::style::Direction::Rtl),
        ))
        .child(PARAGRAPH)
        .into(),
    );
    stage(cells)
}

pub fn white_space() -> Node {
    let text = "Two  spaces,\na newline and\ta tab in one long line of words";
    let modes = [
        WhiteSpace::Normal,
        WhiteSpace::Nowrap,
        WhiteSpace::Pre,
        WhiteSpace::PreWrap,
        WhiteSpace::PreLine,
        WhiteSpace::BreakSpaces,
    ];
    let mut cells: Vec<Node> = modes
        .iter()
        .enumerate()
        .map(|(i, mode)| {
            d(cell(
                i,
                &bg()
                    .white_space(*mode)
                    .font_size(14.0)
                    .line_height(LineHeight::Number(Num(1.3))),
            ))
            .child(text)
            .into()
        })
        .collect();
    cells.push(
        d(cell(
            6,
            &bg()
                .font_size(14.0)
                .overflow_wrap(opsis_scene::style::OverflowWrap::Anywhere),
        ))
        .child("https://opsis.example/a-very-long-address-without-breaks")
        .into(),
    );
    cells.push(
        d(cell(
            7,
            &bg()
                .font_size(14.0)
                .font_family(family("JetBrains Mono"))
                .white_space(WhiteSpace::Pre),
        ))
        .child("fn main() {\n    run();\n}")
        .into(),
    );
    stage(cells)
}

fn placed(left: f64, top: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(left)
        .top(top)
}

fn lines_of(style: Style, rows: usize, text: &str) -> Node {
    stage(vec![
        d(style)
            .children((0..rows).map(|_| d(Style::default()).child(text).into()))
            .into(),
    ])
}

fn color_shape() -> Node {
    lines_of(
        placed(24.0, 16.0)
            .font_size(28.0)
            .font_weight(700.0)
            .line_height(LineHeight::Number(Num(1.15))),
        8,
        "Coloured words",
    )
}

pub fn font_weight() -> Node {
    let weights = [100, 200, 300, 400, 500, 600, 700, 800, 900, 450, 650];
    stage(vec![
        d(placed(24.0, 16.0)
            .display(Display::Grid)
            .grid_template_columns(vec![Track::Fraction(Num(1.0)), Track::Fraction(Num(1.0))])
            .row_gap(4.0)
            .column_gap(32.0))
        .children(weights.into_iter().map(|weight| {
            d(Style::default()
                .font_weight(f64::from(weight))
                .font_size(22.0)
                .line_height(LineHeight::Number(Num(1.1))))
            .child("Weight ")
            .child(weight.to_string())
            .into()
        }))
        .child(
            d(Style::default()
                .font_weight(700.0)
                .font_family(family("JetBrains Mono"))
                .font_size(20.0))
            .child("mono bold"),
        )
        .child(
            d(Style::default()
                .font_weight(800.0)
                .font_family(family("Fraunces"))
                .font_size(22.0))
            .child("serif 800"),
        )
        .into(),
    ])
}

pub fn font_variant_numeric() -> Node {
    let numbers = ["1,111.11", "8,888.88", "4,710.09", "$0.50"];
    let variants = [
        NumericVariant::default(),
        NumericVariant {
            tabular: true,
            ..NumericVariant::default()
        },
        NumericVariant {
            slashed_zero: true,
            ..NumericVariant::default()
        },
        NumericVariant {
            tabular: true,
            slashed_zero: true,
            ..NumericVariant::default()
        },
    ];
    stage(vec![
        d(placed(24.0, 24.0)
            .display(Display::Flex)
            .gap(24.0)
            .font_size(22.0))
        .children(variants.into_iter().map(|variant| {
            d(Style::default()
                .font_variant_numeric(variant)
                .text_align(TextAlign::Right))
            .children(numbers.iter().map(|n| d(Style::default()).child(*n).into()))
            .child(d(Style::default().font_family(family("Geist"))).child("10:01"))
            .child(
                d(Style::default().font_feature_settings(vec![(Tag(*b"ss01"), 1)])).child("a1l0"),
            )
            .into()
        }))
        .into(),
    ])
}

pub fn font_variation_settings() -> Node {
    let fraunces = || Style::default().font_family(family("Fraunces"));
    let inter = || Style::default().font_family(family("Inter"));
    let axes = |list: &[(&[u8; 4], f64)]| -> Vec<(Tag, Num)> {
        list.iter()
            .map(|(tag, value)| (Tag(**tag), Num(*value)))
            .collect()
    };
    let variations = vec![
        fraunces().font_variation_settings(axes(&[(b"opsz", 9.0)])),
        fraunces().font_variation_settings(axes(&[(b"opsz", 144.0)])),
        fraunces().font_variation_settings(axes(&[(b"SOFT", 100.0), (b"WONK", 1.0)])),
        inter().font_variation_settings(axes(&[(b"wght", 250.0)])),
        inter().font_variation_settings(axes(&[(b"wght", 850.0)])),
        inter().font_style(FontStyle::Italic),
        fraunces().font_style(FontStyle::Italic).font_weight(300.0),
    ];
    stage(vec![
        d(placed(24.0, 8.0)
            .font_size(34.0)
            .line_height(LineHeight::Number(Num(1.3))))
        .children(
            variations
                .into_iter()
                .map(|style| d(style).child("Variable faces").into()),
        )
        .into(),
    ])
}

pub fn text_transform() -> Node {
    let transforms = [
        TextTransform::None,
        TextTransform::Uppercase,
        TextTransform::Lowercase,
        TextTransform::Capitalize,
    ];
    stage(vec![
        d(placed(24.0, 24.0)
            .right(24.0)
            .font_size(28.0)
            .line_height(LineHeight::Number(Num(1.5))))
        .children(transforms.into_iter().map(|transform| {
            d(Style::default().text_transform(transform))
                .child("straße and ÉCOLE start here")
                .into()
        }))
        .child(
            d(Style::default()
                .text_transform(TextTransform::Uppercase)
                .letter_spacing(Length::em(0.12))
                .font_size(14.0)
                .font_weight(600.0))
            .child("small caps label"),
        )
        .into(),
    ])
}

fn text_shadow_shape() -> Node {
    lines_of(
        placed(24.0, 16.0)
            .font_size(40.0)
            .font_weight(800.0)
            .line_height(LineHeight::Number(Num(1.6))),
        5,
        "Shadowed",
    )
}

fn text_stroke_shape() -> Node {
    lines_of(
        placed(24.0, 16.0)
            .font_size(52.0)
            .font_weight(900.0)
            .line_height(LineHeight::Number(Num(1.25))),
        4,
        "Outline",
    )
}

fn clipped(index: usize, extra: &Style) -> Style {
    cell(
        index,
        &bg()
            .height(30.0)
            .white_space(WhiteSpace::Nowrap)
            .overflow(Overflow::Hidden)
            .merged(extra),
    )
}

pub fn text_overflow() -> Node {
    let ellipsis = || Style::default().text_overflow(TextOverflow::Ellipsis);
    stage(vec![
        d(clipped(0, &ellipsis())).child(PARAGRAPH).into(),
        d(clipped(
            1,
            &Style::default().text_overflow(TextOverflow::Clip),
        ))
        .child(PARAGRAPH)
        .into(),
        d(clipped(2, &ellipsis().font_size(22.0).font_weight(700.0)))
            .child(PARAGRAPH)
            .into(),
        d(clipped(
            3,
            &ellipsis().font_family(family("JetBrains Mono")),
        ))
        .child(PARAGRAPH)
        .into(),
        d(cell(
            4,
            &bg()
                .display(Display::WebkitBox)
                .line_clamp(3_u16)
                .overflow(Overflow::Hidden)
                .font_size(15.0),
        ))
        .child(PARAGRAPH)
        .child(" ")
        .child(PARAGRAPH)
        .into(),
        d(cell(5, &bg().display(Display::Flex).height(30.0)))
            .child(
                crate::s(
                    Style::default()
                        .min_width(0.0)
                        .white_space(WhiteSpace::Nowrap)
                        .overflow(Overflow::Hidden)
                        .text_overflow(TextOverflow::Ellipsis),
                )
                .child(PARAGRAPH),
            )
            .into(),
    ])
}

const LONG: &str = "A demo film takes a recording, a camera that follows the cursor, captions, music and titles, and puts them in order.";

pub fn wrapping() -> Node {
    stage(vec![
        d(cell(0, &bg().font_size(13.0).height(Length::Auto).width(136.0)))
            .child(LONG)
            .into(),
        d(cell(1, &bg().font_size(16.0).width(100.0)))
            .child(LONG)
            .into(),
        d(cell(2, &bg().font_size(12.0).font_family(family("JetBrains Mono"))))
            .child(LONG)
            .into(),
        d(cell(3, &bg().font_size(15.0)))
            .child("Extra\u{ad}ordinary in\u{ad}com\u{ad}pre\u{ad}hen\u{ad}si\u{ad}bil\u{ad}i\u{ad}ties")
            .into(),
        d(cell(4, &bg().font_size(14.0)))
            .child("Mixed scripts: Grüße, Ελληνικά, Кириллица, Ærøskøbing, ½ → ≥")
            .into(),
        d(cell(5, &bg().font_size(18.0).word_break(WordBreak::KeepAll)))
            .child("Long\u{ad}words\u{ad}without\u{ad}spaces stay")
            .into(),
        d(cell(6, &bg().font_size(14.0).text_wrap(TextWrap::Balance)))
            .child(LONG)
            .into(),
        d(cell(
            7,
            &bg()
                .font_size(30.0)
                .font_weight(800.0)
                .line_height(LineHeight::Number(Num(1.0)))
                .letter_spacing(Length::em(-0.04)),
        ))
        .child("Big headline wraps")
        .into(),
    ])
}

fn spans_shape() -> Node {
    let words = ["Every", "word", "moves", "on", "its", "own"];
    let script = |offset: Style| {
        crate::s(
            Style::default()
                .font_size(Length::percent(75.0))
                .line_height(LineHeight::Number(Num(0.0)))
                .position(Position::Relative)
                .merged(&offset),
        )
        .child("2")
    };
    let first = d(Style::default())
        .child("Plain text with ")
        .child(crate::s(Style::default().font_weight(700.0)).child("strong"))
        .child(", ")
        .child(crate::s(Style::default().font_style(FontStyle::Italic)).child("emphasis"))
        .child(",")
        .child(" ")
        .child(crate::s(Style::default().font_weight(700.0)).child("a coloured run"))
        .child(",")
        .child(" ")
        .child(
            crate::s(
                Style::default()
                    .font_family(family("JetBrains Mono"))
                    .font_size(Length::em(0.8))
                    .background_color(slate())
                    .padding(Edges::axes(Length::ZERO, Length::px(4.0)))
                    .border_radius_all(4.0),
            )
            .child("code"),
        )
        .child(" ")
        .child("and ")
        .child(
            crate::s(
                Style::default()
                    .font_size(Length::em(1.4))
                    .font_family(family("Fraunces")),
            )
            .child("bigger"),
        )
        .child(" words on one line that wraps.");
    let second = d(Style::default().margin(Edges {
        top: Length::px(16.0),
        ..Edges::all(Length::ZERO)
    }))
    .children(words.iter().enumerate().map(|(i, word)| {
        let step = opsis_types::numbers::float(i);
        crate::s(
            Style::default()
                .display(Display::InlineBlock)
                .margin(Edges {
                    right: Length::em(0.25),
                    ..Edges::all(Length::ZERO)
                })
                .translate(
                    Length::ZERO,
                    Length::px(opsis_types::numbers::float(i % 3) * 6.0),
                )
                .opacity(step.mul_add(-0.12, 1.0))
                .rotate(Angle::Deg(Num(step.mul_add(2.0, -5.0)))),
        )
        .child(*word)
        .into()
    }));
    let third = d(Style::default().font_size(18.0).margin(Edges {
        top: Length::px(8.0),
        ..Edges::all(Length::ZERO)
    }))
    .child("H")
    .child(script(Style::default().bottom(Length::em(-0.25))))
    .child("O and E = mc")
    .child(script(Style::default().top(Length::em(-0.5))))
    .child(", ")
    .child(crate::s(Style::default()).child("underlined"))
    .child(", ")
    .child(crate::s(Style::default()).child("struck"))
    .child(",")
    .child(" ")
    .child(crate::s(Style::default()).child("wavy"));
    stage(vec![
        d(placed(24.0, 24.0)
            .right(24.0)
            .font_size(26.0)
            .line_height(LineHeight::Number(Num(1.4))))
        .child(first)
        .child(second)
        .child(third)
        .into(),
    ])
}

fn vertical_align_shape() -> Node {
    let aligns = [
        (VerticalAlign::Baseline, VerticalAlign::Baseline),
        (VerticalAlign::Sub, VerticalAlign::Sub),
        (VerticalAlign::Super, VerticalAlign::Super),
        (VerticalAlign::TextTop, VerticalAlign::TextTop),
        (VerticalAlign::TextBottom, VerticalAlign::TextBottom),
        (VerticalAlign::Middle, VerticalAlign::Middle),
        (VerticalAlign::Top, VerticalAlign::Length(Length::px(4.0))),
        (
            VerticalAlign::Bottom,
            VerticalAlign::Length(Length::percent(-30.0)),
        ),
    ];
    stage(
        aligns
            .into_iter()
            .enumerate()
            .map(|(i, (chip, run))| {
                d(cell(i, &bg().font_size(16.0)))
                    .child("Ag")
                    .child(" ")
                    .child(crate::s(
                        Style::default()
                            .display(Display::InlineBlock)
                            .width(20.0)
                            .height(40.0)
                            .vertical_align(chip),
                    ))
                    .child(" ")
                    .child("text ")
                    .child(
                        crate::s(Style::default().font_size(11.0).vertical_align(run)).child("run"),
                    )
                    .child(" end")
                    .into()
            })
            .collect(),
    )
}

pub fn color() -> Node {
    let line = |k: usize| vec![0, 0, k];
    let colors: Vec<(Vec<usize>, Style)> = vec![
        (vec![0, 0], Style::default().color(hex(0xfa_cc15))),
        (line(0), Style::default().color(hex(0xf8_fafc))),
        (line(1), Style::default().color(hex(0xf9_7316))),
        (line(2), Style::default().color(rgba_of(248, 250, 252, 0.5))),
        (
            line(3),
            Style::default().color(colour(ColorSpace::Oklch, [0.75, 0.18, 150.0], 1.0)),
        ),
        (
            line(4),
            Style::default().color(Color::mix(
                Interpolation {
                    space: ColorSpace::Oklch,
                    hue: HueMethod::Shorter,
                },
                (hex(0xf9_7316), Some(40.0)),
                (hex(0x0e_a5e9), None),
            )),
        ),
        (
            line(6),
            Style::default().color(colour(ColorSpace::Hsl, [280.0, 80.0, 70.0], 1.0)),
        ),
        (
            line(7),
            Style::default().color(Color::TRANSPARENT).outline(Outline {
                width: Length::px(1.0),
                style: BorderStyle::Solid,
                color: Some(hex(0x47_5569)),
                offset: Length::ZERO,
            }),
        ),
    ];
    let list: Vec<(&[usize], Style)> = colors
        .iter()
        .map(|(p, s)| (p.as_slice(), s.clone()))
        .collect();
    styled(color_shape(), &list)
}

pub fn text_shadow() -> Node {
    let shadow = |x: f64, y: f64, blur: f64, color: Color| Shadow {
        inset: false,
        x: Length::px(x),
        y: Length::px(y),
        blur: Length::px(blur),
        spread: Length::ZERO,
        color,
    };
    let lists = [
        vec![shadow(0.0, 2.0, 4.0, rgba_of(0, 0, 0, 0.9))],
        vec![shadow(0.0, 0.0, 12.0, hex(0x0e_a5e9))],
        vec![shadow(3.0, 3.0, 0.0, hex(0xf9_7316))],
        vec![
            shadow(0.0, 0.0, 2.0, hex(0xff_ffff)),
            shadow(0.0, 0.0, 10.0, hex(0xf4_3f5e)),
            shadow(0.0, 0.0, 30.0, hex(0xf4_3f5e)),
        ],
        vec![
            shadow(-2.0, 0.0, 0.0, hex(0x0e_a5e9)),
            shadow(2.0, 0.0, 0.0, hex(0xf4_3f5e)),
        ],
    ];
    let paths: Vec<(Vec<usize>, Style)> = lists
        .into_iter()
        .enumerate()
        .map(|(k, list)| (vec![0, 0, k], Style::default().text_shadow(list)))
        .collect();
    let list: Vec<(&[usize], Style)> = paths
        .iter()
        .map(|(p, s)| (p.as_slice(), s.clone()))
        .collect();
    styled(text_shadow_shape(), &list)
}

pub fn text_stroke() -> Node {
    let stroke = |width: f64, color: Color| TextStroke {
        width: Length::px(width),
        color: Some(color),
    };
    let paths: Vec<(Vec<usize>, Style)> = vec![
        (
            vec![0, 0, 0],
            Style::default()
                .text_stroke(stroke(1.0, hex(0xf8_fafc)))
                .color(Color::TRANSPARENT),
        ),
        (
            vec![0, 0, 1],
            Style::default()
                .text_stroke(stroke(3.0, hex(0x0f_172a)))
                .color(hex(0xf8_fafc))
                .paint_order(PaintOrder::StrokeFirst),
        ),
        (
            vec![0, 0, 2],
            Style::default()
                .text_stroke(stroke(2.0, hex(0xf9_7316)))
                .color(hex(0x0e_a5e9)),
        ),
        (
            vec![0, 0, 3],
            Style::default()
                .text_stroke(stroke(4.0, hex(0x7c_3aed)))
                .color(hex(0xf8_fafc))
                .paint_order(PaintOrder::StrokeFirst),
        ),
    ];
    let list: Vec<(&[usize], Style)> = paths
        .iter()
        .map(|(p, s)| (p.as_slice(), s.clone()))
        .collect();
    styled(text_stroke_shape(), &list)
}

pub fn text_align() -> Node {
    colored(text_align_shape(), &[(&[0, 6, 0], hex(0x7c_3aed))])
}

pub fn vertical_align() -> Node {
    let paths: Vec<Vec<usize>> = (0..8)
        .flat_map(|i| [vec![0, i, 2], vec![0, i, 5]])
        .collect();
    let list: Vec<(&[usize], Color)> = paths
        .iter()
        .map(|p| (p.as_slice(), if p[2] == 2 { warm() } else { cool() }))
        .collect();
    colored(vertical_align_shape(), &list)
}

pub fn spans() -> Node {
    let underline = |style: DecorationStyle, color: Option<Color>| TextDecoration {
        lines: vec![DecorationLine::Underline],
        style,
        color,
        thickness: None,
    };
    let paths: Vec<(Vec<usize>, Style)> = vec![
        (vec![0, 0, 0, 6], Style::default().color(hex(0xf9_7316))),
        (vec![0, 0, 0, 9], Style::default().background_color(slate())),
        (
            vec![0, 0, 2, 5],
            Style::default().text_decoration(underline(DecorationStyle::Solid, None)),
        ),
        (
            vec![0, 0, 2, 7],
            Style::default().text_decoration(TextDecoration {
                lines: vec![DecorationLine::LineThrough],
                style: DecorationStyle::Solid,
                color: None,
                thickness: None,
            }),
        ),
        (
            vec![0, 0, 2, 10],
            Style::default()
                .text_decoration(underline(DecorationStyle::Wavy, Some(hex(0xf4_3f5e)))),
        ),
    ];
    let list: Vec<(&[usize], Style)> = paths
        .iter()
        .map(|(p, s)| (p.as_slice(), s.clone()))
        .collect();
    styled(spans_shape(), &list)
}
