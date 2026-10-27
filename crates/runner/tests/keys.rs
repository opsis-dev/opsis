use std::path::Path;

use opsis_compile::vars::workflow_vars;
use opsis_runner::context::{ExecutorOutput, Executors, Stamps, executor};
use opsis_runner::keys::{KeyInputs, Release, hash_nodes, node_hash};
use opsis_types::workflow::Workflow;
use serde_json::Value;

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
            node("bg", "fx.core.solid", serde_json::json!({})),
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

#[test]
fn keys_ignore_port_listing_order_and_change_with_the_app_version() {
    let ports = |list: &[(&str, &str)]| {
        list.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect::<Vec<_>>()
    };
    let params = serde_json::json!({});
    assert_eq!(
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("clip", "x:clip"), ("audio", "y:audio")]),
            &Release::default()
        ),
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("audio", "y:audio"), ("clip", "x:clip")]),
            &Release::default()
        )
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("clip", "x:clip"), ("audio", "y:audio")]),
            &Release::default()
        ),
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("clip", "y:audio"), ("audio", "x:clip")]),
            &Release::default()
        )
    );
    let release = |app, engine| Release { app, engine };
    assert_ne!(
        node_hash("media.trim", &params, &[], &release("0.1.0", "")),
        node_hash("media.trim", &params, &[], &release("0.2.0", ""))
    );
    assert_ne!(
        node_hash("media.trim", &params, &[], &release("0.1.0", "a:b:c")),
        node_hash("media.trim", &params, &[], &release("0.1.0", "a:d:c"))
    );
}

fn upstream(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn keys_are_stable_under_key_order_and_change_with_params_or_upstream() {
    let clip = upstream(&[("clip", "x:clip")]);
    let key = node_hash(
        "fx.core.title",
        &serde_json::json!({ "title": "Hi", "fontSize": 100 }),
        &clip,
        &Release::default(),
    );
    assert_eq!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "fontSize": 100, "title": "Hi" }),
            &clip,
            &Release::default()
        ),
        key
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "title": "Hello", "fontSize": 100 }),
            &clip,
            &Release::default()
        ),
        key
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "title": "Hi", "fontSize": 100 }),
            &upstream(&[("clip", "y:clip")]),
            &Release::default()
        ),
        key
    );
}

#[test]
fn keys_ignore_the_order_connections_were_made_to_a_port_that_takes_several() {
    let params = serde_json::json!({});
    assert_eq!(
        node_hash(
            "audio.mix",
            &params,
            &upstream(&[("sounds", "a:audio"), ("sounds", "b:audio")]),
            &Release::default()
        ),
        node_hash(
            "audio.mix",
            &params,
            &upstream(&[("sounds", "b:audio"), ("sounds", "a:audio")]),
            &Release::default()
        )
    );
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
