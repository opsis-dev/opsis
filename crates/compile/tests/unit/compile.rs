use indexmap::IndexMap;
use opsis_compile::clip::walk_items;
use opsis_compile::compile::{CompileResult, compile_workflow, film_of, preview_spec};
use opsis_compile::graph::connection_problem;
use opsis_compile::timeline::flatten::{BarKind, flatten_spec};
use opsis_types::actions::ActionKind;
use opsis_types::timeline::{TimelineItem, TimelineSpec};
use opsis_types::values::{ClipValue, CompileValue, NodeResult};
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};

use crate::helpers::{
    Graph, TITLE, action, export_film, make_log, node_result, record_result, template,
};

fn log() -> opsis_types::actions::ActionLog {
    make_log(
        vec![
            action(
                json!({ "id": "a", "kind": "type", "t": 1500, "tEnd": 3200, "point": { "x": 960, "y": 200 }, "rect": { "x": 600, "y": 170, "w": 720, "h": 60 }, "cluster": "add", "text": "Record a real product flow" }),
            ),
            action(
                json!({ "id": "b", "kind": "click", "t": 6000, "tDown": 5900, "tDepart": 5200, "point": { "x": 620, "y": 270 }, "rect": { "x": 600, "y": 250, "w": 40, "h": 40 }, "cluster": "done", "label": "Complete" }),
            ),
            action(
                json!({ "id": "c", "kind": "click", "t": 8000, "point": { "x": 1100, "y": 700 }, "rect": { "x": 1060, "y": 690, "w": 90, "h": 24 }, "cluster": "filter", "label": "Completed filter" }),
            ),
        ],
        12_000.0,
    )
}

fn todo_mvc() -> Workflow {
    serde_json::from_value(template("todomvc")).unwrap()
}

fn with_results(wf: &Workflow, results: Vec<(&str, NodeResult)>) -> CompileResult {
    let results: IndexMap<String, NodeResult> = results
        .into_iter()
        .map(|(k, r)| (k.to_owned(), r))
        .collect();
    compile_workflow(wf, Some(&results))
}

fn todo_film(results: Vec<(&str, NodeResult)>) -> TimelineSpec {
    let wf = todo_mvc();
    film_of(&wf, &with_results(&wf, results), "render").unwrap()
}

fn items_of(spec: &TimelineSpec) -> Vec<TimelineItem> {
    let mut items = Vec::new();
    walk_items(&spec.tracks, &mut |i| items.push(i.clone()));
    items
}

fn recording_in(spec: &TimelineSpec) -> Option<Value> {
    items_of(spec).into_iter().find_map(|i| match i {
        TimelineItem::Recording { props, .. } => Some(serde_json::to_value(props).unwrap()),
        _ => None,
    })
}

fn clip(out: &CompileResult, id: &str) -> ClipValue {
    match out.values.get(id).and_then(|v| v.get("clip")) {
        Some(CompileValue::Clip(c)) => c.clone(),
        other => panic!("{id} has no clip: {other:?}"),
    }
}

fn group_value(item: &TimelineItem) -> Value {
    match item {
        TimelineItem::Group { props, .. } => serde_json::to_value(props).unwrap(),
        other => panic!("not a group: {other:?}"),
    }
}

#[test]
fn compiles_the_sample_to_one_spec_per_export_with_a_placeholder_before_the_first_run() {
    let wf = todo_mvc();
    let out = compile_workflow(&wf, None);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let spec = &film_of(&wf, &out, "render").unwrap();
    assert_eq!(spec.width, 1920);
    let components: Vec<String> = items_of(spec)
        .iter()
        .map(|i| match i {
            TimelineItem::Element { component, .. } => component.clone(),
            TimelineItem::Group { .. } => "group".into(),
            TimelineItem::Recording { .. } => "recording".into(),
        })
        .collect();
    for c in [
        "placeholder",
        "soft-blur-in",
        "shader-mesh-gradient",
        "title",
    ] {
        assert!(components.iter().any(|x| x == c), "{c}");
    }
}

#[test]
fn uses_the_recording_once_the_capture_node_has_a_result() {
    let film = todo_film(vec![("record", record_result(&log()))]);
    let props = recording_in(&film).unwrap();
    assert_eq!(props["src"], "project:.opsis/cache/h/recording.mp4");
    assert_eq!(props["tuning"]["sMax"], 1.74);
    assert_eq!(
        props["keystrokes"],
        json!({ "position": "bottom", "typed": true, "chords": true, "keycap": "#0a0a0a", "ink": "#ffffff", "size": 1.0 })
    );
}

#[test]
fn marks_every_action_of_the_recording_on_the_timeline_at_its_frame() {
    let flat = flatten_spec(&todo_film(vec![("record", record_result(&log()))]));
    let bar = flat
        .bars
        .iter()
        .find(|b| b.kind == BarKind::Recording)
        .unwrap();
    let marks: Vec<_> = flat.marks.iter().filter(|m| m.bar_key == bar.key).collect();
    assert_eq!(
        marks.iter().map(|m| m.kind).collect::<Vec<_>>(),
        [ActionKind::Type, ActionKind::Click, ActionKind::Click]
    );
    assert_eq!(marks[1].label, "Click Complete");
    let speed = bar.props["speed"].as_f64().unwrap();
    let local = (marks[1].frame - bar.from) / bar.rate;
    assert!((local + bar.content_start - 6000.0 / 1000.0 / speed * 30.0).abs() < 0.5);
}

#[test]
fn passes_the_small_preview_copy_along_with_the_full_recording() {
    let mut result = record_result(&log());
    if let Some(CompileValue::Recording(r)) = result.outputs.get_mut("recording") {
        r.proxy_src = Some("project:.opsis/cache/h/recording.preview.mp4".into());
    }
    let props = recording_in(&todo_film(vec![("record", result)])).unwrap();
    assert_eq!(props["src"], "project:.opsis/cache/h/recording.mp4");
    assert_eq!(
        props["proxySrc"],
        "project:.opsis/cache/h/recording.preview.mp4"
    );
}

#[test]
fn sequences_clips_with_overlapping_transitions_and_fills_variables() {
    let spec = &todo_film(vec![("record", record_result(&log()))]);
    let seq = &spec.tracks[0].items;
    assert_eq!(seq.len(), 3);
    let enter = seq[1].base().enter.clone().unwrap();
    assert_eq!(enter.kind, "whip-pan");
    assert_eq!(seq[0].base().exit.as_ref().unwrap().kind, "whip-pan");
    assert_eq!(
        seq[1].base().from,
        seq[0].base().from + seq[0].base().duration_in_frames - enter.duration_in_frames
    );
    assert_eq!(
        spec.duration_in_frames,
        seq[2].base().from + seq[2].base().duration_in_frames
    );
    let outro = items_of(&todo_film(Vec::new()))
        .into_iter()
        .find_map(|i| match i {
            TimelineItem::Element {
                component, props, ..
            } if component == "title" => Some(props),
            _ => None,
        });
    assert_eq!(
        outro.unwrap()["subtitle"],
        "https://demo.playwright.dev/todomvc"
    );
}

fn title_workflow(extra: impl FnOnce(&mut Graph) -> String) -> (Workflow, String) {
    let mut g = Graph::default();
    g.node("title", TITLE, json!({ "clipDuration": 90 }));
    let last = extra(&mut g);
    (g.build(), last)
}

#[test]
fn trims_speeds_and_bypasses_through_groups() {
    let (wf, last) = title_workflow(|g| {
        g.node(
            "trim",
            "timeline.trim",
            json!({ "startMs": 1000, "durationMs": 1000 }),
        );
        g.link("title", "trim");
        "trim".into()
    });
    let trimmed = clip(&compile_workflow(&wf, None), &last);
    assert_eq!(
        (
            trimmed.duration_in_frames,
            group_value(&trimmed.tracks[0].items[0])["startFrom"].clone()
        ),
        (30, json!(30))
    );
    let (wf, last) = title_workflow(|g| {
        g.node("speed", "timeline.speed", json!({ "rate": 2 }));
        g.link("title", "speed");
        "speed".into()
    });
    let sped = clip(&compile_workflow(&wf, None), &last);
    assert_eq!(
        (
            sped.duration_in_frames,
            group_value(&sped.tracks[0].items[0])["remap"].clone()
        ),
        (45, json!([[0.0, 0.0], [45.0, 90.0]]))
    );
    let (mut wf, last) = title_workflow(|g| {
        g.node("loop", "timeline.loop", json!({ "times": 3 }));
        g.link("title", "loop");
        "loop".into()
    });
    wf.nodes
        .iter_mut()
        .find(|n| n.id == "loop")
        .unwrap()
        .disabled = Some(true);
    assert_eq!(
        clip(&compile_workflow(&wf, None), &last).duration_in_frames,
        90
    );
}

#[test]
fn reports_invalid_params_per_node_and_previews_the_selected_nodes_clip() {
    let (mut wf, _) = title_workflow(|_| "title".into());
    wf.nodes[0].params.insert("fontSize".into(), json!("huge"));
    assert_eq!(
        compile_workflow(&wf, None).errors["title"],
        "Font size isn't valid"
    );
    let (wf, _) = title_workflow(|g| {
        g.node("loop", "timeline.loop", json!({ "times": 2 }));
        g.link("title", "loop");
        "loop".into()
    });
    let out = compile_workflow(&wf, None);
    let selected = preview_spec(&wf, &out, Some("title")).unwrap();
    assert_eq!(selected.spec.duration_in_frames, 90);
    assert_eq!(
        preview_spec(&wf, &out, None)
            .unwrap()
            .spec
            .duration_in_frames,
        180
    );
    assert_eq!(
        (selected.spans["title"].from, selected.spans["title"].to),
        (0.0, 90.0)
    );
}

#[test]
fn wraps_layers_that_keep_their_layout_in_a_box_at_the_projects_size() {
    let mut g = Graph::default();
    g.node("bg", "fx.solid", json!({ "clipDuration": 60 }));
    g.node("t", TITLE, json!({ "clipDuration": 60 }));
    g.node("stack", "timeline.stack", json!({ "keepLayout": true }));
    g.link_ports("bg", "stack", "clip", "in1")
        .link_ports("t", "stack", "clip", "in2");
    g.node("out", "output.render", json!({}));
    g.link("stack", "out");
    let spec = export_film(&g.build(), None, "out");
    let props = group_value(&spec.tracks[0].items[0]);
    assert_eq!(props["design"], json!({ "width": 1920, "height": 1080 }));
    assert_eq!(props["tracks"].as_array().unwrap().len(), 2);
    let mut ids: Vec<String> = flatten_spec(&spec)
        .bars
        .into_iter()
        .map(|b| b.node_id)
        .collect();
    ids.sort();
    assert_eq!(ids, ["bg", "t"]);
}

fn with_sounds(sounds: &[(&str, Value)]) -> TimelineSpec {
    let mut tagged = log();
    for a in &mut tagged.actions {
        if a.id == "b" {
            a.node_id = Some("complete".into());
        }
    }
    let mut g = Graph::default();
    let cam = g.recorded_take(&[("complete", "browser.click", json!({ "target": "Complete" }))]);
    g.node("mix", "audio.mix", json!({}));
    g.link(&cam, "mix");
    for (id, params) in sounds {
        let mut all = json!({ "src": "app:sfx/pop.wav" });
        all.as_object_mut()
            .unwrap()
            .extend(params.as_object().unwrap().clone());
        g.node(id, "audio.sfx", all);
        g.link_ports(id, "mix", "audio", "sounds");
    }
    g.node("out", "output.render", json!({}));
    g.link("mix", "out");
    let wf = g.build();
    film_of(
        &wf,
        &with_results(&wf, vec![("record", record_result(&tagged))]),
        "out",
    )
    .unwrap()
}

#[test]
fn takes_any_number_of_sounds_and_plays_a_pinned_one_on_its_action() {
    let spec = with_sounds(&[
        ("s1", json!({ "atMs": 500 })),
        ("s2", json!({ "atMs": 500 })),
        ("s3", json!({ "atMs": 500 })),
        ("s4", json!({ "atMs": 500 })),
    ]);
    assert_eq!(
        spec.audio
            .iter()
            .map(|a| a.node_id.clone().unwrap())
            .collect::<Vec<_>>(),
        ["s1", "s2", "s3", "s4"]
    );
    let spec = with_sounds(&[(
        "pinned",
        json!({ "atAction": "complete", "offsetMs": -200 }),
    )]);
    let mark = flatten_spec(&spec)
        .marks
        .into_iter()
        .find(|m| m.node_id.as_deref() == Some("complete"))
        .unwrap();
    let sound = spec
        .audio
        .iter()
        .find(|a| a.node_id.as_deref() == Some("pinned"))
        .unwrap();
    assert_eq!(f64::from(sound.from), mark.frame.round() - 6.0);
}

fn sequence(cuts: Value, ports: &[&str]) -> Vec<TimelineItem> {
    let mut g = Graph::default();
    for (i, port) in ports.iter().enumerate() {
        g.node(&format!("c{i}"), TITLE, json!({ "clipDuration": 60 }));
        g.link_ports(&format!("c{i}"), "seq", "clip", port);
    }
    g.node(
        "seq",
        "timeline.sequence",
        json!({ "transition": "fade", "transitionDuration": 10, "cuts": cuts }),
    );
    g.node("out", "output.render", json!({}));
    g.link("seq", "out");
    export_film(&g.build(), None, "out").tracks[0].items.clone()
}

fn enters(items: &[TimelineItem]) -> Vec<Option<(String, u32, bool)>> {
    items
        .iter()
        .map(|i| {
            i.base()
                .enter
                .as_ref()
                .map(|e| (e.kind.clone(), e.duration_in_frames, e.hold == Some(true)))
        })
        .collect()
}

fn cut(kind: &str, frames: u32, hold: bool) -> Option<(String, u32, bool)> {
    Some((kind.to_owned(), frames, hold))
}

#[test]
fn uses_each_cuts_own_transition_a_plain_cut_for_none_and_the_last_entry_for_a_port() {
    let three = ["in1", "in2", "in3"];
    let items = sequence(
        json!([{ "before": "in3", "transition": "wipe", "transitionDuration": 20, "keepWhole": true }]),
        &three,
    );
    assert_eq!(
        enters(&items),
        [None, cut("fade", 10, false), cut("wipe", 20, true)]
    );
    assert_eq!(items[1].base().exit.as_ref().unwrap().kind, "wipe");
    let items = sequence(
        json!([{ "before": "in2", "transition": "none", "transitionDuration": 15 }]),
        &three,
    );
    assert_eq!(enters(&items), [None, None, cut("fade", 10, false)]);
    let items = sequence(
        json!([
            { "before": "in5", "transition": "wipe", "transitionDuration": 20 },
            { "before": "in2", "transition": "slide", "transitionDuration": 12 },
            { "before": "in2", "transition": "iris", "transitionDuration": 8 }
        ]),
        &["in1", "in2"],
    );
    assert_eq!(enters(&items), [None, cut("iris", 8, false)]);
}

#[test]
fn joins_sound_effects_on_one_port_and_wont_take_a_file_as_a_sound() {
    let mut g = Graph::default();
    g.node("bg", "fx.solid", json!({ "clipDuration": 90 }));
    g.node(
        "click",
        "audio.sfx",
        json!({ "src": "assets/click.wav", "atMs": 0 }),
    );
    g.node(
        "whoosh",
        "audio.sfx",
        json!({ "src": "assets/whoosh.wav", "atMs": 500 }),
    );
    g.node("mix", "audio.mix", json!({}));
    g.link("bg", "mix")
        .link_ports("click", "mix", "audio", "sounds")
        .link_ports("whoosh", "mix", "audio", "sounds");
    let mixed = clip(&compile_workflow(&g.build(), None), "mix");
    assert_eq!(
        mixed
            .audio
            .iter()
            .map(|a| (a.src.as_str(), a.from))
            .collect::<Vec<_>>(),
        [
            ("project:assets/click.wav", 0),
            ("project:assets/whoosh.wav", 15)
        ]
    );
    let mut g = Graph::default();
    g.node(
        "file",
        "media.import",
        json!({ "path": "assets/voice.wav" }),
    );
    g.node("mix", "audio.mix", json!({}));
    g.link_ports("file", "mix", "media", "sounds");
    assert_eq!(
        connection_problem(&g.build(), "file", "media", "mix", "sounds")
            .unwrap()
            .message,
        "A file can't connect to a sound input."
    );
}

#[test]
fn hands_a_screenshot_and_a_recording_to_the_file_tools_as_files() {
    let mut g = Graph::default();
    let session = g.node("session", "browser.open", json!({}));
    let shot = g.node("shot", "capture.still", json!({}));
    let rec = g.node("rec", "capture.record", json!({}));
    g.chain(&[session.clone(), shot]).chain(&[session, rec]);
    g.node("trim", "media.trim", json!({}));
    g.node("convert", "media.convert", json!({}));
    g.link_ports("shot", "trim", "image", "media").link_ports(
        "rec",
        "convert",
        "recording",
        "media",
    );
    let shot_result = node_result(
        json!({ "outputs": { "image": { "type": "image", "src": "project:shot.png", "width": 800, "height": 600 } } }),
    );
    let take = record_result(&make_log(Vec::new(), 4000.0));
    let out = with_results(&g.build(), vec![("shot", shot_result), ("rec", take)]);
    let value = |id: &str| serde_json::to_value(&out.values[id]["media"]).unwrap();
    assert_eq!(
        value("trim"),
        json!({ "type": "media", "kind": "image", "src": "project:shot.png", "width": 800, "height": 600 })
    );
    let converted = value("convert");
    assert_eq!(
        (
            converted["kind"].clone(),
            converted["src"].clone(),
            converted["durationMs"].clone()
        ),
        (
            json!("video"),
            json!("project:.opsis/cache/h/recording.mp4"),
            json!(4000)
        )
    );
}

#[test]
fn an_export_takes_its_film_from_the_connected_clip_and_a_skipped_node_passes_its_first_input() {
    let mut g = Graph::default();
    g.node("title", "fx.title", json!({ "clipDuration": 45 }));
    g.node("film", "output.render", json!({}));
    g.node("loose", "output.render", json!({ "fileName": "loose" }));
    g.link("title", "film");
    let wf = g.build();
    let out = compile_workflow(&wf, None);
    assert_eq!(film_of(&wf, &out, "film").unwrap().duration_in_frames, 45);
    assert!(film_of(&wf, &out, "loose").is_none());
    assert!(out.values["film"].is_empty());
    let mut g = Graph::default();
    g.node("red", "fx.solid", json!({ "color": "#ff0000" }));
    g.node("blue", "fx.solid", json!({ "color": "#0000ff" }));
    g.node("seq", "timeline.sequence", json!({}));
    g.link_ports("blue", "seq", "clip", "in2")
        .link_ports("red", "seq", "clip", "in1");
    let mut wf = g.build();
    wf.nodes
        .iter_mut()
        .find(|n| n.id == "seq")
        .unwrap()
        .disabled = Some(true);
    let passed = clip(&compile_workflow(&wf, None), "seq");
    match &passed.tracks[0].items[0] {
        TimelineItem::Element { props, .. } => assert_eq!(props["color"], "#ff0000"),
        other => panic!("{other:?}"),
    }
}
