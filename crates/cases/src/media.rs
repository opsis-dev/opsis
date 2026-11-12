use opsis_scene::node::{Node, img};
use opsis_scene::paint::Filter;
use opsis_scene::style::{Align, Display, ImageRendering, ObjectFit, Overflow, Style};
use opsis_scene::values::{Angle, Length, Num};

use crate::{cell, d, pct, slate, stage};

const PHOTO: &str = "app:placeholders/launch-earth.jpg";
const LOGO: &str = "app:placeholders/logo-mark.svg";
const WIDE: &str = "app:placeholders/screen-wide.svg";

fn bg() -> Style {
    Style::default().background_color(slate())
}

fn centred() -> Style {
    bg().display(Display::Grid)
        .align_items(Align::Center)
        .justify_items(Align::Center)
}

fn filled(fit: ObjectFit) -> Style {
    Style::default()
        .width(pct(100.0))
        .height(pct(100.0))
        .object_fit(fit)
}

pub fn img_case() -> Node {
    stage(vec![
        d(cell(0, &bg()))
            .child(img(PHOTO, Style::default().width(pct(100.0))))
            .into(),
        d(cell(1, &centred()))
            .child(img(LOGO, Style::default().width(80.0).height(80.0)))
            .into(),
        d(cell(2, &bg()))
            .child(img(WIDE, filled(ObjectFit::Contain)))
            .into(),
        d(cell(
            3,
            &bg().overflow(Overflow::Hidden).border_radius_all(20.0),
        ))
        .child(img(PHOTO, filled(ObjectFit::Cover).max_width(Length::Auto)))
        .into(),
        d(cell(4, &bg()))
            .child(img(
                PHOTO,
                Style::default()
                    .width(40.0)
                    .height(40.0)
                    .object_fit(ObjectFit::Cover)
                    .image_rendering(ImageRendering::Pixelated),
            ))
            .child(img(
                PHOTO,
                Style::default().width(300.0).max_width(pct(100.0)),
            ))
            .into(),
        d(cell(5, &centred()))
            .child(img(
                LOGO,
                Style::default()
                    .width(120.0)
                    .height(120.0)
                    .opacity(0.5)
                    .rotate(Angle::Deg(Num(15.0))),
            ))
            .into(),
        d(cell(6, &bg()))
            .child(img(
                PHOTO,
                filled(ObjectFit::Cover)
                    .max_width(Length::Auto)
                    .filter(vec![
                        Filter::Blur(Length::px(3.0)),
                        Filter::Brightness(Num(0.8)),
                    ])
                    .scale(1.2, 1.2),
            ))
            .into(),
        d(cell(7, &bg()))
            .child(img(
                PHOTO,
                Style::default()
                    .width(120.0)
                    .height(600.0)
                    .max_width(Length::Auto),
            ))
            .into(),
    ])
}

fn noise() -> Vec<u8> {
    const SEED: u32 = 12_345;
    const MULTIPLIER: u32 = 48_271;
    const MODULUS: u32 = 2_147_483_647;
    let mut state = SEED;
    let mut pixels = Vec::with_capacity(160 * 90 * 4);
    for _ in 0..160 * 90 {
        state = state.wrapping_mul(MULTIPLIER) % MODULUS;
        let grey = u8::try_from(u64::from(state) * 255 / u64::from(MODULUS)).unwrap_or(u8::MAX);
        pixels.extend_from_slice(&[grey, grey, grey, 255]);
    }
    pixels
}

pub fn canvas_case() -> Node {
    use opsis_scene::drawing::{Baseline, Cap, Command, Composite, Drawing, Fill, Font};
    use opsis_scene::node::drawing;
    use opsis_scene::style::Position;

    use crate::paints::{hex, rgba_of};

    let n = Num;
    let fill = |color| Command::FillStyle(Fill::Color(color));
    let stroke = |color| Command::StrokeStyle(Fill::Color(color));
    let font = |family: &str, size: f64, weight: f64| {
        Command::Font(Font {
            families: vec![family.to_owned()],
            size: n(size),
            weight: n(weight),
            italic: false,
        })
    };
    let commands = vec![
        Command::ClearRect([n(0.0), n(0.0), n(592.0), n(312.0)]),
        fill(hex(0xf9_7316)),
        Command::FillRect([n(10.0), n(10.0), n(120.0), n(80.0)]),
        stroke(hex(0xf8_fafc)),
        Command::LineWidth(n(4.0)),
        Command::StrokeRect([n(150.0), n(10.0), n(120.0), n(80.0)]),
        Command::BeginPath,
        Command::Arc {
            center: (n(350.0), n(50.0)),
            radius: n(40.0),
            start: n(0.0),
            end: n(std::f64::consts::PI * 1.5),
            anticlockwise: false,
        },
        Command::ClosePath,
        fill(hex(0x0e_a5e9)),
        Command::Fill,
        Command::BeginPath,
        Command::MoveTo(n(410.0), n(90.0)),
        Command::BezierTo([n(440.0), n(0.0), n(500.0), n(0.0), n(520.0), n(90.0)]),
        Command::QuadraticTo(n(550.0), n(40.0), n(580.0), n(90.0)),
        Command::LineWidth(n(3.0)),
        stroke(hex(0x22_c55e)),
        Command::LineCap(Cap::Round),
        Command::Stroke,
        Command::FillStyle(Fill::Linear {
            from: (n(10.0), n(0.0)),
            to: (n(190.0), n(0.0)),
            stops: vec![(n(0.0), hex(0x7c_3aed)), (n(1.0), hex(0xf4_3f5e))],
        }),
        Command::FillRect([n(10.0), n(110.0), n(180.0), n(80.0)]),
        Command::FillStyle(Fill::Radial {
            from: (n(290.0), n(150.0), n(5.0)),
            to: (n(290.0), n(150.0), n(60.0)),
            stops: vec![
                (n(0.0), hex(0xfa_cc15)),
                (n(1.0), rgba_of(250, 204, 21, 0.0)),
            ],
        }),
        Command::FillRect([n(210.0), n(110.0), n(160.0), n(80.0)]),
        Command::Save,
        Command::GlobalAlpha(n(0.6)),
        Command::Composite(Composite::Screen),
        Command::Translate(n(470.0), n(150.0)),
        Command::Rotate(n(0.3)),
        fill(hex(0x0e_a5e9)),
        Command::FillRect([n(-60.0), n(-30.0), n(120.0), n(60.0)]),
        Command::Restore,
        font("Inter", 32.0, 700.0),
        fill(hex(0xf8_fafc)),
        Command::TextBaseline(Baseline::Alphabetic),
        Command::FillText {
            text: "Canvas text".to_owned(),
            at: (n(10.0), n(240.0)),
        },
        font("JetBrains Mono", 18.0, 400.0),
        fill(hex(0x94_a3b8)),
        Command::FillText {
            text: "width 119".to_owned(),
            at: (n(10.0), n(270.0)),
        },
        Command::Shadow {
            color: rgba_of(0, 0, 0, 0.8),
            blur: n(8.0),
            x: n(0.0),
            y: n(4.0),
        },
        fill(hex(0xf9_7316)),
        font("Inter", 28.0, 800.0),
        Command::FillText {
            text: "Shadowed".to_owned(),
            at: (n(220.0), n(250.0)),
        },
        Command::Shadow {
            color: opsis_scene::color::Color::TRANSPARENT,
            blur: n(8.0),
            x: n(0.0),
            y: n(4.0),
        },
        Command::PutImageData {
            at: (420, 210),
            width: 160,
            height: 90,
            pixels: noise(),
        },
    ];
    stage(vec![
        drawing(
            Drawing {
                width: 592,
                height: 312,
                commands,
            },
            Style::default()
                .position(Position::Absolute)
                .left(24.0)
                .top(24.0),
        )
        .into(),
    ])
}
