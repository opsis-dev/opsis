use opsis_compile::mix::eq::{RANGE_DB, coefficients, curve, highpass_db, response_db};
use opsis_types::mix::{EqBand, EqType};
use proptest::prelude::*;

const RATE: f64 = 48_000.0;

fn kind() -> impl Strategy<Value = EqType> {
    proptest::sample::select(vec![EqType::Lowshelf, EqType::Peak, EqType::Highshelf])
}

proptest! {
    #[test]
    fn a_flat_band_changes_nothing(kind in kind(), freq in 40f64..15_000.0, q in 0.3f64..8.0, at in 30f64..18_000.0) {
        let band = EqBand { kind, freq, gain: 0.0, q };
        prop_assert!(response_db(&coefficients(&band, RATE), at, RATE).abs() < 1e-6);
    }

    #[test]
    fn a_bell_reaches_its_gain_at_its_frequency(freq in 100f64..10_000.0, gain in -18f64..18.0, q in 0.5f64..4.0) {
        let band = EqBand { kind: EqType::Peak, freq, gain, q };
        prop_assert!((response_db(&coefficients(&band, RATE), freq, RATE) - gain).abs() < 0.05);
    }

    #[test]
    fn a_cut_mirrors_a_boost(kind in kind(), freq in 60f64..12_000.0, gain in 0.5f64..18.0, q in 0.4f64..4.0, at in 30f64..18_000.0) {
        let boost = response_db(&coefficients(&EqBand { kind, freq, gain, q }, RATE), at, RATE);
        let cut = response_db(&coefficients(&EqBand { kind, freq, gain: -gain, q }, RATE), at, RATE);
        prop_assert!((boost + cut).abs() < 1e-6);
    }

    #[test]
    fn the_low_cut_only_lowers_and_fades_above_its_cutoff(cutoff in 20f64..400.0, at in 20f64..20_000.0) {
        let db = highpass_db(cutoff, at);
        prop_assert!(db <= 0.0);
        if at > cutoff * 10.0 {
            prop_assert!(db > -0.01);
        }
    }

    #[test]
    fn the_curve_stays_in_its_range(gain in -40f64..40.0, steps in 1u32..200) {
        let bands = vec![EqBand { kind: EqType::Peak, freq: 1000.0, gain, q: 1.0 }];
        let points = curve(&bands, true, Some(80.0), steps);
        prop_assert_eq!(points.len(), usize::try_from(steps).unwrap_or(0) + 1);
        prop_assert!(points.iter().all(|db| db.abs() <= RANGE_DB));
    }
}

#[test]
fn an_eq_that_is_off_is_flat() {
    let bands = vec![EqBand {
        kind: EqType::Peak,
        freq: 1000.0,
        gain: 12.0,
        q: 1.0,
    }];
    assert!(
        curve(&bands, false, None, 32)
            .iter()
            .all(|db| db.abs() < 1e-9)
    );
}
