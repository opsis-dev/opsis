use std::path::Path;

use opsis_compile::vars::workflow_vars;
use opsis_runner::context::{ExecutorOutput, Executors, Stamps, executor};
use opsis_runner::keys::{KeyInputs, Release, hash_nodes, node_hash};
use opsis_types::workflow::Workflow;
use proptest::prelude::*;
use serde_json::{Map, Value};

fn workflow(nodes: Value, edges: Value) -> Workflow {
    serde_json::from_value(
        serde_json::json!({ "version": 1, "id": "x", "name": "x", "nodes": nodes, "edges": edges }),
    )
    .unwrap()
}

fn node(id: &str, kind: &str, params: Value) -> Value {
    serde_json::json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 }, "params": params })
}

fn edge(source: &str, source_handle: &str, target: &str, target_handle: &str) -> Value {
    serde_json::json!({ "id": format!("{source}-{target}"), "source": source, "sourceHandle": source_handle, "target": target, "targetHandle": target_handle })
}

fn key_with(wf: &Workflow, dir: &Path, executors: &Executors, id: &str) -> String {
    let vars = workflow_vars(&wf.name, &wf.variables);
    let order: Vec<_> = wf.nodes.iter().collect();
    hash_nodes(
        &order,
        &KeyInputs {
            workflow: wf,
            vars: &vars,
            project_dir: dir,
            executors,
            release: Release {
                app: "test",
                engine: "",
            },
        },
    )[id]
        .clone()
}

fn key_of(wf: &Workflow, dir: &Path, id: &str) -> String {
    key_with(wf, dir, &Executors::default(), id)
}

#[test]
fn changes_every_key_after_a_node_when_skip_is_turned_on_or_off() {
    let mut wf = workflow(
        serde_json::json!([
            node("bg", "fx.solid", serde_json::json!({})),
            node("spot", "camera.crop", serde_json::json!({})),
            node("out", "output.render", serde_json::json!({}))
        ]),
        serde_json::json!([
            edge("bg", "clip", "spot", "clip"),
            edge("spot", "clip", "out", "clip")
        ]),
    );
    let shown = key_of(&wf, Path::new("/nowhere"), "out");
    wf.nodes[1].disabled = Some(true);
    let skipped = key_of(&wf, Path::new("/nowhere"), "out");
    wf.nodes[1].disabled = Some(false);
    assert_ne!(skipped, shown);
    assert_eq!(key_of(&wf, Path::new("/nowhere"), "out"), shown);
}

#[test]
fn changes_every_key_after_a_stamped_executor_when_its_stamp_changes() {
    let dir = tempfile::tempdir().unwrap();
    let wf = workflow(
        serde_json::json!([
            node("open", "browser.open", serde_json::json!({})),
            node(
                "login",
                "browser.signIn",
                serde_json::json!({ "secretName": "admin" })
            ),
            node(
                "go",
                "browser.navigate",
                serde_json::json!({ "url": "https://example.com" })
            ),
            node("rec", "capture.record", serde_json::json!({}))
        ]),
        serde_json::json!([
            edge("open", "session", "login", "session"),
            edge("login", "session", "go", "session"),
            edge("go", "session", "rec", "session")
        ]),
    );
    let file = dir.path().join("saved");
    let read = file.clone();
    let mut executors = Executors::default();
    executors
        .add(
            "browser.signIn",
            executor(|_, _| async { Ok(ExecutorOutput::default()) }).stamped(move |_, params| {
                let name = params["secretName"].as_str().unwrap_or_default();
                std::fs::read_to_string(&read)
                    .map(|text| Stamps::from([(name.to_owned(), text)]))
                    .unwrap_or_default()
            }),
        )
        .unwrap();
    let unsaved = key_with(&wf, dir.path(), &executors, "rec");
    assert_eq!(key_of(&wf, dir.path(), "rec"), unsaved);
    std::fs::write(&file, "first").unwrap();
    let saved = key_with(&wf, dir.path(), &executors, "rec");
    std::fs::write(&file, "a later sign-in").unwrap();
    assert_ne!(saved, unsaved);
    assert_ne!(key_with(&wf, dir.path(), &executors, "rec"), saved);
    assert_eq!(key_of(&wf, dir.path(), "rec"), unsaved);
}

fn params() -> impl Strategy<Value = Map<String, Value>> {
    prop::collection::btree_map(
        "[a-z]{1,6}",
        prop_oneof![
            any::<i32>().prop_map(Value::from),
            "[a-z ]{0,6}".prop_map(Value::from),
            any::<bool>().prop_map(Value::from)
        ],
        0..6,
    )
    .prop_map(|m| m.into_iter().collect())
}

fn upstream() -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec(
        (
            prop::sample::select(vec!["clip", "audio", "sounds", "in1"]),
            "[a-z]{1,4}:(clip|audio)",
        )
            .prop_map(|(port, from)| (port.to_owned(), from)),
        0..6,
    )
}

fn reversed(params: &Map<String, Value>) -> Value {
    Value::Object(
        params
            .iter()
            .rev()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

proptest! {
    #[test]
    fn keys_ignore_the_order_of_settings_and_connections(
        params in params(),
        ports in upstream(),
        shift in 0usize..6,
    ) {
        let mut moved = ports.clone();
        if !moved.is_empty() {
            let by = shift % moved.len();
            moved.rotate_left(by);
        }
        moved.reverse();
        prop_assert_eq!(
            node_hash("fx.title", &Value::Object(params.clone()), &ports, &Release::default()),
            node_hash("fx.title", &reversed(&params), &moved, &Release::default())
        );
    }

    #[test]
    fn keys_change_with_any_setting_connection_or_release(
        params in params(),
        ports in upstream(),
        key in "[a-z]{1,6}",
        value in any::<i32>(),
    ) {
        let key_of = |params: &Map<String, Value>, ports: &[(String, String)], release: &Release<'_>| {
            node_hash("fx.title", &Value::Object(params.clone()), ports, release)
        };
        let base = key_of(&params, &ports, &Release::default());
        let mut changed = params.clone();
        let next = Value::from(value);
        if changed.get(&key) != Some(&next) {
            changed.insert(key, next);
            prop_assert_ne!(&key_of(&changed, &ports, &Release::default()), &base);
        }
        let mut rewired = ports.clone();
        rewired.push(("clip".to_owned(), "zz:clip".to_owned()));
        prop_assert_ne!(&key_of(&params, &rewired, &Release::default()), &base);
        let app = Release { app: "0.2.0", engine: "" };
        let engine = Release { app: "", engine: "a:d:c" };
        prop_assert_ne!(&key_of(&params, &ports, &app), &base);
        prop_assert_ne!(key_of(&params, &ports, &engine), base);
    }
}

#[test]
fn changes_the_key_of_a_node_that_draws_text_when_the_project_fonts_change() {
    let mut wf = workflow(
        serde_json::json!([
            node("compare", "image.beforeAfter", serde_json::json!({})),
            node("crop", "image.crop", serde_json::json!({}))
        ]),
        serde_json::json!([]),
    );
    let dir = Path::new("/nowhere");
    let (labelled, cropped) = (key_of(&wf, dir, "compare"), key_of(&wf, dir, "crop"));
    wf.settings.fonts.main = "Fraunces".to_owned();
    assert_ne!(key_of(&wf, dir, "compare"), labelled);
    assert_eq!(key_of(&wf, dir, "crop"), cropped);
}
