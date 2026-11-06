use opsis_compile::canvas::{SNAP_GRID, snap_to_grid, staggered};
use opsis_types::workflow::Position;
use proptest::prelude::*;

fn on_grid(value: f64) -> bool {
    (value / SNAP_GRID - (value / SNAP_GRID).round()).abs() < 1e-9
}

proptest! {
    #[test]
    fn snapping_lands_on_the_grid_within_half_a_cell(x in -1e5f64..1e5, y in -1e5f64..1e5) {
        let snapped = snap_to_grid(Position { x, y });
        prop_assert!(on_grid(snapped.x) && on_grid(snapped.y));
        prop_assert!((snapped.x - x).abs() <= SNAP_GRID / 2.0 + 1e-9);
        prop_assert!((snapped.y - y).abs() <= SNAP_GRID / 2.0 + 1e-9);
        prop_assert_eq!(snap_to_grid(snapped), snapped);
    }

    #[test]
    fn staggered_files_never_land_on_each_other(x in -1e4f64..1e4, y in -1e4f64..1e4, count in 2usize..20) {
        let at = Position { x, y };
        let spots: Vec<Position> = (0..count).map(|i| staggered(at, i)).collect();
        prop_assert!(spots.windows(2).all(|pair| pair[1].x > pair[0].x && pair[1].y > pair[0].y));
    }
}
