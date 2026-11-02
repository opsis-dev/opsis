use opsis_compile::remap::{
    Remap, RemapSegment, RemapSide, inverse_remap, remap_duration, remap_frame, remap_from_segments,
};
use opsis_compile::time::{
    chapter_time, frames_to_ms, frames_to_recording_ms, ms_to_frames, ms_to_frames_from,
    recording_ms_to_frames, whole_frames,
};
use opsis_compile::timeline::camera_path::{CameraView, camera_at, camera_key_at, move_camera_key};
use opsis_types::numbers::exact_eq;
use opsis_types::timeline::CameraPath;
use proptest::prelude::*;
use serde_json::{Value, json};

const RATES: [u32; 8] = [12, 24, 25, 30, 48, 50, 60, 120];

#[test]
fn writes_chapter_times_as_a_clock() {
    assert_eq!(
        [chapter_time(3_725_000.0), chapter_time(20_000.0)],
        ["1:02:05", "0:20"]
    );
}

proptest! {
    #[test]
    fn frames_survive_a_trip_through_milliseconds(frames in 1u32..200_000, fps in prop::sample::select(RATES.to_vec())) {
        prop_assert_eq!(ms_to_frames(frames_to_ms(frames, fps), fps), frames);
        let ms = frames_to_ms(frames, fps);
        prop_assert!(exact_eq(ms.fract(), 0.0), "{ms}");
    }

    #[test]
    fn rounds_to_the_nearest_whole_frame_and_never_below_the_minimum(
        frames in -1e6..1e6f64,
        min in 0u32..10,
        ms in 0.0..1e7f64,
        fps in prop::sample::select(RATES.to_vec()),
    ) {
        let whole = whole_frames(frames, min);
        prop_assert!(whole >= min);
        if frames.round() >= f64::from(min) {
            prop_assert!(exact_eq(f64::from(whole), frames.round()), "{whole} != {}", frames.round());
        }
        prop_assert!(ms_to_frames(ms, fps) >= 1);
        prop_assert_eq!(ms_to_frames_from(ms, fps, 0), whole_frames(ms / 1000.0 * f64::from(fps), 0));
    }

    #[test]
    fn places_a_recording_moment_and_finds_it_again_at_any_speed(
        ms in 0.0..1e7f64,
        speed in 0.05..20.0f64,
        fps in prop::sample::select(RATES.to_vec()),
    ) {
        let frames = recording_ms_to_frames(ms, speed, fps);
        prop_assert!((frames_to_recording_ms(frames, speed, fps) - ms).abs() <= f64::mul_add(ms, 1e-12, 1e-9));
        prop_assert!(recording_ms_to_frames(ms, speed * 2.0, fps) <= frames);
    }
}

fn path() -> CameraPath {
    serde_json::from_value(json!({ "keys": [
        { "frame": 60, "yaw": 40, "pitch": 10, "distance": 2, "height": 0.5, "ease": "linear" },
        { "frame": 0, "yaw": -20, "pitch": 0, "distance": 1, "height": 0, "ease": "linear" }
    ] }))
    .unwrap()
}

const fn view(yaw: f64, pitch: f64, distance: f64, height: f64) -> CameraView {
    CameraView {
        yaw,
        pitch,
        distance,
        height,
    }
}

#[test]
fn lands_on_each_camera_key_blends_between_and_holds_the_ends() {
    let p = path();
    assert_eq!(camera_at(Some(&p), 0.0, false), view(-20.0, 0.0, 1.0, 0.0));
    assert_eq!(camera_at(Some(&p), 60.0, false), view(40.0, 10.0, 2.0, 0.5));
    assert_eq!(camera_at(Some(&p), 30.0, false), view(10.0, 5.0, 1.5, 0.25));
    let single = CameraPath {
        keys: vec![p.keys[0].clone()],
    };
    assert_eq!(camera_at(Some(&single), 10.0, false).yaw, 40.0);
    assert_eq!(camera_at(Some(&p), 500.0, false).yaw, 40.0);
    assert_eq!(
        camera_at(Some(&CameraPath { keys: Vec::new() }), 10.0, false),
        view(0.0, 0.0, 1.0, 0.0)
    );
    assert_eq!(camera_at(Some(&p), 60.0, true), view(0.0, 0.0, 1.0, 0.0));
    assert!((camera_at(Some(&p), 30.0, true).yaw + 10.0).abs() < 1e-9);
}

fn camera_keys() -> impl Strategy<Value = CameraPath> {
    prop::collection::btree_map(
        0u32..600,
        (-90.0..90.0f64, -30.0..30.0f64, 1.0..3.0f64, -0.5..0.5f64),
        1..6,
    )
    .prop_map(|keys| {
        let keys: Vec<Value> = keys
            .into_iter()
            .rev()
            .map(|(frame, (yaw, pitch, distance, height))| {
                json!({ "frame": frame, "yaw": yaw, "pitch": pitch, "distance": distance, "height": height, "ease": "linear" })
            })
            .collect();
        serde_json::from_value(json!({ "keys": keys })).unwrap()
    })
}

proptest! {
    #[test]
    fn lands_on_every_key_stays_between_neighbours_and_holds_the_ends(path in camera_keys(), frame in -50.0..700.0f64) {
        let mut keys = path.keys.clone();
        keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
        for key in &keys {
            prop_assert_eq!(camera_at(Some(&path), key.frame, false), view(key.yaw, key.pitch, key.distance, key.height));
        }
        let seen = camera_at(Some(&path), frame, false);
        let (first, last) = (&keys[0], &keys[keys.len() - 1]);
        if frame <= first.frame {
            prop_assert_eq!(seen, view(first.yaw, first.pitch, first.distance, first.height));
        } else if frame >= last.frame {
            prop_assert_eq!(seen, view(last.yaw, last.pitch, last.distance, last.height));
        } else {
            let after = keys.iter().position(|k| k.frame > frame).unwrap();
            let (a, b) = (&keys[after - 1], &keys[after]);
            let between = |x: f64, p: f64, q: f64| x >= p.min(q) - 1e-9 && x <= p.max(q) + 1e-9;
            prop_assert!(between(seen.yaw, a.yaw, b.yaw));
            prop_assert!(between(seen.pitch, a.pitch, b.pitch));
            prop_assert!(between(seen.distance, a.distance, b.distance));
            prop_assert!(between(seen.height, a.height, b.height));
        }
    }
}

#[test]
fn moves_a_camera_key_in_time_and_adds_one_at_the_playhead() {
    let moved = move_camera_key(&path(), 0, 90.0);
    assert_eq!(
        moved
            .keys
            .iter()
            .map(|k| (k.frame, k.yaw))
            .collect::<Vec<_>>(),
        [(60.0, 40.0), (90.0, -20.0)]
    );
    let added = camera_key_at(&path(), 30.4);
    assert_eq!(
        added.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        [0.0, 30.0, 60.0]
    );
    assert_eq!(
        (
            added.keys[1].yaw,
            added.keys[1].pitch,
            added.keys[1].distance
        ),
        (10.0, 5.0, 1.5)
    );
}

fn segments() -> impl Strategy<Value = (f64, Vec<RemapSegment>)> {
    (
        100u32..2000,
        prop::collection::vec((1u32..200, 1u32..200, 0.1..8.0f64), 0..5),
    )
        .prop_map(|(length, parts)| {
            let mut at = 0.0;
            let mut list = Vec::new();
            for (gap, span, rate) in parts {
                let from = at + f64::from(gap);
                let to = from + f64::from(span);
                list.push(RemapSegment { from, to, rate });
                at = to;
            }
            (f64::from(length), list)
        })
}

fn held_remap() -> impl Strategy<Value = Remap> {
    prop::collection::vec((1u32..50, 0u32..50), 1..8).prop_map(|steps| {
        let mut points = vec![(0.0, 0.0)];
        for (out, inner) in steps {
            let (o, i) = *points.last().unwrap();
            points.push((o + f64::from(out), i + f64::from(inner)));
        }
        points
    })
}

proptest! {
    #[test]
    fn plays_a_speed_ramp_forward_and_finds_every_moment_again((length, segments) in segments(), share in 0.0..1.0f64) {
        let remap = remap_from_segments(length, &segments);
        let duration = remap_duration(&remap);
        prop_assert!(remap.windows(2).all(|w| w[0].0 <= w[1].0 && w[0].1 <= w[1].1));
        let out = share * duration;
        let inner = remap_frame(&remap, out);
        prop_assert!((0.0..=length + 1e-9).contains(&inner));
        prop_assert!(remap_frame(&remap, out + 1.0) >= inner);
        for side in [RemapSide::Start, RemapSide::End] {
            prop_assert!((inverse_remap(&remap, inner, side) - out).abs() < 1e-2);
        }
        prop_assert!((remap_frame(&remap, duration) - length).abs() < 1e-9);
    }

    #[test]
    fn finds_where_a_held_moment_starts_and_ends(remap in held_remap(), share in 0.0..1.0f64) {
        let last = remap.last().unwrap().1;
        let r = (share * last).round();
        let (start, end) = (
            inverse_remap(&remap, r, RemapSide::Start),
            inverse_remap(&remap, r, RemapSide::End),
        );
        prop_assert!(start <= end);
        prop_assert!((remap_frame(&remap, start) - r).abs() < 1e-9);
        prop_assert!((remap_frame(&remap, end) - r).abs() < 1e-9);
        prop_assert!(inverse_remap(&remap, -1.0, RemapSide::Start) < remap[0].0);
    }
}

proptest::proptest! {
    #[test]
    fn eases_each_speed_change_inside_its_own_part(
        from in 0.0..10_000.0f64,
        length in 1.0..10_000.0f64,
        rate in 0.1..64.0f64,
        ease in 0.0..3_000.0f64,
    ) {
        let parts = opsis_compile::remap::eased_segments(&[(from, from + length, rate)], ease);
        proptest::prop_assert!((parts[0].0 - from).abs() < 1e-6);
        proptest::prop_assert!((parts[parts.len() - 1].1 - (from + length)).abs() < 1e-6);
        for pair in parts.windows(2) {
            proptest::prop_assert!((pair[0].1 - pair[1].0).abs() < 1e-6, "{parts:?}");
        }
        let (low, high) = (rate.min(1.0), rate.max(1.0));
        proptest::prop_assert!(parts.iter().all(|p| p.2 >= low - 1e-9 && p.2 <= high + 1e-9));
        proptest::prop_assert!(parts.iter().any(|p| (p.2 - rate).abs() < 1e-9));
    }
}
