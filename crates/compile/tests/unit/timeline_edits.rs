use opsis_compile::edits::timeline::{
    HandleKind, Outcome, TimelineEdits, apply_removal, outer_handle,
};
use opsis_compile::ids::SaltedIds;
use opsis_compile::registry::registry;
use opsis_compile::timeline::flatten::{AudioBar, ChainLink, FlatTimeline, VideoBar};
use opsis_types::edits::TimelineEdit;
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit};
use opsis_types::workflow::{PortType, Workflow};
use serde_json::{Value, json};

use crate::helpers::{Graph, TITLE, film, flat_film, near};

const DELTA: f64 = 6.0;
const FED_FAMILY_INPUTS: usize = 6;
const SOURCE: &str = "out";

fn run(
    wf: &Workflow,
    change: impl FnOnce(&mut TimelineEdits<'_>, &FlatTimeline) -> Outcome,
) -> (Outcome, Workflow) {
    let flat = flat_film(wf);
    let mut ids = SaltedIds::new("t");
    let mut edits = TimelineEdits {
        workflow: wf.clone(),
        fps: wf.settings.fps,
        source: Some(SOURCE.to_owned()),
        ids: &mut ids,
    };
    let outcome = change(&mut edits, &flat);
    (outcome, edits.workflow)
}

fn applied(
    wf: &Workflow,
    change: impl FnOnce(&mut TimelineEdits<'_>, &FlatTimeline) -> Outcome,
) -> Workflow {
    let (outcome, next) = run(wf, change);
    outcome.unwrap_or_else(|e| panic!("the edit failed: {e}"));
    next
}

fn refused(
    wf: &Workflow,
    change: impl FnOnce(&mut TimelineEdits<'_>, &FlatTimeline) -> Outcome,
) -> bool {
    run(wf, change).0.is_err()
}

fn bar_of<'a>(flat: &'a FlatTimeline, node: &str) -> &'a VideoBar {
    flat.bars
        .iter()
        .find(|b| b.node_id == node)
        .unwrap_or_else(|| panic!("no bar for {node}"))
}

fn audio_of<'a>(flat: &'a FlatTimeline, node: &str) -> &'a AudioBar {
    flat.audio
        .iter()
        .find(|a| a.node_id.as_deref() == Some(node))
        .unwrap_or_else(|| panic!("no sound for {node}"))
}

fn len(from: f64, to: f64) -> f64 {
    to - from
}

fn by_start(flat: &FlatTimeline) -> Vec<(f64, f64)> {
    let mut spans: Vec<(f64, f64)> = flat
        .bars
        .iter()
        .map(|b| (b.from.round(), b.to.round()))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    spans
}

fn param<'a>(wf: &'a Workflow, kind: &str, key: &str) -> Option<&'a Value> {
    wf.nodes
        .iter()
        .find(|n| n.kind == kind)
        .and_then(|n| n.params.get(key))
}

fn count(wf: &Workflow, kind: &str) -> usize {
    wf.nodes.iter().filter(|n| n.kind == kind).count()
}

fn sequence_of(clips: &[(&str, u32)], extra: Value) -> Workflow {
    film(|g| {
        for (i, (id, frames)) in clips.iter().enumerate() {
            g.node(
                id,
                TITLE,
                json!({ "title": id.to_uppercase(), "clipDuration": frames }),
            );
            g.link_ports(id, "seq", "clip", &format!("in{}", i + 1));
        }
        let mut params = json!({ "transition": "none" });
        if let (Some(all), Value::Object(more)) = (params.as_object_mut(), extra) {
            all.extend(more);
        }
        g.node("seq", "timeline.sequence", params);
        "seq".into()
    })
}

fn single() -> Workflow {
    film(|g| g.node("t", TITLE, json!({ "title": "Hello", "clipDuration": 90 })))
}

const HANDLES: [HandleKind; 4] = [
    HandleKind::Move,
    HandleKind::TrimStart,
    HandleKind::TrimEnd,
    HandleKind::Speed,
];

fn act(
    edits: &mut TimelineEdits<'_>,
    kind: HandleKind,
    bar: &VideoBar,
    link: &ChainLink,
) -> Outcome {
    match kind {
        HandleKind::Move => edits.move_bar(bar, DELTA),
        HandleKind::TrimStart => edits.trim_bar(bar, true, DELTA),
        HandleKind::TrimEnd => edits.trim_bar(bar, false, -DELTA),
        HandleKind::Speed => edits.speed_bar(bar, (len(link.from, link.to) / 2.0).round()),
    }
}

fn expected(kind: HandleKind, link: &ChainLink) -> f64 {
    match kind {
        HandleKind::Speed => (len(link.from, link.to) / 2.0).round(),
        _ => DELTA,
    }
}

fn measured(kind: HandleKind, before: &VideoBar, after: &VideoBar) -> f64 {
    match kind {
        HandleKind::Move => after.from - before.from,
        HandleKind::TrimStart | HandleKind::TrimEnd => {
            len(before.from, before.to) - len(after.from, after.to)
        }
        HandleKind::Speed => len(after.from, after.to),
    }
}

fn tolerance(kind: HandleKind) -> f64 {
    if kind == HandleKind::Speed { 1.0 } else { 0.5 }
}

fn handle_ref(link: &ChainLink, kind: HandleKind) -> Option<&ParamRef> {
    let h = link.handles.as_ref()?;
    match kind {
        HandleKind::Move => h.move_to.as_ref(),
        HandleKind::TrimStart => h.trim_start.as_ref(),
        HandleKind::TrimEnd => h.trim_end.as_ref(),
        HandleKind::Speed => h.speed.as_ref(),
    }
}

struct RoundTrip {
    kind: HandleKind,
    owner: String,
    off: f64,
    added: usize,
}

fn feed_inputs(g: &mut Graph, id: &str, kind: &str, family_inputs: usize) {
    let def = registry().get(kind).expect("a node definition");
    let mut ports: Vec<(String, PortType)> = def
        .inputs
        .iter()
        .map(|(port, spec)| ((*port).to_owned(), spec.kind))
        .collect();
    if let Some(family) = &def.family {
        ports.extend((0..family_inputs).map(|i| (family.port(i), family.kind())));
    }
    for (i, (port, kind)) in ports.iter().enumerate() {
        let port = port.as_str();
        let src = format!("{id}_{port}");
        match kind {
            PortType::Clip => {
                g.node(
                    &src,
                    TITLE,
                    json!({ "title": format!("Clip {}", i + 1), "clipDuration": 90 + i * 15 }),
                );
                g.link_ports(&src, id, "clip", port);
            }
            PortType::Media => {
                g.node(&src, "media.import", json!({ "path": "assets/clip.mp4" }));
                g.link_ports(&src, id, "media", port);
            }
            _ => {}
        }
    }
}

fn round_trips(kind: &str, params: Value) -> Vec<RoundTrip> {
    round_trips_fed(kind, params, FED_FAMILY_INPUTS)
}

fn round_trips_fed(kind: &str, params: Value, family_inputs: usize) -> Vec<RoundTrip> {
    let wf = film(|g| {
        g.node("x", kind, params);
        feed_inputs(g, "x", kind, family_inputs);
        "x".into()
    });
    let before = flat_film(&wf);
    let mut trips = Vec::new();
    for bar in &before.bars {
        for handle in HANDLES {
            let Some(owner) = outer_handle(bar, handle) else {
                continue;
            };
            let Some(node) = handle_ref(owner, handle).and_then(|r| wf.node(&r.node_id)) else {
                continue;
            };
            let next = applied(&wf, |e, _| act(e, handle, bar, owner));
            let after_flat = flat_film(&next);
            let after = after_flat
                .bars
                .iter()
                .find(|b| b.key == bar.key)
                .unwrap_or_else(|| panic!("{kind} bar after its {handle:?} edit"));
            trips.push(RoundTrip {
                kind: handle,
                owner: node.kind.clone(),
                off: measured(handle, bar, after) - expected(handle, owner),
                added: next.nodes.len() - wf.nodes.len(),
            });
        }
    }
    trips
}

fn clip_kinds() -> Vec<String> {
    registry()
        .all()
        .iter()
        .filter(|d| d.has_compile() && d.output("clip").is_some() && !d.hidden)
        .map(|d| d.kind.to_string())
        .collect()
}

#[test]
fn every_handle_moves_its_bar_in_a_sequence_of_twelve_clips() {
    let trips = round_trips_fed("timeline.sequence", json!({}), 12);
    assert!(trips.len() >= 12, "{} round trips", trips.len());
    let misses: Vec<String> = trips
        .iter()
        .filter(|t| t.added != 0 || t.off.abs() > tolerance(t.kind))
        .map(|t| format!("{:?} owned by {}: off by {}", t.kind, t.owner, t.off))
        .collect();
    assert_eq!(misses, Vec::<String>::new());
}

#[test]
fn every_handle_moves_its_bar_by_the_delta() {
    let mut misses = Vec::new();
    let mut owners: Vec<(HandleKind, String)> = Vec::new();
    for kind in clip_kinds() {
        for trip in round_trips(&kind, json!({})) {
            if trip.added != 0 || trip.off.abs() > tolerance(trip.kind) {
                misses.push(format!(
                    "{kind} {:?}: off by {} frames, {} nodes added",
                    trip.kind, trip.off, trip.added
                ));
            }
            owners.push((trip.kind, trip.owner));
        }
    }
    assert_eq!(misses, Vec::<String>::new());
    let owned =
        |kind: HandleKind, owner: &str| owners.iter().any(|(k, o)| *k == kind && o == owner);
    assert!(owned(HandleKind::Move, "timeline.delay"));
    assert!(owned(HandleKind::TrimStart, "timeline.trim"));
    assert!(owned(HandleKind::TrimEnd, "timeline.trim"));
    assert!(owned(HandleKind::TrimEnd, TITLE));
    assert!(owned(HandleKind::Speed, "timeline.speed"));
}

#[test]
fn trims_an_image_media_file_through_its_length() {
    let image = round_trips(
        "media.import",
        json!({ "path": "assets/still.png", "stillMs": 4000 }),
    );
    assert!(
        image
            .iter()
            .any(|t| t.kind == HandleKind::TrimEnd && t.owner == "media.import" && t.added == 0)
    );
    assert!(image.iter().all(|t| t.off.abs() <= tolerance(t.kind)));
}

#[test]
fn adds_a_delay_to_move_a_clip_that_has_none() {
    let wf = single();
    let (outcome, next) = run(&wf, |e, flat| e.move_bar(&flat.bars[0], 12.0));
    let done = outcome.unwrap();
    assert!(
        next.nodes.iter().any(
            |n| n.kind == "timeline.delay" && n.label.as_deref() == Some("Delay from timeline")
        )
    );
    assert!(near(flat_film(&next).bars[0].from, 12.0, 0.5));
    assert!(done.message.unwrap().contains("Delay from timeline"));
}

#[test]
fn moves_a_sped_up_clip_by_the_frames_dragged() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 90 }));
        g.node("speed", "timeline.speed", json!({ "rate": 2 }));
        g.link("t", "speed");
        "speed".into()
    });
    let next = applied(&wf, |e, flat| e.move_bar(bar_of(flat, "t"), 12.0));
    assert_eq!(param(&next, "timeline.delay", "ms"), Some(&json!(400)));
    assert!(near(bar_of(&flat_film(&next), "t").from, 12.0, 0.5));
}

#[test]
fn adds_a_trim_to_trim_the_start_of_a_clip_that_has_none() {
    let wf = single();
    let next = applied(&wf, |e, flat| e.trim_bar(&flat.bars[0], true, 30.0));
    assert_eq!(count(&next, "timeline.trim"), 1);
    let bar = &flat_film(&next).bars[0];
    assert!(near(len(bar.from, bar.to), 60.0, 0.5));
}

#[test]
fn adds_a_speed_to_stretch_a_clip() {
    let wf = single();
    let next = applied(&wf, |e, flat| e.speed_bar(&flat.bars[0], 45.0));
    assert_eq!(param(&next, "timeline.speed", "rate"), Some(&json!(2)));
    let bar = &flat_film(&next).bars[0];
    assert!(near(len(bar.from, bar.to), 45.0, 0.5));
}

#[test]
fn speeds_a_clip_up_to_64_times_and_no_further() {
    let wf = single();
    let rate = |w: &Workflow| param(w, "timeline.speed", "rate").cloned();
    let fast = applied(&wf, |e, flat| e.speed_bar(&flat.bars[0], 2.0));
    assert_eq!(rate(&fast), Some(json!(45)));
    let faster = applied(&fast, |e, flat| e.speed_bar(&flat.bars[0], 1.0));
    assert_eq!(rate(&faster), Some(json!(64)));
    let once = applied(&wf, |e, flat| e.speed_bar(&flat.bars[0], 1.0));
    assert_eq!(rate(&once), Some(json!(64)));
}

#[test]
fn slips_the_content_without_changing_the_length() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 90 }));
        g.node(
            "trim",
            "timeline.trim",
            json!({ "startMs": 0, "durationMs": 1000 }),
        );
        g.link("t", "trim");
        "trim".into()
    });
    let before = flat_film(&wf);
    let trimmed = &before.bars[0];
    assert!(near(len(trimmed.from, trimmed.to), 30.0, 0.5));
    let next = applied(&wf, |e, flat| e.slip_bar(&flat.bars[0], 10.0));
    let slipped = flat_film(&next).bars.remove(0);
    assert!(near(len(slipped.from, slipped.to), 30.0, 0.5));
    assert!(near(
        slipped.content_start - trimmed.content_start,
        10.0,
        0.5
    ));
}

#[test]
fn adds_a_trim_to_slip_a_clip_that_has_none() {
    let wf = single();
    let next = applied(&wf, |e, flat| e.slip_bar(&flat.bars[0], 10.0));
    assert_eq!(count(&next, "timeline.trim"), 1);
}

#[test]
fn wont_move_a_sequenced_clip_before_the_one_before_it() {
    let wf = sequence_of(&[("a", 60), ("b", 60)], json!({}));
    assert!(refused(&wf, |e, flat| e.move_bar(bar_of(flat, "b"), -10.0)));
}

#[test]
fn splits_a_clip_at_the_playhead_into_two_parts_in_a_sequence() {
    let wf = single();
    let next = applied(&wf, |e, flat| e.split_bar(&flat.bars[0], 30.0));
    let after = flat_film(&next);
    assert_eq!(by_start(&after), [(0.0, 30.0), (30.0, 90.0)]);
    let mut bars = after.bars.clone();
    bars.sort_by(|a, b| a.from.total_cmp(&b.from));
    assert!(near(bars[1].content_start, 30.0, 0.5));
    assert_eq!(
        param(&next, "timeline.sequence", "transition"),
        Some(&json!("none"))
    );
}

#[test]
fn splits_inside_a_cut_only_sequence_without_nesting_another_one() {
    let wf = sequence_of(&[("a", 60), ("b", 60)], json!({}));
    let next = applied(&wf, |e, flat| e.split_bar(bar_of(flat, "a"), 20.0));
    assert_eq!(count(&next, "timeline.sequence"), 1);
    assert_eq!(
        by_start(&flat_film(&next)),
        [(0.0, 20.0), (20.0, 60.0), (60.0, 120.0)]
    );
}

#[test]
fn splits_in_place_inside_a_sequence_with_a_transition_keeping_each_cut_on_its_clip() {
    let wf = sequence_of(
        &[("a", 60), ("b", 60), ("c", 60)],
        json!({
            "transition": "fade",
            "transitionDuration": 10,
            "cuts": [{ "before": "in3", "transition": "wipe", "transitionDuration": 20, "keepWhole": false }],
        }),
    );
    let next = applied(&wf, |e, flat| e.split_bar(bar_of(flat, "a"), 20.0));
    assert_eq!(count(&next, "timeline.sequence"), 1);
    assert_eq!(
        param(&next, "timeline.sequence", "cuts"),
        Some(&json!([
            { "before": "in4", "transition": "wipe", "transitionDuration": 20, "keepWhole": false },
            { "before": "in2", "transition": "none", "transitionDuration": 15, "keepWhole": false, "easing": "linear" },
        ]))
    );
    let after = flat_film(&next);
    let marks: Vec<(String, Option<String>)> = after
        .transitions
        .iter()
        .map(|t| (t.kind.clone(), t.slot.as_ref().map(|s| s.port.clone())))
        .collect();
    assert_eq!(
        marks,
        [
            ("fade".to_owned(), Some("in3".to_owned())),
            ("wipe".to_owned(), Some("in4".to_owned())),
        ]
    );
    assert_eq!(
        by_start(&after),
        [(0.0, 20.0), (20.0, 60.0), (50.0, 110.0), (90.0, 150.0)]
    );
}

#[test]
fn keeps_a_cuts_own_transition_when_rolling_it() {
    let wf = sequence_of(
        &[("a", 60), ("b", 60)],
        json!({ "cuts": [{ "before": "in2", "transition": "wipe", "transitionDuration": 10, "keepWhole": true }] }),
    );
    let next = applied(&wf, |e, flat| {
        e.roll_cut(bar_of(flat, "a"), bar_of(flat, "b"), 12.0)
    });
    let cuts = |w: &Workflow| w.node("seq").and_then(|n| n.params.get("cuts")).cloned();
    assert_eq!(cuts(&next), cuts(&wf));
    let kinds: Vec<String> = flat_film(&next)
        .transitions
        .iter()
        .map(|t| t.kind.clone())
        .collect();
    assert_eq!(kinds, ["wipe"]);
}

#[test]
fn rolls_the_cut_between_two_clips_without_changing_the_total() {
    let wf = sequence_of(&[("a", 60), ("b", 60)], json!({}));
    let next = applied(&wf, |e, flat| {
        e.roll_cut(bar_of(flat, "a"), bar_of(flat, "b"), 12.0)
    });
    let after = flat_film(&next);
    assert!(near(bar_of(&after, "a").to, 72.0, 0.5));
    assert!(near(bar_of(&after, "b").from, 72.0, 0.5));
    assert!(near(bar_of(&after, "b").to, 120.0, 0.5));
    assert!(refused(&wf, |e, flat| {
        e.roll_cut(bar_of(flat, "a"), bar_of(flat, "b"), -12.0)
    }));
}

#[test]
fn rolls_through_the_neighbour_of_a_dragged_edge() {
    let wf = sequence_of(&[("a", 60), ("b", 60)], json!({}));
    let key = bar_of(&flat_film(&wf), "b").key.clone();
    let next = applied(&wf, |e, flat| {
        e.apply(
            TimelineEdit::RollStart {
                bar: key,
                delta: 12.0,
            },
            flat,
        )
    });
    let after = flat_film(&next);
    assert!(near(bar_of(&after, "a").to, 72.0, 0.5));
    assert!(near(bar_of(&after, "b").to, 120.0, 0.5));
}

#[test]
fn removes_the_nodes_that_only_feed_one_clip() {
    let wf = film(|g| {
        g.node("a", TITLE, json!({ "clipDuration": 60 }));
        g.node(
            "trim",
            "timeline.trim",
            json!({ "startMs": 0, "durationMs": 1000 }),
        );
        g.node("b", TITLE, json!({ "clipDuration": 60 }));
        g.node("seq", "timeline.sequence", json!({ "transition": "none" }));
        g.link("a", "trim")
            .link_ports("trim", "seq", "clip", "in1")
            .link_ports("b", "seq", "clip", "in2");
        "seq".into()
    });
    let flat = flat_film(&wf);
    let mut ids = SaltedIds::new("t");
    let edits = TimelineEdits {
        workflow: wf.clone(),
        fps: wf.settings.fps,
        source: Some(SOURCE.to_owned()),
        ids: &mut ids,
    };
    let removal = edits.selection_removal(&flat, &["a".to_owned()]).unwrap();
    assert_eq!(removal.node_ids, ["a", "trim"]);
    let mut removed = wf.clone();
    apply_removal(&removal, &mut removed);
    assert!(removed.node("a").is_none() && removed.node("trim").is_none());
}

fn with_sound() -> Workflow {
    film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 300 }));
        g.node(
            "music",
            "audio.music",
            json!({ "src": "assets/song.mp3", "startMs": 1000, "fadeInMs": 0 }),
        );
        g.node(
            "fx",
            "audio.sfx",
            json!({ "src": "app:sfx/whoosh.wav", "atMs": 2000 }),
        );
        g.node("mix", "audio.mix", json!({}));
        g.link("t", "mix")
            .link_ports("music", "mix", "audio", "sounds")
            .link_ports("fx", "mix", "audio", "sounds");
        "mix".into()
    })
}

#[test]
fn moves_trims_and_sets_the_volume_of_music_and_sound_effects() {
    let wf = with_sound();
    let before = flat_film(&wf);
    let music = audio_of(&before, "music");
    let fx = audio_of(&before, "fx");
    assert_eq!((music.from, fx.from), (30.0, 60.0));

    let moved = applied(&wf, |e, flat| e.move_audio(audio_of(flat, "fx"), DELTA));
    assert_eq!(audio_of(&flat_film(&moved), "fx").from - fx.from, DELTA);

    let trimmed = applied(&wf, |e, flat| {
        e.trim_audio(audio_of(flat, "music"), true, DELTA)
    });
    let after = flat_film(&trimmed);
    let trimmed_music = audio_of(&after, "music");
    assert_eq!(trimmed_music.from - music.from, DELTA);
    assert_eq!(trimmed_music.trim_before - music.trim_before, DELTA);
    assert_eq!(
        len(music.from, music.to) - len(trimmed_music.from, trimmed_music.to),
        DELTA
    );

    let ended = applied(&wf, |e, flat| {
        e.trim_audio(audio_of(flat, "music"), false, -DELTA)
    });
    let after = flat_film(&ended);
    let ended_music = audio_of(&after, "music");
    assert_eq!(
        len(music.from, music.to) - len(ended_music.from, ended_music.to),
        DELTA
    );

    let louder = applied(&wf, |e, flat| e.set_audio_volume(audio_of(flat, "fx"), 1.5));
    assert_eq!(
        louder.node("fx").and_then(|n| n.params.get("volume")),
        Some(&json!(1.5))
    );
}

#[test]
fn doesnt_trim_sound_effects() {
    let wf = with_sound();
    assert!(refused(&wf, |e, flat| {
        e.trim_audio(audio_of(flat, "fx"), false, -3.0)
    }));
}

#[test]
fn moves_through_a_new_node_when_the_move_settings_node_is_gone() {
    let wf = sequence_of(&[("a", 60), ("b", 60)], json!({}));
    let flat = flat_film(&wf);
    let mut bar = bar_of(&flat, "b").clone();
    let owner = bar
        .chain
        .iter()
        .rposition(|l| l.handles.as_ref().is_some_and(|h| h.trim_end.is_some()))
        .expect("a trim end handle");
    let handles = bar.chain[owner].handles.clone().unwrap_or_default();
    bar.chain[owner].handles = Some(EditHandles {
        move_to: Some(ParamRef {
            node_id: "gone".into(),
            param: "ms".into(),
            unit: ParamUnit::Ms,
            scale: None,
            index: None,
            linked: None,
        }),
        ..handles
    });
    let moved = applied(&wf, |e, _| e.move_bar(&bar, DELTA));
    assert_eq!(count(&moved, "timeline.delay"), 1);
}

fn camera_keys() -> Value {
    json!({ "keys": [
        { "frame": 0, "yaw": -30, "pitch": 0, "distance": 1.5, "height": 0, "ease": "in-out" },
        { "frame": 60, "yaw": 0, "pitch": 0, "distance": 1, "height": 0, "ease": "in-out" },
    ] })
}

fn three_film(kind: &str, preset: Value) -> Workflow {
    film(|g| {
        g.node("intro", TITLE, json!({ "title": "Hi", "clipDuration": 30 }));
        let mut params = json!({ "camera": camera_keys(), "clipDuration": 90 });
        if let (Some(all), Value::Object(more)) = (params.as_object_mut(), preset) {
            all.extend(more);
        }
        g.node("three", kind, params);
        g.node("seq", "timeline.sequence", json!({ "transition": "none" }));
        g.link_ports("intro", "seq", "clip", "in1")
            .link_ports("three", "seq", "clip", "in2");
        "seq".into()
    })
}

fn key_frames(flat: &FlatTimeline) -> Vec<f64> {
    flat.camera_keys.iter().map(|m| m.frame.round()).collect()
}

#[test]
fn shows_the_keys_of_a_keyframed_3d_element_where_they_play() {
    let flat = flat_film(&three_film("fx.logo-3d", json!({ "motion": "keyframes" })));
    let marks: Vec<(String, u32, f64)> = flat
        .camera_keys
        .iter()
        .map(|m| (m.node_id.clone(), m.index, m.frame.round()))
        .collect();
    assert_eq!(
        marks,
        [("three".to_owned(), 0, 30.0), ("three".to_owned(), 1, 90.0)]
    );
}

#[test]
fn shows_no_keys_while_a_preset_move_drives_the_camera() {
    let flat = flat_film(&three_film("fx.logo-3d", json!({ "motion": "spin" })));
    assert!(flat.camera_keys.is_empty());
}

#[test]
fn moves_a_key_by_the_drag_and_lands_where_it_was_dropped() {
    let wf = three_film("fx.logo-3d", json!({ "motion": "keyframes" }));
    let moved = applied(&wf, |e, flat| {
        e.move_camera_key(&flat.camera_keys[1], -15.0)
    });
    assert_eq!(key_frames(&flat_film(&moved)), [30.0, 75.0]);
}

#[test]
fn shows_and_moves_the_keys_of_a_3d_screen_around_a_clip() {
    let wf = film(|g| {
        g.node("intro", TITLE, json!({ "title": "Hi", "clipDuration": 90 }));
        g.node(
            "screen",
            "fx.screen-3d",
            json!({ "move": "keyframes", "camera": camera_keys() }),
        );
        g.link("intro", "screen");
        "screen".into()
    });
    let flat = flat_film(&wf);
    assert_eq!(key_frames(&flat), [0.0, 60.0]);
    let moved = applied(&wf, |e, flat| e.move_camera_key(&flat.camera_keys[0], 20.0));
    assert_eq!(key_frames(&flat_film(&moved)), [20.0, 60.0]);
}

#[test]
fn ripples_the_sounds_after_a_moved_clip() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 300 }));
        g.node("delay", "timeline.delay", json!({ "ms": 0 }));
        g.node("layers", "timeline.stack", json!({}));
        g.node(
            "fx",
            "audio.sfx",
            json!({ "src": "app:sfx/whoosh.wav", "atMs": 2000 }),
        );
        g.node("mix", "audio.mix", json!({}));
        g.link("t", "delay")
            .link_ports("delay", "layers", "clip", "in1")
            .link("layers", "mix")
            .link_ports("fx", "mix", "audio", "sounds");
        "mix".into()
    });
    let before = flat_film(&wf);
    let key = bar_of(&before, "t").key.clone();
    let next = applied(&wf, |e, flat| {
        e.apply(
            TimelineEdit::Move {
                bar: key,
                delta: 30.0,
                ripple: true,
            },
            flat,
        )
    });
    let after = flat_film(&next);
    assert!(near(bar_of(&after, "t").from, 30.0, 0.5));
    assert_eq!(
        audio_of(&after, "fx").from,
        audio_of(&before, "fx").from + 30.0
    );
}

#[test]
fn ripples_the_sounds_of_a_moved_film_only_once() {
    let wf = with_sound();
    let before = flat_film(&wf);
    let key = bar_of(&before, "t").key.clone();
    let next = applied(&wf, |e, flat| {
        e.apply(
            TimelineEdit::Move {
                bar: key,
                delta: 30.0,
                ripple: true,
            },
            flat,
        )
    });
    let after = flat_film(&next);
    assert!(near(bar_of(&after, "t").from, 30.0, 0.5));
    assert_eq!(
        audio_of(&after, "fx").from,
        audio_of(&before, "fx").from + 30.0
    );
    assert_eq!(
        audio_of(&after, "music").from,
        audio_of(&before, "music").from + 30.0
    );
}
