use opsis_scene::color::{Color, ColorSpace, HueMethod, Interpolation};
use opsis_scene::css::{background, gradient, length};
use opsis_scene::paint::{ColorStop, Gradient, Image, LinearDirection, RadialShape, RadialSize};
use opsis_scene::values::{Angle, Length, Num, Position2};
use proptest::prelude::*;

fn length_text(length: &Length) -> String {
    match length {
        Length::Px(p) => format!("{}px", p.0),
        Length::Percent(Num(n)) => format!("{n}%"),
        Length::Em(Num(n)) => format!("{n}em"),
        Length::Rem(Num(n)) => format!("{n}rem"),
        Length::Vw(Num(n)) => format!("{n}vw"),
        Length::Vh(Num(n)) => format!("{n}vh"),
        other => panic!("not printed: {other:?}"),
    }
}

fn color_text(color: &Color) -> String {
    let Color::Components {
        space,
        components,
        alpha,
    } = color
    else {
        panic!("not printed: {color:?}");
    };
    let [a, b, c] = components.map(|n| n.0);
    match space {
        ColorSpace::Srgb => format!("color(srgb {a} {b} {c} / {})", alpha.0),
        ColorSpace::Oklch => format!("oklch({a} {b} {c} / {})", alpha.0),
        other => panic!("not printed: {other:?}"),
    }
}

fn stops_text(stops: &[ColorStop]) -> String {
    stops
        .iter()
        .map(|s| {
            let mut out = color_text(&s.color);
            for at in s.at.iter().chain(s.until.iter()) {
                out.push(' ');
                out.push_str(&length_text(at));
            }
            out
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn interpolation_text(interpolation: Interpolation) -> String {
    if interpolation == Interpolation::DEFAULT {
        return String::new();
    }
    let space = match interpolation.space {
        ColorSpace::Oklch => "oklch",
        ColorSpace::Oklab => "oklab",
        _ => "srgb",
    };
    let hue = match interpolation.hue {
        HueMethod::Shorter => "",
        HueMethod::Longer => " longer hue",
        HueMethod::Increasing => " increasing hue",
        HueMethod::Decreasing => " decreasing hue",
    };
    format!(" in {space}{hue}")
}

fn position_text(position: &Position2) -> String {
    format!("{} {}", length_text(&position.x), length_text(&position.y))
}

fn gradient_text(gradient: &Gradient) -> String {
    match gradient {
        Gradient::Linear {
            direction: LinearDirection::Angle(angle),
            stops,
            repeating,
            interpolation,
        } => format!(
            "{}linear-gradient({}deg{}, {})",
            if *repeating { "repeating-" } else { "" },
            angle.degrees(),
            interpolation_text(*interpolation),
            stops_text(stops)
        ),
        Gradient::Radial {
            shape,
            center,
            stops,
            repeating,
            interpolation,
            ..
        } => format!(
            "{}radial-gradient({} at {}{}, {})",
            if *repeating { "repeating-" } else { "" },
            if *shape == RadialShape::Circle {
                "circle"
            } else {
                "ellipse"
            },
            position_text(center),
            interpolation_text(*interpolation),
            stops_text(stops)
        ),
        Gradient::Conic {
            from,
            center,
            stops,
            repeating,
            interpolation,
        } => format!(
            "{}conic-gradient(from {}deg at {}{}, {})",
            if *repeating { "repeating-" } else { "" },
            from.degrees(),
            position_text(center),
            interpolation_text(*interpolation),
            stops_text(stops)
        ),
        other @ Gradient::Linear { .. } => panic!("not printed: {other:?}"),
    }
}

fn unit() -> impl Strategy<Value = f64> {
    (0u32..=100).prop_map(|n| f64::from(n) / 100.0)
}

fn colors() -> impl Strategy<Value = Color> {
    prop_oneof![
        (unit(), unit(), unit(), unit()).prop_map(|(r, g, b, a)| Color::srgb_alpha(r, g, b, a)),
        (unit(), unit(), 0u32..360, unit()).prop_map(|(l, c, h, a)| Color::oklch(
            l,
            c / 2.0,
            f64::from(h),
            a
        )),
    ]
}

fn lengths() -> impl Strategy<Value = Length> {
    prop_oneof![
        (0u32..400).prop_map(|n| Length::px(f64::from(n))),
        (0u32..100).prop_map(|n| Length::percent(f64::from(n))),
        (0u32..10).prop_map(|n| Length::em(f64::from(n))),
    ]
}

fn stops() -> impl Strategy<Value = Vec<ColorStop>> {
    prop::collection::vec(
        (
            colors(),
            prop::option::of((lengths(), prop::option::of(lengths()))),
        )
            .prop_map(|(color, at)| ColorStop {
                color,
                until: at.as_ref().and_then(|(_, u)| u.clone()),
                at: at.map(|(a, _)| a),
            }),
        2..6,
    )
}

fn interpolations() -> impl Strategy<Value = Interpolation> {
    prop_oneof![
        Just(Interpolation::DEFAULT),
        Just(Interpolation {
            space: ColorSpace::Oklch,
            hue: HueMethod::Longer,
        }),
        Just(Interpolation {
            space: ColorSpace::Oklab,
            hue: HueMethod::Shorter,
        }),
    ]
}

fn centers() -> impl Strategy<Value = Position2> {
    (lengths(), lengths()).prop_map(|(x, y)| Position2 { x, y })
}

fn gradients() -> impl Strategy<Value = Gradient> {
    prop_oneof![
        (0u32..360, stops(), any::<bool>(), interpolations()).prop_map(
            |(deg, stops, repeating, interpolation)| Gradient::Linear {
                direction: LinearDirection::Angle(Angle::deg(f64::from(deg))),
                stops,
                repeating,
                interpolation,
            }
        ),
        (
            any::<bool>(),
            centers(),
            stops(),
            any::<bool>(),
            interpolations()
        )
            .prop_map(|(circle, center, stops, repeating, interpolation)| {
                Gradient::Radial {
                    shape: if circle {
                        RadialShape::Circle
                    } else {
                        RadialShape::Ellipse
                    },
                    size: RadialSize::FarthestCorner,
                    center,
                    stops,
                    repeating,
                    interpolation,
                }
            }),
        (
            0u32..360,
            centers(),
            stops(),
            any::<bool>(),
            interpolations()
        )
            .prop_map(
                |(deg, center, stops, repeating, interpolation)| Gradient::Conic {
                    from: Angle::deg(f64::from(deg)),
                    center,
                    stops,
                    repeating,
                    interpolation,
                }
            ),
    ]
}

proptest! {
    #[test]
    fn a_printed_gradient_reads_back_the_same(g in gradients()) {
        let text = gradient_text(&g);
        let parsed = gradient(&text);
        prop_assert!(parsed.is_some(), "unread: {}", text);
        prop_assert_eq!(gradient_text(&parsed.expect("a gradient")), text);
    }

    #[test]
    fn a_printed_length_reads_back_the_same(l in lengths()) {
        prop_assert_eq!(length(&length_text(&l)), Some(l));
    }

    #[test]
    fn a_background_keeps_every_layer_in_order(list in prop::collection::vec(gradients(), 1..4), color in colors()) {
        let text = list.iter().map(gradient_text).chain([color_text(&color)]).collect::<Vec<_>>().join(", ");
        let parsed = background(&text).expect("a background");
        prop_assert_eq!(parsed.layers.len(), list.len());
        prop_assert!(parsed.color.is_some());
        for (layer, want) in parsed.layers.iter().zip(&list) {
            let Image::Gradient(got) = &layer.image else {
                return Err(TestCaseError::fail("not a gradient"));
            };
            prop_assert_eq!(gradient_text(got), gradient_text(want));
        }
    }
}
