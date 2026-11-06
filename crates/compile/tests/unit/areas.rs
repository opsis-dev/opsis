use opsis_compile::areas::{big_enough, box_from, moved_box};
use opsis_types::actions::Bounds;
use proptest::prelude::*;

const ROUNDING: f64 = 0.0015;

fn inside(area: Bounds) -> bool {
    area.x >= 0.0
        && area.y >= 0.0
        && area.x + area.width <= 1.0 + ROUNDING
        && area.y + area.height <= 1.0 + ROUNDING
}

proptest! {
    #[test]
    fn a_drawn_box_is_the_same_from_either_corner(
        x0 in -0.5f64..1.5, y0 in -0.5f64..1.5, x1 in -0.5f64..1.5, y1 in -0.5f64..1.5,
    ) {
        let drawn = box_from(x0, y0, x1, y1);
        prop_assert_eq!(drawn, box_from(x1, y1, x0, y0));
        prop_assert!(drawn.width >= 0.0 && drawn.height >= 0.0);
        prop_assert!(inside(drawn));
    }

    #[test]
    fn a_moved_box_keeps_its_size_and_stays_in_the_frame(
        x0 in 0f64..1.0, y0 in 0f64..1.0, x1 in 0f64..1.0, y1 in 0f64..1.0,
        dx in -2f64..2.0, dy in -2f64..2.0,
    ) {
        let origin = box_from(x0, y0, x1, y1);
        let moved = moved_box(origin, dx, dy);
        prop_assert!((moved.width - origin.width).abs() < f64::EPSILON);
        prop_assert!((moved.height - origin.height).abs() < f64::EPSILON);
        prop_assert!(inside(moved));
    }

    #[test]
    fn a_click_without_a_drag_is_too_small_to_keep(x in 0f64..1.0, y in 0f64..1.0) {
        prop_assert!(!big_enough(box_from(x, y, x, y)));
    }
}
