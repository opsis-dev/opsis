use opsis_compile::edits::EditError;
use opsis_compile::edits::keyframes::apply_keyframe_edit;
use opsis_compile::edits::timeline::TimelineEdits;
use opsis_compile::envelope::envelope_track;
use opsis_compile::ids::SaltedIds;
use opsis_types::edits::{KeyframeEdit, TimelineEdit};
use opsis_types::timeline::{CameraPath, Ease, EnvelopeKey, Keyframe};
use opsis_types::workflow::Workflow;
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::helpers::{TITLE, film, flat_film};

fn titled(params: Value) -> Workflow {
    film(|g| {
        g.node("t", TITLE, params);
        "t".into()
    })
}

fn keys(wf: &Workflow, track: &str) -> Vec<Keyframe> {
    let raw = wf.node("t").unwrap().params.get("envelope").cloned();
    raw.and_then(|e| e.get(track).cloned())
        .map_or_default(|k| serde_json::from_value(k).unwrap())
}

fn edited(wf: &Workflow, edit: KeyframeEdit) -> Workflow {
    let mut next = wf.clone();
    apply_keyframe_edit(&mut next, edit).unwrap();
    next
}

fn key(frame: f64, value: f64) -> Keyframe {
    Keyframe {
        frame,
        value,
        ease: Ease::Linear,
    }
}

#[test]
fn adds_a_key_a_second_after_the_last_or_at_rest() {
    let wf = titled(json!({ "title": "Hi" }));
    let append = || KeyframeEdit::AppendKey {
        node_id: "t".into(),
        track: EnvelopeKey::Opacity,
    };
    let once = edited(&wf, append());
    assert_eq!(keys(&once, "opacity"), [key(0.0, 1.0)]);
    let twice = edited(&once, append());
    assert_eq!(keys(&twice, "opacity"), [key(0.0, 1.0), key(30.0, 1.0)]);
}

#[test]
fn adds_a_key_on_the_line_where_it_is_placed() {
    let wf = titled(json!({ "envelope": { "x": [
        { "frame": 0, "value": 0, "ease": "linear" },
        { "frame": 20, "value": 1, "ease": "linear" },
    ] } }));
    let added = edited(
        &wf,
        KeyframeEdit::AddKey {
            node_id: "t".into(),
            track: EnvelopeKey::X,
            frame: 5.4,
        },
    );
    assert_eq!(keys(&added, "x")[1], key(5.0, 0.25));
}

const VISUAL: [EnvelopeKey; 4] = [
    EnvelopeKey::Opacity,
    EnvelopeKey::X,
    EnvelopeKey::Y,
    EnvelopeKey::Scale,
];

fn ease() -> impl Strategy<Value = Ease> {
    prop::sample::select(vec![
        Ease::Linear,
        Ease::In,
        Ease::Out,
        Ease::InOut,
        Ease::Hold,
    ])
}

fn keyframe_edit() -> impl Strategy<Value = KeyframeEdit> {
    let track = prop::sample::select(VISUAL.to_vec());
    let node_id = || "t".to_owned();
    prop_oneof![
        (track.clone(), -100.0..1000.0f64).prop_map(move |(track, frame)| KeyframeEdit::AddKey {
            node_id: node_id(),
            track,
            frame
        }),
        track
            .clone()
            .prop_map(move |track| KeyframeEdit::AppendKey {
                node_id: node_id(),
                track
            }),
        (
            track.clone(),
            0u32..6,
            prop::option::of(-100.0..1000.0f64),
            prop::option::of(-10.0..10.0f64),
            prop::option::of(ease()),
        )
            .prop_map(
                move |(track, index, frame, value, ease)| KeyframeEdit::SetKey {
                    node_id: node_id(),
                    track,
                    index,
                    frame,
                    value,
                    ease
                }
            ),
        (track, 0u32..6).prop_map(move |(track, index)| KeyframeEdit::RemoveKey {
            node_id: node_id(),
            track,
            index
        }),
    ]
}

proptest! {
    #[test]
    fn keeps_values_in_range_frames_whole_and_keys_in_order(edits in prop::collection::vec(keyframe_edit(), 1..25)) {
        let mut wf = titled(json!({ "title": "Hi" }));
        for edit in edits {
            let before = wf.clone();
            if apply_keyframe_edit(&mut wf, edit).is_err() {
                prop_assert_eq!(&wf, &before);
            }
            for track in VISUAL {
                let range = envelope_track(track);
                let list = keys(&wf, track.name());
                prop_assert!(list.iter().all(|k| k.frame >= 0.0 && k.frame.fract() == 0.0));
                prop_assert!(list.iter().all(|k| (range.min..=range.max).contains(&k.value)));
                prop_assert!(list.windows(2).all(|w| w[0].frame <= w[1].frame));
            }
        }
    }
}

#[test]
fn moves_a_key_past_the_next_one_with_its_value_clamped() {
    let wf = titled(json!({ "envelope": { "scale": [
        { "frame": 0, "value": 1, "ease": "linear" },
        { "frame": 30, "value": 2, "ease": "linear" },
    ] } }));
    let set = edited(
        &wf,
        KeyframeEdit::SetKey {
            node_id: "t".into(),
            track: EnvelopeKey::Scale,
            index: 0,
            frame: Some(45.6),
            value: Some(9.0),
            ease: Some(Ease::Hold),
        },
    );
    assert_eq!(
        keys(&set, "scale"),
        [
            key(30.0, 2.0),
            Keyframe {
                frame: 46.0,
                value: 3.0,
                ease: Ease::Hold
            }
        ]
    );
}

#[test]
fn drops_a_track_when_its_last_key_goes() {
    let wf = titled(json!({ "envelope": {
        "opacity": [{ "frame": 0, "value": 0.5, "ease": "linear" }],
        "x": [{ "frame": 0, "value": 0.2, "ease": "linear" }],
    } }));
    let removed = edited(
        &wf,
        KeyframeEdit::RemoveKey {
            node_id: "t".into(),
            track: EnvelopeKey::Opacity,
            index: 0,
        },
    );
    let envelope = removed.node("t").unwrap().params["envelope"].clone();
    assert_eq!(envelope.get("opacity"), None);
    assert_eq!(keys(&removed, "x"), [key(0.0, 0.2)]);
}

#[test]
fn refuses_keys_it_cannot_read() {
    let mut wf = titled(json!({ "envelope": { "x": "nope" } }));
    let refused = apply_keyframe_edit(
        &mut wf,
        KeyframeEdit::AppendKey {
            node_id: "t".into(),
            track: EnvelopeKey::X,
        },
    );
    assert_eq!(refused, Err(EditError::UnreadableKeys("t".into())));
}

#[test]
fn adds_camera_keys_where_the_camera_is_and_keeps_dials_in_range() {
    let mut wf = film(|g| {
        g.node(
            "t",
            "fx.logo-3d",
            json!({ "motion": "keyframes", "camera": { "keys": [
                { "frame": 0, "yaw": -30, "pitch": 0, "distance": 1, "height": 0, "ease": "linear" },
                { "frame": 60, "yaw": 30, "pitch": 0, "distance": 1, "height": 0, "ease": "linear" },
            ] } }),
        );
        "t".into()
    });
    let camera = |wf: &Workflow| -> CameraPath {
        serde_json::from_value(wf.node("t").unwrap().params["camera"].clone()).unwrap()
    };
    apply_keyframe_edit(
        &mut wf,
        KeyframeEdit::AddCameraKey {
            node_id: "t".into(),
            frame: 30.0,
        },
    )
    .unwrap();
    assert_eq!(camera(&wf).keys[1].yaw, 0.0);
    apply_keyframe_edit(
        &mut wf,
        KeyframeEdit::SetCameraKey {
            node_id: "t".into(),
            index: 1,
            frame: None,
            yaw: Some(400.0),
            pitch: None,
            distance: Some(0.0),
            height: None,
            ease: None,
        },
    )
    .unwrap();
    let key = camera(&wf).keys[1].clone();
    assert_eq!((key.yaw, key.distance), (180.0, 0.3));
    apply_keyframe_edit(
        &mut wf,
        KeyframeEdit::RemoveCameraKey {
            node_id: "t".into(),
            index: 0,
        },
    )
    .unwrap();
    assert_eq!(camera(&wf).keys.len(), 2);
}

#[test]
fn edits_keys_from_the_timeline_too() {
    let wf = titled(json!({ "envelope": { "opacity": [
        { "frame": 0, "value": 1, "ease": "linear" },
    ] } }));
    let flat = flat_film(&wf);
    let mut ids = SaltedIds::new("t");
    let mut edits = TimelineEdits {
        workflow: wf.clone(),
        fps: wf.settings.fps,
        source: Some("out".to_owned()),
        ids: &mut ids,
    };
    edits
        .apply(
            TimelineEdit::Keyframes {
                edit: KeyframeEdit::SetKey {
                    node_id: "t".into(),
                    track: EnvelopeKey::Opacity,
                    index: 0,
                    frame: Some(12.0),
                    value: Some(-1.0),
                    ease: None,
                },
            },
            &flat,
        )
        .unwrap();
    assert_eq!(keys(&edits.workflow, "opacity"), [key(12.0, 0.0)]);
}

#[test]
fn finds_a_key_where_it_is_stored_even_out_of_order() {
    let wf = titled(json!({ "envelope": { "x": [
        { "frame": 20, "value": 0.5, "ease": "linear" },
        { "frame": 0, "value": 0, "ease": "linear" },
    ] } }));
    let set = edited(
        &wf,
        KeyframeEdit::SetKey {
            node_id: "t".into(),
            track: EnvelopeKey::X,
            index: 0,
            frame: None,
            value: Some(0.75),
            ease: None,
        },
    );
    assert_eq!(keys(&set, "x"), [key(0.0, 0.0), key(20.0, 0.75)]);
}
