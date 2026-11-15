use opsis_render::footage::remap_rate;
use opsis_render::native::{ahead_for, fitted_ahead};
use proptest::prelude::*;

proptest! {
    #[test]
    fn the_allowance_is_the_most_frames_ahead_whose_windows_fit_half_the_free_memory(
        ahead in 0usize..200,
        per_frame in 0u64..50_000_000,
        fixed in 0u64..100_000_000,
        free in 0u64..64_000_000_000,
    ) {
        let bytes = |frames: usize| fixed + per_frame * u64::try_from(frames).unwrap();
        let fitted = fitted_ahead(ahead, &bytes, free);
        prop_assert!(fitted >= 1);
        prop_assert!(fitted <= ahead.max(1));
        prop_assert!(fitted == 1 || bytes(fitted) <= free / 2);
        prop_assert!(fitted >= ahead || bytes(fitted + 1) > free / 2);
    }

    #[test]
    fn every_worker_may_draw_two_frames_ahead(workers in 1usize..256) {
        prop_assert!(ahead_for(workers) >= workers * 2);
        prop_assert!(ahead_for(workers + 1) >= ahead_for(workers));
    }

    #[test]
    fn a_remap_reads_at_least_as_fast_as_its_steepest_stretch(steps in proptest::collection::vec((1u32..40, 0u32..200), 1..12)) {
        let mut remap = vec![(0.0, 0.0)];
        for (outer, inner) in steps {
            let (x, y) = *remap.last().unwrap();
            remap.push((x + f64::from(outer), y + f64::from(inner)));
        }
        let rate = remap_rate(&remap);
        prop_assert!(rate >= 1.0);
        for w in remap.windows(2) {
            prop_assert!(rate >= (w[1].1 - w[0].1) / (w[1].0 - w[0].0) - 1e-12);
        }
    }
}
