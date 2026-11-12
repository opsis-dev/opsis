use opsis_scene::node::Node;
use opsis_scene::paint::Transform;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, Outline, Overflow, Position, Style, TransformStyle,
    Visibility,
};
use opsis_scene::values::{Angle, Edges, Length, Num, Position2, Px};

use crate::paints::{hex, layer, linear, repeating, rgba_of, stop, stop_span};
use crate::{cell, d, pct, slate, stage, styled, warm};

fn bg() -> Style {
    Style::default().background_color(slate())
}

const fn px(value: f64) -> Length {
    Length::px(value)
}

const fn deg(value: f64) -> Angle {
    Angle::Deg(Num(value))
}

const fn translate(x: f64, y: f64) -> Transform {
    Transform::Translate(px(x), px(y), Length::ZERO)
}

const fn scale(x: f64, y: f64) -> Transform {
    Transform::Scale(Num(x), Num(y), Num(1.0))
}

fn origin(x: Length, y: Length) -> Style {
    Style::default().transform_origin((x, y, Length::ZERO))
}

fn outlined() -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: px(2.0),
        style: BorderStyle::Solid,
        color: None,
    })
}

fn centred() -> Style {
    bg().display(Display::Grid)
        .align_items(Align::Center)
        .justify_items(Align::Center)
}

fn translate_case_shape() -> Node {
    let moves = vec![
        Style::default().translate(Length::ZERO, Length::ZERO),
        Style::default().translate(px(20.0), px(30.0)),
        Style::default().translate(pct(50.0), Length::ZERO),
        Style::default().translate(pct(-25.0), pct(120.0)),
        Style::default().translate(px(10.5), px(0.25)),
        Style::default().transform(vec![translate(30.0, -10.0)]),
        Style::default().transform(vec![translate(40.0, 0.0), translate(0.0, 60.0)]),
        Style::default()
            .translate(px(20.0), px(20.0))
            .transform(vec![translate(10.0, 10.0)]),
    ];
    stage(
        moves
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().overflow(Overflow::Hidden)))
                    .child(d(Style::default().width(50.0).height(50.0).merged(&style)))
                    .into()
            })
            .collect(),
    )
}

fn rotate_case_shape() -> Node {
    let turns = vec![
        Style::default().rotate(deg(0.0)),
        Style::default().rotate(deg(15.0)),
        Style::default().rotate(deg(45.0)),
        Style::default().rotate(deg(-90.0)),
        Style::default().rotate(Angle::Turn(Num(0.5))),
        Style::default().rotate(deg(3.3)),
        Style::default().transform(vec![Transform::Rotate(deg(30.0))]),
        Style::default()
            .rotate(deg(20.0))
            .merged(&origin(Length::ZERO, Length::ZERO)),
    ];
    stage(
        turns
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &centred()))
                    .child(
                        d(Style::default()
                            .width(80.0)
                            .height(40.0)
                            .border(outlined())
                            .font_size(14.0)
                            .merged(&style))
                        .child("Rotate"),
                    )
                    .into()
            })
            .collect(),
    )
}

fn scale_case_shape() -> Node {
    let scales = vec![
        Style::default().scale(1.0, 1.0),
        Style::default().scale(0.5, 0.5),
        Style::default().scale(1.4, 1.4),
        Style::default().scale(1.5, 0.6),
        Style::default().scale(-1.0, 1.0),
        Style::default().scale(0.001, 0.001),
        Style::default().transform(vec![scale(0.7, 1.2)]),
        Style::default()
            .scale(0.8, 0.8)
            .merged(&origin(Length::ZERO, Length::ZERO)),
    ];
    stage(
        scales
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &centred()))
                    .child(
                        d(Style::default()
                            .width(70.0)
                            .height(70.0)
                            .border_radius_all(12.0)
                            .font_weight(700.0)
                            .display(Display::Grid)
                            .align_items(Align::Center)
                            .justify_items(Align::Center)
                            .merged(&style))
                        .child("Ab"),
                    )
                    .into()
            })
            .collect(),
    )
}

fn transform_case_shape() -> Node {
    let transforms = vec![
        vec![],
        vec![
            translate(10.0, 20.0),
            Transform::Rotate(deg(20.0)),
            scale(0.8, 0.8),
        ],
        vec![
            scale(0.8, 0.8),
            Transform::Rotate(deg(20.0)),
            translate(10.0, 20.0),
        ],
        vec![Transform::Skew(deg(20.0), Angle::Zero)],
        vec![Transform::Skew(deg(-10.0), deg(15.0))],
        vec![Transform::Matrix([0.9, 0.3, -0.3, 0.9, 10.0, 0.0].map(Num))],
        vec![
            Transform::Translate(pct(-50.0), pct(-50.0), Length::ZERO),
            Transform::Rotate(deg(-12.0)),
        ],
        vec![Transform::Rotate(deg(45.0)), scale(1.5, 1.0)],
    ];
    stage(
        transforms
            .into_iter()
            .enumerate()
            .map(|(i, transform)| {
                let (left, top) = if i == 6 {
                    (pct(50.0), pct(50.0))
                } else {
                    (px(30.0), px(40.0))
                };
                d(cell(i, &bg()))
                    .child(d(Style::default()
                        .position(Position::Absolute)
                        .left(left)
                        .top(top)
                        .width(70.0)
                        .height(50.0)
                        .transform(transform)))
                    .into()
            })
            .collect(),
    )
}

fn transform_origin_case_shape() -> Node {
    let origins = [
        (pct(50.0), pct(50.0)),
        (Length::ZERO, Length::ZERO),
        (pct(100.0), pct(100.0)),
        (Length::ZERO, pct(100.0)),
        (px(20.0), px(80.0)),
        (pct(50.0), Length::ZERO),
        (pct(100.0), pct(50.0)),
        (px(-20.0), px(-20.0)),
    ];
    let square = || {
        Style::default()
            .position(Position::Absolute)
            .left(33.0)
            .top(39.0)
            .width(70.0)
            .height(70.0)
    };
    stage(
        origins
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                d(cell(i, &bg()))
                    .child(d(square()))
                    .child(d(square().merged(&origin(x, y)).transform(vec![
                        Transform::Rotate(deg(25.0)),
                        scale(0.7, 0.7),
                    ])))
                    .into()
            })
            .collect(),
    )
}

fn perspective_case_shape() -> Node {
    let depth = |value: f64| Style::default().perspective(Px(value));
    let scenes: Vec<(Style, Style)> = vec![
        (
            depth(400.0),
            Style::default().transform(vec![Transform::RotateY(deg(35.0))]),
        ),
        (
            depth(200.0),
            Style::default().transform(vec![Transform::RotateX(deg(45.0))]),
        ),
        (
            depth(800.0),
            Style::default().transform(vec![
                Transform::RotateY(deg(-60.0)),
                Transform::RotateX(deg(10.0)),
            ]),
        ),
        (
            depth(300.0).perspective_origin(Position2 {
                x: Length::ZERO,
                y: Length::ZERO,
            }),
            Style::default().transform(vec![Transform::RotateY(deg(30.0))]),
        ),
        (
            Style::default(),
            Style::default().transform(vec![
                Transform::Perspective(Some(Px(300.0))),
                Transform::RotateY(deg(45.0)),
                Transform::Translate(Length::ZERO, Length::ZERO, px(-40.0)),
            ]),
        ),
        (
            depth(500.0),
            Style::default().transform(vec![Transform::Translate(
                Length::ZERO,
                Length::ZERO,
                px(60.0),
            )]),
        ),
        (
            depth(400.0).transform_style(TransformStyle::Preserve3d),
            Style::default().transform(vec![
                Transform::RotateX(deg(60.0)),
                Transform::Rotate(deg(30.0)),
            ]),
        ),
        (
            depth(250.0),
            Style::default()
                .transform(vec![Transform::Rotate3d(
                    Num(1.0),
                    Num(1.0),
                    Num(0.0),
                    deg(50.0),
                )])
                .backface_visibility(Visibility::Hidden),
        ),
    ];
    stage(
        scenes
            .into_iter()
            .enumerate()
            .map(|(i, (scene, card))| {
                d(cell(i, &centred().merged(&scene)))
                    .child(
                        d(Style::default()
                            .width(80.0)
                            .height(80.0)
                            .border_radius_all(8.0)
                            .border(outlined())
                            .display(Display::Grid)
                            .align_items(Align::Center)
                            .justify_items(Align::Center)
                            .font_weight(700.0)
                            .merged(&card))
                        .child("3D"),
                    )
                    .into()
            })
            .collect(),
    )
}

fn matrix3d_case_shape() -> Node {
    let matrices: [[f64; 16]; 8] = [
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        [
            0.9, 0.1, 0.0, 0.001, -0.1, 0.9, 0.0, 0.0005, 0.0, 0.0, 1.0, 0.0, 10.0, 5.0, 0.0, 1.0,
        ],
        [
            1.0, 0.0, 0.0, 0.002, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.003, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        [
            0.8, 0.2, 0.0, -0.0015, 0.1, 1.0, 0.0, 0.001, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        [
            1.2, 0.0, 0.0, 0.0025, 0.0, 0.8, 0.0, -0.001, 0.0, 0.0, 1.0, 0.0, -10.0, 10.0, 0.0, 1.0,
        ],
        [
            0.7, -0.3, 0.0, 0.0, 0.3, 0.7, 0.0, 0.002, 0.0, 0.0, 1.0, 0.0, 20.0, 0.0, 0.0, 1.1,
        ],
        [
            1.0, 0.4, 0.0, 0.004, 0.0, 1.0, 0.0, 0.004, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    ];
    stage(
        matrices
            .into_iter()
            .enumerate()
            .map(|(i, matrix)| {
                d(cell(i, &bg()))
                    .child(d(Style::default()
                        .position(Position::Absolute)
                        .left(0.0)
                        .top(0.0)
                        .width(100.0)
                        .height(100.0)
                        .border(outlined())
                        .merged(&origin(Length::ZERO, Length::ZERO))
                        .transform(vec![Transform::Matrix3d(matrix.map(Num))])))
                    .into()
            })
            .collect(),
    )
}

pub fn translate_case() -> Node {
    let style = Style::default().background_color(warm());
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(translate_case_shape(), &list)
}

pub fn rotate_case() -> Node {
    let style = Style::default().background(vec![layer(linear(
        90.0,
        vec![stop(hex(0x0e_a5e9)), stop(hex(0x22_c55e))],
    ))]);
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(rotate_case_shape(), &list)
}

pub fn scale_case() -> Node {
    let style = Style::default().background(vec![layer(linear(
        90.0,
        vec![stop(hex(0xf9_7316)), stop(hex(0x7c_3aed))],
    ))]);
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(scale_case_shape(), &list)
}

pub fn transform_case() -> Node {
    let style = Style::default().background(vec![layer(linear(
        135.0,
        vec![stop(hex(0xf4_3f5e)), stop(hex(0xfa_cc15))],
    ))]);
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(transform_case_shape(), &list)
}

pub fn perspective_case() -> Node {
    let style = Style::default().background(vec![layer(linear(
        135.0,
        vec![stop(hex(0x0e_a5e9)), stop(hex(0x7c_3aed))],
    ))]);
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(perspective_case_shape(), &list)
}

pub fn transform_origin_case() -> Node {
    let outline = Style::default().outline(Outline {
        width: px(1.0),
        style: BorderStyle::Dashed,
        color: Some(hex(0x94_a3b8)),
        offset: Length::ZERO,
    });
    let violet = Style::default().background_color(rgba_of(124, 58, 237, 0.85));
    let paths: Vec<(Vec<usize>, Style)> = (0..8)
        .flat_map(|i| {
            [
                (vec![0, i, 0], outline.clone()),
                (vec![0, i, 1], violet.clone()),
            ]
        })
        .collect();
    let list: Vec<(&[usize], Style)> = paths
        .iter()
        .map(|(p, s)| (p.as_slice(), s.clone()))
        .collect();
    styled(transform_origin_case_shape(), &list)
}

pub fn matrix3d_case() -> Node {
    let style = Style::default()
        .background_color(hex(0xf9_7316))
        .background(vec![layer(repeating(linear(
            90.0,
            vec![
                stop_span(hex(0xf9_7316), Length::ZERO, px(10.0)),
                stop_span(hex(0x0f_172a), px(10.0), px(20.0)),
            ],
        )))]);
    let cells: Vec<Vec<usize>> = (0..8).map(|i| vec![0, i, 0]).collect();
    let list: Vec<(&[usize], Style)> = cells
        .iter()
        .map(|p| (p.as_slice(), style.clone()))
        .collect();
    styled(matrix3d_case_shape(), &list)
}
