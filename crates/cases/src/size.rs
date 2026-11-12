use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::{Display, Overflow, Position, Style};
use opsis_scene::values::{Edges, Length};

use crate::{accent, cell, colored, cool, d, leaf, pct, rose, s, slate, stage, warm};

fn bg() -> Style {
    Style::default().background_color(slate())
}

fn width_height_shape() -> Node {
    stage(vec![
        d(cell(0, &bg()))
            .child(d(Style::default().width(80.0).height(40.0)))
            .into(),
        d(cell(1, &bg()))
            .child(d(Style::default().width(pct(50.0)).height(pct(75.0))))
            .into(),
        d(cell(2, &bg()))
            .child(d(Style::default().padding(Edges::all(Length::px(8.0)))).child("auto"))
            .into(),
        d(cell(3, &bg().display(Display::Flex)))
            .child(d(Style::default()
                .height(pct(100.0))
                .width(Length::sum([pct(100.0), Length::px(-40.0)]))))
            .into(),
        d(cell(4, &bg()))
            .child(d(Style::default().width(pct(100.0)).height(12.5)))
            .child(d(Style::default().width(99.5).height(20.25)))
            .into(),
        d(cell(5, &bg()))
            .child(s(Style::default().width(100.0).height(100.0)).child("inline"))
            .into(),
        d(cell(6, &bg().padding(Edges::all(Length::px(16.0)))))
            .child(d(Style::default().height(pct(100.0))))
            .into(),
        d(cell(7, &bg()))
            .child(d(Style::default().width(120.0).aspect_ratio(16.0 / 9.0)))
            .into(),
    ])
}

const PARAGRAPH: &str = "Opsis turns takes into finished films, one frame at a time.";

fn min_max_width_shape() -> Node {
    stage(vec![
        d(cell(0, &bg()))
            .child(d(Style::default().max_width(90.0).font_size(12.0)).child(PARAGRAPH))
            .into(),
        d(cell(1, &bg()))
            .child(d(Style::default()
                .width(20.0)
                .min_width(100.0)
                .height(40.0)))
            .into(),
        d(cell(2, &bg()))
            .child(d(Style::default()
                .width(pct(100.0))
                .max_width(pct(60.0))
                .height(40.0)))
            .into(),
        d(cell(3, &bg().display(Display::Flex)))
            .child(d(Style::default()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(pct(0.0))
                .min_width(0.0)
                .max_width(70.0)
                .height(40.0)))
            .child(d(Style::default()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(pct(0.0))
                .min_width(50.0)
                .height(40.0)))
            .into(),
        d(cell(4, &bg()))
            .child(d(Style::default().height(300.0).max_height(80.0)))
            .into(),
        d(cell(5, &bg()))
            .child(d(Style::default().min_height(120.0)).child("short"))
            .into(),
        d(cell(
            6,
            &bg()
                .display(Display::Flex)
                .align_items(opsis_scene::style::Align::FlexStart),
        ))
        .child(
            d(Style::default()
                .max_width(Length::min([Length::px(80.0), pct(50.0)]))
                .font_size(12.0))
            .child(PARAGRAPH),
        )
        .into(),
        d(cell(7, &bg()))
            .child(d(Style::default()
                .width(Length::max([Length::px(40.0), pct(75.0)]))
                .height(40.0)))
            .into(),
    ])
}

fn square() -> Style {
    Style::default().width(40.0).height(40.0)
}

fn position_shape() -> Node {
    let abs = || square().position(Position::Absolute);
    stage(vec![
        d(cell(0, &bg()))
            .child(d(square()))
            .child(d(square()
                .position(Position::Relative)
                .left(30.0)
                .top(-10.0)))
            .child(d(square()))
            .into(),
        d(cell(1, &bg()))
            .child(d(abs().left(10.0).top(10.0)))
            .child(d(abs().right(10.0).bottom(10.0)))
            .into(),
        d(cell(2, &bg()))
            .child(d(Style::default()
                .position(Position::Absolute)
                .left(20.0)
                .right(20.0)
                .top(30.0)
                .bottom(50.0)))
            .into(),
        d(cell(3, &bg()))
            .child(
                d(Style::default()
                    .position(Position::Relative)
                    .left(20.0)
                    .top(20.0)
                    .width(90.0)
                    .height(90.0))
                .child(d(abs().right(-10.0).bottom(-10.0))),
            )
            .into(),
        d(cell(4, &bg()))
            .child(d(abs().left(pct(50.0)).top(pct(50.0))))
            .into(),
        d(cell(5, &bg()))
            .child(d(abs()
                .left(pct(50.0))
                .top(pct(50.0))
                .translate(pct(-50.0), pct(-50.0))))
            .into(),
        d(cell(6, &bg().overflow(Overflow::Hidden)))
            .child(d(abs().left(-20.0).top(-20.0)))
            .child(d(abs().right(-20.0).bottom(-20.0)))
            .into(),
        d(cell(7, &bg()))
            .child(d(abs().right(pct(10.0)).top(pct(10.0))))
            .child(d(Style::default()
                .position(Position::Absolute)
                .left(10.0)
                .top(70.0)
                .width(Length::Auto)
                .right(60.0)
                .height(30.0)))
            .into(),
    ])
}

fn inset_shape() -> Node {
    let insets: [Style; 8] = [
        Style::default().inset(Length::ZERO),
        Style::default().inset(20.0),
        Style::default()
            .top(10.0)
            .bottom(10.0)
            .left(40.0)
            .right(40.0),
        Style::default()
            .top(8.0)
            .right(16.0)
            .bottom(32.0)
            .left(48.0),
        Style::default().inset(pct(25.0)),
        Style::default().inset(-10.0),
        Style::default()
            .top(0.0)
            .right(Length::Auto)
            .bottom(Length::Auto)
            .left(0.0)
            .width(50.0)
            .height(50.0),
        Style::default()
            .top(Length::Auto)
            .right(0.0)
            .bottom(0.0)
            .left(Length::Auto)
            .width(pct(50.0))
            .height(pct(30.0)),
    ];
    stage(
        insets
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().overflow(Overflow::Hidden)))
                    .child(d(Style::default()
                        .position(Position::Absolute)
                        .merged(&style)))
                    .into()
            })
            .collect(),
    )
}

fn stacked(left: f64, top: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(left)
        .top(top)
        .width(60.0)
        .height(60.0)
}

fn z_index_shape() -> Node {
    let layer = |left: f64, top: f64, z: i32| d(stacked(left, top).z_index(z));
    stage(vec![
        d(cell(0, &bg()))
            .child(d(stacked(10.0, 10.0)))
            .child(d(stacked(40.0, 40.0)))
            .child(d(stacked(70.0, 70.0)))
            .into(),
        d(cell(1, &bg()))
            .child(layer(10.0, 10.0, 3))
            .child(layer(40.0, 40.0, 2))
            .child(layer(70.0, 70.0, 1))
            .into(),
        d(cell(2, &bg().isolation(true)))
            .child(d(stacked(20.0, 20.0)))
            .child(layer(50.0, 50.0, -1))
            .into(),
        d(cell(3, &bg()))
            .child(d(stacked(10.0, 10.0).z_index(1).opacity(0.99)).child(layer(30.0, 30.0, 10)))
            .child(layer(40.0, 40.0, 2))
            .into(),
        d(cell(4, &bg()))
            .child(d(stacked(10.0, 10.0).transform(vec![
                Transform::Translate(Length::ZERO, Length::ZERO, Length::ZERO),
            ])))
            .child(d(stacked(40.0, 40.0)))
            .into(),
        d(cell(5, &bg()))
            .child(d(stacked(40.0, 40.0).position(Position::Relative)))
            .child(d(stacked(10.0, 10.0).position(Position::Static)))
            .into(),
        d(cell(6, &bg()))
            .child(layer(10.0, 10.0, 5))
            .child(layer(30.0, 30.0, 5))
            .child(layer(50.0, 50.0, 4))
            .into(),
        d(cell(7, &bg()))
            .child(d(
                stacked(10.0, 10.0).filter(vec![Filter::Blur(Length::ZERO)])
            ))
            .child(layer(40.0, 40.0, 0))
            .child(d(stacked(25.0, 70.0)))
            .into(),
    ])
}

pub fn width_height() -> Node {
    colored(
        width_height_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 1, 0], cool()),
            (&[0, 2, 0], leaf()),
            (&[0, 3, 0], accent()),
            (&[0, 4, 0], warm()),
            (&[0, 4, 1], cool()),
            (&[0, 5, 0], leaf()),
            (&[0, 6, 0], accent()),
            (&[0, 7, 0], cool()),
        ],
    )
}

pub fn min_max_width() -> Node {
    colored(
        min_max_width_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 1, 0], cool()),
            (&[0, 2, 0], leaf()),
            (&[0, 3, 0], warm()),
            (&[0, 3, 1], cool()),
            (&[0, 4, 0], leaf()),
            (&[0, 5, 0], warm()),
            (&[0, 6, 0], cool()),
            (&[0, 7, 0], leaf()),
        ],
    )
}

pub fn position() -> Node {
    colored(
        position_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 0, 1], cool()),
            (&[0, 0, 2], leaf()),
            (&[0, 1, 0], warm()),
            (&[0, 1, 1], cool()),
            (&[0, 2, 0], leaf()),
            (&[0, 3, 0], accent()),
            (&[0, 3, 0, 0], rose()),
            (&[0, 4, 0], warm()),
            (&[0, 5, 0], cool()),
            (&[0, 6, 0], leaf()),
            (&[0, 6, 1], rose()),
            (&[0, 7, 0], warm()),
            (&[0, 7, 1], accent()),
        ],
    )
}

pub fn inset() -> Node {
    colored(
        inset_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 1, 0], cool()),
            (&[0, 2, 0], leaf()),
            (&[0, 3, 0], rose()),
            (&[0, 4, 0], warm()),
            (&[0, 5, 0], cool()),
            (&[0, 6, 0], leaf()),
            (&[0, 7, 0], rose()),
        ],
    )
}

pub fn z_index() -> Node {
    colored(
        z_index_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 0, 1], cool()),
            (&[0, 0, 2], leaf()),
            (&[0, 1, 0], warm()),
            (&[0, 1, 1], cool()),
            (&[0, 1, 2], leaf()),
            (&[0, 2, 0], warm()),
            (&[0, 2, 1], cool()),
            (&[0, 3, 0], warm()),
            (&[0, 3, 0, 0], rose()),
            (&[0, 3, 1], cool()),
            (&[0, 4, 0], warm()),
            (&[0, 4, 1], cool()),
            (&[0, 5, 0], cool()),
            (&[0, 5, 1], warm()),
            (&[0, 6, 0], warm()),
            (&[0, 6, 1], accent()),
            (&[0, 6, 2], leaf()),
            (&[0, 7, 0], warm()),
            (&[0, 7, 1], cool()),
            (&[0, 7, 2], rose()),
        ],
    )
}
