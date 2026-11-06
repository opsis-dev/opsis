use std::collections::BTreeMap;

use opsis_compile::timeline::view::{
    Loop, SnapKind, SnapTarget, active_at, format_time, loop_of, ruler_step, snap_frame, snap_span,
    wrap_in_loop,
};
use proptest::prelude::*;

fn target() -> impl Strategy<Value = SnapTarget> {
    (
        0u32..2000,
        proptest::sample::select(vec![
            SnapKind::Playhead,
            SnapKind::Edge,
            SnapKind::Marker,
            SnapKind::Beat,
        ]),
    )
        .prop_map(|(frame, kind)| SnapTarget {
            frame: f64::from(frame),
            kind,
        })
}

proptest! {
    #[test]
    fn snapping_lands_on_the_nearest_target_in_reach(
        targets in proptest::collection::vec(target(), 0..12),
        frame in -100f64..2100.0,
        threshold in 0u32..40,
    ) {
        let threshold = f64::from(threshold);
        let snapped = snap_frame(frame, &targets, threshold);
        let nearest = targets.iter().map(|t| (t.frame - frame).abs()).fold(f64::INFINITY, f64::min);
        let in_reach = nearest <= threshold;
        prop_assert_eq!(snapped.target.is_some(), in_reach);
        prop_assert!((snapped.frame - snapped.target.map_or(frame, |t| t.frame)).abs() < f64::EPSILON);
        if in_reach {
            prop_assert!(((snapped.frame - frame).abs() - nearest).abs() < 1e-9);
        }
    }

    #[test]
    fn a_moved_span_puts_its_closer_end_on_a_target(
        targets in proptest::collection::vec(target(), 0..12),
        from in 0u32..1000,
        length in 1u32..500,
        delta in -300f64..300.0,
        threshold in 0u32..40,
    ) {
        let (from, to, threshold) = (f64::from(from), f64::from(from + length), f64::from(threshold));
        let snapped = snap_span(from, to, delta, &targets, threshold);
        let ends = [from, to].map(|edge| snap_frame(edge + delta, &targets, threshold));
        prop_assert_eq!(snapped.target.is_some(), ends.iter().any(|e| e.target.is_some()));
        if let Some(t) = snapped.target {
            let landed = [from + snapped.delta, to + snapped.delta];
            prop_assert!(landed.iter().any(|l| (l - t.frame).abs() < 1e-9));
            let closest = ends
                .iter()
                .zip([from, to])
                .filter(|(e, _)| e.target.is_some())
                .map(|(e, edge)| (e.frame - (edge + delta)).abs())
                .fold(f64::INFINITY, f64::min);
            prop_assert!(((snapped.delta - delta).abs() - closest).abs() < 1e-9);
        } else {
            prop_assert!((snapped.delta - delta).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn active_spans_hold_the_frame_and_come_sorted(
        spans in proptest::collection::btree_map("[a-z]{1,4}", (0u32..1000, 1u32..500), 0..8),
        frame in 0u32..1600,
    ) {
        let spans: BTreeMap<String, (f64, f64)> = spans
            .into_iter()
            .map(|(id, (from, length))| (id, (f64::from(from), f64::from(from + length))))
            .collect();
        let frame = f64::from(frame);
        let active = active_at(&spans, frame);
        let mut sorted = active.clone();
        sorted.sort();
        prop_assert_eq!(&active, &sorted);
        for (id, (from, to)) in &spans {
            prop_assert_eq!(active.contains(id), *from <= frame && frame < *to);
        }
    }

    #[test]
    fn a_loop_plays_the_out_frame_and_stays_inside_the_film(
        in_point in proptest::option::of(-50i32..400),
        out_point in proptest::option::of(-50i32..400),
        duration in 1u32..300,
    ) {
        let range = loop_of(in_point.map(f64::from), out_point.map(f64::from), f64::from(duration));
        prop_assert!(range.from >= 0.0);
        prop_assert!(range.to <= f64::from(duration));
        prop_assert!(range.from < range.to);
    }

    #[test]
    fn wrapping_keeps_a_position_in_its_place_in_the_cycle(
        from in 0u32..1000,
        length in 1u32..500,
        position in -10_000f64..10_000.0,
    ) {
        let range = Loop { from: f64::from(from), to: f64::from(from + length) };
        let wrapped = wrap_in_loop(position, range);
        prop_assert!(wrapped >= range.from && wrapped < range.to);
        let cycles = (position - wrapped) / (range.to - range.from);
        prop_assert!((cycles - cycles.round()).abs() < 1e-6);
        if position >= range.from && position < range.to {
            prop_assert!((wrapped - position).abs() < f64::EPSILON);
        }
    }
}

#[test]
fn the_whole_film_loops_without_in_and_out_points() {
    assert_eq!(
        loop_of(None, None, 300.0),
        Loop {
            from: 0.0,
            to: 300.0
        }
    );
    assert_eq!(
        loop_of(Some(30.0), Some(120.0), 300.0),
        Loop {
            from: 30.0,
            to: 121.0
        }
    );
}

#[test]
fn ruler_ticks_space_out_with_the_zoom() {
    assert_eq!(ruler_step(30.0, 30, 72.0), (5.0, 1.0));
    assert_eq!(ruler_step(1.0, 30, 72.0), (150.0, 30.0));
    assert_eq!(ruler_step(0.05, 30, 72.0), (1800.0, 360.0));
    assert_eq!(ruler_step(0.001, 30, 72.0), (18_000.0, 1800.0));
}

#[test]
fn times_read_as_minutes_and_seconds_or_timecode() {
    assert_eq!(format_time(0.0, 30, false), "0:00");
    assert_eq!(format_time(45.0, 30, false), "0:01.5");
    assert_eq!(format_time(1830.0, 30, false), "1:01");
    assert_eq!(format_time(108_031.0, 30, true), "01:00:01:01");
}
