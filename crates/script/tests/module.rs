mod support;

use indexmap::IndexMap;
use opsis_types::edits::{
    EdgeEnds, KeyframeEdit, MarkerEdit, MixEdit, NodePatch, SettingsPatch, TimelineEdit,
    WorkflowEdit,
};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::fonts::FontRole;
use opsis_types::shell::PlaybackCall;
use opsis_types::timeline::EnvelopeKey;
use opsis_types::workflow::Position;
use serde_json::{Map, Value, json};
use support::{Asked, FakeHost, run_with};

fn script(source: &str) -> (std::rc::Rc<FakeHost>, support::Ran) {
    let dir = tempfile::tempdir().unwrap();
    let host = FakeHost::new();
    let ran = run_with(
        dir.path(),
        std::rc::Rc::clone(&host),
        source,
        std::sync::Arc::default(),
    );
    (host, ran)
}

fn asked(source: &str) -> Vec<Asked> {
    let (host, ran) = script(source);
    assert!(
        ran.ending.failure.is_none(),
        "{:?}\n{:?}",
        ran.ending.failure,
        ran.lines
    );
    host.asked.borrow().clone()
}

fn params(value: &Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn reaches_every_node_and_connection_edit() {
    let edits = asked(
        r#"
local shot = opsis.nodes.add { type = "browser.navigate", id = "shot", label = "Landing", position = { x = 1, y = 2 }, params = { url = "https://example.com" } }
shot.label = "Landing page"
shot.params.waitUntil = "load"
shot.disabled = true
shot.position = { x = 5, y = 6 }
opsis.nodes.move { shot = { x = 7, y = 8 } }
local edge = opsis.edges.connect("session", "session", shot, "session")
opsis.edges.disconnect(edge)
local group = opsis.nodes.group({ shot, "open" }, "Pair")
opsis.nodes.ungroup(group)
opsis.nodes.paste { nodes = {}, edges = {}, offset = { x = 10, y = 0 } }
shot:remove()
"#,
    );
    let expected = [
        WorkflowEdit::AddNode {
            kind: "browser.navigate".into(),
            id: Some("shot".into()),
            label: Some("Landing".into()),
            position: Some(Position { x: 1.0, y: 2.0 }),
            params: Some(params(&json!({ "url": "https://example.com" }))),
        },
        WorkflowEdit::UpdateNode {
            id: "shot".into(),
            patch: NodePatch {
                label: Some("Landing page".into()),
                ..NodePatch::default()
            },
        },
        WorkflowEdit::UpdateNode {
            id: "shot".into(),
            patch: NodePatch {
                params: Some(params(&json!({ "waitUntil": "load" }))),
                ..NodePatch::default()
            },
        },
        WorkflowEdit::UpdateNode {
            id: "shot".into(),
            patch: NodePatch {
                disabled: Some(true),
                ..NodePatch::default()
            },
        },
        WorkflowEdit::UpdateNode {
            id: "shot".into(),
            patch: NodePatch {
                position: Some(Position { x: 5.0, y: 6.0 }),
                ..NodePatch::default()
            },
        },
        WorkflowEdit::MoveNodes {
            positions: IndexMap::from([("shot".to_owned(), Position { x: 7.0, y: 8.0 })]),
        },
        WorkflowEdit::AddEdge {
            ends: EdgeEnds {
                source: "session".into(),
                source_handle: "session".into(),
                target: "shot".into(),
                target_handle: "session".into(),
            },
        },
        WorkflowEdit::RemoveEdges {
            ids: vec!["session.session->shot.session".into()],
        },
    ];
    let got: Vec<WorkflowEdit> = edits
        .into_iter()
        .filter_map(|a| match a {
            Asked::Edit(edit) => Some(edit),
            _ => None,
        })
        .collect();
    assert_eq!(got[..expected.len()], expected[..]);
    assert!(
        matches!(&got[8], WorkflowEdit::Group { ids, title, .. } if ids == &["shot", "open"] && title == "Pair")
    );
    assert!(matches!(&got[9], WorkflowEdit::Ungroup { .. }));
    assert!(
        matches!(&got[10], WorkflowEdit::Paste { offset, .. } if opsis_types::numbers::exact_eq(offset.x, 10.0))
    );
    assert_eq!(
        got[11],
        WorkflowEdit::RemoveNodes {
            ids: vec!["shot".into()]
        }
    );
}

#[test]
fn reaches_the_project_variables_fonts_mixer_markers_and_keyframes() {
    let edits = asked(
        r#"
opsis.project.name = "Renamed"
opsis.project.settings.fps = 60
opsis.variables.set("Greeting", "Hello")
opsis.variables.rename("Greeting", "Hi")
opsis.fonts.role("heading", "Inter")
opsis.mixer.edit { op = "addBus" }
opsis.timeline.markers { op = "add", atMs = 1500 }
opsis.timeline.keyframes { op = "addKey", nodeId = "open", track = "opacity", frame = 3 }
"#,
    );
    let got: Vec<WorkflowEdit> = edits
        .into_iter()
        .filter_map(|a| match a {
            Asked::Edit(edit) => Some(edit),
            _ => None,
        })
        .collect();
    let mut variables = IndexMap::new();
    variables.insert(
        "Site address".to_owned(),
        "https://demo.playwright.dev/todomvc".to_owned(),
    );
    variables.insert("Greeting".to_owned(), "Hello".to_owned());
    assert_eq!(
        got,
        vec![
            WorkflowEdit::Rename {
                name: "Renamed".into()
            },
            WorkflowEdit::SetSettings {
                patch: SettingsPatch {
                    fps: Some(60),
                    ..SettingsPatch::default()
                }
            },
            WorkflowEdit::SetVariables { variables },
            WorkflowEdit::RenameVariable {
                from: "Greeting".into(),
                to: "Hi".into()
            },
            WorkflowEdit::SetFontRole {
                role: FontRole::Heading,
                family: "Inter".into()
            },
            WorkflowEdit::Mix {
                edit: MixEdit::AddBus
            },
            WorkflowEdit::Markers {
                edit: MarkerEdit::Add { at_ms: 1500.0 }
            },
            WorkflowEdit::Keyframes {
                edit: KeyframeEdit::AddKey {
                    node_id: "open".into(),
                    track: EnvelopeKey::Opacity,
                    frame: 3.0
                }
            },
        ]
    );
}

#[test]
fn reaches_timeline_runs_playback_selection_and_saving() {
    let edits = asked(
        r#"
opsis.timeline.edit { op = "move", bar = "b", delta = 2, ripple = false }
local run = opsis.run.start { nodes = { opsis.nodes.open } }
assert(run.id == "r1")
opsis.run.start { from = "open" }
opsis.run.stop()
opsis.run.clear_results()
opsis.run.clear_log()
opsis.playback.seek(12)
opsis.playback.play()
opsis.playback.pause()
opsis.playback.rate(2)
opsis.selection.set { "open", opsis.nodes.session }
assert(opsis.selection.get()[1] == "a")
assert(opsis.project.save())
assert(opsis.fonts.use("inter") == "Inter")
assert(run:result("render").file == ".opsis/out/film.mp4")
"#,
    );
    assert_eq!(
        edits,
        vec![
            Asked::Timeline(TimelineEdit::Move {
                bar: "b".into(),
                delta: 2.0,
                ripple: false
            }),
            Asked::Run(RunTargets {
                target_node_ids: Some(vec!["open".into()]),
                from_node_id: None
            }),
            Asked::Run(RunTargets {
                target_node_ids: None,
                from_node_id: Some("open".into())
            }),
            Asked::StopRun,
            Asked::ClearResults,
            Asked::ClearLog,
            Asked::Playback(PlaybackCall::Seek { frame: 12.0 }),
            Asked::Playback(PlaybackCall::Play),
            Asked::Playback(PlaybackCall::Pause),
            Asked::Playback(PlaybackCall::Rate { rate: 2.0 }),
            Asked::Select(vec!["open".into(), "session".into()]),
            Asked::Save,
            Asked::UseFont("inter".into()),
        ]
    );
}

#[test]
fn reads_nodes_live_and_says_when_one_is_gone() {
    let (_, ran) = script(
        r#"
local open = opsis.nodes.open
assert(open.type == "browser.navigate" and open.params.waitUntil == "load")
assert(opsis.nodes.get("open") == open)
local count = 0
for id, node in pairs(opsis.nodes) do count = count + 1 end
assert(count == #opsis.nodes and count > 3)
assert(#opsis.nodes.find { type = "browser.navigate" } >= 1)
open.params.waitUntil = "networkidle"
assert(opsis.nodes.open.params.waitUntil == "networkidle")
assert(opsis.project.name == "Sample: to-do app")
assert(opsis.version == "9.9.9")
assert(opsis.validate(opsis.project.workflow).ok)
open:remove()
return open.label
"#,
    );
    let failure = ran.ending.failure.unwrap();
    assert!(
        failure.message.ends_with("This node is gone."),
        "{}",
        failure.message
    );
    assert_eq!(failure.line, Some(15));
}

#[test]
fn iterates_a_runs_events_until_it_is_done() {
    let dir = tempfile::tempdir().unwrap();
    let host = FakeHost::new();
    *host.events.borrow_mut() = vec![
        RunEvent::NodeStart {
            run_id: "other".into(),
            node_id: "x".into(),
            at: 0,
        },
        RunEvent::NodeStart {
            run_id: "r1".into(),
            node_id: "open".into(),
            at: 1,
        },
        RunEvent::NodeError {
            run_id: "r1".into(),
            node_id: "open".into(),
            message: "Page not found".into(),
            stack: None,
            trace_id: None,
        },
        RunEvent::RunDone {
            run_id: "r1".into(),
            ok: false,
            cancelled: false,
            error: None,
            failed_node_ids: None,
            skipped_node_ids: None,
            outputs: Vec::new(),
            at: 2,
        },
    ];
    let ran = run_with(
        dir.path(),
        host,
        r#"
local seen = {}
local run = opsis.run.start()
for event in run:events() do
  seen[#seen + 1] = event.type
end
assert(table.concat(seen, ",") == "node:start,node:error,run:done", table.concat(seen, ","))
assert(run:missed() == 0)
run:wait()
"#,
        std::sync::Arc::default(),
    );
    assert!(ran.ending.failure.is_none(), "{:?}", ran.ending.failure);
}

#[test]
fn makes_an_empty_list_that_crosses_as_a_list() {
    let (_, ran) = script("return { list = opsis.list(), map = {} }");
    assert_eq!(ran.ending.value, Some(json!({ "list": [], "map": {} })));
}

const LEFT_OUT: [(&str, &str); 16] = [
    ("get", "/openapi.json"),
    ("post", "/open"),
    ("get", "/settings"),
    ("get", "/engine"),
    ("get", "/paths"),
    ("get", "/about"),
    ("get", "/templates"),
    ("get", "/projects"),
    ("post", "/projects"),
    ("get", "/project/run/last"),
    ("get", "/project/traces"),
    ("post", "/project/undo"),
    ("post", "/project/redo"),
    ("get", "/project/script"),
    ("post", "/project/script"),
    ("post", "/project/script/stop"),
];

const LUA_CALLS: [(&str, &str, &str); 20] = [
    ("get", "/status", "opsis.version"),
    ("get", "/nodes", "opsis.catalog"),
    ("get", "/nodes/{type}", "opsis.catalog"),
    ("post", "/validate", "opsis.validate"),
    ("get", "/project", "opsis.project.workflow"),
    ("put", "/project", "opsis.project.replace"),
    ("post", "/project/nodes", "opsis.nodes.add"),
    ("patch", "/project/nodes/{id}", "opsis.nodes.get"),
    ("delete", "/project/nodes/{id}", "opsis.nodes.remove"),
    ("post", "/project/edges", "opsis.edges.connect"),
    ("delete", "/project/edges/{id}", "opsis.edges.disconnect"),
    ("patch", "/project/settings", "opsis.project.settings"),
    ("put", "/project/variables", "opsis.variables.set"),
    ("get", "/project/problems", "opsis.project.problems"),
    ("post", "/project/save", "opsis.project.save"),
    ("get", "/project/run", "opsis.run.active"),
    ("post", "/project/run", "opsis.run.start"),
    ("post", "/project/run/stop", "opsis.run.stop"),
    ("get", "/project/run/events", "opsis.run.start"),
    ("get", "/fonts", "opsis.fonts.list"),
];

#[test]
fn has_a_lua_call_for_every_local_api_route_but_the_ones_left_out() {
    let document = opsis_api::openapi::openapi_document("0");
    let mut missing = Vec::new();
    for (path, methods) in document["paths"].as_object().unwrap() {
        for method in methods.as_object().unwrap().keys() {
            let route = (method.as_str(), path.as_str());
            let known =
                LEFT_OUT.contains(&route) || LUA_CALLS.iter().any(|(m, p, _)| (*m, *p) == route);
            if !known {
                missing.push(format!("{method} {path}"));
            }
        }
    }
    assert!(missing.is_empty(), "no Lua call for {missing:?}");
    let checks: Vec<String> = LUA_CALLS
        .iter()
        .map(|(_, _, call)| format!("assert({call} ~= nil, \"{call}\")"))
        .collect();
    let (_, ran) = script(&checks.join("\n"));
    assert!(ran.ending.failure.is_none(), "{:?}", ran.ending.failure);
}

#[test]
fn stops_a_script_waiting_on_a_runs_events() {
    let dir = tempfile::tempdir().unwrap();
    let host = FakeHost::new();
    host.running.set(true);
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&stop);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let started = std::time::Instant::now();
    let ran = run_with(dir.path(), host, "opsis.run.start():wait()", stop);
    assert!(ran.ending.stopped, "{:?}", ran.ending);
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
}
