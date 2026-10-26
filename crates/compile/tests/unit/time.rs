use opsis_compile::remap::{
    RemapSegment, RemapSide, inverse_remap, remap_duration, remap_frame, remap_from_segments,
};
use opsis_compile::time::{
    chapter_time, frames_to_ms, frames_to_recording_ms, ms_to_frames, ms_to_frames_from,
    recording_ms_to_frames, whole_frames,
};
use opsis_compile::timeline::camera_path::{CameraView, camera_at, camera_key_at, move_camera_key};
use opsis_types::timeline::CameraPath;
use serde_json::json;

#[test]
fn rounds_times_to_whole_frames_and_milliseconds() {
    assert_eq!(ms_to_frames(1000.0, 30), 30);
    assert_eq!(ms_to_frames(0.0, 30), 1);
    assert_eq!(ms_to_frames_from(0.0, 30, 0), 0);
    assert_eq!(
        [
            whole_frames(2.6, 0),
            whole_frames(-4.0, 0),
            whole_frames(0.2, 1)
        ],
        [3, 0, 1]
    );
    assert_eq!(
        [frames_to_ms(30.0, 30), frames_to_ms(1.0, 30)],
        [1000.0, 33.0]
    );
    assert_eq!(
        [chapter_time(3_725_000.0), chapter_time(20_000.0)],
        ["1:02:05", "0:20"]
    );
}

#[test]
fn places_a_recording_moment_on_the_timeline_at_the_playback_speed_and_back() {
    assert_eq!(recording_ms_to_frames(2000.0, 2.0, 30), 30.0);
    assert_eq!(recording_ms_to_frames(1000.0, 0.5, 30), 60.0);
    assert_eq!(frames_to_recording_ms(30.0, 2.0, 30), 2000.0);
}

fn path() -> CameraPath {
    serde_json::from_value(json!({ "keys": [
        { "frame": 60, "yaw": 40, "pitch": 10, "distance": 2, "height": 0.5, "ease": "linear" },
        { "frame": 0, "yaw": -20, "pitch": 0, "distance": 1, "height": 0, "ease": "linear" }
    ] }))
    .unwrap()
}

fn view(yaw: f64, pitch: f64, distance: f64, height: f64) -> CameraView {
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
    let forward: Vec<CameraView> = [0.0, 15.0, 30.0, 45.0, 60.0]
        .map(|f| camera_at(Some(&p), f, false))
        .to_vec();
    let mut backward: Vec<CameraView> = [60.0, 45.0, 30.0, 15.0, 0.0]
        .map(|f| camera_at(Some(&p), f, false))
        .to_vec();
    backward.reverse();
    assert_eq!(backward, forward);
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

#[test]
fn maps_frames_through_remaps_and_back() {
    let fast = vec![(0.0, 0.0), (10.0, 40.0), (20.0, 50.0)];
    assert_eq!(
        [
            remap_frame(&fast, 5.0),
            remap_frame(&fast, 15.0),
            remap_frame(&fast, 25.0)
        ],
        [20.0, 45.0, 55.0]
    );
    let remap = remap_from_segments(
        100.0,
        &[RemapSegment {
            from: 20.0,
            to: 60.0,
            rate: 4.0,
        }],
    );
    assert_eq!(
        (remap_duration(&remap), remap_frame(&remap, 30.0)),
        (70.0, 60.0)
    );
    assert_eq!(
        [
            inverse_remap(&fast, 20.0, RemapSide::Start),
            inverse_remap(&fast, 20.0, RemapSide::End)
        ],
        [5.0, 5.0]
    );
    assert_eq!(
        [
            inverse_remap(&fast, 55.0, RemapSide::Start),
            inverse_remap(&fast, 55.0, RemapSide::End)
        ],
        [25.0, 25.0]
    );
    let held = [(0.0, 0.0), (10.0, 10.0), (20.0, 10.0), (30.0, 20.0)];
    assert_eq!(
        [
            inverse_remap(&held, 10.0, RemapSide::Start),
            inverse_remap(&held, 10.0, RemapSide::End)
        ],
        [10.0, 20.0]
    );
    assert_eq!(
        inverse_remap(&[(5.0, 10.0), (15.0, 20.0)], 4.0, RemapSide::Start),
        -1.0
    );
}
