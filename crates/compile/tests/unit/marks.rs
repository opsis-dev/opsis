use opsis_compile::marks::{
    MarkPoints, Point, drawn, kind_of, moved, points_of, step_numbers, two_ended, with_end,
};
use opsis_types::numbers::exact_eq;
use proptest::prelude::*;
use serde_json::{Value, json};

const KINDS: [&str; 5] = ["arrow", "rectangle", "oval", "step", "text"];

fn unit() -> impl Strategy<Value = f64> {
    (0u32..=1000).prop_map(|n| f64::from(n) / 1000.0)
}

fn points() -> impl Strategy<Value = MarkPoints> {
    (unit(), unit(), unit(), unit()).prop_map(|(x, y, x2, y2)| MarkPoints { x, y, x2, y2 })
}

fn point() -> impl Strategy<Value = Point> {
    (-2f64..3.0, -2f64..3.0).prop_map(|(x, y)| Point { x, y })
}

fn on_picture(p: MarkPoints) -> bool {
    [p.x, p.y, p.x2, p.y2].into_iter().all(|v| {
        let scaled = v * 1000.0;
        (0.0..=1.0).contains(&v) && (scaled - scaled.round()).abs() < 1e-9
    })
}

proptest! {
    #[test]
    fn a_moved_mark_keeps_its_shape_and_stays_on_the_picture(p in points(), dx in -2f64..3.0, dy in -2f64..3.0) {
        let m = moved(p, dx, dy);
        prop_assert!(on_picture(m));
        prop_assert!(((m.x2 - m.x) - (p.x2 - p.x)).abs() < 0.0015);
        prop_assert!(((m.y2 - m.y) - (p.y2 - p.y)).abs() < 0.0015);
        let left = p.x.min(p.x2);
        let right = p.x.max(p.x2);
        let fits = dx >= -left && dx <= 1.0 - right;
        let at_edge = m.x.min(m.x2) < 0.001 || m.x.max(m.x2) > 0.999;
        let followed = ((m.x - p.x) - dx).abs() < 0.005;
        prop_assert!(followed || (!fits && at_edge));
    }

    #[test]
    fn dragging_an_end_moves_only_that_end(p in points(), second in any::<bool>(), at in point()) {
        let m = with_end(p, second, at);
        prop_assert!(on_picture(m));
        let (held, other, before) = if second {
            ((m.x2, m.y2), (m.x, m.y), (p.x, p.y))
        } else {
            ((m.x, m.y), (m.x2, m.y2), (p.x2, p.y2))
        };
        prop_assert_eq!(other, before);
        prop_assert!((held.0 - at.x.clamp(0.0, 1.0)).abs() < 0.001);
        prop_assert!((held.1 - at.y.clamp(0.0, 1.0)).abs() < 0.001);
    }

    #[test]
    fn two_ended_marks_need_a_drag_and_the_rest_land_on_a_click(
        kind in proptest::sample::select(KINDS.to_vec()),
        start in point(),
        end in point(),
    ) {
        let mark = drawn(kind, start, end);
        let long = (end.x - start.x).hypot(end.y - start.y) >= 0.01;
        prop_assert_eq!(mark.is_some(), !two_ended(kind) || long);
        if let Some(m) = mark {
            prop_assert!(on_picture(m));
            prop_assert!(two_ended(kind) || (exact_eq(m.x, m.x2) && exact_eq(m.y, m.y2)));
        }
    }

    #[test]
    fn steps_count_up_in_list_order(
        kinds in proptest::collection::vec(proptest::sample::select(vec!["arrow", "rectangle", "oval", "step", "text", "nonsense"]), 0..12),
        first in 0i64..999,
    ) {
        let marks: Vec<Value> = kinds.iter().map(|k| json!({ "kind": k })).collect();
        let numbers = step_numbers(&marks, first);
        let steps: Vec<i64> = numbers.iter().flatten().copied().collect();
        prop_assert_eq!(steps.clone(), (0..steps.len()).map(|i| first + i64::try_from(i).unwrap_or(0)).collect::<Vec<_>>());
        for (mark, number) in marks.iter().zip(&numbers) {
            prop_assert_eq!(number.is_some(), kind_of(mark) == "step");
        }
    }
}

#[test]
fn unknown_kinds_are_arrows_and_missing_points_are_zero() {
    assert_eq!(kind_of(&json!({ "kind": "nonsense" })), "arrow");
    assert_eq!(
        points_of(&json!({ "x": 0.5, "y": "no" })),
        MarkPoints {
            x: 0.5,
            y: 0.0,
            x2: 0.0,
            y2: 0.0
        }
    );
}
