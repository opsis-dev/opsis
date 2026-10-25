use opsis_compile::check::check_workflow;
use opsis_compile::graph::IssueLevel;
use opsis_compile::recording::default_recording_props;
use opsis_compile::text::{
    display_address, format_bytes, format_duration, format_precise_duration, format_when, humanize,
    pascal_case, percent_text, plural, plural_with, signed_text, single_quoted,
};
use opsis_types::actions::{ActionLog, Viewport};
use opsis_types::timeline::TimelineSpec;
use serde_json::{Value, json};

use crate::helpers::template;

#[test]
fn writes_sizes_in_binary_units_and_nothing_for_unknown_ones() {
    assert_eq!(
        [
            format_bytes(512.0),
            format_bytes(2048.0),
            format_bytes(5_242_880.0),
            format_bytes(3.0 * 1024f64.powi(3))
        ],
        ["512 B", "2.0 KB", "5.0 MB", "3.0 GB"]
    );
    assert_eq!([format_bytes(-1.0), format_bytes(f64::NAN)], ["", ""]);
}

#[test]
fn writes_durations_in_seconds_minutes_and_hours() {
    let cases = [
        (400.0, "0.4 s"),
        (1500.0, "1.5 s"),
        (14_520.0, "14.5 s"),
        (65_000.0, "1 min 5 s"),
        (119_600.0, "2 min"),
        (36_000_000.0, "10 h"),
        (5_400_000.0, "1 h 30 min"),
        (3_605_000.0, "1 h 5 s"),
    ];
    for (ms, text) in cases {
        assert_eq!(format_duration(ms), text, "{ms}");
    }
    assert_eq!(
        [
            format_precise_duration(12.0),
            format_precise_duration(1500.0),
            format_precise_duration(61_234.0)
        ],
        ["0.012 s", "1.5 s", "1 min 1.234 s"]
    );
}

#[test]
fn writes_keys_as_names_and_labels() {
    assert_eq!(
        [pascal_case("my_icon-x"), pascal_case("arrow-up-0-1")],
        ["MyIconX", "ArrowUp01"]
    );
    assert_eq!(
        [
            humanize("fooBar"),
            humanize("end_time"),
            humanize("ease-in-out"),
            humanize("x2Y")
        ],
        ["Foo bar", "End time", "Ease in out", "X2 y"]
    );
}

#[test]
fn shows_addresses_without_scheme_www_or_trailing_slash() {
    assert_eq!(
        display_address("https://www.saucedemo.com/", false),
        "saucedemo.com"
    );
    assert_eq!(
        display_address("https://www.example.com/a/?x=1", false),
        "example.com/a"
    );
    assert_eq!(
        display_address("https://www.example.com/a/?x=1", true),
        "example.com/a?x=1"
    );
    assert_eq!(
        display_address("{{baseUrl}}/login/", false),
        "{{baseUrl}}/login"
    );
}

#[test]
fn quotes_text_for_a_shell_and_writes_numbers_people_read() {
    assert_eq!(single_quoted("/home/me/api.sock"), "'/home/me/api.sock'");
    assert_eq!(
        single_quoted("/home/o'brien/api.sock"),
        "'/home/o'\\''brien/api.sock'"
    );
    assert_eq!(
        [
            percent_text(0.504, f64::INFINITY),
            percent_text(1.7, f64::INFINITY),
            percent_text(1.7, 100.0)
        ],
        ["50%", "170%", "100%"]
    );
    assert_eq!(format_when("2026-10-01T09:41:00"), "1 Oct 2026, 09:41");
    assert_eq!(format_when("next week"), "next week");
    assert_eq!(
        [
            plural(1.0, "clip"),
            plural(3.0, "clip"),
            plural_with(0.0, "font family", "font families")
        ],
        ["1 clip", "3 clips", "0 font families"]
    );
    assert_eq!(
        [
            signed_text(-14.25, Some(1), false),
            signed_text(1.24, Some(1), true),
            signed_text(1.24, Some(1), false),
            signed_text(-0.01, Some(1), true),
            signed_text(-23.0, None, false)
        ],
        ["−14.2", "+1.2", "1.2", "0.0", "−23"]
    );
}

fn spec_with(item: Value) -> Value {
    json!({ "width": 1920, "height": 1080, "fps": 30, "durationInFrames": 30, "tracks": [{ "id": "t", "items": [item] }] })
}

fn accepts(item: Value) -> bool {
    serde_json::from_value::<TimelineSpec>(spec_with(item)).is_ok()
}

fn recording_props() -> Value {
    let log = ActionLog {
        viewport: Viewport {
            width: 1280.0,
            height: 720.0,
        },
        dpr: 1.0,
        t0: 0.0,
        duration_ms: 1000.0,
        actions: Vec::new(),
        cursor_track: None,
    };
    serde_json::to_value(default_recording_props(
        "project:take.mp4".into(),
        log,
        "Take".into(),
    ))
    .unwrap()
}

#[test]
fn reads_timeline_items_by_kind_and_refuses_props_that_dont_fit() {
    let place = json!({ "id": "i", "from": 0, "durationInFrames": 30 });
    let with = |extra: Value| {
        let mut item = place.clone();
        item.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        item
    };
    let recording = with(json!({ "kind": "recording", "props": recording_props() }));
    let element =
        with(json!({ "kind": "element", "component": "core/title", "props": { "title": "Hi" } }));
    let group = with(
        json!({ "kind": "group", "props": { "tracks": [{ "id": "n", "items": [recording] }] } }),
    );
    assert!(accepts(group.clone()) && accepts(element));
    let parsed: TimelineSpec = serde_json::from_value(spec_with(group.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&parsed).unwrap()["tracks"][0]["items"][0]["props"]["tracks"][0]["id"],
        "n"
    );
    let mut bad_log = recording_props();
    bad_log["log"] = json!("take");
    let mut bad_style = recording_props();
    bad_style["style"] = json!("fancy");
    assert!(!accepts(with(
        json!({ "kind": "group", "props": recording_props() })
    )));
    assert!(!accepts(with(
        json!({ "kind": "recording", "props": bad_log })
    )));
    assert!(!accepts(with(json!({ "kind": "element", "props": {} }))));
    assert!(!accepts(with(
        json!({ "component": "core/group", "props": { "tracks": [] } })
    )));
    let bad = with(json!({ "kind": "recording", "props": bad_style }));
    assert!(!accepts(with(
        json!({ "kind": "group", "props": { "tracks": [{ "id": "n", "items": [bad] }] } })
    )));
}

#[test]
fn accepts_a_template_project_and_fails_an_empty_required_input() {
    let result = check_workflow(&template("todomvc"));
    assert!(result.ok);
    assert!(result.workflow.unwrap().nodes.len() > 5);
    let mut wf = template("todomvc");
    let edges = wf["edges"].as_array_mut().unwrap();
    edges.retain(|e| !(e["target"] == "render" && e["targetHandle"] == "clip"));
    let result = check_workflow(&wf);
    assert!(!result.ok);
    assert!(
        result
            .problems
            .iter()
            .any(|p| p.issue.level == IssueLevel::Error
                && p.issue.node_id.as_deref() == Some("render")
                && p.issue.message == "Connect something to “Clip”")
    );
}

#[test]
fn reports_format_problems_with_a_path() {
    let result = check_workflow(
        &json!({ "version": 1, "id": "x", "name": "x", "nodes": [{ "id": "a" }], "edges": [] }),
    );
    assert!(!result.ok && result.workflow.is_none());
    assert!(
        result.problems.iter().any(|p| p
            .path
            .as_deref()
            .is_some_and(|path| path.starts_with("nodes[0]"))),
        "{:?}",
        result.problems
    );
}

#[test]
fn reports_unknown_types_bad_connections_duplicate_ids_and_invalid_settings() {
    let mut wf = template("todomvc");
    let first = wf["nodes"][0].clone();
    let nodes = wf["nodes"].as_array_mut().unwrap();
    nodes.push(json!({ "id": "ghost", "type": "nope.nothing", "position": { "x": 0, "y": 0 }, "params": {} }));
    nodes.push(first.clone());
    wf["edges"].as_array_mut().unwrap().push(json!({ "id": "bad", "source": first["id"], "sourceHandle": "no-such-port", "target": "ghost", "targetHandle": "in" }));
    let result = check_workflow(&wf);
    assert!(!result.ok);
    let messages: Vec<&str> = result
        .problems
        .iter()
        .map(|p| p.issue.message.as_str())
        .collect();
    assert!(
        messages.contains(&"This node type isn't available")
            && messages.contains(&"Two nodes share this id"),
        "{messages:?}"
    );
    assert!(
        result
            .problems
            .iter()
            .any(|p| p.issue.edge_id.as_deref() == Some("bad"))
    );
    let mut wf = template("todomvc");
    let navigate = wf["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["type"] == "browser.navigate")
        .unwrap();
    navigate["params"]["url"] = json!(42);
    let id = navigate["id"].as_str().unwrap().to_owned();
    let result = check_workflow(&wf);
    assert!(
        result
            .problems
            .iter()
            .any(|p| p.issue.level == IssueLevel::Error
                && p.issue.node_id.as_deref() == Some(id.as_str())
                && p.issue.message == "Page address isn't valid"),
        "{:?}",
        result.problems
    );
}
