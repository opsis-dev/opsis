use opsis_compile::clip::{item_has_sound, walk_items};
use opsis_compile::compile::compile_workflow;
use opsis_compile::mix::levels::{apply_mix_levels, duck_spans, with_master_stem};
use opsis_compile::mix::plan::{MixPlan, build_mix_plan, channel_identity};
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_types::mix::{Mix, Strip, db_to_gain};
use opsis_types::timeline::{AudioItem, TimelineItem, TimelineSpec};
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};

use crate::helpers::{Graph, TITLE, film, film_spec};

fn project(mix: Option<Mix>) -> (Workflow, TimelineSpec) {
    let mut g = Graph::default();
    g.node("t", TITLE, json!({ "clipDuration": 300 }));
    g.node(
        "clipfile",
        "media.import",
        json!({ "path": "assets/clip.mp4" }),
    );
    g.node(
        "music",
        "audio.music",
        json!({ "src": "assets/song.mp3", "fadeInMs": 0, "fadeOutMs": 0 }),
    );
    g.node(
        "fx",
        "audio.sfx",
        json!({ "src": "app:sfx/whoosh.wav", "atMs": 2000 }),
    );
    g.node("layers", "timeline.stack", json!({}));
    g.node("mix", "audio.mix", json!({}));
    g.node("out", "output.render", json!({}));
    g.link_ports("t", "layers", "clip", "in1")
        .link_ports("clipfile", "layers", "clip", "in2")
        .link("layers", "mix")
        .link_ports("music", "mix", "audio", "sounds")
        .link_ports("fx", "mix", "audio", "sounds")
        .link("mix", "out");
    let mut wf = g.build();
    wf.mix = mix;
    let spec = film_spec(&wf);
    (wf, spec)
}

fn strip(patch: Value) -> Strip {
    serde_json::from_value(patch).unwrap()
}

fn sound(spec: &TimelineSpec, id: &str) -> AudioItem {
    spec.audio
        .iter()
        .find(|a| a.node_id.as_deref() == Some(id))
        .cloned()
        .unwrap_or_else(|| panic!("sound {id}"))
}

fn live(spec: &TimelineSpec, mix: &Mix) -> TimelineSpec {
    apply_mix_levels(spec, Some(mix), &flatten_spec(spec))
}

fn gain(mix: &Mix, id: &str) -> f64 {
    sound(&live(&project(None).1, mix), id).gain.unwrap_or(1.0)
}

fn with_duck(mut mix: Mix, patch: Value) -> Mix {
    for bus in &mut mix.buses {
        if bus.id == "music" {
            let mut duck = serde_json::to_value(&bus.duck).unwrap();
            let base = json!({ "on": true, "keys": ["effects"], "amount": 12, "attack": 100 });
            duck.as_object_mut()
                .unwrap()
                .extend(base.as_object().unwrap().clone());
            duck.as_object_mut()
                .unwrap()
                .extend(patch.as_object().unwrap().clone());
            bus.duck = serde_json::from_value(duck).unwrap();
        }
    }
    mix
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn multiplies_channel_bus_and_master_faders() {
    let mut mix = Mix::default();
    mix.channels
        .insert("fx".into(), strip(json!({ "fader": -6 })));
    mix.buses
        .iter_mut()
        .find(|b| b.id == "effects")
        .unwrap()
        .fader = -6.0;
    mix.master.fader = 6.0;
    assert!(close(gain(&mix, "fx"), db_to_gain(-6.0)));
    assert!(close(gain(&mix, "music"), db_to_gain(6.0)));
}

#[test]
fn mutes_and_solos_channels_and_buses() {
    let mut mix = Mix::default();
    mix.channels
        .insert("fx".into(), strip(json!({ "solo": true })));
    assert_eq!((gain(&mix, "music"), gain(&mix, "fx")), (0.0, 1.0));
    mix.buses.iter_mut().find(|b| b.id == "music").unwrap().solo = true;
    assert_eq!(gain(&mix, "music"), 1.0);
    mix.channels
        .insert("music".into(), strip(json!({ "mute": true })));
    assert_eq!(gain(&mix, "music"), 0.0);
}

#[test]
fn applies_gains_to_sounds_and_clip_sound_in_the_live_preview() {
    let mut mix = Mix::default();
    mix.channels
        .insert("music".into(), strip(json!({ "fader": -20 })));
    mix.channels
        .insert("clipfile".into(), strip(json!({ "mute": true })));
    let (_, spec) = project(Some(mix.clone()));
    let shown = live(&spec, &mix);
    assert!(close(sound(&shown, "music").gain.unwrap(), 0.1));
    assert_eq!(sound(&shown, "fx").gain.unwrap_or(1.0), 1.0);
    let mut video = None;
    walk_items(&shown.tracks, &mut |i| {
        if i.base().node_id.as_deref() == Some("clipfile")
            && let TimelineItem::Element { props, .. } = i
        {
            video = Some(props.clone());
        }
    });
    let props = video.unwrap();
    assert_eq!(
        (props["muted"].clone(), props["volume"].clone()),
        (json!(true), json!(0.0))
    );
}

#[test]
fn turns_bus_ducking_into_a_live_approximation_that_ignores_short_sounds() {
    let mix = with_duck(Mix::default(), json!({}));
    let (_, spec) = project(Some(mix.clone()));
    let shown = live(&spec, &mix);
    let music = sound(&shown, "music").duck.unwrap();
    assert_eq!(music.keys, ["fx"]);
    assert!(close(music.level, db_to_gain(-12.0)));
    assert!(sound(&shown, "fx").duck.is_none());
    let ignoring = with_duck(Mix::default(), json!({ "ignoreShorterMs": 5000 }));
    assert!(sound(&live(&spec, &ignoring), "music").duck.is_none());
}

#[test]
fn ducks_only_for_clips_whose_sound_plays() {
    let music_over = |rate: f64| {
        let mut g = Graph::default();
        g.node(
            "music",
            "audio.music",
            json!({ "src": "assets/song.mp3", "fadeInMs": 0, "fadeOutMs": 0 }),
        );
        g.node(
            "clipfile",
            "media.import",
            json!({ "path": "assets/clip.mp4" }),
        );
        g.node("fast", "timeline.speed", json!({ "rate": rate }));
        g.node("mix", "audio.mix", json!({}));
        g.node("out", "output.render", json!({}));
        g.link("clipfile", "fast")
            .link("fast", "mix")
            .link_ports("music", "mix", "audio", "sounds")
            .link("mix", "out");
        let spec = film_spec(&g.build());
        let mut any = false;
        walk_items(&spec.tracks, &mut |i| any |= item_has_sound(i));
        assert!(any);
        sound(
            &live(
                &spec,
                &with_duck(Mix::default(), json!({ "keys": ["clips"] })),
            ),
            "music",
        )
        .duck
    };
    assert_eq!(music_over(1.0).unwrap().keys, ["clipfile"]);
    assert!(music_over(2.0).is_none());
}

#[test]
fn plays_only_the_master_stem_when_one_is_ready() {
    let (_, spec) = project(None);
    let stem = with_master_stem(&spec, "project:.opsis/cache/mix/m.wav", "m");
    assert!(stem.audio.is_empty());
    assert_eq!(stem.mix.as_ref().unwrap().hash, "m");
    let mut sounding = 0;
    walk_items(&stem.tracks, &mut |i| {
        sounding += usize::from(item_has_sound(i))
    });
    assert_eq!(sounding, 0);
}

fn plan_of(spec: &TimelineSpec, mix: Option<&Mix>) -> MixPlan {
    build_mix_plan(spec, mix, &flatten_spec(spec))
}

#[test]
fn lists_channels_in_connection_order_routed_to_the_default_buses() {
    let mut g = Graph::default();
    g.node("t", TITLE, json!({ "clipDuration": 300 }));
    g.node(
        "late",
        "audio.sfx",
        json!({ "src": "app:sfx/whoosh.wav", "atMs": 5000 }),
    );
    g.node(
        "early",
        "audio.sfx",
        json!({ "src": "app:sfx/whip.wav", "atMs": 1000 }),
    );
    g.node("mix", "audio.mix", json!({}));
    g.node("out", "output.render", json!({}));
    g.link("t", "mix")
        .link_ports("late", "mix", "audio", "sounds")
        .link_ports("early", "mix", "audio", "sounds")
        .link("mix", "out");
    let wf = g.build();
    let spec = compile_workflow(&wf, None)
        .specs
        .shift_remove("out")
        .unwrap();
    assert_eq!(
        plan_of(&spec, None)
            .channels
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ["late", "early"]
    );
    let plan = plan_of(&project(None).1, None);
    let routes: Vec<(String, Value, String)> = plan
        .channels
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                serde_json::to_value(c.kind).unwrap(),
                c.bus.clone(),
            )
        })
        .collect();
    assert_eq!(
        routes,
        [
            ("music".into(), json!("music"), "music".into()),
            ("fx".into(), json!("effect"), "effects".into()),
            ("clipfile".into(), json!("clip"), "clips".into())
        ]
    );
    assert_eq!(
        plan.buses
            .iter()
            .find(|b| b.id == "effects")
            .unwrap()
            .channels,
        ["fx"]
    );
    let fx = &plan.channels.iter().find(|c| c.id == "fx").unwrap().sounds[0];
    assert_eq!(
        (fx.src.as_str(), fx.at, fx.offset, fx.looped),
        ("app:sfx/whoosh.wav", 60, 0, false)
    );
}

#[test]
fn changes_the_plan_key_but_not_the_channel_stem_key_when_only_routing_changes() {
    let (_, spec) = project(None);
    let a = plan_of(&spec, None);
    let mut mix = Mix::default();
    mix.channels
        .insert("fx".into(), strip(json!({ "mute": true })));
    let b = plan_of(&spec, Some(&mix));
    assert_ne!(a.key, b.key);
    let fx = |p: &MixPlan| p.channels.iter().find(|c| c.id == "fx").unwrap().clone();
    let (fa, fb) = (fx(&a), fx(&b));
    assert_eq!(
        serde_json::to_value(channel_identity(&a, &fa)).unwrap(),
        serde_json::to_value(channel_identity(&b, &fb)).unwrap()
    );
    assert!(
        b.buses
            .iter()
            .find(|x| x.id == "effects")
            .unwrap()
            .channels
            .is_empty()
    );
}

fn duck_spec(audio: Value) -> TimelineSpec {
    let mut spec = serde_json::to_value(film_spec(&film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 300 }))
    })))
    .unwrap();
    spec["audio"] = audio;
    spec["durationInFrames"] = json!(300);
    serde_json::from_value(spec).unwrap()
}

fn effect(id: &str, from: i32, extra: Value) -> Value {
    let mut item = json!({ "id": id, "src": "app:sfx/whoosh.wav", "from": from, "role": "effect", "lengthFrames": 10, "channel": id });
    if let (Some(item), Value::Object(extra)) = (item.as_object_mut(), extra) {
        item.extend(extra);
    }
    item
}

fn keys(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn merges_overlapping_duck_spans() {
    let spec = duck_spec(json!([
        effect("a", 20, json!({})),
        effect("b", 0, json!({})),
        effect("c", 5, json!({ "lengthFrames": 7 })),
        effect("d", 30, json!({})),
    ]));
    assert_eq!(
        duck_spans(&spec, [], &keys(&["a", "b", "c", "d"])),
        [(0.0, 12.0), (20.0, 40.0)]
    );
}

#[test]
fn ducks_under_the_key_channels_but_not_other_or_keep_music_sounds() {
    let spec = duck_spec(json!([
        { "id": "m", "src": "song.mp3", "from": 0, "role": "music", "channel": "m" },
        effect("a", 30, json!({})),
        effect("b", 100, json!({ "keepMusic": true })),
    ]));
    assert_eq!(
        duck_spans(&spec, [], &keys(&["a", "b", "v"])),
        [(30.0, 40.0)]
    );
}

#[test]
fn ducks_effects_of_unknown_length_to_the_end_unless_placed_shorter() {
    let open = duck_spec(
        json!([{ "id": "a", "src": "x.wav", "from": 0, "role": "effect", "channel": "a" }]),
    );
    assert_eq!(duck_spans(&open, [], &keys(&["a"])), [(0.0, 300.0)]);
    let placed = duck_spec(
        json!([{ "id": "b", "src": "x.wav", "from": 100, "role": "effect", "channel": "b", "durationInFrames": 12 }]),
    );
    assert_eq!(duck_spans(&placed, [], &keys(&["b"])), [(100.0, 112.0)]);
}

#[test]
fn ducks_under_the_clips_with_sound_on_a_key_channel() {
    let spec = duck_spec(json!([effect("a", 0, json!({}))]));
    let clips = [("v", 60.0, 150.0), ("q", 0.0, 20.0)];
    assert_eq!(duck_spans(&spec, clips, &keys(&["v"])), [(60.0, 150.0)]);
    assert_eq!(
        duck_spans(&spec, clips, &keys(&["a", "v"])),
        [(0.0, 10.0), (60.0, 150.0)]
    );
}
