use opsis_scene::color::{Color, ColorSpace, HueMethod, Interpolation};
use opsis_scene::node::Node;
use opsis_scene::style::{ContainerType, Display, GridLine, LineHeight, Position, Style, Track};
use opsis_scene::values::{ContainerUnit, Edges, Length, Num};

use crate::paints::{colour, hex, interpolated, layer, linear, space, stop, stop_at};
use crate::{cell, colored, cool, d, leaf, pct, rose, slate, stage, warm};

fn bg() -> Style {
    Style::default().background_color(slate())
}

const fn px(value: f64) -> Length {
    Length::px(value)
}

fn sized(width: Length, height: Length) -> Style {
    Style::default().width(width).height(height)
}

const fn cq(unit: ContainerUnit, value: f64) -> Length {
    Length::Container(unit, Num(value))
}

fn calc_shape() -> Node {
    stage(vec![
        d(cell(0, &bg()))
            .child(d(sized(
                Length::sum([pct(100.0), px(-30.0)]),
                Length::sum([pct(50.0), px(10.0)]),
            )))
            .into(),
        d(cell(1, &bg()))
            .child(d(Style::default()
                .position(Position::Absolute)
                .left(Length::sum([pct(50.0), px(-20.0)]))
                .top(Length::sum([pct(25.0).scaled(2.0), px(-20.0)]))
                .width(40.0)
                .height(40.0)))
            .into(),
        d(cell(2, &bg().font_size(20.0)))
            .child(d(sized(
                Length::sum([Length::em(3.0), px(10.0)]),
                Length::em(1.0).scaled(2.0),
            )))
            .into(),
        d(cell(3, &bg()))
            .child(d(Style::default()
                .margin(Edges::all(Length::sum([px(10.0), pct(5.0)])))
                .height(Length::sum([
                    pct(100.0),
                    Length::sum([px(10.0), pct(5.0)]).scaled(-2.0),
                ]))))
            .into(),
        d(cell(4, &bg()))
            .child(d(sized(
                Length::clamp(px(20.0), pct(50.0), px(60.0)),
                px(40.0),
            )))
            .child(d(sized(Length::min([px(200.0), pct(80.0)]), px(40.0))))
            .child(d(sized(Length::max([px(10.0), pct(30.0)]), px(40.0))))
            .into(),
        d(cell(5, &bg()))
            .child(d(sized(px(50.0), px(50.0)).translate(
                Length::sum([pct(100.0), px(4.0)]),
                Length::sum([pct(50.0), px(-10.0)]),
            )))
            .into(),
    ])
}

fn grid(columns: usize) -> Style {
    Style::default()
        .position(Position::Absolute)
        .inset(px(24.0))
        .display(Display::Grid)
        .grid_template_columns(vec![Track::Repeat(
            opsis_scene::style::RepeatCount::Times(u16::try_from(columns).expect("a column count")),
            vec![Track::Fraction(Num(1.0))],
        )])
        .gap(4.0)
}

fn container_units_shape() -> Node {
    let size = |index: usize, extra: Style| {
        cell(
            index,
            &bg().container_type(ContainerType::Size).merged(&extra),
        )
    };
    stage(vec![
        d(size(0, Style::default()))
            .child(d(sized(
                cq(ContainerUnit::Cqw, 50.0),
                cq(ContainerUnit::Cqh, 25.0),
            )))
            .into(),
        d(size(1, Style::default()))
            .child(
                d(Style::default()
                    .font_size(cq(ContainerUnit::Cqw, 20.0))
                    .font_weight(800.0)
                    .line_height(LineHeight::Number(Num(1.0))))
                .child("Fit"),
            )
            .into(),
        d(size(2, Style::default()))
            .child(d(sized(
                cq(ContainerUnit::Cqmin, 40.0),
                cq(ContainerUnit::Cqmax, 40.0),
            )))
            .into(),
        d(cell(
            3,
            &bg()
                .container_type(ContainerType::InlineSize)
                .height(Length::Auto),
        ))
        .child(
            d(Style::default()
                .padding(Edges::all(cq(ContainerUnit::Cqi, 5.0)))
                .font_size(cq(ContainerUnit::Cqi, 12.0)))
            .child("inline"),
        )
        .into(),
        d(size(4, Style::default().width(280.0).height(148.0)))
            .child(
                d(Style::default()
                    .font_size(cq(ContainerUnit::Cqh, 15.0))
                    .font_weight(700.0)
                    .margin(Edges {
                        left: cq(ContainerUnit::Cqw, 10.0),
                        ..Edges::all(Length::ZERO)
                    }))
                .child("Wide box"),
            )
            .into(),
    ])
}

fn relative_units_shape() -> Node {
    let block = |w: Length, h: Length| -> Node { d(sized(w, h)).into() };
    stage(vec![
        d(cell(0, &bg().font_size(10.0)))
            .child(block(Length::em(5.0), Length::em(3.0)))
            .into(),
        d(cell(1, &bg().font_size(20.0)))
            .child(block(Length::em(5.0), Length::em(3.0)))
            .into(),
        d(cell(2, &bg().font_size(20.0)))
            .child(
                d(Style::default().font_size(Length::em(0.5)))
                    .child(block(Length::em(5.0), Length::em(3.0))),
            )
            .into(),
        d(cell(3, &bg()))
            .child(block(Length::Rem(Num(5.0)), Length::Rem(Num(2.0))))
            .into(),
        d(cell(4, &bg()))
            .child(block(Length::Vw(Num(25.0)), Length::Vh(Num(10.0))))
            .into(),
        d(cell(5, &bg().font_size(24.0)))
            .child(
                d(Style::default()
                    .letter_spacing(Length::em(0.5))
                    .line_height(LineHeight::Length(Length::em(2.0)))
                    .padding(Edges::all(Length::em(0.5))))
                .child("em"),
            )
            .into(),
        d(cell(6, &bg()))
            .child(
                d(sized(pct(75.0), pct(50.0)).padding(Edges::all(pct(10.0))))
                    .child(block(pct(50.0), pct(100.0))),
            )
            .into(),
        d(cell(7, &bg().font_size(18.0)))
            .child(
                d(Style::default()
                    .width(Length::Ch(Num(4.0)))
                    .font_family(vec!["JetBrains Mono".to_owned()]))
                .child("abcd"),
            )
            .child(block(Length::Ex(Num(4.0)), Length::Lh(Num(2.0))))
            .into(),
    ])
}

fn swatch(min_height: f64, style: Style) -> Node {
    d(style.min_height(min_height)).into()
}

fn mixed(space: ColorSpace, hue: HueMethod, (a, share): (Color, f64), b: Color) -> Color {
    Color::mix(Interpolation { space, hue }, (a, Some(share)), (b, None))
}

pub fn color_mix() -> Node {
    let shares = [0.0, 10.0, 25.0, 50.0, 75.0, 90.0, 100.0];
    let spaces = [
        ColorSpace::Oklch,
        ColorSpace::Oklab,
        ColorSpace::Srgb,
        ColorSpace::SrgbLinear,
    ];
    let mut cells: Vec<Node> = Vec::new();
    for space in spaces {
        for share in shares {
            cells.push(swatch(
                44.0,
                Style::default().background_color(mixed(
                    space,
                    HueMethod::Shorter,
                    (hex(0xf9_7316), share),
                    hex(0x0e_a5e9),
                )),
            ));
        }
    }
    for share in shares {
        cells.push(swatch(
            44.0,
            Style::default().background_color(mixed(
                ColorSpace::Oklch,
                HueMethod::Shorter,
                (hex(0xf8_fafc), share),
                Color::TRANSPARENT,
            )),
        ));
    }
    for share in shares {
        cells.push(swatch(
            44.0,
            Style::default().background_color(mixed(
                ColorSpace::Oklch,
                HueMethod::Longer,
                (colour(ColorSpace::Oklch, [0.7, 0.2, 20.0], 1.0), share),
                colour(ColorSpace::Oklch, [0.7, 0.2, 200.0], 1.0),
            )),
        ));
    }
    stage(vec![d(grid(7)).children(cells).into()])
}

pub fn oklab() -> Node {
    let stops = [-0.3, -0.15, 0.0, 0.15, 0.3];
    let mut cells: Vec<Node> = Vec::new();
    for a in stops {
        for b in stops {
            cells.push(swatch(
                40.0,
                Style::default().background_color(colour(ColorSpace::Oklab, [0.7, a, b], 1.0)),
            ));
        }
    }
    for within in [ColorSpace::Oklab, ColorSpace::Srgb] {
        cells.push(
            d(Style::default()
                .grid_column((GridLine::Line(1), GridLine::Line(-1)))
                .min_height(30.0)
                .background(vec![layer(interpolated(
                    linear(90.0, vec![stop(hex(0x00_00ff)), stop(hex(0xff_ff00))]),
                    space(within),
                ))]))
            .into(),
        );
    }
    stage(vec![d(grid(5)).children(cells).into()])
}

pub fn oklch() -> Node {
    let lightness = [0.3, 0.45, 0.6, 0.75, 0.9];
    let hues = [0.0, 45.0, 90.0, 150.0, 210.0, 260.0, 320.0];
    let mut cells: Vec<Node> = Vec::new();
    for l in lightness {
        for h in hues {
            cells.push(swatch(
                40.0,
                Style::default().background_color(colour(ColorSpace::Oklch, [l, 0.2, h], 1.0)),
            ));
        }
    }
    let row = |stops| {
        d(Style::default()
            .grid_column((GridLine::Line(1), GridLine::Line(-1)))
            .background(vec![layer(linear(90.0, stops))]))
        .into()
    };
    cells.push(row(vec![
        stop(colour(ColorSpace::Oklch, [0.7, 0.37, 30.0], 1.0)),
        stop_at(
            colour(ColorSpace::Oklch, [0.7, 0.37, 150.0], 1.0),
            pct(50.0),
        ),
        stop(colour(ColorSpace::Oklch, [0.7, 0.37, 270.0], 1.0)),
    ]));
    cells.push(row(vec![
        stop(colour(ColorSpace::Oklch, [0.7, 0.1, 30.0], 0.2)),
        stop(colour(ColorSpace::Oklch, [0.95, 0.05, 90.0], 1.0)),
    ]));
    stage(vec![d(grid(7)).children(cells).into()])
}

pub fn calc() -> Node {
    colored(
        calc_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 1, 0], cool()),
            (&[0, 2, 0], leaf()),
            (&[0, 3, 0], rose()),
            (&[0, 4, 0], warm()),
            (&[0, 4, 1], cool()),
            (&[0, 4, 2], leaf()),
            (&[0, 5, 0], rose()),
        ],
    )
}

pub fn container_units() -> Node {
    colored(
        container_units_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 2, 0], cool()),
            (&[0, 3, 0], leaf()),
        ],
    )
}

pub fn relative_units() -> Node {
    colored(
        relative_units_shape(),
        &[
            (&[0, 0, 0], warm()),
            (&[0, 1, 0], cool()),
            (&[0, 2, 0, 0], leaf()),
            (&[0, 3, 0], rose()),
            (&[0, 4, 0], warm()),
            (&[0, 5, 0], cool()),
            (&[0, 6, 0], leaf()),
            (&[0, 6, 0, 0], rose()),
            (&[0, 7, 0], warm()),
            (&[0, 7, 1], cool()),
        ],
    )
}
