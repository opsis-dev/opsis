use opsis_paint::color::{interpolate, resolve};
use opsis_scene::color::{Color, ColorSpace, HueMethod, Interpolation};
use opsis_scene::values::{Alpha, Num};
use proptest::prelude::*;

const SPACES: [ColorSpace; 10] = [
    ColorSpace::Srgb,
    ColorSpace::SrgbLinear,
    ColorSpace::DisplayP3,
    ColorSpace::Hsl,
    ColorSpace::Hwb,
    ColorSpace::Lab,
    ColorSpace::Lch,
    ColorSpace::Oklab,
    ColorSpace::Oklch,
    ColorSpace::XyzD65,
];

fn close(a: [f32; 4], b: [f32; 4], slack: f32) -> bool {
    a.iter().zip(&b).all(|(x, y)| (x - y).abs() <= slack)
}

fn srgb(rgb: [f64; 3]) -> Color {
    Color::Components {
        space: ColorSpace::Srgb,
        components: rgb.map(Num),
        alpha: Alpha(1.0),
    }
}

fn channels() -> impl Strategy<Value = [f64; 3]> {
    [0.0..=1.0_f64, 0.0..=1.0_f64, 0.0..=1.0_f64]
}

proptest! {
    #[test]
    fn interpolation_ends_on_its_two_colours(a in channels(), b in channels(), space in 0..SPACES.len()) {
        let first = resolve(&srgb(a), &Color::BLACK);
        let second = resolve(&srgb(b), &Color::BLACK);
        for (t, want) in [(0.0, first), (1.0, second)] {
            let got = interpolate(SPACES[space], HueMethod::Shorter, (first, second), t);
            prop_assert!(close(got.srgb(), want.srgb(), 1e-3), "{:?} at {t}", SPACES[space]);
        }
    }

    #[test]
    fn mixing_in_shares_below_a_hundred_lowers_the_alpha(a in channels(), share in 1.0..99.0_f64) {
        let mixed = Color::mix(
            Interpolation::default(),
            (srgb(a), Some(share / 2.0)),
            (srgb(a), Some(share / 2.0)),
        );
        let got = resolve(&mixed, &Color::BLACK);
        let want = resolve(&srgb(a), &Color::BLACK).srgb();
        prop_assert!((f64::from(got.alpha()) - share / 100.0).abs() < 1e-5);
        prop_assert!(close([got.srgb()[0], got.srgb()[1], got.srgb()[2], want[3]], want, 1e-4));
    }

    #[test]
    fn a_missing_share_takes_the_rest_of_a_hundred(a in channels(), b in channels(), share in 0.0..100.0_f64) {
        let one = Color::mix(Interpolation::default(), (srgb(a), Some(share)), (srgb(b), None));
        let both = Color::mix(Interpolation::default(), (srgb(a), Some(share)), (srgb(b), Some(100.0 - share)));
        prop_assert!(close(resolve(&one, &Color::BLACK).srgb(), resolve(&both, &Color::BLACK).srgb(), 1e-6));
    }
}
