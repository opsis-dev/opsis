use opsis_scene::shutter::{sample_opacity, shutter_offsets};
use opsis_types::numbers::float;
use proptest::prelude::*;

proptest! {
    #[test]
    fn offsets_start_at_zero_and_stay_inside_the_open_shutter(angle in 0.0f64..720.0, samples in 0u32..32) {
        let offsets = shutter_offsets(angle, samples);
        let open = angle.clamp(0.0, 360.0) / 360.0;
        prop_assert!(offsets[0].abs() < f64::EPSILON);
        prop_assert!(offsets.iter().all(|&o| o >= 0.0 && (o < open || offsets.len() == 1)));
        prop_assert!(offsets.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn stacked_samples_weigh_the_same(samples in 1usize..32) {
        let weights: Vec<f64> = (0..samples)
            .map(|k| sample_opacity(k) * ((k + 1)..samples).map(|j| 1.0 - sample_opacity(j)).product::<f64>())
            .collect();
        for weight in weights {
            prop_assert!((weight - 1.0 / float(samples)).abs() < 1e-12);
        }
    }
}
