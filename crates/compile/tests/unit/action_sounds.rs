use opsis_compile::clip::clip_to_spec;
use opsis_compile::edits::timeline::{Outcome, TimelineEdits};
use opsis_compile::ids::SaltedIds;
use opsis_compile::mix::plan::build_mix_plan;
use opsis_compile::node::CardInputs;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_types::actions::ActionLog;
use opsis_types::numbers::{whole, whole_signed};
use opsis_types::timeline::{AudioItem, AudioRole};
use opsis_types::values::{AudioValue, ClipValue, CompileValue, Hits};
use opsis_types::workflow::WorkflowSettings;
use serde_json::{Value, json};

use crate::helpers::{
    CLIP, Graph, action, card_texts, clip_value, compile_clip, compile_node, export_film, make_log,
    record_result, zoomed,
};

fn take_log() -> ActionLog {
    let point = json!({ "x": 400, "y": 300 });
    make_log(
        [
            json!({ "id": "c", "kind": "click", "t": 2000, "tDown": 2000, "point": point, "nodeId": "save" }),
            json!({ "id": "d", "kind": "click", "t": 3170, "tDown": 3000, "tSecondDown": 3170, "point": point, "nodeId": "open" }),
            json!({ "id": "m", "kind": "click", "t": 4000, "tDown": 4000, "point": point, "silent": true, "nodeId": "quiet" }),
            json!({ "id": "t", "kind": "type", "t": 5000, "tDown": 5000, "tEnd": 5600, "keyTimes": [5100, 5200, 5300], "text": "abc", "point": point, "nodeId": "name" }),
            json!({ "id": "q", "kind": "type", "t": 6000, "tDown": 6000, "tEnd": 6300, "keyTimes": [6100], "text": "x", "point": point, "silent": true, "nodeId": "hushed" }),
            json!({ "id": "k", "kind": "key", "t": 7000, "text": "Enter", "nodeId": "send" }),
        ]
        .into_iter()
        .map(action)
        .collect(),
        9000.0,
    )
}

fn take(own: &[(&str, Value)]) -> ClipValue {
    zoomed(None, own, Some(take_log()), json!({ "speed": 1 }))
}

fn sounds(clip: &ClipValue, params: Value) -> AudioValue {
    match compile_node(
        "audio.actionSounds",
        params,
        json!({ CLIP: clip_value(clip) }),
    )
    .shift_remove("audio")
    {
        Some(CompileValue::Audio(audio)) => audio,
        other => panic!("{other:?}"),
    }
}

fn mark_frame(clip: &ClipValue, action_id: &str) -> i32 {
    let flat = flatten_spec(&clip_to_spec(clip, &WorkflowSettings::default()));
    let mark = flat
        .marks
        .iter()
        .find(|m| m.key.ends_with(&format!(":{action_id}")))
        .unwrap_or_else(|| panic!("no mark for {action_id}"));
    whole_signed(mark.frame)
}

fn froms(audio: &AudioValue) -> Vec<i32> {
    audio.items.iter().map(|i| i.from).collect()
}

#[test]
fn sounds_every_click_and_keystroke_except_silent_actions() {
    let audio = sounds(&take(&[]), json!({}));
    assert_eq!(audio.hits, Some(Hits { clicks: 4, keys: 4 }));
    let click = |item: &AudioItem| item.src == "app:sfx/mouse-click.wav";
    assert_eq!(audio.items.iter().filter(|i| click(i)).count(), 4);
    assert_eq!(
        audio
            .items
            .iter()
            .filter(|i| i.src == "app:sfx/key-press.wav")
            .count(),
        4
    );
    let at = |ms: f64| whole_signed(ms / 1000.0 * 30.0);
    assert_eq!(
        froms(&audio),
        [
            2000.0, 3000.0, 3170.0, 5000.0, 5100.0, 5200.0, 5300.0, 7000.0
        ]
        .map(at)
    );
    let mut ids: Vec<&str> = audio.items.iter().map(|i| i.id.as_str()).collect();
    ids.dedup();
    assert_eq!(ids.len(), audio.items.len());
}

#[test]
fn puts_every_hit_on_one_quiet_effects_channel_that_never_ducks_the_music() {
    let audio = sounds(&take(&[]), json!({ "vary": false, "volume": 0.5 }));
    for item in &audio.items {
        assert_eq!(item.channel.as_deref(), Some("n"));
        assert_eq!(item.role, Some(AudioRole::Effect));
        assert_eq!(item.keep_music, Some(true));
        assert_eq!(item.volume, Some(0.5));
        assert_eq!(item.pitch, None);
        assert!(item.length_frames.is_some_and(|f| f > 0));
    }
    let varied = sounds(&take(&[]), json!({ "volume": 0.5 }));
    let pitches: Vec<f64> = varied.items.iter().filter_map(|i| i.pitch).collect();
    assert_eq!(pitches.len(), varied.items.len());
    assert!(pitches.iter().all(|p| (0.95..=1.05).contains(p)));
    assert!(pitches.windows(2).any(|w| w[0] != w[1]));
    assert_eq!(
        varied,
        sounds(&take(&[]), json!({ "volume": 0.5 })),
        "the same hits vary the same way every compile"
    );
}

#[test]
fn turns_clicks_or_keys_off_and_uses_the_chosen_sounds() {
    let keys_only = sounds(&take(&[]), json!({ "clicks": false }));
    assert_eq!(keys_only.hits, Some(Hits { clicks: 0, keys: 4 }));
    let clicks_only = sounds(
        &take(&[]),
        json!({ "keys": false, "clickSound": "assets/tap.wav", "clickSoundMs": 100 }),
    );
    assert_eq!(clicks_only.hits, Some(Hits { clicks: 4, keys: 0 }));
    assert!(
        clicks_only
            .items
            .iter()
            .all(|i| i.src == "project:assets/tap.wav" && i.length_frames == Some(3))
    );
}

#[test]
fn follows_the_clicks_through_a_trim_a_speed_ramp_and_a_speed_change() {
    let ramp = (
        "camera.speedRamp",
        json!({ "segments": [{ "fromMs": 1000, "toMs": 4500, "rate": 2 }] }),
    );
    let ramped = take(&[ramp]);
    let trimmed = compile_clip(
        "timeline.trim",
        json!({ "startMs": 1500, "durationMs": 0 }),
        json!({ CLIP: clip_value(&take(&[])) }),
    );
    let faster = compile_clip(
        "timeline.speed",
        json!({ "rate": 2 }),
        json!({ CLIP: clip_value(&take(&[])) }),
    );
    for clip in [take(&[]), ramped, trimmed, faster] {
        let audio = sounds(&clip, json!({ "keys": false }));
        let clicks = froms(&audio);
        assert_eq!(clicks[0], mark_frame(&clip, "c"), "{clicks:?}");
        assert_eq!(clicks[3], mark_frame(&clip, "t"), "{clicks:?}");
        assert!(clicks.windows(2).all(|w| w[0] <= w[1]), "{clicks:?}");
    }
    let plain = froms(&sounds(&take(&[]), json!({ "keys": false })));
    let fast = froms(&sounds(
        &compile_clip(
            "timeline.speed",
            json!({ "rate": 2 }),
            json!({ CLIP: clip_value(&take(&[])) }),
        ),
        json!({ "keys": false }),
    ));
    assert_eq!(fast[0], plain[0] / 2);
}

#[test]
fn says_how_many_clicks_and_keys_it_sounds_on_its_card() {
    let mut g = Graph::default();
    let cam = g.recorded_take(&[]);
    g.node("sounds", "audio.actionSounds", json!({}));
    g.link(&cam, "sounds");
    let wf = g.build();
    let results = indexmap::IndexMap::from([("record".to_owned(), record_result(&take_log()))]);
    let compiled = opsis_compile::compile::compile_workflow(&wf, Some(&results));
    let node = wf.nodes.iter().find(|n| n.id == "sounds").unwrap();
    let inputs = opsis_compile::cards::card_inputs(&wf, &compiled, node, None);
    assert_eq!(inputs.hits, Some(Hits { clicks: 4, keys: 4 }));
    let rows = card_texts("audio.actionSounds", json!({ "keys": false }), inputs);
    assert_eq!(rows[0], "4 clicks · Mouse click");
    assert_eq!(rows[1], "No keys");
    assert_eq!(
        card_texts("audio.actionSounds", json!({}), CardInputs::default())[0],
        "Connect a clip with a take"
    );
}

#[test]
fn adds_the_hits_to_the_film_through_add_sound() {
    let mut g = Graph::default();
    let cam = g.recorded_take(&[]);
    g.node("sounds", "audio.actionSounds", json!({}));
    g.node("mix", "audio.mix", json!({}));
    g.node("out", "output.render", json!({}));
    g.link(&cam, "sounds")
        .link(&cam, "mix")
        .link_ports("sounds", "mix", "audio", "sounds")
        .link("mix", "out");
    let results = indexmap::IndexMap::from([("record".to_owned(), record_result(&take_log()))]);
    let film = export_film(&g.build(), Some(&results), "out");
    let hits: Vec<&AudioItem> = film
        .audio
        .iter()
        .filter(|a| a.channel.as_deref() == Some("sounds"))
        .collect();
    assert_eq!(hits.len(), 8);
    let flat = flatten_spec(&film);
    let bars: Vec<_> = flat
        .audio
        .iter()
        .filter(|bar| bar.channel == "sounds")
        .collect();
    assert_eq!(bars.len(), 1, "the hits share one bar");
    assert_eq!(bars[0].key, "hits:sounds");
    let ticks = bars[0].hits.as_ref().unwrap();
    assert_eq!(ticks.len(), 8);
    assert_eq!(bars[0].from, ticks[0]);
    assert!(hits.iter().all(|h| h.hit == Some(true)));
    let plan = build_mix_plan(&film, None, &flat);
    let channel = plan.channels.iter().find(|c| c.id == "sounds").unwrap();
    let placed: Vec<u32> = channel.sounds.iter().map(|s| s.at).collect();
    let expected: Vec<u32> = hits.iter().map(|h| whole(f64::from(h.from))).collect();
    assert_eq!(placed, expected);
}

#[test]
fn sets_the_volume_of_every_hit_from_their_bar_and_refuses_to_move_them() {
    let mut g = Graph::default();
    let cam = g.recorded_take(&[]);
    g.node("sounds", "audio.actionSounds", json!({}));
    g.node("mix", "audio.mix", json!({}));
    g.node("out", "output.render", json!({}));
    g.link(&cam, "sounds")
        .link(&cam, "mix")
        .link_ports("sounds", "mix", "audio", "sounds")
        .link("mix", "out");
    let wf = g.build();
    let results = indexmap::IndexMap::from([("record".to_owned(), record_result(&take_log()))]);
    let flat = flatten_spec(&export_film(&wf, Some(&results), "out"));
    let bar = flat.audio.iter().find(|b| b.channel == "sounds").unwrap();
    let edit = |change: &dyn Fn(&mut TimelineEdits<'_>) -> Outcome| {
        let mut ids = SaltedIds::new("t");
        let mut edits = TimelineEdits {
            workflow: wf.clone(),
            fps: wf.settings.fps,
            source: Some("out".to_owned()),
            ids: &mut ids,
        };
        let outcome = change(&mut edits);
        (outcome, edits.workflow)
    };
    let (outcome, louder) = edit(&|e| e.set_audio_volume(bar, 0.9));
    outcome.unwrap();
    assert_eq!(
        louder.node("sounds").and_then(|n| n.params.get("volume")),
        Some(&json!(0.9))
    );
    assert!(edit(&|e| e.move_audio(bar, 6.0)).0.is_err());
}
