use opsis_compile::mix::controls::{
    KnobRange, START_DEG, SWEEP_DEG, angle_of, db_to_gain, format_db, format_hz, format_level_db,
    format_ms, format_pan, fraction_to_value, level_at, sample_at, share, snap_value,
    value_to_fraction,
};
use opsis_types::numbers::exact_eq;
use proptest::prelude::*;

fn range() -> impl Strategy<Value = KnobRange> {
    (
        1f64..100.0,
        1f64..1000.0,
        proptest::sample::select(vec![0.01, 0.1, 0.5, 1.0]),
        any::<bool>(),
    )
        .prop_map(|(min, width, step, log)| KnobRange {
            min,
            max: min + width,
            step,
            log,
        })
}

proptest! {
    #[test]
    fn knob_values_stay_in_range_and_on_their_step(r in range(), fraction in -1f64..2.0) {
        let value = fraction_to_value(r, fraction);
        prop_assert!(value >= r.min - 1e-9 && value <= r.max + 1e-9);
        let snapped = snap_value(r, value);
        prop_assert!((snapped - value).abs() < r.step / 2.0 + 1e-9);
    }

    #[test]
    fn a_knob_round_trips_through_its_fraction(r in range(), fraction in 0f64..1.0) {
        let value = fraction_to_value(r, fraction);
        let back = value_to_fraction(r, value);
        let again = fraction_to_value(r, back);
        prop_assert!((again - value).abs() <= r.step + 1e-9);
    }

    #[test]
    fn knob_angles_sweep_from_the_start(fraction in 0f64..1.0) {
        let angle = angle_of(fraction);
        prop_assert!((START_DEG..=START_DEG + SWEEP_DEG).contains(&angle));
    }

    #[test]
    fn samples_come_from_inside_the_list(values in proptest::collection::vec(-100f64..0.0, 0..20), frame in -10f64..40.0) {
        let sampled = sample_at(&values, frame);
        prop_assert_eq!(sampled.is_some(), !values.is_empty());
        if let Some(v) = sampled {
            prop_assert!(values.contains(&v));
        }
    }

    #[test]
    fn a_channel_share_is_a_fraction(master in -150f64..0.0, channel in -150f64..0.0) {
        let s = share(&[master], &[channel], 0.0);
        prop_assert!(s > 0.0 && s <= 1.0);
    }

    #[test]
    fn the_held_peak_is_at_least_the_peak(peaks in proptest::collection::vec(-100f64..0.0, 1..30), frame in 0f64..40.0, hold in 0usize..10) {
        let level = level_at(&peaks, &peaks, frame, hold).expect("a level");
        prop_assert!(level.hold >= level.peak);
    }

    #[test]
    fn louder_faders_give_more_gain(a in -59f64..12.0, b in -59f64..12.0) {
        let (low, high) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(db_to_gain(low) <= db_to_gain(high));
    }
}

#[test]
fn channel_shares_match_the_master() {
    assert!((share(&[-10.0, -10.0], &[-10.0, -20.0], 0.0) - 1.0).abs() < 1e-9);
    assert!((share(&[-10.0, -10.0], &[-10.0, -20.0], 1.0) - 0.1).abs() < 1e-9);
    assert!((share(&[], &[-10.0], 0.0) - 1.0).abs() < f64::EPSILON);
    assert!((share(&[-150.0], &[-10.0], 0.0) - 1.0).abs() < f64::EPSILON);
    assert!(exact_eq(db_to_gain(-60.0), 0.0));
}

#[test]
fn readings_read_like_the_mixer() {
    assert_eq!(format_db(-70.0), "\u{2212}\u{221e}");
    assert_eq!(format_db(3.0), "+3.0");
    assert_eq!(format_db(-6.04), "\u{2212}6.0");
    assert_eq!(format_db(0.0), "0.0");
    assert_eq!(format_level_db(-90.0), "\u{2212}90.0");
    assert_eq!(format_hz(80.0), "80");
    assert_eq!(format_hz(2500.0), "2.5k");
    assert_eq!(format_hz(12_000.0), "12k");
    assert_eq!(format_ms(12.34), "12.3");
    assert_eq!(format_ms(250.0), "250");
    assert_eq!(format_pan(0.0), "C");
    assert_eq!(format_pan(-0.25), "L25");
    assert_eq!(format_pan(1.0), "R100");
}
