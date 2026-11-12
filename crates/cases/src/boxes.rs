use opsis_scene::color::{Color, ColorSpace};
use opsis_scene::node::{Node, img};
use opsis_scene::paint::{
    AssetRef, Image, ImageLayer, LayerSize, RadialShape, RadialSize, Repeat, Shadow, Side,
    Transform,
};
use opsis_scene::style::{
    BorderSide, BorderStyle, BoxSizing, ObjectFit, Outline, Overflow, Position, Style,
};
use opsis_scene::values::{Angle, Corners, Edges, Length, Num, Position2};

use crate::paints::{
    at, colour, conic, degrees, hex, hexa, interpolated, layer, linear, linear_to, radial,
    repeating, rgba_of, space, stop, stop_at, stop_span, transparent,
};
use crate::{cell, d, ink, leaf, pct, slate, stage, warm};

const PARAGRAPH: &str = "Opsis turns takes into finished films, one frame at a time.";
const PHOTO: &str = "app:placeholders/launch-earth.jpg";
const SCREEN: &str = "app:placeholders/screen-tall.svg";

fn bg() -> Style {
    Style::default().background_color(slate())
}

const fn px(value: f64) -> Length {
    Length::px(value)
}

const fn side(width: f64, style: BorderStyle, color: Color) -> BorderSide {
    BorderSide {
        width: px(width),
        style,
        color: Some(color),
    }
}

fn solid(width: f64, color: Color) -> Edges<BorderSide> {
    Edges::all(side(width, BorderStyle::Solid, color))
}

const fn edges(top: f64, right: f64, bottom: f64, left: f64) -> Edges<Length> {
    Edges {
        top: Length::px(top),
        right: Length::px(right),
        bottom: Length::px(bottom),
        left: Length::px(left),
    }
}

fn cells(styles: Vec<Style>) -> Node {
    stage(
        styles
            .into_iter()
            .enumerate()
            .map(|(i, style)| d(cell(i, &style)).into())
            .collect(),
    )
}

fn layers(list: Vec<ImageLayer>) -> Style {
    Style::default().background(list)
}

const fn shadow(inset: bool, (x, y): (f64, f64), blur: f64, spread: f64, color: Color) -> Shadow {
    Shadow {
        inset,
        x: px(x),
        y: px(y),
        blur: px(blur),
        spread: px(spread),
        color,
    }
}

const fn corners(tl: f64, tr: f64, br: f64, bl: f64) -> Corners<(Length, Length)> {
    Corners {
        top_left: (px(tl), px(tl)),
        top_right: (px(tr), px(tr)),
        bottom_right: (px(br), px(br)),
        bottom_left: (px(bl), px(bl)),
    }
}

const fn centre() -> Position2 {
    at(50.0, 50.0)
}

fn photo_layer() -> ImageLayer {
    ImageLayer::of(Image::Url(AssetRef(PHOTO.to_owned())))
}

fn dots() -> ImageLayer {
    layer(radial(
        RadialShape::Circle,
        RadialSize::FarthestCorner,
        centre(),
        vec![stop_at(ink(), px(2.0)), stop_at(transparent(), px(2.5))],
    ))
}

pub fn background_color() -> Node {
    cells(vec![
        Style::default().background_color(hex(0xf9_7316)),
        Style::default().background_color(rgba_of(14, 165, 233, 0.5)),
        Style::default().background_color(rgba_of(255, 255, 255, 0.08)),
        Style::default().background_color(colour(ColorSpace::Hsl, [280.0, 70.0, 50.0], 1.0)),
        Style::default().background_color(hex(0x22_c55e)),
        Style::default()
            .background_color(transparent())
            .outline(Outline {
                width: px(1.0),
                style: BorderStyle::Solid,
                color: Some(ink()),
                offset: Length::ZERO,
            }),
        Style::default().background_color(hex(0xff_ffff)),
        Style::default().background_color(hexa(0x0000_0080)),
    ])
}

fn sized(l: ImageLayer, w: Length, h: Length) -> ImageLayer {
    ImageLayer {
        size: LayerSize::Explicit(w, h),
        ..l
    }
}

fn placed(l: ImageLayer, x: Length, y: Length) -> ImageLayer {
    ImageLayer {
        position: Position2 { x, y },
        ..l
    }
}

pub fn background_layers() -> Node {
    let photo_over = ImageLayer {
        size: LayerSize::Explicit(pct(200.0), Length::Auto),
        position: at(30.0, 50.0),
        repeat: (Repeat::NoRepeat, Repeat::NoRepeat),
        ..photo_layer()
    };
    cells(vec![
        layers(vec![sized(dots(), px(16.0), px(16.0))]),
        layers(vec![placed(
            sized(dots(), px(20.0), px(20.0)),
            px(10.0),
            px(10.0),
        )])
        .background_color(hex(0x7c_3aed)),
        layers(vec![ImageLayer {
            repeat: (Repeat::NoRepeat, Repeat::NoRepeat),
            ..sized(
                layer(linear(
                    180.0,
                    vec![stop(hex(0xf9_7316)), stop(hex(0x0e_a5e9))],
                )),
                pct(50.0),
                pct(50.0),
            )
        }]),
        layers(vec![ImageLayer {
            repeat: (Repeat::Repeat, Repeat::NoRepeat),
            ..sized(
                layer(linear(
                    180.0,
                    vec![stop(hex(0x22_c55e)), stop(hex(0x0f_172a))],
                )),
                px(40.0),
                px(40.0),
            )
        }]),
        layers(vec![ImageLayer {
            size: LayerSize::Cover,
            position: centre(),
            ..photo_layer()
        }]),
        layers(vec![ImageLayer {
            size: LayerSize::Contain,
            repeat: (Repeat::NoRepeat, Repeat::NoRepeat),
            ..photo_layer()
        }])
        .background_color(slate()),
        layers(vec![
            layer(linear(
                180.0,
                vec![
                    stop(rgba_of(15, 23, 42, 0.2)),
                    stop(rgba_of(15, 23, 42, 0.9)),
                ],
            )),
            photo_over,
        ]),
        layers(vec![
            placed(sized(dots(), px(12.0), px(12.0)), pct(100.0), pct(100.0)),
            sized(
                layer(linear(
                    90.0,
                    vec![stop(hex(0xf4_3f5e)), stop(hex(0xfa_cc15))],
                )),
                pct(100.0),
                pct(100.0),
            ),
        ]),
    ])
}

pub fn border_radius() -> Node {
    let gradient = || {
        layers(vec![layer(linear(
            135.0,
            vec![stop(hex(0xf9_7316)), stop(hex(0x7c_3aed))],
        ))])
    };
    let radii = vec![
        Style::default().border_radius_all(8.0),
        Style::default().border_radius_all(24.0),
        Style::default().border_radius_all(pct(50.0)),
        Style::default().border_radius_all(9999.0).height(60.0),
        Style::default().border_radius(corners(40.0, 0.0, 40.0, 0.0)),
        Style::default().border_radius(corners(10.0, 20.0, 30.0, 40.0)),
        Style::default().border_radius(Corners::all((pct(50.0), pct(20.0)))),
        Style::default().border_radius(corners(60.0, 0.0, 12.0, 0.0)),
    ];
    cells(
        radii
            .into_iter()
            .map(|style| gradient().merged(&style))
            .collect(),
    )
}

pub fn box_shadow() -> Node {
    let shadows: Vec<(Vec<Shadow>, Style)> = vec![
        (
            vec![shadow(false, (0.0, 4.0), 12.0, 0.0, rgba_of(0, 0, 0, 0.6))],
            Style::default(),
        ),
        (
            vec![shadow(false, (0.0, 0.0), 0.0, 4.0, hex(0xf9_7316))],
            Style::default(),
        ),
        (
            vec![shadow(
                false,
                (0.0, 20.0),
                40.0,
                -10.0,
                rgba_of(124, 58, 237, 0.9),
            )],
            Style::default().border_radius_all(20.0),
        ),
        (
            vec![shadow(true, (0.0, 0.0), 20.0, 0.0, rgba_of(0, 0, 0, 0.9))],
            Style::default(),
        ),
        (
            vec![
                shadow(true, (0.0, 2.0), 0.0, 0.0, rgba_of(255, 255, 255, 0.4)),
                shadow(false, (0.0, 8.0), 24.0, 0.0, rgba_of(14, 165, 233, 0.6)),
            ],
            Style::default().border_radius_all(12.0),
        ),
        (
            vec![
                shadow(false, (8.0, 8.0), 0.0, 0.0, hex(0x22_c55e)),
                shadow(false, (16.0, 16.0), 0.0, 0.0, hex(0xf4_3f5e)),
            ],
            Style::default(),
        ),
        (
            vec![shadow(
                false,
                (0.0, 0.0),
                40.0,
                10.0,
                rgba_of(250, 204, 21, 0.5),
            )],
            Style::default().border_radius_all(pct(50.0)),
        ),
        (
            vec![
                shadow(
                    false,
                    (-10.0, -10.0),
                    30.0,
                    0.0,
                    rgba_of(255, 255, 255, 0.2),
                ),
                shadow(false, (10.0, 10.0), 30.0, 0.0, rgba_of(0, 0, 0, 0.8)),
            ],
            Style::default().border_radius_all(24.0),
        ),
    ];
    cells(
        shadows
            .into_iter()
            .map(|(list, extra)| {
                bg().inset(Length::Auto)
                    .width(100.0)
                    .height(100.0)
                    .margin(Edges {
                        left: px(18.0),
                        top: px(24.0),
                        ..Edges::all(Length::ZERO)
                    })
                    .box_shadow(list)
                    .merged(&extra)
            })
            .collect(),
    )
}

fn gradient_cells(list: Vec<Vec<ImageLayer>>, round: impl Fn(usize) -> Option<Length>) -> Node {
    cells(
        list.into_iter()
            .enumerate()
            .map(|(i, l)| {
                let style = layers(l);
                match round(i) {
                    Some(r) => style.border_radius_all(r),
                    None => style,
                }
            })
            .collect(),
    )
}

pub fn linear_gradient() -> Node {
    let one = |g| vec![layer(g)];
    gradient_cells(
        vec![
            one(linear(
                180.0,
                vec![stop(hex(0xf9_7316)), stop(hex(0x0e_a5e9))],
            )),
            one(linear(
                90.0,
                vec![
                    stop(hex(0xf9_7316)),
                    stop_at(hex(0x7c_3aed), pct(40.0)),
                    stop(hex(0x22_c55e)),
                ],
            )),
            one(linear(
                135.0,
                vec![
                    stop(rgba_of(255, 255, 255, 0.9)),
                    stop_at(rgba_of(255, 255, 255, 0.0), pct(70.0)),
                ],
            )),
            one(linear_to(
                Some(Side::Right),
                Some(Side::Top),
                vec![stop(hex(0x0f_172a)), stop(hex(0xf4_3f5e))],
            )),
            one(interpolated(
                linear(180.0, vec![stop(hex(0xf9_7316)), stop(hex(0x0e_a5e9))]),
                space(ColorSpace::Oklch),
            )),
            one(interpolated(
                linear(45.0, vec![stop(hex(0x7c_3aed)), stop(hex(0xfa_cc15))]),
                space(ColorSpace::Oklab),
            )),
            one(linear(
                180.0,
                vec![
                    stop_span(hex(0x22_c55e), Length::ZERO, pct(30.0)),
                    stop_span(hex(0x0e_a5e9), pct(30.0), pct(60.0)),
                    stop_at(hex(0xf4_3f5e), pct(60.0)),
                ],
            )),
            vec![
                layer(linear(
                    200.0,
                    vec![stop(transparent()), stop_at(hex(0x00_0000), pct(80.0))],
                )),
                layer(linear(
                    90.0,
                    vec![stop(hex(0xf9_7316)), stop(hex(0x7c_3aed))],
                )),
            ],
        ],
        |_| None,
    )
}

pub fn radial_gradient() -> Node {
    let one = |g| vec![layer(g)];
    let ellipse = RadialShape::Ellipse;
    let circle = RadialShape::Circle;
    let far = || RadialSize::FarthestCorner;
    gradient_cells(
        vec![
            one(radial(
                ellipse,
                far(),
                centre(),
                vec![stop(hex(0xf9_7316)), stop(hex(0x0f_172a))],
            )),
            one(radial(
                circle,
                far(),
                centre(),
                vec![
                    stop(hex(0xf8_fafc)),
                    stop_at(hex(0x7c_3aed), pct(60.0)),
                    stop_at(transparent(), pct(61.0)),
                ],
            )),
            one(radial(
                circle,
                far(),
                at(30.0, 30.0),
                vec![stop(hex(0x0e_a5e9)), stop_at(transparent(), pct(70.0))],
            )),
            one(radial(
                ellipse,
                far(),
                at(50.0, 0.0),
                vec![stop(hex(0x22_c55e)), stop_at(hex(0x0f_172a), pct(80.0))],
            )),
            one(radial(
                ellipse,
                RadialSize::ClosestSide,
                centre(),
                vec![stop(hex(0xf4_3f5e)), stop(slate())],
            )),
            one(radial(
                ellipse,
                far(),
                at(80.0, 90.0),
                vec![
                    stop(hex(0xfa_cc15)),
                    stop_at(hex(0x7c_3aed), pct(50.0)),
                    stop(hex(0x0f_172a)),
                ],
            )),
            one(interpolated(
                radial(
                    ellipse,
                    far(),
                    centre(),
                    vec![stop(hex(0xf9_7316)), stop(hex(0x0e_a5e9))],
                ),
                space(ColorSpace::Oklch),
            )),
            one(radial(
                ellipse,
                RadialSize::Explicit(px(120.0), px(60.0)),
                centre(),
                vec![
                    stop(rgba_of(255, 255, 255, 0.8)),
                    stop(rgba_of(255, 255, 255, 0.0)),
                ],
            )),
        ],
        |_| None,
    )
}

pub fn conic_gradient() -> Node {
    let one = |g| vec![layer(g)];
    let orange = || hex(0xf9_7316);
    let blue = || hex(0x0e_a5e9);
    gradient_cells(
        vec![
            one(conic(
                0.0,
                centre(),
                vec![
                    stop(orange()),
                    stop(blue()),
                    stop(hex(0x22_c55e)),
                    stop(orange()),
                ],
            )),
            one(conic(
                90.0,
                centre(),
                vec![stop(hex(0x7c_3aed)), stop(hex(0xf4_3f5e))],
            )),
            one(conic(
                0.0,
                at(30.0, 70.0),
                vec![stop(hex(0xfa_cc15)), stop(hex(0x0f_172a))],
            )),
            one(conic(
                0.0,
                centre(),
                vec![
                    stop_span(hex(0x22_c55e), Length::ZERO, pct(25.0)),
                    stop_span(slate(), pct(25.0), pct(100.0)),
                ],
            )),
            one(interpolated(
                conic(
                    0.0,
                    centre(),
                    vec![stop(orange()), stop(blue()), stop(orange())],
                ),
                space(ColorSpace::Oklch),
            )),
            one(conic(
                45.0,
                centre(),
                vec![
                    stop(rgba_of(255, 255, 255, 0.9)),
                    stop_at(rgba_of(255, 255, 255, 0.0), pct(50.0)),
                    stop(rgba_of(255, 255, 255, 0.9)),
                ],
            )),
            one(repeating(conic(
                0.0,
                centre(),
                vec![
                    stop_span(hex(0xf8_fafc), Length::ZERO, degrees(10.0)),
                    stop_span(hex(0x0f_172a), degrees(10.0), degrees(20.0)),
                ],
            ))),
            one(conic(
                180.0,
                at(50.0, 0.0),
                vec![stop(blue()), stop(hex(0x7c_3aed)), stop(blue())],
            )),
        ],
        |i| (i % 2 == 0).then(|| pct(50.0)),
    )
}

pub fn repeating_gradient() -> Node {
    let one = |g| vec![layer(g)];
    gradient_cells(
        vec![
            one(repeating(linear(
                45.0,
                vec![
                    stop_span(hex(0xf9_7316), Length::ZERO, px(10.0)),
                    stop_span(hex(0x0f_172a), px(10.0), px(20.0)),
                ],
            ))),
            one(repeating(linear(
                90.0,
                vec![
                    stop_span(rgba_of(255, 255, 255, 0.15), Length::ZERO, px(1.0)),
                    stop_span(transparent(), px(1.0), px(12.0)),
                ],
            ))),
            one(repeating(radial(
                RadialShape::Circle,
                RadialSize::FarthestCorner,
                centre(),
                vec![
                    stop_span(hex(0x0e_a5e9), Length::ZERO, px(6.0)),
                    stop_span(hex(0x0f_172a), px(6.0), px(14.0)),
                ],
            ))),
            one(repeating(linear(
                0.0,
                vec![stop(hex(0x22_c55e)), stop_at(hex(0x7c_3aed), pct(25.0))],
            ))),
            vec![
                layer(repeating(linear(
                    180.0,
                    vec![
                        stop_span(transparent(), Length::ZERO, px(3.0)),
                        stop_span(rgba_of(0, 0, 0, 0.5), px(3.0), px(4.0)),
                    ],
                ))),
                layer(linear(
                    180.0,
                    vec![stop(hex(0xf4_3f5e)), stop(hex(0xfa_cc15))],
                )),
            ],
            one(repeating(radial(
                RadialShape::Ellipse,
                RadialSize::FarthestCorner,
                at(20.0, 20.0),
                vec![
                    stop_span(hex(0xf8_fafc), Length::ZERO, px(2.0)),
                    stop_span(transparent(), px(2.0), px(9.0)),
                ],
            ))),
            one(repeating(linear(
                -30.0,
                vec![
                    stop_span(slate(), Length::ZERO, px(8.0)),
                    stop_span(hex(0x47_5569), px(8.0), px(16.0)),
                ],
            ))),
            one(repeating(conic(
                0.0,
                centre(),
                vec![
                    stop_span(hex(0x0e_a5e9), Length::ZERO, degrees(15.0)),
                    stop_span(hex(0x0f_172a), degrees(15.0), degrees(30.0)),
                ],
            ))),
        ],
        |_| None,
    )
}

pub fn border() -> Node {
    let none = || side(0.0, BorderStyle::Solid, ink());
    let borders = vec![
        Style::default().border(solid(1.0, ink())),
        Style::default().border(solid(6.0, warm())),
        Style::default()
            .border(solid(4.0, rgba_of(255, 255, 255, 0.3)))
            .border_radius_all(16.0),
        Style::default().border(Edges {
            top: side(8.0, BorderStyle::Solid, hex(0x0e_a5e9)),
            bottom: side(2.0, BorderStyle::Solid, leaf()),
            left: none(),
            right: none(),
        }),
        Style::default().border(Edges {
            top: side(2.0, BorderStyle::Solid, warm()),
            right: side(10.0, BorderStyle::Solid, hex(0x0e_a5e9)),
            bottom: side(18.0, BorderStyle::Solid, leaf()),
            left: side(4.0, BorderStyle::Solid, hex(0xf4_3f5e)),
        }),
        Style::default()
            .border(Edges::all(side(3.0, BorderStyle::Dashed, hex(0xfa_cc15))))
            .border_radius_all(12.0),
        Style::default().border(Edges::all(side(2.0, BorderStyle::Dotted, ink()))),
        Style::default()
            .border(solid(12.0, hex(0x7c_3aed)))
            .border_radius_all(pct(50.0)),
    ];
    cells(
        borders
            .into_iter()
            .map(|style| bg().merged(&style))
            .collect(),
    )
}

pub fn padding() -> Node {
    let paddings = vec![
        Style::default().padding(Edges::all(Length::ZERO)),
        Style::default().padding(Edges::all(Length::px(12.0))),
        Style::default().padding(edges(4.0, 24.0, 4.0, 24.0)),
        Style::default().padding(edges(2.0, 8.0, 30.0, 40.0)),
        Style::default().padding(Edges::all(pct(10.0))),
        Style::default().padding(edges(20.0, 0.0, 0.0, 50.0)),
        Style::default()
            .padding(Edges::all(Length::em(0.75)))
            .font_size(24.0),
        Style::default()
            .padding(Edges::all(Length::px(16.0)))
            .margin(Edges::all(Length::ZERO))
            .box_sizing(BoxSizing::ContentBox)
            .width(104.0)
            .height(116.0),
    ];
    stage(
        paddings
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().merged(&style)))
                    .child(
                        d(Style::default()
                            .background_color(warm())
                            .height(pct(100.0))
                            .min_height(20.0))
                        .child("Aa"),
                    )
                    .into()
            })
            .collect(),
    )
}

pub fn overflow() -> Node {
    let hidden = || Style::default().overflow(Overflow::Hidden);
    let overflows = vec![
        Style::default().overflow(Overflow::Visible),
        hidden(),
        hidden().border_radius_all(32.0),
        hidden().border_radius_all(pct(50.0)),
        Style::default()
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Visible),
        Style::default().overflow(Overflow::Clip),
        hidden()
            .transform(vec![Transform::Rotate(Angle::Deg(Num(10.0)))])
            .border_radius_all(16.0),
        hidden().font_size(28.0),
    ];
    stage(
        overflows
            .into_iter()
            .enumerate()
            .map(|(i, style)| {
                d(cell(i, &bg().height(100.0).merged(&style)))
                    .child(d(Style::default()
                        .position(Position::Absolute)
                        .left(-20.0)
                        .top(40.0)
                        .width(180.0)
                        .height(30.0)
                        .background_color(warm())))
                    .child(d(Style::default()
                        .width(60.0)
                        .height(140.0)
                        .background_color(hex(0x0e_a5e9))
                        .margin(Edges {
                            left: Length::px(50.0),
                            ..Edges::all(Length::ZERO)
                        })))
                    .child(
                        d(Style::default()
                            .position(Position::Absolute)
                            .left(4.0)
                            .top(4.0)
                            .width(200.0)
                            .color(leaf()))
                        .child(PARAGRAPH),
                    )
                    .into()
            })
            .collect(),
    )
}

pub fn object_fit() -> Node {
    let fits = [
        (PHOTO, ObjectFit::Cover, None),
        (PHOTO, ObjectFit::Contain, None),
        (PHOTO, ObjectFit::Fill, None),
        (PHOTO, ObjectFit::None, None),
        (PHOTO, ObjectFit::Cover, Some(at(0.0, 0.0))),
        (PHOTO, ObjectFit::ScaleDown, None),
        (SCREEN, ObjectFit::Contain, None),
        (SCREEN, ObjectFit::Cover, Some(at(50.0, 0.0))),
    ];
    stage(
        fits.into_iter()
            .enumerate()
            .map(|(i, (source, fit, position))| {
                let style = Style::default()
                    .width(pct(100.0))
                    .height(pct(100.0))
                    .max_width(Length::Auto)
                    .object_fit(fit);
                let style = match position {
                    Some(p) => style.object_position(p),
                    None => style,
                };
                d(cell(i, &bg())).child(img(source, style)).into()
            })
            .collect(),
    )
}
