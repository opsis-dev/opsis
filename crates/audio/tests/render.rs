use opsis_audio::render::{Pace, fill, frames_in};
use proptest::prelude::*;

proptest! {
    #[test]
    fn filling_moves_by_the_step_for_every_frame(
        samples in proptest::collection::vec(-1f32..1.0, 0..200),
        position in -10f64..120.0,
        step in 0.1f64..3.0,
        frames in 1usize..64,
        channels in 1usize..4,
    ) {
        let mut out = vec![9.0f32; frames * channels];
        let end = fill(&mut out, channels, &samples, position, Pace { step, volume: 1.0 });
        let frames_f = f64::from(u32::try_from(frames).unwrap_or(0));
        prop_assert!((end - step.mul_add(frames_f, position)).abs() < 1e-6);
        prop_assert!(out.iter().all(|s| (-1.0..=1.0).contains(s)));
    }

    #[test]
    fn outside_the_stem_is_silence(
        samples in proptest::collection::vec(-1f32..1.0, 2..100),
        frames in 1usize..16,
    ) {
        let total = frames_in(&samples);
        let mut out = vec![9.0f32; frames * 2];
        let start = f64::from(u32::try_from(total).unwrap_or(0)) + 1.0;
        fill(&mut out, 2, &samples, start, Pace { step: 1.0, volume: 1.0 });
        prop_assert!(out.iter().all(|s| s.abs() < f32::EPSILON));
    }

    #[test]
    fn a_whole_step_copies_the_stem(
        pairs in proptest::collection::vec((-1f32..1.0, -1f32..1.0), 1..50),
    ) {
        let samples: Vec<f32> = pairs.iter().flat_map(|(l, r)| [*l, *r]).collect();
        let mut out = vec![0.0f32; samples.len()];
        fill(&mut out, 2, &samples, 0.0, Pace { step: 1.0, volume: 1.0 });
        prop_assert_eq!(out, samples);
    }
}
