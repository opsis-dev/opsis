use std::path::Path;

use opsis_compile::graph::topo_sort;
use opsis_compile::vars::workflow_vars;
use opsis_runner::keys::{KeyInputs, hash_nodes, node_hash};
use opsis_types::workflow::Workflow;
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn cache_keys_match_the_ts_runner() {
    let f = fixture("run/keys.json");
    for case in f["nodeHashes"].as_array().unwrap() {
        let upstream: Vec<(String, String)> =
            serde_json::from_value(case["upstream"].clone()).unwrap();
        assert_eq!(
            node_hash(
                case["type"].as_str().unwrap(),
                &case["params"],
                &upstream,
                "0.1.0"
            ),
            case["hash"].as_str().unwrap(),
            "{case}"
        );
    }
    for case in f["workflows"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let wf: Workflow = serde_json::from_value(
            fixture(&format!("compile/workflows/{name}.json"))["workflow"].clone(),
        )
        .unwrap();
        let vars = workflow_vars(&wf.name, &wf.variables);
        let order = topo_sort(&wf).sorted;
        let hashes = hash_nodes(
            &order,
            &KeyInputs {
                workflow: &wf,
                vars: &vars,
                project_dir: Path::new("/nowhere"),
                app_version: "0.1.0",
            },
        );
        for (id, expected) in case["hashes"].as_object().unwrap() {
            assert_eq!(
                hashes.get(id).map(String::as_str),
                expected.as_str(),
                "{name} {id}"
            );
        }
        assert_eq!(
            hashes.len(),
            case["hashes"].as_object().unwrap().len(),
            "{name}"
        );
    }
}

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

fn key_of(wf: &Workflow, dir: &Path, id: &str) -> String {
    let vars = workflow_vars(&wf.name, &wf.variables);
    let order: Vec<_> = wf.nodes.iter().collect();
    hash_nodes(
        &order,
        &KeyInputs {
            workflow: wf,
            vars: &vars,
            project_dir: dir,
            app_version: "test",
        },
    )[id]
        .clone()
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
fn changes_the_takes_key_when_a_new_sign_in_is_saved() {
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
    let unsaved = key_of(&wf, dir.path(), "rec");
    let file = opsis_types::secrets::secret_file(dir.path(), "admin");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "first").unwrap();
    let saved = key_of(&wf, dir.path(), "rec");
    std::fs::write(&file, "a later sign-in").unwrap();
    assert_ne!(saved, unsaved);
    assert_ne!(key_of(&wf, dir.path(), "rec"), saved);
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
            ""
        ),
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("audio", "y:audio"), ("clip", "x:clip")]),
            ""
        )
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("clip", "x:clip"), ("audio", "y:audio")]),
            ""
        ),
        node_hash(
            "fx.core.title",
            &params,
            &ports(&[("clip", "y:audio"), ("audio", "x:clip")]),
            ""
        )
    );
    assert_ne!(
        node_hash("media.trim", &params, &[], "0.1.0"),
        node_hash("media.trim", &params, &[], "0.2.0")
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
        "",
    );
    assert_eq!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "fontSize": 100, "title": "Hi" }),
            &clip,
            ""
        ),
        key
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "title": "Hello", "fontSize": 100 }),
            &clip,
            ""
        ),
        key
    );
    assert_ne!(
        node_hash(
            "fx.core.title",
            &serde_json::json!({ "title": "Hi", "fontSize": 100 }),
            &upstream(&[("clip", "y:clip")]),
            ""
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
            ""
        ),
        node_hash(
            "audio.mix",
            &params,
            &upstream(&[("sounds", "b:audio"), ("sounds", "a:audio")]),
            ""
        )
    );
}
