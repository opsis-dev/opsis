use indexmap::IndexMap;
use opsis_compile::clip::recording_props;
use opsis_compile::compile::compile_workflow;
use opsis_compile::footage::footage_uses;
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CardInputs, InputTake};
use opsis_compile::terminal::{default_look, grid, screen_size, theme_named};
use opsis_types::recording::ScreenProps;
use opsis_types::terminal::TerminalAction;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{CompileValue, TakeKind};
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::helpers::{
    Graph, card_texts, compile_clip, make_log, parse_params, style_entries, take_recording,
};

fn terminal_take(cursor: bool) -> Value {
    let mut log = serde_json::to_value(make_log(Vec::new(), 4000.0)).unwrap();
    log["viewport"] = json!({ "width": 80, "height": 24 });
    log["actions"] = json!([{
        "id": "type", "kind": "type", "t": 500, "text": "ls",
        "point": { "x": 11.0, "y": 0.5 }, "rect": { "x": 10.0, "y": 0.0, "w": 2.0, "h": 1.0 }
    }]);
    log["cursorTrack"] = if cursor {
        json!([{ "t": 0, "x": 40.5, "y": 12.5 }])
    } else {
        Value::Null
    };
    log["caretTrack"] = json!([{ "t": 0, "col": 2, "row": 0 }, { "t": 900, "col": 12, "row": 3 }]);
    json!({
        "type": "recording", "log": log, "durationMs": 4000, "nodeId": "term", "edits": [],
        "source": { "kind": "terminal", "cast": "project:.opsis/cache/t/take.cast", "cols": 80, "rows": 24 }
    })
}

fn clip_of(take: &Value, style: &[(&str, Value)], params: Value) -> opsis_types::values::ClipValue {
    let mut inputs = json!({ "recording": take });
    if !style.is_empty() {
        inputs["style"] =
            json!({ "type": "recording-style", "entries": style_entries(style, "s") });
    }
    compile_clip("camera.take", params, inputs)
}

#[test]
fn builds_a_session_from_the_chain_of_terminal_actions() {
    let mut g = Graph::default();
    g.node("open", "terminal.open", json!({ "cols": 100, "rows": 30 }));
    g.node("start", "terminal.startRecording", json!({}));
    g.node("run", "terminal.run", json!({ "command": "ls -la" }));
    g.node("keys", "terminal.keys", json!({ "keys": "Control+c" }));
    g.node("rec", "capture.terminal", json!({}));
    let chain = ["open", "start", "run", "keys", "rec"];
    for pair in chain.windows(2) {
        g.link_ports(pair[0], pair[1], "terminal", "terminal");
    }
    let wf = g.build();
    let compiled = compile_workflow(&wf, None);
    let Some(CompileValue::TerminalSession(session)) = compiled.values["keys"].get("terminal")
    else {
        panic!("no session");
    };
    assert_eq!((session.cols, session.rows), (100, 30));
    let kinds: Vec<&str> = session
        .steps
        .iter()
        .map(|s| match &s.action {
            TerminalAction::StartRecording => "start",
            TerminalAction::Run { command, .. } if command == "ls -la" => "run",
            TerminalAction::Keys { keys, .. } if keys == "Control+c" => "keys",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, ["start", "run", "keys"]);
    let Some(CompileValue::Recording(take)) = compiled.values["rec"].get("recording") else {
        panic!("no take");
    };
    assert_eq!(take.kind(), TakeKind::Terminal);
}

#[test]
fn draws_a_terminal_take_as_its_own_screen_in_pixels_of_its_look() {
    let clip = clip_of(&terminal_take(false), &[], json!({}));
    let props = recording_props(&clip)[0];
    let ScreenProps::Terminal {
        cols,
        rows,
        look,
        cast,
        ..
    } = &props.screen
    else {
        panic!("not a terminal screen");
    };
    assert_eq!(
        (*cols, *rows, cast.as_str()),
        (80, 24, "project:.opsis/cache/t/take.cast")
    );
    let cells = grid(look, 80, 24);
    assert_eq!(props.log.viewport, screen_size(&cells));
    let rect = props.log.actions[0].rect.unwrap();
    assert_eq!((rect.x, rect.w), (cells.x(10.0), cells.width * 2.0));
    assert!(!props.pointer.shown);
    assert!((props.speed - 1.0).abs() < f64::EPSILON);
    let with_pointer = clip_of(&terminal_take(true), &[], json!({}));
    assert!(recording_props(&with_pointer)[0].pointer.shown);
}

proptest! {
    #[test]
    fn keeps_every_mark_on_its_cells_whatever_the_look(size in 10u32..48, spacing in 0u32..4, padding in 0u32..80) {
        let look = json!({ "fontSize": size, "letterSpacing": spacing, "padding": padding });
        let clip = clip_of(&terminal_take(true), &[("camera.followCursor", json!({})), ("terminal.look", look)], json!({}));
        let props = recording_props(&clip)[0];
        let cells = props.log.cells.unwrap();
        let rect = props.log.actions[0].rect.unwrap();
        prop_assert!((cells.col(rect.x) - 10.0).abs() < 1e-9);
        prop_assert!((cells.row(rect.y) - 0.0).abs() < 1e-9);
        prop_assert!((rect.w / cells.width - 2.0).abs() < 1e-9);
        let follow = props.extra_beats.first().and_then(|b| b.point).unwrap();
        prop_assert!((cells.col(follow.x) - 40.5).abs() < 1e-9);
        prop_assert!((cells.row(follow.y) - 12.5).abs() < 1e-9);
    }
}

#[test]
fn follows_the_text_cursor_when_a_terminal_take_has_no_pointer() {
    let clip = clip_of(
        &terminal_take(false),
        &[("camera.followCursor", json!({ "everyMs": 200 }))],
        json!({}),
    );
    let props = recording_props(&clip)[0];
    let cells = props.log.cells.unwrap();
    let points: Vec<(f64, f64)> = props
        .extra_beats
        .iter()
        .filter_map(|b| b.point)
        .map(|p| (cells.col(p.x), cells.row(p.y)))
        .collect();
    assert_eq!(points, [(2.5, 0.5), (12.5, 3.5)]);
}

#[test]
fn plays_each_kind_of_take_at_its_usual_speed_until_set_by_hand() {
    let browser = clip_of(&take_recording(None), &[], json!({}));
    assert!((recording_props(&browser)[0].speed - 1.25).abs() < f64::EPSILON);
    let set = clip_of(&terminal_take(false), &[], json!({ "speed": 2 }));
    assert!((recording_props(&set)[0].speed - 2.0).abs() < f64::EPSILON);
}

fn take_card(kind: TakeKind, style: &[&str]) -> Vec<String> {
    card_texts(
        "camera.take",
        json!({}),
        CardInputs {
            takes: IndexMap::from([(
                "recording".to_owned(),
                Some(InputTake {
                    kind,
                    duration_ms: 3000.0,
                    cursor_track: true,
                }),
            )]),
            style: IndexMap::from([(
                "style".to_owned(),
                style.iter().map(|s| (*s).to_owned()).collect(),
            )]),
            ..CardInputs::default()
        },
    )
}

#[test]
fn leaves_a_browser_take_as_it_is_under_a_terminal_look_and_says_so() {
    let plain = clip_of(&take_recording(None), &[], json!({}));
    let looked = clip_of(
        &take_recording(None),
        &[("terminal.look", json!({ "fontSize": 30 }))],
        json!({}),
    );
    assert_eq!(recording_props(&plain), recording_props(&looked));
    assert!(
        take_card(TakeKind::Browser, &["terminal.look"])
            .contains(&"Terminal look does nothing on a browser take".to_owned())
    );
    assert!(
        !take_card(TakeKind::Terminal, &["terminal.look"])
            .iter()
            .any(|t| t.contains("does nothing"))
    );
}

#[test]
fn gives_a_terminal_screen_its_footage_use_and_frames_address() {
    let clip = clip_of(
        &terminal_take(false),
        &[("terminal.look", json!({ "theme": "Dracula" }))],
        json!({}),
    );
    let uses = footage_uses(&clip.tracks);
    assert_eq!(uses.len(), 1);
    assert_eq!(
        uses[0].terminal.as_ref().map(|t| t.name.as_str()),
        Some("Dracula")
    );
    let plain = footage_uses(&clip_of(&terminal_take(false), &[], json!({})).tracks);
    assert_ne!(plain[0].id, uses[0].id);
    let spec: TimelineSpec = serde_json::from_value(json!({
        "width": 1920, "height": 1080, "fps": 30, "durationInFrames": clip.duration_in_frames,
        "tracks": clip.tracks, "audio": []
    }))
    .unwrap();
    let spec = TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some("http://127.0.0.1:9/p/".to_owned()),
        }),
        ..spec
    };
    let hosted = hosted(&spec);
    let props = serde_json::to_value(&hosted.tracks[0].items[0]).unwrap();
    let url = props["props"]["screen"]["framesUrl"].as_str().unwrap();
    assert!(
        url.starts_with("http://127.0.0.1:9/p/.opsis/footage/") && url.ends_with("/frames"),
        "{url}"
    );
}

#[test]
fn checks_themes_and_text_patterns() {
    assert!(theme_named("Dracula").is_some() && theme_named("Default").is_some());
    assert_eq!(
        parse_params("terminal.look", json!({ "theme": "No such theme" })).unwrap_err(),
        "There's no theme with this name"
    );
    assert!(parse_params("effect.redactTake", json!({ "textToHide": ["sk-[0-9]+"] })).is_ok());
    assert!(parse_params("effect.redactTake", json!({ "textToHide": ["sk-[0-9"] })).is_err());
    assert_eq!(
        parse_params("terminal.run", json!({ "command": " " })).unwrap_err(),
        "Type a command"
    );
    assert_eq!(default_look(None).theme.name, "Default");
}

#[test]
fn names_each_step_after_its_node() {
    let mut g = Graph::default();
    g.node("open", "terminal.open", json!({}));
    g.node("run", "terminal.run", json!({}));
    g.link_ports("open", "run", "terminal", "terminal");
    let compiled = compile_workflow(&g.build(), None);
    let Some(CompileValue::TerminalSession(session)) = compiled.values["run"].get("terminal")
    else {
        panic!("no session");
    };
    assert_eq!(session.steps[0].label, "Run command");
}

#[test]
fn plays_a_tape_from_its_result_and_says_what_it_left_out() {
    let none = crate::helpers::compile_node_with(
        "terminal.playTape",
        json!({ "file": "project:demo.tape" }),
        json!({}),
        None,
    )
    .unwrap();
    assert!(matches!(&none["style"], CompileValue::RecordingStyle(s) if s.entries.is_empty()));
    assert!(
        matches!(&none["recording"], CompileValue::Recording(r) if r.kind() == TakeKind::Terminal && r.file().is_none())
    );
    let style = json!({ "type": "recording-style", "entries": style_entries(&[("terminal.look", json!({ "fontSize": 30 }))], "play") });
    let result = crate::helpers::node_result(json!({
        "outputs": { "recording": terminal_take(false), "style": style },
        "measured": [{ "type": "tape", "steps": 7, "ignored": ["Set FontFamily", "Output"], "warnings": ["There's no theme called “X”, so the usual colours are used"] }]
    }));
    let played = crate::helpers::compile_node_with(
        "terminal.playTape",
        json!({ "file": "project:demo.tape" }),
        json!({}),
        Some(&result),
    )
    .unwrap();
    assert!(matches!(&played["style"], CompileValue::RecordingStyle(s) if s.entries.len() == 1));
    assert!(matches!(&played["recording"], CompileValue::Recording(r) if r.file().is_some()));
    let rows: Vec<String> = crate::helpers::node_card_with(
        "terminal.playTape",
        json!({ "file": "project:tapes/demo.tape" }),
        CardInputs::default(),
        Some(&result),
    )
    .into_iter()
    .map(|r| r.text)
    .collect();
    assert_eq!(
        rows,
        vec![
            "demo.tape",
            "4 s",
            "7 actions",
            "There's no theme called “X”, so the usual colours are used",
        ]
    );
}

#[test]
fn a_terminal_look_without_a_theme_keeps_the_takes_own_colours() {
    let mut take = terminal_take(false);
    take["source"]["theme"] = serde_json::to_value(theme_named("Dracula").unwrap()).unwrap();
    let clip = clip_of(
        &take,
        &[("terminal.look", json!({ "fontSize": 30 }))],
        json!({}),
    );
    let ScreenProps::Terminal { look, .. } = &recording_props(&clip)[0].screen else {
        panic!("not a terminal screen");
    };
    assert_eq!((look.theme.name.as_str(), look.font_size), ("Dracula", 30));
}

#[test]
fn export_tape_names_its_file_and_claims_it() {
    let texts = card_texts(
        "output.tape",
        json!({ "fileName": "demo" }),
        CardInputs::default(),
    );
    assert_eq!(texts, vec!["demo.tape", "Connect the terminal actions"]);
}

#[test]
fn a_cast_file_says_its_length_size_and_events() {
    let result = crate::helpers::node_result(json!({
        "outputs": { "recording": terminal_take(false) },
        "measured": [{ "type": "cast", "events": 120 }]
    }));
    let rows: Vec<String> = crate::helpers::node_card_with(
        "terminal.castFile",
        json!({ "file": "project:casts/demo.cast" }),
        CardInputs::default(),
        Some(&result),
    )
    .into_iter()
    .map(|r| r.text)
    .collect();
    assert_eq!(rows, vec!["demo.cast", "4 s", "80 × 24", "120 events"]);
    assert_eq!(
        card_texts("terminal.castFile", json!({}), CardInputs::default()),
        vec!["Choose a cast file"]
    );
}

#[test]
fn texture_and_lens_on_a_terminal_take_become_its_page_effects() {
    let take = clip_of(&terminal_take(false), &[], json!({}));
    let grade = |kind: &str, params: Value| json!({ "type": "grade", "entries": [{ "nodeId": "g", "type": kind, "params": params }] });
    let graded = compile_clip(
        "effect.gradeFootage",
        json!({}),
        json!({ "clip": crate::helpers::clip_value(&take), "grade": grade("grade.texture", json!({ "vignette": 0.6, "grain": 0.3 })) }),
    );
    let ScreenProps::Terminal { effects, chain, .. } = &recording_props(&graded)[0].screen else {
        panic!("not a terminal screen");
    };
    let effects = effects.clone().unwrap();
    assert!((effects.vignette - 0.6).abs() < 1e-9 && (effects.grain - 0.3).abs() < 1e-9);
    assert_eq!(chain.grade.len(), 1);
    let coloured = compile_clip(
        "effect.gradeFootage",
        json!({}),
        json!({ "clip": crate::helpers::clip_value(&take), "grade": grade("grade.colour", json!({})) }),
    );
    let ScreenProps::Terminal { effects, .. } = &recording_props(&coloured)[0].screen else {
        panic!("not a terminal screen");
    };
    assert_eq!(effects, &None);
}
