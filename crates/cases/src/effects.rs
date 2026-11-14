use opsis_scene::color::{Color, ColorSpace};
use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::paint::{
    AssetRef, BlendMode, BoxArea, ClipPath, FillRule, Filter, Image, ImageLayer, LayerSize,
    MaskLayer, MaskMode, RadialShape, RadialSize, Repeat, Shadow, ShapeRadius, Transform,
};
use opsis_scene::style::{Align, BorderSide, BorderStyle, Display, Position, Style};
use opsis_scene::values::{Angle, Edges, Length, Num, Position2};

use crate::paints::{
    at, conic, hex, interpolated, layer, linear, radial, repeating, rgba_of, space, stop, stop_at,
    stop_span, transparent,
};
use crate::{cell, cool, d, leaf, pct, s, slate, stage, warm};

const SAMPLE: &str = "Sphinx of black quartz";
const PHOTO: &str = "app:placeholders/launch-earth.jpg";
const LOGO: &str = "app:placeholders/logo-mark.svg";

const fn px(value: f64) -> Length {
    Length::px(value)
}

fn bg() -> Style {
    Style::default().background_color(slate())
}

fn absolute(left: f64, top: f64, width: f64, height: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(left)
        .top(top)
        .width(width)
        .height(height)
}

fn centred(style: Style) -> Style {
    style
        .display(Display::Grid)
        .align_items(Align::Center)
        .justify_items(Align::Center)
}

const fn shadow(x: f64, y: f64, blur: f64, color: Color) -> Shadow {
    Shadow {
        inset: false,
        x: px(x),
        y: px(y),
        blur: px(blur),
        spread: Length::ZERO,
        color,
    }
}

fn photo() -> ImageLayer {
    ImageLayer::of(Image::Url(AssetRef(PHOTO.to_owned())))
}

fn brand() -> ImageLayer {
    layer(linear(
        135.0,
        vec![
            stop(hex(0xf9_7316)),
            stop_at(hex(0x7c_3aed), pct(60.0)),
            stop(hex(0x0e_a5e9)),
        ],
    ))
}

pub fn opacity() -> Node {
    let levels = [1.0, 0.75, 0.5, 0.25, 0.1, 0.004, 0.0];
    let mut cells: Vec<Node> = levels
        .iter()
        .enumerate()
        .map(|(i, level)| {
            d(cell(i, &bg()))
                .child(d(absolute(10.0, 10.0, 80.0, 80.0)
                    .background_color(warm())
                    .opacity(*level)))
                .child(d(absolute(46.0, 46.0, 80.0, 80.0)
                    .background_color(cool())
                    .opacity(*level)))
                .into()
        })
        .collect();
    cells.push(
        d(cell(7, &bg()))
            .child(
                d(Style::default()
                    .position(Position::Absolute)
                    .inset(px(10.0))
                    .opacity(0.5))
                .child(d(absolute(0.0, 0.0, 80.0, 80.0).background_color(warm())))
                .child(d(absolute(36.0, 36.0, 80.0, 80.0).background_color(leaf()))),
            )
            .into(),
    );
    stage(cells)
}

pub fn filter_blur() -> Node {
    let blurs = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 40.0];
    stage(
        blurs
            .iter()
            .enumerate()
            .map(|(i, radius)| {
                d(cell(i, &bg()))
                    .child(
                        d(Style::default()
                            .position(Position::Absolute)
                            .inset(px(16.0))
                            .filter(vec![Filter::Blur(px(*radius))]))
                        .child(d(Style::default()
                            .width(60.0)
                            .height(40.0)
                            .background_color(hex(0xf9_7316))))
                        .child(
                            d(Style::default()
                                .font_weight(700.0)
                                .font_size(18.0)
                                .margin(Edges {
                                    top: px(8.0),
                                    ..Edges::all(Length::ZERO)
                                }))
                            .child(SAMPLE),
                        ),
                    )
                    .into()
            })
            .collect(),
    )
}

pub fn filter_color() -> Node {
    let filters: Vec<Vec<Filter>> = vec![
        vec![],
        vec![Filter::Brightness(Num(1.6))],
        vec![Filter::Brightness(Num(0.4))],
        vec![Filter::Saturate(Num(0.0))],
        vec![Filter::Saturate(Num(2.5))],
        vec![Filter::Contrast(Num(1.8))],
        vec![Filter::Sepia(Num(1.0))],
        vec![
            Filter::HueRotate(Angle::Deg(Num(120.0))),
            Filter::Contrast(Num(0.8)),
            Filter::Brightness(Num(1.1)),
        ],
    ];
    let bands = layer(linear(
        180.0,
        vec![
            stop_at(transparent(), pct(60.0)),
            stop_span(hex(0xf9_7316), pct(60.0), pct(80.0)),
            stop_at(hex(0x0e_a5e9), pct(80.0)),
        ],
    ));
    let cover = ImageLayer {
        size: LayerSize::Cover,
        position: at(50.0, 50.0),
        ..photo()
    };
    stage(
        filters
            .into_iter()
            .enumerate()
            .map(|(i, list)| {
                d(cell(
                    i,
                    &Style::default()
                        .filter(list)
                        .background(vec![bands.clone(), cover.clone()]),
                ))
                .into()
            })
            .collect(),
    )
}

fn triangle() -> SvgElement {
    SvgElement::new("svg")
        .attr("width", "70")
        .attr("height", "70")
        .attr("viewBox", "0 0 70 70")
        .child(
            SvgElement::new("path")
                .attr("d", "M35 4 L66 62 L4 62 Z")
                .attr("fill", "#f8fafc"),
        )
}

pub fn filter_drop_shadow() -> Node {
    let shadows: Vec<Vec<Filter>> = vec![
        vec![Filter::DropShadow(shadow(
            0.0,
            4.0,
            6.0,
            rgba_of(0, 0, 0, 0.8),
        ))],
        vec![Filter::DropShadow(shadow(6.0, 6.0, 0.0, hex(0xf9_7316)))],
        vec![Filter::DropShadow(shadow(0.0, 0.0, 12.0, hex(0x0e_a5e9)))],
        vec![
            Filter::DropShadow(shadow(0.0, 0.0, 2.0, hex(0xff_ffff))),
            Filter::DropShadow(shadow(0.0, 0.0, 10.0, hex(0x7c_3aed))),
        ],
        vec![Filter::DropShadow(shadow(
            -8.0,
            10.0,
            4.0,
            rgba_of(34, 197, 94, 0.7),
        ))],
        vec![
            Filter::DropShadow(shadow(0.0, 20.0, 20.0, rgba_of(0, 0, 0, 0.9))),
            Filter::Brightness(Num(1.2)),
        ],
        vec![
            Filter::Blur(px(2.0)),
            Filter::DropShadow(shadow(0.0, 0.0, 6.0, hex(0xfa_cc15))),
        ],
        vec![Filter::DropShadow(shadow(0.0, 0.0, 0.0, transparent()))],
    ];
    stage(
        shadows
            .into_iter()
            .enumerate()
            .map(|(i, list)| {
                d(cell(i, &centred(bg())))
                    .child(
                        d(centred(Style::default().filter(list)))
                            .child(svg(triangle(), Style::default()))
                            .child(
                                s(Style::default().font_weight(800.0).font_size(20.0)).child("Hi"),
                            ),
                    )
                    .into()
            })
            .collect(),
    )
}

pub fn mix_blend_mode() -> Node {
    let modes = [
        BlendMode::Normal,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Lighten,
        BlendMode::Multiply,
        BlendMode::Darken,
        BlendMode::PlusLighter,
        BlendMode::Difference,
    ];
    let backdrop = layer(linear(
        90.0,
        vec![
            stop(hex(0x0f_172a)),
            stop_at(hex(0xf9_7316), pct(50.0)),
            stop(hex(0xf8_fafc)),
        ],
    ));
    let disc = layer(radial(
        RadialShape::Circle,
        RadialSize::FarthestCorner,
        at(50.0, 50.0),
        vec![
            stop(hex(0x0e_a5e9)),
            stop_at(hex(0x7c_3aed), pct(70.0)),
            stop_at(transparent(), pct(71.0)),
        ],
    ));
    stage(
        modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                d(cell(
                    i,
                    &Style::default()
                        .background(vec![backdrop.clone()])
                        .isolation(true),
                ))
                .child(d(Style::default()
                    .position(Position::Absolute)
                    .inset(px(20.0))
                    .border_radius_all(pct(50.0))
                    .background(vec![disc.clone()])
                    .mix_blend_mode(*mode)))
                .into()
            })
            .collect(),
    )
}

pub fn backdrop_filter() -> Node {
    let filters: Vec<Vec<Filter>> = vec![
        vec![Filter::Blur(px(4.0))],
        vec![Filter::Blur(px(16.0))],
        vec![Filter::Blur(px(12.0)), Filter::Saturate(Num(1.6))],
        vec![Filter::Brightness(Num(0.5))],
        vec![Filter::Blur(px(8.0)), Filter::Brightness(Num(1.3))],
        vec![Filter::Contrast(Num(1.5)), Filter::Blur(px(2.0))],
        vec![Filter::Blur(px(24.0))],
        vec![],
    ];
    let stripes = ImageLayer {
        blend: BlendMode::Overlay,
        ..layer(repeating(linear(
            45.0,
            vec![
                stop_span(hex(0xf9_7316), Length::ZERO, px(8.0)),
                stop_span(hex(0x0f_172a), px(8.0), px(16.0)),
            ],
        )))
    };
    let picture = ImageLayer {
        blend: BlendMode::Overlay,
        ..photo()
    };
    stage(
        filters
            .into_iter()
            .enumerate()
            .map(|(i, list)| {
                d(cell(
                    i,
                    &Style::default()
                        .background(vec![stripes.clone(), picture.clone()])
                        .overflow(opsis_scene::style::Overflow::Hidden),
                ))
                .child(d(Style::default()
                    .position(Position::Absolute)
                    .left(16.0)
                    .top(30.0)
                    .right(16.0)
                    .bottom(30.0)
                    .border_radius_all(16.0)
                    .background_color(rgba_of(255, 255, 255, 0.12))
                    .border(Edges::all(BorderSide {
                        width: px(1.0),
                        style: BorderStyle::Solid,
                        color: Some(rgba_of(255, 255, 255, 0.3)),
                    }))
                    .backdrop_filter(list)))
                .into()
            })
            .collect(),
    )
}

pub fn background_clip_text() -> Node {
    let one = |g| vec![layer(g)];
    let fills: Vec<(Vec<ImageLayer>, Style)> = vec![
        (
            one(linear(
                90.0,
                vec![stop(hex(0xf9_7316)), stop(hex(0x7c_3aed))],
            )),
            Style::default(),
        ),
        (
            one(linear(
                180.0,
                vec![stop_at(hex(0xf8_fafc), pct(40.0)), stop(hex(0x0e_a5e9))],
            )),
            Style::default(),
        ),
        (
            one(radial(
                RadialShape::Circle,
                RadialSize::FarthestCorner,
                at(30.0, 50.0),
                vec![stop(hex(0xfa_cc15)), stop_at(hex(0xf4_3f5e), pct(60.0))],
            )),
            Style::default(),
        ),
        (
            one(repeating(linear(
                45.0,
                vec![
                    stop_span(hex(0x22_c55e), Length::ZERO, px(4.0)),
                    stop_span(hex(0x0e_a5e9), px(4.0), px(8.0)),
                ],
            ))),
            Style::default(),
        ),
        (
            vec![ImageLayer {
                size: LayerSize::Explicit(pct(200.0), pct(100.0)),
                position: at(30.0, 0.0),
                ..layer(linear(
                    90.0,
                    vec![
                        stop(rgba_of(255, 255, 255, 0.2)),
                        stop_at(hex(0xff_ffff), pct(50.0)),
                        stop(rgba_of(255, 255, 255, 0.2)),
                    ],
                ))
            }],
            Style::default(),
        ),
        (
            one(conic(
                0.0,
                at(50.0, 50.0),
                vec![
                    stop(hex(0xf9_7316)),
                    stop(hex(0x0e_a5e9)),
                    stop(hex(0x22_c55e)),
                    stop(hex(0xf9_7316)),
                ],
            )),
            Style::default(),
        ),
        (
            one(interpolated(
                linear(90.0, vec![stop(hex(0xf4_3f5e)), stop(hex(0x0e_a5e9))]),
                space(ColorSpace::Oklch),
            )),
            Style::default(),
        ),
        (
            one(linear(
                180.0,
                vec![stop(hex(0xf8_fafc)), stop(hex(0x94_a3b8))],
            )),
            Style::default().filter(vec![Filter::DropShadow(shadow(
                0.0,
                2.0,
                4.0,
                rgba_of(0, 0, 0, 0.8),
            ))]),
        ),
    ];
    stage(
        fills
            .into_iter()
            .enumerate()
            .map(|(i, (layers, extra))| {
                d(cell(
                    i,
                    &Style::default()
                        .font_size(44.0)
                        .font_weight(900.0)
                        .line_height(opsis_scene::style::LineHeight::Number(Num(1.0)))
                        .letter_spacing(Length::em(-0.03))
                        .color(transparent())
                        .background_clip_text(true)
                        .background(layers)
                        .merged(&extra),
                ))
                .child("Big type")
                .into()
            })
            .collect(),
    )
}

fn polygon(points: &[(f64, f64)]) -> ClipPath {
    ClipPath::Polygon {
        fill: FillRule::Nonzero,
        points: points.iter().map(|(x, y)| (pct(*x), pct(*y))).collect(),
    }
}

pub fn clip_path() -> Node {
    let clips = vec![
        ClipPath::Inset {
            edges: Edges::all(px(10.0)),
            round: None,
        },
        ClipPath::Inset {
            edges: Edges {
                top: pct(10.0),
                right: pct(20.0),
                bottom: pct(30.0),
                left: Length::ZERO,
            },
            round: Some(opsis_scene::values::Corners::all((px(16.0), px(16.0)))),
        },
        ClipPath::Inset {
            edges: Edges {
                top: Length::ZERO,
                right: pct(50.0),
                bottom: Length::ZERO,
                left: Length::ZERO,
            },
            round: None,
        },
        polygon(&[(50.0, 0.0), (100.0, 100.0), (0.0, 100.0)]),
        polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 70.0), (0.0, 100.0)]),
        ClipPath::Circle {
            radius: ShapeRadius::Length(pct(40.0)),
            center: at(50.0, 50.0),
        },
        ClipPath::Circle {
            radius: ShapeRadius::Length(px(60.0)),
            center: Position2 {
                x: Length::ZERO,
                y: Length::ZERO,
            },
        },
        ClipPath::Path {
            fill: FillRule::Nonzero,
            data: "M10 70 C 40 0, 90 0, 120 70 S 60 140, 10 70 Z".to_owned(),
        },
    ];
    stage(
        clips
            .into_iter()
            .enumerate()
            .map(|(i, clip)| {
                d(cell(
                    i,
                    &Style::default().clip_path(clip).background(vec![brand()]),
                ))
                .into()
            })
            .collect(),
    )
}

pub fn clip_path_url() -> Node {
    let defs = SvgElement::new("svg")
        .attr("width", "0")
        .attr("height", "0")
        .child(
            SvgElement::new("defs")
                .child(
                    SvgElement::new("clipPath")
                        .attr("id", "case-star")
                        .attr("clipPathUnits", "objectBoundingBox")
                        .child(SvgElement::new("path").attr(
                            "d",
                            "M0.5 0 L0.62 0.38 L1 0.38 L0.69 0.6 L0.81 1 L0.5 0.76 L0.19 1 L0.31 0.6 L0 0.38 L0.38 0.38 Z",
                        )),
                )
                .child(
                    SvgElement::new("clipPath")
                        .attr("id", "case-blobs")
                        .child(
                            SvgElement::new("circle")
                                .attr("cx", "40")
                                .attr("cy", "40")
                                .attr("r", "35"),
                        )
                        .child(
                            SvgElement::new("circle")
                                .attr("cx", "100")
                                .attr("cy", "100")
                                .attr("r", "45"),
                        ),
                )
                .child(
                    SvgElement::new("clipPath").attr("id", "case-text").child(
                        SvgElement::new("text")
                            .attr("x", "4")
                            .attr("y", "90")
                            .attr("font-size", "72")
                            .attr("font-weight", "900")
                            .attr("font-family", "Inter")
                            .text("Op"),
                    ),
                ),
        );
    let clipped = |index: usize, id: &str, background: Style| -> Node {
        d(cell(
            index,
            &background.clip_path(ClipPath::Url(format!("#{id}"))),
        ))
        .into()
    };
    stage(vec![
        svg(defs, Style::default().position(Position::Absolute)).into(),
        clipped(
            0,
            "case-star",
            Style::default().background(vec![layer(linear(
                180.0,
                vec![stop(hex(0xfa_cc15)), stop(hex(0xf9_7316))],
            ))]),
        ),
        clipped(
            1,
            "case-blobs",
            Style::default().background(vec![layer(linear(
                90.0,
                vec![stop(hex(0x0e_a5e9)), stop(hex(0x7c_3aed))],
            ))]),
        ),
        clipped(
            2,
            "case-text",
            Style::default().background(vec![layer(linear(
                180.0,
                vec![stop(hex(0x22_c55e)), stop(hex(0x0e_a5e9))],
            ))]),
        ),
        clipped(
            3,
            "case-star",
            Style::default()
                .background_color(hex(0xf8_fafc))
                .transform(vec![
                    Transform::Rotate(Angle::Deg(Num(20.0))),
                    Transform::Scale(Num(0.8), Num(0.8), Num(1.0)),
                ]),
        ),
    ])
}

fn mask(layer: ImageLayer) -> MaskLayer {
    MaskLayer {
        layer: ImageLayer {
            origin: BoxArea::BorderBox,
            ..layer
        },
        mode: MaskMode::Alpha,
    }
}

pub fn mask_image() -> Node {
    let black = || hex(0x00_0000);
    let masks: Vec<MaskLayer> = vec![
        mask(layer(linear(
            180.0,
            vec![stop(black()), stop(transparent())],
        ))),
        mask(layer(linear(
            90.0,
            vec![
                stop(transparent()),
                stop_span(black(), pct(30.0), pct(70.0)),
                stop(transparent()),
            ],
        ))),
        mask(layer(radial(
            RadialShape::Circle,
            RadialSize::FarthestCorner,
            at(50.0, 50.0),
            vec![
                stop_at(black(), pct(40.0)),
                stop_at(transparent(), pct(70.0)),
            ],
        ))),
        mask(ImageLayer {
            size: LayerSize::Explicit(px(12.0), px(12.0)),
            ..layer(radial(
                RadialShape::Circle,
                RadialSize::FarthestCorner,
                at(50.0, 50.0),
                vec![stop_at(black(), px(3.0)), stop_at(transparent(), px(3.5))],
            ))
        }),
        mask(ImageLayer {
            size: LayerSize::Explicit(pct(50.0), pct(50.0)),
            repeat: (Repeat::NoRepeat, Repeat::NoRepeat),
            position: at(100.0, 100.0),
            ..layer(linear(180.0, vec![stop(black()), stop(black())]))
        }),
        mask(layer(repeating(linear(
            90.0,
            vec![
                stop_span(black(), Length::ZERO, px(6.0)),
                stop_span(transparent(), px(6.0), px(12.0)),
            ],
        )))),
        mask(ImageLayer {
            size: LayerSize::Contain,
            repeat: (Repeat::NoRepeat, Repeat::NoRepeat),
            position: at(50.0, 50.0),
            ..ImageLayer::of(Image::Url(AssetRef(LOGO.to_owned())))
        }),
        mask(layer(linear(
            135.0,
            vec![stop_at(black(), pct(50.0)), stop(rgba_of(0, 0, 0, 0.2))],
        ))),
    ];
    stage(
        masks
            .into_iter()
            .enumerate()
            .map(|(i, m)| {
                d(cell(
                    i,
                    &Style::default().background(vec![brand()]).mask(vec![m]),
                ))
                .into()
            })
            .collect(),
    )
}
