use opsis_scene::clock::{AUTHORED_FPS, Clock, js_round};
use opsis_types::numbers::exact_eq;
use proptest::prelude::*;

fn frames() -> impl Strategy<Value = f64> {
    (0u32..2_000).prop_map(f64::from)
}

fn lengths() -> impl Strategy<Value = f64> {
    (1u32..600).prop_map(f64::from)
}

proptest! {
    #[test]
    fn nested_sequences_shift_like_one(frame in frames(), outer in 0u32..500, inner in 0u32..500, total in lengths()) {
        let clock = Clock::new(frame, 30, 4_000.0);
        let (outer, inner) = (f64::from(outer), f64::from(inner));
        let nested = clock.sequence(outer, None).and_then(|c| c.sequence(inner, Some(total)));
        let once = clock.sequence(outer + inner, Some(total));
        prop_assert_eq!(nested.map(|c| c.frame), once.map(|c| c.frame));
    }

    #[test]
    fn a_sequence_shows_only_its_own_frames(frame in frames(), from in 0u32..1_000, length in lengths()) {
        let (from, length) = (f64::from(from), length);
        let shown = Clock::new(frame, 30, 4_000.0).sequence(from, Some(length));
        prop_assert_eq!(shown.is_some(), frame >= from && frame < from + length);
        if let Some(inner) = shown {
            prop_assert!(exact_eq(inner.frame, frame - from));
            prop_assert!(inner.duration_in_frames <= length);
        }
    }

    #[test]
    fn a_loop_repeats_inside_its_length(frame in frames(), length in lengths()) {
        let clock = Clock::new(frame, 30, 2_000.0);
        if let Some(inner) = clock.loop_every(length) {
            prop_assert!(inner.frame >= 0.0 && inner.frame < length);
            prop_assert!(exact_eq(inner.frame % length, frame % length) || frame >= length * ((2_000.0 / length).ceil() - 1.0));
        }
    }

    #[test]
    fn the_authored_clock_runs_at_thirty_and_maps_back(frame in frames(), fps in prop::sample::select(vec![24u32, 25, 30, 50, 60])) {
        let clock = Clock::new(frame, fps, 900.0);
        let authored = clock.authored();
        prop_assert_eq!(authored.fps, AUTHORED_FPS);
        prop_assert!((clock.real_frames(authored.frame) - frame).abs() < 1e-9);
        prop_assert!((authored.seconds() - clock.seconds()).abs() < 1e-9);
    }

    #[test]
    fn freezing_keeps_the_rate_and_length(frame in frames(), held in frames()) {
        let clock = Clock::new(frame, 30, 900.0);
        let frozen = clock.freeze(held);
        prop_assert!(exact_eq(frozen.frame, held));
        prop_assert_eq!((frozen.fps, frozen.duration_in_frames.to_bits()), (clock.fps, clock.duration_in_frames.to_bits()));
    }

    #[test]
    fn rounding_goes_half_up_like_javascript(whole in -1_000i32..1_000) {
        let x = f64::from(whole);
        prop_assert!(exact_eq(js_round(x + 0.5), x + 1.0));
        prop_assert!(exact_eq(js_round(x + 0.49), x));
        prop_assert!(exact_eq(js_round(x), x));
    }
}
