use opsis_scene::color::Color;
use opsis_scene::node::Node;
use opsis_scene::style::{
    Align, Display, FlexDirection, FlexWrap, GridLine, Overflow, RepeatCount, Style, Track,
    WhiteSpace, WordBreak,
};
use opsis_scene::values::{Edges, Length, Num};

use crate::{accent, cell, cool, d, leaf, rainbow, slate, stage, tinted, warm};

const PARAGRAPH: &str = "Opsis turns takes into finished films, one frame at a time.";

fn aligned_items() -> Vec<Node> {
    vec![
        d(Style::default()
            .width(30.0)
            .min_height(20.0)
            .font_size(10.0))
        .child("a")
        .into(),
        d(Style::default()
            .width(30.0)
            .min_height(50.0)
            .font_size(24.0))
        .child("b")
        .into(),
        d(Style::default()
            .width(30.0)
            .min_height(30.0)
            .font_size(14.0)
            .padding(Edges {
                top: Length::px(10.0),
                ..Edges::all(Length::ZERO)
            }))
        .child("c")
        .into(),
    ]
}

pub fn align_items() -> Node {
    let aligns = [
        Align::Stretch,
        Align::FlexStart,
        Align::Center,
        Align::FlexEnd,
        Align::Baseline,
        Align::Start,
        Align::End,
    ];
    let mut cells: Vec<Node> = aligns
        .iter()
        .enumerate()
        .map(|(i, align)| {
            d(cell(
                i,
                &bg().display(Display::Flex).align_items(*align).gap(4.0),
            ))
            .children(tinted(aligned_items(), &rainbow()))
            .into()
        })
        .collect();
    cells.push(
        d(cell(
            7,
            &bg()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(Align::Center)
                .gap(4.0),
        ))
        .children(tinted(aligned_items(), &rainbow()))
        .child(d(Style::default()
            .align_self(Align::FlexEnd)
            .width(30.0)
            .height(10.0)
            .background_color(warm())))
        .into(),
    );
    stage(cells)
}

pub fn flex_shrink() -> Node {
    let row = || bg().display(Display::Flex);
    let bar = |width: f64, color: Color| {
        Style::default()
            .width(width)
            .height(30.0)
            .background_color(color)
    };
    let plain = |color: Color| Style::default().background_color(color);
    stage(vec![
        d(cell(0, &row()))
            .child(d(bar(100.0, warm())))
            .child(d(bar(100.0, cool())))
            .into(),
        d(cell(1, &row()))
            .child(d(bar(100.0, warm()).flex_shrink(0.0)))
            .child(d(bar(100.0, cool())))
            .into(),
        d(cell(2, &row()))
            .child(d(bar(100.0, warm()).flex_shrink(3.0)))
            .child(d(bar(100.0, cool()).flex_shrink(1.0)))
            .into(),
        d(cell(3, &row().font_size(12.0)))
            .child(d(plain(warm())).child(PARAGRAPH))
            .child(d(plain(cool()).flex_shrink(0.0)).child("fixed"))
            .into(),
        d(cell(4, &row().font_size(12.0)))
            .child(
                d(plain(leaf())
                    .min_width(0.0)
                    .white_space(WhiteSpace::Nowrap)
                    .overflow(Overflow::Hidden))
                .child(PARAGRAPH),
            )
            .child(d(plain(cool()).flex_shrink(0.0)).child("end"))
            .into(),
        d(cell(
            5,
            &bg()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column),
        ))
        .child(d(plain(warm()).height(100.0)))
        .child(d(plain(cool()).height(100.0).flex_shrink(0.0)))
        .into(),
        d(cell(6, &row()))
            .child(d(plain(warm()).flex_basis(120.0).height(30.0)))
            .child(d(plain(leaf()).flex_basis(60.0).height(30.0)))
            .into(),
        d(cell(7, &row().font_size(12.0)))
            .child(d(plain(warm()).word_break(WordBreak::BreakAll)).child("Supercalifragilistic"))
            .child(d(plain(cool())).child("Unbreakableword"))
            .into(),
    ])
}

pub fn place_items() -> Node {
    let grid = || Style::default().display(Display::Grid);
    let place = |style: Style, align: Align, justify: Align| {
        style.align_items(align).justify_items(justify)
    };
    let places = vec![
        place(grid(), Align::Center, Align::Center),
        place(grid(), Align::Start, Align::Start),
        place(grid(), Align::End, Align::End),
        place(grid(), Align::Center, Align::Start),
        place(grid(), Align::Stretch, Align::Stretch),
        grid()
            .align_content(Align::Center)
            .justify_content(Align::Center),
        place(
            Style::default().display(Display::Flex),
            Align::Center,
            Align::Center,
        )
        .justify_content(Align::Center),
        place(
            grid()
                .grid_template_columns(vec![Track::Fraction(Num(1.0)), Track::Fraction(Num(1.0))]),
            Align::Center,
            Align::End,
        ),
    ];
    stage(
        places
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().merged(&style)))
                    .children(tinted(
                        [
                            d(Style::default().min_width(40.0).min_height(30.0)).into(),
                            d(Style::default().min_width(20.0).min_height(20.0)).into(),
                            d(Style::default().width(30.0).height(10.0)).into(),
                        ],
                        &rainbow(),
                    ))
                    .into()
            })
            .collect(),
    )
}

fn bg() -> Style {
    Style::default().background_color(slate())
}

fn sized(width: f64, height: f64) -> Node {
    d(Style::default().width(width).height(height)).into()
}

fn chips() -> Vec<Node> {
    tinted(
        [sized(30.0, 30.0), sized(30.0, 30.0), sized(30.0, 30.0)],
        &rainbow(),
    )
}

pub fn display() -> Node {
    stage(vec![
        d(cell(0, &bg().display(Display::Block)))
            .children(chips())
            .into(),
        d(cell(1, &bg().display(Display::Flex)))
            .children(chips())
            .into(),
        d(cell(
            2,
            &bg()
                .display(Display::Grid)
                .grid_template_columns(vec![Track::Fraction(Num(1.0)), Track::Fraction(Num(1.0))]),
        ))
        .children(chips())
        .into(),
        d(cell(3, &bg()))
            .children(tinted(
                [
                    crate::s(Style::default()).child("inline").into(),
                    crate::s(Style::default()).child(" spans ").into(),
                    crate::s(Style::default())
                        .child("wrap along the line")
                        .into(),
                ],
                &rainbow(),
            ))
            .into(),
        d(cell(4, &bg()))
            .children(tinted(
                (0..3).map(|_| {
                    d(Style::default()
                        .width(30.0)
                        .height(30.0)
                        .display(Display::InlineBlock))
                    .into()
                }),
                &rainbow(),
            ))
            .into(),
        d(cell(5, &bg()))
            .children(tinted(
                [
                    sized(30.0, 30.0),
                    d(Style::default()
                        .width(30.0)
                        .height(30.0)
                        .display(Display::None))
                    .into(),
                    sized(30.0, 30.0),
                ],
                &rainbow(),
            ))
            .into(),
        d(cell(6, &bg()))
            .child(
                d(Style::default()
                    .display(Display::InlineFlex)
                    .background_color(accent())
                    .gap(4.0)
                    .padding(opsis_scene::values::Edges::all(Length::px(4.0))))
                .children(chips()),
            )
            .child("text after")
            .into(),
        d(cell(7, &bg()))
            .child(
                d(Style::default().display(Display::Contents))
                    .children(tinted([sized(30.0, 30.0)], &rainbow())),
            )
            .child(
                d(Style::default()
                    .display(Display::InlineGrid)
                    .grid_template_columns(vec![
                        Track::Length(Length::px(20.0)),
                        Track::Length(Length::px(20.0)),
                    ])
                    .gap(2.0))
                .children(chips()),
            )
            .into(),
    ])
}

fn items() -> Vec<Node> {
    tinted(
        [sized(30.0, 30.0), sized(40.0, 20.0), sized(20.0, 40.0)],
        &rainbow(),
    )
}

pub fn flex_direction() -> Node {
    let directions = [
        FlexDirection::Row,
        FlexDirection::Column,
        FlexDirection::RowReverse,
        FlexDirection::ColumnReverse,
    ];
    let mut cells: Vec<Node> = directions
        .iter()
        .enumerate()
        .map(|(i, dir)| {
            d(cell(i, &bg().display(Display::Flex).flex_direction(*dir)))
                .children(items())
                .into()
        })
        .collect();
    cells.extend(directions.iter().enumerate().map(|(i, dir)| {
        d(cell(
            i + 4,
            &bg()
                .display(Display::Flex)
                .flex_direction(*dir)
                .gap(8.0)
                .padding(opsis_scene::values::Edges::all(Length::px(8.0)))
                .align_items(Align::Center),
        ))
        .children(items())
        .into()
    }));
    stage(cells)
}

pub fn justify_content() -> Node {
    let justify = [
        Align::FlexStart,
        Align::Center,
        Align::FlexEnd,
        Align::SpaceBetween,
        Align::SpaceAround,
        Align::SpaceEvenly,
        Align::Normal,
    ];
    let three = || tinted((0..3).map(|_| sized(24.0, 30.0)), &rainbow());
    let mut cells: Vec<Node> = justify
        .iter()
        .enumerate()
        .map(|(i, j)| {
            d(cell(
                i,
                &bg()
                    .display(Display::Flex)
                    .justify_content(*j)
                    .align_items(Align::Center),
            ))
            .children(three())
            .into()
        })
        .collect();
    cells.push(
        d(cell(
            7,
            &bg()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .justify_content(Align::SpaceBetween)
                .align_items(Align::Center),
        ))
        .children(three())
        .into(),
    );
    stage(cells)
}

pub fn gap() -> Node {
    let grid2 = || vec![Track::Fraction(Num(1.0)), Track::Fraction(Num(1.0))];
    let gaps: Vec<Style> = vec![
        Style::default().display(Display::Flex).gap(0.0),
        Style::default().display(Display::Flex).gap(6.0),
        Style::default()
            .display(Display::Flex)
            .row_gap(12.0)
            .column_gap(2.0)
            .flex_wrap(FlexWrap::Wrap),
        Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(10.0),
        Style::default()
            .display(Display::Grid)
            .grid_template_columns(grid2())
            .gap(8.0),
        Style::default()
            .display(Display::Grid)
            .grid_template_columns(grid2())
            .row_gap(16.0)
            .column_gap(2.0),
        Style::default()
            .display(Display::Flex)
            .gap(Length::percent(5.0)),
        Style::default()
            .display(Display::Flex)
            .gap(Length::em(0.5))
            .font_size(30.0),
    ];
    stage(
        gaps.into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().merged(&style)))
                    .children(tinted((0..4).map(|_| sized(24.0, 24.0)), &rainbow()))
                    .into()
            })
            .collect(),
    )
}

pub fn flex() -> Node {
    let base = |s: Style| s;
    let sets: Vec<[Style; 3]> = vec![
        [one(), one(), one()],
        [grow(1.0), grow(2.0), grow(3.0)],
        [none().width(40.0), one(), none().width(20.0)],
        [
            one(),
            Style::default()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(Length::Auto)
                .width(50.0),
            Style::default()
                .flex_grow(0.0)
                .flex_shrink(0.0)
                .flex_basis(30.0),
        ],
        [
            Style::default().flex_grow(1.0),
            Style::default(),
            Style::default().flex_grow(2.0),
        ],
        [
            auto_flex(),
            Style::default()
                .flex_grow(0.0)
                .flex_shrink(1.0)
                .flex_basis(Length::Auto)
                .width(60.0),
            auto_flex(),
        ],
        [one().min_width(80.0), one(), one()],
        [grow(0.5), grow(0.25), grow(0.0)],
    ];
    stage(
        sets.into_iter()
            .enumerate()
            .map(|(i, [a, b, c])| {
                d(cell(
                    i,
                    &bg().display(Display::Flex).align_items(Align::FlexStart),
                ))
                .child(d(base(
                    Style::default().height(40.0).background_color(warm()),
                )
                .merged(&a)))
                .child(d(base(
                    Style::default().height(60.0).background_color(cool()),
                )
                .merged(&b)))
                .child(d(base(
                    Style::default().height(80.0).background_color(leaf()),
                )
                .merged(&c)))
                .into()
            })
            .collect(),
    )
}

fn one() -> Style {
    grow(1.0)
}

fn grow(n: f64) -> Style {
    Style::default()
        .flex_grow(n)
        .flex_shrink(1.0)
        .flex_basis(Length::percent(0.0))
}

fn none() -> Style {
    Style::default()
        .flex_grow(0.0)
        .flex_shrink(0.0)
        .flex_basis(Length::Auto)
}

fn auto_flex() -> Style {
    Style::default()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .flex_basis(Length::Auto)
}

pub fn flex_wrap() -> Node {
    let wraps: Vec<Style> = vec![
        Style::default().flex_wrap(FlexWrap::Nowrap),
        Style::default().flex_wrap(FlexWrap::Wrap),
        Style::default().flex_wrap(FlexWrap::WrapReverse),
        Style::default().flex_wrap(FlexWrap::Wrap).gap(6.0),
        Style::default()
            .flex_wrap(FlexWrap::Wrap)
            .justify_content(Align::Center)
            .gap(4.0),
        Style::default()
            .flex_wrap(FlexWrap::Wrap)
            .align_content(Align::SpaceBetween),
        Style::default()
            .flex_wrap(FlexWrap::Wrap)
            .flex_direction(FlexDirection::Column)
            .height(100.0),
        Style::default()
            .flex_wrap(FlexWrap::Wrap)
            .align_content(Align::Center)
            .align_items(Align::FlexEnd),
    ];
    stage(
        wraps
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().display(Display::Flex).merged(&style)))
                    .children(tinted(
                        (0..7u32).map(|j| {
                            sized(f64::from(30 + (j % 3) * 6), f64::from(20 + (j % 2) * 10))
                        }),
                        &rainbow(),
                    ))
                    .into()
            })
            .collect(),
    )
}

pub fn grid_template_columns() -> Node {
    let fr = |n: f64| Track::Fraction(Num(n));
    let px = |n: f64| Track::Length(Length::px(n));
    let grids: Vec<Style> = vec![
        Style::default().grid_template_columns(vec![fr(1.0), fr(1.0), fr(1.0)]),
        Style::default().grid_template_columns(vec![px(40.0), fr(1.0)]),
        Style::default()
            .grid_template_columns(vec![Track::Repeat(
                RepeatCount::Times(3),
                vec![Track::MinMax(Box::new(px(0.0)), Box::new(fr(1.0)))],
            )])
            .gap(4.0),
        Style::default()
            .grid_template_columns(vec![Track::Auto, fr(1.0), Track::Auto])
            .gap(4.0),
        Style::default()
            .grid_template_columns(vec![fr(2.0), fr(1.0)])
            .grid_auto_rows(vec![px(30.0)])
            .gap(2.0),
        Style::default()
            .grid_template_columns(vec![Track::Repeat(RepeatCount::AutoFill, vec![px(30.0)])])
            .gap(4.0),
        Style::default()
            .grid_template_columns(vec![fr(1.0), fr(1.0)])
            .grid_template_rows(vec![fr(1.0), fr(2.0)]),
        Style::default()
            .grid_template_columns(vec![
                Track::Length(Length::percent(25.0)),
                Track::Length(Length::percent(50.0)),
                Track::Length(Length::percent(25.0)),
            ])
            .align_items(Align::End),
    ];
    let _ = GridLine::Auto;
    stage(
        grids
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().display(Display::Grid).merged(&style)))
                    .children(tinted(
                        (0..6).map(|j| {
                            let item = d(Style::default()
                                .min_height(20.0)
                                .min_width(10.0)
                                .font_size(10.0));
                            if j == 1 {
                                item.child("auto").into()
                            } else {
                                item.into()
                            }
                        }),
                        &rainbow(),
                    ))
                    .into()
            })
            .collect(),
    )
}
