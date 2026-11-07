use opsis_audio::dsp::{FilterState, Smoother, pan_stereo};
use opsis_compile::mix::eq::coefficients;
use opsis_types::mix::{EqBand, EqType};
use proptest::prelude::*;

proptest! {
    #[test]
    fn a_flat_filter_leaves_the_sound_alone(input in proptest::collection::vec(-1f64..1.0, 1..200), freq in 50f64..10_000.0) {
        let filter = coefficients(&EqBand { kind: EqType::Peak, freq, gain: 0.0, q: 1.0 }, 48_000.0);
        let mut state = FilterState::default();
        for x in &input {
            prop_assert!((state.process(&filter, *x) - x).abs() < 1e-9);
        }
    }

    #[test]
    fn smoothing_moves_towards_the_target_without_passing_it(start in -10f64..10.0, target in -10f64..10.0, steps in 1usize..500) {
        let mut smoother = Smoother::new(start, 0.02, 48_000.0);
        let mut last = start;
        for _ in 0..steps {
            let value = smoother.next(target);
            prop_assert!((target - value).abs() <= (target - last).abs() + 1e-12);
            last = value;
        }
    }

    #[test]
    fn centred_pan_changes_nothing_and_any_pan_keeps_its_side(left in -1f64..1.0, right in -1f64..1.0, pan in -1f64..1.0) {
        let (l, r) = pan_stereo(left, right, 0.0);
        prop_assert!((l - left).abs() < 1e-9 && r.abs() <= 1.0 + 1e-9 && (r - right).abs() < 1e-9);
        let (pl, pr) = pan_stereo(1.0, 1.0, pan);
        if pan < -0.01 {
            prop_assert!(pl > pr);
        } else if pan > 0.01 {
            prop_assert!(pr > pl);
        }
    }
}
