use indexmap::IndexMap;
use opsis_compile::clip::recording_props;
use opsis_compile::compile::compile_workflow;
use opsis_compile::graph::{connection_problem_now, validate_workflow};
use opsis_compile::inspector::take_kind;
use opsis_compile::node::{CardInputs, InputTake};
use opsis_compile::ports::{refused_conversion, value_converts};
use opsis_compile::registry::registry;
use opsis_types::values::{CompileValue, NodeResult, TakeKind};
use opsis_types::workflow::{PortType, Workflow};
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::helpers::{Graph, card_texts, make_log, node_result, take_recording, zoomed};

fn terminal_take() -> Value {
    let log = make_log(Vec::new(), 2000.0);
    json!({
        "type": "recording", "log": log, "durationMs": 2000, "nodeId": "term", "edits": [],
        "source": { "kind": "terminal", "cast": "project:take.cast", "cols": 80, "rows": 24 }
    })
}

fn value(v: Value) -> CompileValue {
    serde_json::from_value(v).unwrap()
}

const DEFAULT_STYLES: [&str; 2] = ["camera.camera", "camera.motionBlur"];

fn props_with(defaults: &[&str]) -> Value {
    let style: Vec<(&str, Value)> = defaults.iter().map(|k| (*k, json!({}))).collect();
    serde_json::to_value(recording_props(&zoomed(Some(&style), &[], None, None))[0]).unwrap()
}

proptest! {
    #[test]
    fn default_camera_shot_and_motion_blur_change_nothing(
        picked in proptest::sample::subsequence(DEFAULT_STYLES.to_vec(), 0..=2).prop_shuffle()
    ) {
        prop_assert_eq!(props_with(&picked), props_with(&[]));
    }
}

#[test]
fn applies_the_camera_before_any_other_style_so_a_far_move_never_loses_its_pans() {
    let camera = ("camera.camera", json!({ "longestPan": 900 }));
    let pan = |style: &[(&str, Value)], own: &[(&str, Value)]| {
        serde_json::to_value(recording_props(&zoomed(Some(style), own, None, None))[0])
            .unwrap()["camera"]["tuning"]["longestPanPx"]
            .clone()
    };
    let following = (
        "camera.camera",
        json!({ "follows": "cursor", "zoomOutOnJumps": false, "longestPan": 900 }),
    );
    assert!(pan(&[following], &[]).as_f64().unwrap() > 1.0e6);
    let far = [(
        "camera.move",
        json!({ "fromSpot": true, "zoomOutWhenFar": false }),
    )];
    assert!(pan(std::slice::from_ref(&camera), &far).as_f64().unwrap() > 1.0e6);
    assert_eq!(pan(&[camera], &[]), json!(900.0));
}

#[test]
fn turns_only_a_browser_take_into_a_file() {
    let browser = value(take_recording(None));
    let terminal = value(terminal_take());
    assert!(value_converts(&browser, PortType::Media));
    assert!(!value_converts(&terminal, PortType::Media));
    assert!(value_converts(&terminal, PortType::Recording));
    assert!(refused_conversion(&browser, PortType::Media).is_none());
    assert!(
        refused_conversion(&terminal, PortType::Media)
            .unwrap()
            .starts_with("A terminal take has no video file")
    );
}

fn with_terminal_take() -> (Workflow, IndexMap<String, NodeResult>) {
    let mut g = Graph::default();
    g.node("rec", "capture.record", json!({}));
    g.node("grade", "media.applyGrade", json!({}));
    g.node("take", "camera.take", json!({}));
    g.link_ports("rec", "take", "recording", "recording");
    let results = IndexMap::from([(
        "rec".to_owned(),
        node_result(json!({ "outputs": { "recording": terminal_take() } })),
    )]);
    (g.build(), results)
}

#[test]
fn refuses_a_terminal_take_into_a_file_input_and_flags_one_already_there() {
    let (wf, results) = with_terminal_take();
    let compiled = compile_workflow(&wf, Some(&results));
    let refused = connection_problem_now(&wf, &compiled, "rec", "recording", "grade", "media");
    assert!(
        refused
            .unwrap()
            .message
            .starts_with("A terminal take has no video file")
    );
    let plain = compile_workflow(&wf, None);
    assert!(connection_problem_now(&wf, &plain, "rec", "recording", "grade", "media").is_none());

    let mut g = Graph::default();
    g.node("rec", "capture.record", json!({}));
    g.node("grade", "media.applyGrade", json!({}));
    g.link_ports("rec", "grade", "recording", "media");
    let wf = g.build();
    let compiled = compile_workflow(&wf, Some(&results));
    let issues = validate_workflow(&wf, Some(&compiled));
    assert!(
        issues
            .iter()
            .any(|i| i.edge_id.is_some()
                && i.message.starts_with("A terminal take has no video file"))
    );
}

#[test]
fn finds_the_take_a_style_node_reaches_for_its_inspector() {
    let (mut wf, results) = with_terminal_take();
    let mut g = Graph::default();
    g.node("frame", "camera.frame", json!({}));
    let frame = g.build();
    wf.nodes.extend(frame.nodes);
    wf.edges.push(
        serde_json::from_value(json!({ "id": "f", "source": "frame", "sourceHandle": "style", "target": "take", "targetHandle": "style" }))
            .unwrap(),
    );
    let compiled = compile_workflow(&wf, Some(&results));
    assert_eq!(take_kind(&wf, &compiled, "frame"), Some(TakeKind::Terminal));
    assert_eq!(take_kind(&wf, &compiled, "take"), Some(TakeKind::Terminal));
    assert_eq!(
        take_kind(&wf, &compile_workflow(&wf, None), "frame"),
        Some(TakeKind::Browser)
    );
}

fn show_if_take_values(schema: &Value, out: &mut Vec<Value>) {
    match schema {
        Value::Object(map) => {
            if let Some(kinds) = map.get("x-show-if-take") {
                out.push(kinds.clone());
            }
            map.values().for_each(|v| show_if_take_values(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| show_if_take_values(v, out)),
        _ => {}
    }
}

#[test]
fn names_only_take_kinds_in_show_if_take() {
    let mut found = Vec::new();
    for def in registry().visible() {
        show_if_take_values(def.params.json(), &mut found);
    }
    assert_ne!(found, [] as [Value; 0]);
    for kinds in found {
        let kinds = kinds.as_array().unwrap();
        assert_ne!(kinds.as_slice(), [] as [Value; 0]);
        for kind in kinds {
            assert!(
                serde_json::from_value::<TakeKind>(kind.clone()).is_ok(),
                "{kind}"
            );
        }
    }
}

#[test]
fn names_the_file_a_take_is_saved_as_by_its_kind() {
    let card = |kind: Option<TakeKind>| {
        card_texts(
            "output.take",
            json!({ "fileName": "Launch day" }),
            CardInputs {
                takes: IndexMap::from([(
                    "recording".to_owned(),
                    kind.map(|kind| InputTake {
                        kind,
                        duration_ms: 1000.0,
                        cursor_track: false,
                    }),
                )]),
                ..CardInputs::default()
            },
        )
    };
    assert_eq!(card(Some(TakeKind::Browser)), ["launch-day.mp4"]);
    assert_eq!(card(Some(TakeKind::Terminal)), ["launch-day.cast"]);
    assert_eq!(card(None), ["Connect a take"]);
}

#[test]
fn says_which_kind_of_take_a_clip_is_made_from() {
    let texts = card_texts(
        "camera.take",
        json!({}),
        CardInputs {
            takes: IndexMap::from([(
                "recording".to_owned(),
                Some(InputTake {
                    kind: TakeKind::Browser,
                    duration_ms: 6000.0,
                    cursor_track: true,
                }),
            )]),
            ..CardInputs::default()
        },
    );
    assert!(
        texts.contains(&"Browser take · 4.8 s".to_owned()),
        "{texts:?}"
    );
    assert!(texts.contains(&"The usual style".to_owned()));
}

#[test]
fn a_saved_take_passes_its_file_on_once_it_has_run() {
    let before =
        crate::helpers::compile_node_with("output.take", json!({}), json!({}), None).unwrap();
    assert!(before.is_empty());
    let saved = json!({
        "type": "media", "src": "project:.opsis/out/launch-day.cast", "kind": "terminal-take",
        "durationMs": 4000, "hasAudio": false
    });
    let result = crate::helpers::node_result(json!({ "outputs": { "media": saved } }));
    let after =
        crate::helpers::compile_node_with("output.take", json!({}), json!({}), Some(&result))
            .unwrap();
    let media = after["media"].as_media().unwrap();
    assert_eq!(media.kind, opsis_types::values::MediaKind::TerminalTake);
    assert_eq!(
        opsis_compile::nodes::output::take_media_kind(TakeKind::Browser),
        opsis_types::values::MediaKind::Video
    );
}

#[test]
fn no_node_takes_a_terminal_take_file_as_a_picture_or_sound() {
    use opsis_compile::node::{FileNeed, TERMINAL_TAKE_FILE};
    use opsis_types::values::{MediaKind, MediaValue};
    let cast = MediaValue {
        src: Some("project:take.cast".to_owned()),
        kind: opsis_compile::asset_ref::kind_from_path("take.cast"),
        duration_ms: Some(4000),
        width: None,
        height: None,
        has_audio: Some(false),
    };
    assert_eq!(cast.kind, MediaKind::TerminalTake);
    for need in [
        FileNeed::Image,
        FileNeed::Picture,
        FileNeed::Moving,
        FileNeed::Sound,
        FileNeed::SoundFile,
    ] {
        assert!(need.refuses(&cast), "{need:?}");
        assert_eq!(need.message_for(&cast), TERMINAL_TAKE_FILE);
    }
}
