use opsis_compile::clip::{
    ItemExtra, SequenceSlots, clip_to_spec, empty_clip, group_item, item_clip, mark_layers,
    recording_clip, scene_item_count, scene_sequence, sequence_clips, stack_clips, wrap_clip,
    wrap_sounds,
};
use opsis_compile::recording::default_recording_props;
use opsis_compile::remap::remap_frame;
use opsis_compile::timeline::flatten::{flatten_spec, node_spans};
use opsis_golden::typed;
use opsis_types::timeline::{
    AudioItem, GroupProps, ItemBase, ParamRef, SequencerProps, TimelineItem, TransitionRef,
};
use opsis_types::values::ClipValue;
use opsis_types::workflow::WorkflowSettings;
use serde_json::{Map, Value, json};

fn settings() -> WorkflowSettings {
    WorkflowSettings {
        width: 1920,
        height: 1080,
        fps: 30,
        ..WorkflowSettings::default()
    }
}

fn props(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

fn solid(id: &str, frames: u32) -> ClipValue {
    item_clip(id, "solid", Map::new(), frames, ItemExtra::default())
}

fn fade(kind: &str, frames: u32, hold: bool) -> TransitionRef {
    TransitionRef {
        kind: kind.into(),
        duration_in_frames: frames,
        params: None,
        hold: hold.then_some(true),
        easing: None,
    }
}

fn plain() -> SequenceSlots<'static> {
    SequenceSlots {
        ports: None,
        slot_node: None,
    }
}

fn pop(id: &str) -> AudioItem {
    typed(
        &json!({ "id": format!("{id}:a"), "src": "app:sfx/pop.wav", "from": 0, "volume": 1, "role": "effect" }),
    )
}

fn with_sound(mut clip: ClipValue, id: &str) -> ClipValue {
    clip.audio = vec![pop(id)];
    clip
}

fn group(item: &TimelineItem) -> Option<&GroupProps> {
    match item {
        TimelineItem::Group { props, .. } => Some(props),
        _ => None,
    }
}

fn span(item: &TimelineItem) -> (u32, u32) {
    (item.base().from, item.base().duration_in_frames)
}

#[test]
fn clamps_a_transition_to_the_shorter_clip_and_records_spans_per_node() {
    let seq = sequence_clips(
        "s",
        &[solid("a", 10), solid("b", 100)],
        &[None, Some(fade("fade", 30, false))],
        plain(),
    );
    assert_eq!(
        seq.tracks[0].items[1]
            .base()
            .enter
            .as_ref()
            .unwrap()
            .duration_in_frames,
        9
    );
    assert_eq!(seq.duration_in_frames, 101);
    let spec = clip_to_spec(
        &sequence_clips("s", &[solid("a", 30), solid("b", 30)], &[], plain()),
        &settings(),
    );
    let spans: Vec<(String, f64, f64)> = node_spans(&spec)
        .into_iter()
        .map(|(k, r)| (k, r.from, r.to))
        .collect();
    assert_eq!(
        spans,
        [("a".to_owned(), 0.0, 30.0), ("b".to_owned(), 30.0, 60.0)]
    );
}

#[test]
fn held_transitions_keep_every_clip_whole() {
    let title = |id: &str, f: u32| {
        with_sound(
            item_clip(id, "title", Map::new(), f, ItemExtra::default()),
            id,
        )
    };
    let seq = sequence_clips(
        "seq",
        &[title("a", 45), title("b", 60), title("c", 30)],
        &[
            None,
            Some(fade("fade", 30, true)),
            Some(fade("fade", 10, true)),
        ],
        plain(),
    );
    assert_eq!(seq.duration_in_frames, 45 + 30 + 60 + 10 + 30);
    let items = &seq.tracks[0].items;
    assert_eq!(
        [span(&items[0]), span(&items[1]), span(&items[2])],
        [(0, 75), (45, 100), (135, 40)]
    );
    let remap_a = group(&items[0])
        .and_then(|g| g.remap.clone())
        .unwrap_or_default();
    assert_eq!(
        [remap_frame(&remap_a, 44.0), remap_frame(&remap_a, 74.0)],
        [44.0, 44.0]
    );
    let remap_b = group(&items[1])
        .and_then(|g| g.remap.clone())
        .unwrap_or_default();
    assert_eq!(
        [
            remap_frame(&remap_b, 10.0),
            remap_frame(&remap_b, 30.0),
            remap_frame(&remap_b, 89.0),
            remap_frame(&remap_b, 99.0)
        ],
        [0.0, 0.0, 59.0, 59.0]
    );
    assert_eq!(
        seq.audio.iter().map(|a| a.from).collect::<Vec<_>>(),
        [0, 75, 145]
    );
}

#[test]
fn held_transitions_delay_a_clips_own_sound_and_are_not_flagged() {
    let video = |id: &str, f: u32| {
        item_clip(
            id,
            "video",
            props(json!({ "src": format!("project:{id}.mp4"), "muted": false })),
            f,
            ItemExtra::default(),
        )
    };
    let sounds = |hold: bool| -> Vec<(String, f64, f64, f64)> {
        let seq = sequence_clips(
            "seq",
            &[video("a", 45), video("b", 60)],
            &[None, Some(fade("fade", 30, hold))],
            plain(),
        );
        flatten_spec(&clip_to_spec(&seq, &settings()))
            .audio
            .into_iter()
            .map(|a| (a.channel, a.from, a.to, a.trim_before))
            .collect()
    };
    let row = |c: &str, a: f64, b: f64| (c.to_owned(), a, b, 0.0);
    assert_eq!(sounds(false), [row("a", 0.0, 45.0), row("b", 15.0, 75.0)]);
    assert_eq!(sounds(true), [row("a", 0.0, 45.0), row("b", 75.0, 135.0)]);
    let parts = [
        item_clip("a", "title", Map::new(), 45, ItemExtra::default()),
        item_clip("b", "title", Map::new(), 45, ItemExtra::default()),
    ];
    let covers = |hold: bool| {
        flatten_spec(&clip_to_spec(
            &sequence_clips(
                "seq",
                &parts,
                &[None, Some(fade("fade", 30, hold))],
                plain(),
            ),
            &settings(),
        ))
        .transitions[0]
            .covers
    };
    assert!((covers(false) - 30.0 / 45.0).abs() < 1e-9);
    assert_eq!(covers(true), 0.0);
}

#[test]
fn a_speed_change_drops_a_clips_own_sound() {
    let video = item_clip(
        "a",
        "video",
        props(json!({ "src": "project:a.mp4", "muted": false })),
        60,
        ItemExtra::default(),
    );
    let base: ItemBase = typed(&json!({ "id": "g", "from": 0, "durationInFrames": 30 }));
    let grouped = group_item(
        base,
        GroupProps {
            tracks: video.tracks.clone(),
            remap: Some(vec![(0.0, 0.0), (30.0, 60.0)]),
            ..GroupProps::default()
        },
    );
    let clip = ClipValue {
        tracks: vec![typed(&json!({ "id": "t", "items": [grouped] }))],
        duration_in_frames: 30,
        ..video
    };
    let flat = flatten_spec(&clip_to_spec(&clip, &settings()));
    assert_eq!(
        flat.bars
            .iter()
            .map(|b| (b.node_id.as_str(), b.has_sound))
            .collect::<Vec<_>>(),
        [("a", true)]
    );
    assert!(flat.audio.is_empty());
}

fn sequencer() -> SequencerProps {
    SequencerProps {
        component: "spring-settle".into(),
        props: Map::new(),
        backgrounds: vec![None, None],
    }
}

fn layered(offsets: &[u32], front: ClipValue) -> ClipValue {
    let ports = ["in1".to_owned(), "in2".to_owned()];
    let move_for = |index: usize| -> Option<ParamRef> {
        Some(typed(
            &json!({ "nodeId": "l", "param": "offsetsMs", "unit": "ms", "index": index }),
        ))
    };
    mark_layers(stack_clips(
        "l",
        &[solid("x", 30), front],
        offsets,
        Some(&ports),
        Some(&move_for),
    ))
}

fn scenes(clip: &ClipValue) -> Vec<TimelineItem> {
    group(&clip.tracks[0].items[0])
        .expect("sequencer group")
        .tracks[0]
        .items
        .clone()
}

fn scene(port: &str, clip: ClipValue) -> (String, ClipValue) {
    (port.to_owned(), clip)
}

#[test]
fn plays_scenes_one_after_another_with_the_pause_between_them() {
    let clip = scene_sequence(
        "s",
        &[scene("in1", solid("a", 30)), scene("in3", solid("b", 45))],
        5,
        sequencer(),
    );
    assert_eq!(clip.duration_in_frames, 80);
    assert_eq!(
        group(&clip.tracks[0].items[0]).unwrap().sequencer.as_ref(),
        Some(&sequencer())
    );
    let placed: Vec<(u32, u32, String, String)> = scenes(&clip)
        .iter()
        .map(|s| {
            let slot = s.base().slot.clone().unwrap();
            (
                s.base().from,
                s.base().duration_in_frames,
                slot.node_id,
                slot.port,
            )
        })
        .collect();
    assert_eq!(
        placed,
        [
            (0, 30, "s".into(), "in1".into()),
            (35, 45, "s".into(), "in3".into())
        ]
    );
    let spans = node_spans(&clip_to_spec(&clip, &settings()));
    assert_eq!(
        (
            spans["a"].from,
            spans["a"].to,
            spans["b"].from,
            spans["b"].to
        ),
        (0.0, 30.0, 35.0, 80.0)
    );
}

#[test]
fn counts_scene_items_and_gives_each_layer_its_own_item() {
    let layers = layered(&[0, 12], solid("y", 30));
    assert_eq!(scene_item_count(&solid("p", 30)), 1);
    assert_eq!(scene_item_count(&layers), 2);
    assert_eq!(
        scene_item_count(&wrap_clip(&layers, "w", GroupProps::default(), None)),
        1
    );
    let clip = scene_sequence(
        "s",
        &[scene("in1", solid("p", 30)), scene("in2", layers)],
        0,
        sequencer(),
    );
    let tracks: Vec<usize> = scenes(&clip)
        .iter()
        .map(|s| group(s).unwrap().tracks.len())
        .collect();
    assert_eq!(tracks, [1, 2]);
}

#[test]
fn starts_a_scenes_layers_together_and_keeps_a_length_that_follows_one_layer() {
    let clip = scene_sequence(
        "s",
        &[scene("in1", layered(&[0, 12], solid("y", 30)))],
        0,
        sequencer(),
    );
    let first = &scenes(&clip)[0];
    assert_eq!(first.base().duration_in_frames, 30);
    let layers: Vec<TimelineItem> = group(first)
        .unwrap()
        .tracks
        .iter()
        .map(|t| t.items[0].clone())
        .collect();
    assert_eq!(
        layers.iter().map(|l| l.base().from).collect::<Vec<_>>(),
        [0, 0]
    );
    assert!(layers.iter().all(|l| {
        l.base()
            .handles
            .as_ref()
            .and_then(|h| h.move_to.as_ref())
            .is_none()
    }));
    let slots: Vec<(String, String)> = layers
        .iter()
        .map(|l| l.base().slot.clone().map(|s| (s.node_id, s.port)).unwrap())
        .collect();
    assert_eq!(
        slots,
        [("l".into(), "in1".into()), ("l".into(), "in2".into())]
    );
    let short = ClipValue {
        duration_in_frames: 20,
        ..layered(&[0, 12], solid("y", 30))
    };
    assert_eq!(
        scene_sequence("s", &[scene("in1", short)], 0, sequencer()).duration_in_frames,
        20
    );
}

#[test]
fn moves_a_layers_sound_back_with_it_and_every_scenes_sound_to_its_start() {
    let front = with_sound(solid("y", 30), "y");
    let clip = scene_sequence(
        "s",
        &[
            scene("in1", with_sound(solid("p", 40), "p")),
            scene("in2", layered(&[0, 12], front)),
        ],
        2,
        sequencer(),
    );
    assert_eq!(
        clip.audio.iter().map(|a| a.from).collect::<Vec<_>>(),
        [0, 42]
    );
}

#[test]
fn compiles_built_in_items_to_their_kind() {
    let log = typed(
        &json!({ "viewport": { "width": 1280, "height": 720 }, "dpr": 1, "t0": 0, "durationMs": 1000, "actions": [] }),
    );
    let recording = recording_clip(
        "rec",
        default_recording_props("project:take.mp4".into(), log, "Take".into()),
        30,
        ItemExtra::default(),
    );
    let item = recording.tracks[0].items[0].clone();
    assert!(matches!(item, TimelineItem::Recording { .. }));
    assert!(group(&item).is_none());
    let placeholder = item_clip(
        "p",
        "placeholder",
        props(json!({ "title": "T", "subtitle": "S" })),
        30,
        ItemExtra::default(),
    );
    let wrapped = wrap_clip(&placeholder, "w", GroupProps::default(), None);
    let seq = sequence_clips("s", &[placeholder.clone(), wrapped], &[], plain());
    assert!(
        seq.tracks
            .iter()
            .flat_map(|t| &t.items)
            .all(|i| matches!(i, TimelineItem::Group { .. }))
    );
    assert!(
        matches!(&placeholder.tracks[0].items[0], TimelineItem::Element { component, .. } if component == "placeholder")
    );
}

fn beep(from: i32, frames: Option<u32>) -> AudioItem {
    let mut value = json!({ "id": "b", "src": "project:beeps.wav", "from": from });
    if let Some(f) = frames {
        value["durationInFrames"] = json!(f);
    }
    typed(&value)
}

fn placed(items: Vec<AudioItem>) -> Vec<(i32, Option<u32>, u32, f64)> {
    items
        .into_iter()
        .map(|a| {
            (
                a.from,
                a.duration_in_frames,
                a.trim_before.unwrap_or(0),
                a.rate.unwrap_or(1.0),
            )
        })
        .collect()
}

#[test]
fn wraps_sounds_through_trims_repeats_speed_and_the_clips_end() {
    let start = |f| GroupProps {
        start_from: Some(f),
        ..GroupProps::default()
    };
    assert_eq!(
        placed(wrap_sounds(&[beep(0, Some(120))], &start(30), 90)),
        [(0, Some(90), 30, 1.0)]
    );
    assert_eq!(
        placed(wrap_sounds(&[beep(60, Some(10))], &start(30), 90)),
        [(30, Some(10), 0, 1.0)]
    );
    assert_eq!(wrap_sounds(&[beep(10, Some(10))], &start(30), 90), []);
    let looped = wrap_sounds(
        &[beep(15, Some(10))],
        &GroupProps {
            loop_every: Some(60),
            ..GroupProps::default()
        },
        180,
    );
    assert_eq!(
        looped
            .iter()
            .map(|a| (a.id.as_str(), a.from))
            .collect::<Vec<_>>(),
        [("b", 15), ("b:r1", 75), ("b:r2", 135)]
    );
    let sped = GroupProps {
        remap: Some(vec![(0.0, 0.0), (60.0, 120.0)]),
        ..GroupProps::default()
    };
    assert_eq!(
        placed(wrap_sounds(&[beep(30, Some(60))], &sped, 60)),
        [(15, Some(30), 0, 2.0)]
    );
    assert_eq!(
        placed(wrap_sounds(&[beep(0, None)], &GroupProps::default(), 50)),
        [(0, Some(50), 0, 1.0)]
    );
}

fn spans_of(clips: &[u32], cuts: &[Option<TransitionRef>]) -> (u32, Vec<(u32, u32)>, Vec<u32>) {
    let seq = sequence_clips(
        "s",
        &clips.iter().map(|f| empty_clip(*f)).collect::<Vec<_>>(),
        cuts,
        plain(),
    );
    let items = &seq.tracks[0].items;
    (
        seq.duration_in_frames,
        items
            .iter()
            .map(|i| (i.base().from, i.base().from + i.base().duration_in_frames))
            .collect(),
        items
            .iter()
            .map(|i| i.base().enter.as_ref().map_or(0, |e| e.duration_in_frames))
            .collect(),
    )
}

#[test]
fn never_lets_a_clips_two_transitions_overlap() {
    let (film, items, cuts) = spans_of(
        &[30, 30, 30],
        &[
            None,
            Some(fade("fade", 60, false)),
            Some(fade("fade", 60, false)),
        ],
    );
    assert_eq!((cuts, film), (vec![0, 15, 15], 60));
    let most = (0..film)
        .map(|f| items.iter().filter(|(a, b)| f >= *a && f < *b).count())
        .max()
        .unwrap();
    assert_eq!(most, 2);
    assert_eq!(
        spans_of(&[30, 60], &[None, Some(fade("fade", 60, false))]).2,
        [0, 29]
    );
    assert_eq!(
        spans_of(&[30, 1], &[None, Some(fade("fade", 10, false))]).2,
        [0, 0]
    );
    let (film, _, cuts) = spans_of(&[30, 30], &[None, Some(fade("fade", 45, true))]);
    assert_eq!((cuts, film), (vec![0, 45], 105));
}
