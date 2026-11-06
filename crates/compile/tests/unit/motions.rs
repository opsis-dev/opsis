use opsis_compile::motions::{Ease, eased, motion_spec, offsets, sample};
use opsis_types::looks::NodeMotion;
use proptest::prelude::*;

fn motion() -> impl Strategy<Value = NodeMotion> {
    proptest::sample::select(NodeMotion::ALL)
}

fn ease() -> impl Strategy<Value = Ease> {
    proptest::sample::select(vec![
        Ease::InOut,
        Ease::Out,
        Ease::Back,
        Ease::Linear,
        Ease::Step,
    ])
}

proptest! {
    #[test]
    fn keyframes_run_from_start_to_end_in_order(motion in motion()) {
        let stops = offsets(&motion_spec(motion).frames);
        prop_assert!((stops[0]).abs() < 1e-9);
        prop_assert!((stops[stops.len() - 1] - 1.0).abs() < 1e-9);
        prop_assert!(stops.windows(2).all(|pair| pair[1] >= pair[0]));
    }

    #[test]
    fn a_motion_starts_and_ends_on_its_outer_keyframes(motion in motion()) {
        let spec = motion_spec(motion);
        let first = spec.frames.first().unwrap().pose;
        let last = spec.frames.last().unwrap().pose;
        prop_assert_eq!(sample(&spec, 0.0), first);
        prop_assert_eq!(sample(&spec, f64::from(spec.ms)), last);
    }

    #[test]
    fn every_ease_runs_from_zero_to_one(ease in ease()) {
        prop_assert!(eased(ease, 0.0).abs() < 1e-6);
        prop_assert!((eased(ease, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn smooth_eases_never_turn_back(
        ease in proptest::sample::select(vec![Ease::InOut, Ease::Out, Ease::Linear]),
        a in 0.0f64..1.0,
        b in 0.0f64..1.0,
    ) {
        let (low, high) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(eased(ease, low) <= eased(ease, high) + 1e-6);
    }
}
