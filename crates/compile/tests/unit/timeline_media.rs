use opsis_compile::timeline::media::{
    WaveSpan, pick_level, shaped, tile_index, tile_start, wave_column,
};
use opsis_types::analysis::WaveformLevel;
use proptest::prelude::*;

fn level(rate: f64, count: usize) -> WaveformLevel {
    WaveformLevel {
        rate,
        min: vec![-0.5; count],
        max: vec![0.5; count],
    }
}

proptest! {
    #[test]
    fn shaped_amplitudes_keep_their_sign_and_stay_in_range(value in -4f64..4.0) {
        let shown = shaped(value);
        prop_assert!((-1.0..=1.0).contains(&shown));
        prop_assert!(shown == 0.0 || shown.signum() == value.signum());
    }

    #[test]
    fn louder_reads_taller(a in 0f64..1.0, b in 0f64..1.0) {
        let (low, high) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(shaped(low) <= shaped(high));
    }

    #[test]
    fn the_picked_level_is_the_coarsest_that_still_fills_each_pixel(
        rates in proptest::collection::vec(1u32..2000, 1..6),
        px in 1u32..3000,
    ) {
        let levels: Vec<WaveformLevel> = rates.iter().map(|&r| level(f64::from(r), 4)).collect();
        let picked = pick_level(&levels, f64::from(px)).map(|l| l.rate);
        let finer: Vec<f64> = rates.iter().map(|&r| f64::from(r)).filter(|&r| r >= f64::from(px)).collect();
        let expected = finer
            .iter()
            .copied()
            .fold(None, |best: Option<f64>, r| Some(best.map_or(r, |b| b.min(r))))
            .or_else(|| rates.iter().map(|&r| f64::from(r)).fold(None, |best: Option<f64>, r| Some(best.map_or(r, |b| b.max(r)))));
        prop_assert_eq!(picked, expected);
    }

    #[test]
    fn columns_only_draw_inside_the_bar(
        from in 0f64..500.0,
        length in 1f64..500.0,
        abs in -100f64..1200.0,
    ) {
        let span = WaveSpan { from, to: from + length, trim_before: 0.0, looped: false, volume: 1.0, duration_s: 100.0 };
        let column = wave_column(&level(100.0, 100_000), span, abs, 30.0, 1.0);
        prop_assert_eq!(column.is_some(), abs >= from && abs < from + length);
    }

    #[test]
    fn tiles_start_on_the_grid_left_of_the_view(
        left in 0f64..10_000.0,
        offset in 0f64..2000.0,
        width in 1f64..200.0,
    ) {
        let start = tile_start(left, offset, width);
        prop_assert!(start <= offset + 1e-6);
        prop_assert!(offset - start < width + 1e-6);
        let cycles = (left + start) / width;
        prop_assert!((cycles - cycles.round()).abs() < 1e-6);
    }

    #[test]
    fn tile_indexes_stay_in_the_strip(ms in -1000f64..100_000.0, interval in 1f64..2000.0, count in 1u32..200) {
        prop_assert!(tile_index(ms, interval, count) < count);
    }
}
