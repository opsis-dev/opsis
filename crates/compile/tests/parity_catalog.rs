use std::path::PathBuf;

use opsis_compile::node::Platform;
use opsis_compile::registry::registry;
use opsis_types::json::first_difference;
use serde_json::{Value, json};

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/compile")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).expect("fixture exists")).expect("json")
}

fn ports(list: &[(&'static str, opsis_compile::node::PortSpec)]) -> Value {
    Value::Array(
        list.iter()
            .map(|(id, p)| json!({ "id": id, "type": p.kind.id(), "label": p.label.as_deref(), "optional": p.optional, "many": p.many }))
            .collect(),
    )
}

fn platform_name(p: Platform) -> &'static str {
    match p {
        Platform::Linux => "linux",
        Platform::Mac => "darwin",
        Platform::Windows => "win32",
    }
}

#[test]
fn node_metadata_matches() {
    let expected = fixture("nodes.json");
    let expected = expected.as_array().unwrap();
    let defs = registry().all();
    let mut problems = Vec::new();
    let kinds: Vec<&str> = defs.iter().map(|d| d.kind.as_ref()).collect();
    let wanted: Vec<&str> = expected
        .iter()
        .map(|n| n["type"].as_str().unwrap())
        .collect();
    if kinds != wanted {
        problems.push(format!(
            "order or set differs:\n rust {kinds:?}\n ts   {wanted:?}"
        ));
    }
    for node in expected {
        let kind = node["type"].as_str().unwrap();
        let Some(def) = registry().get(kind) else {
            continue;
        };
        let host = def.host_defaults.map(|f| {
            let mut out = serde_json::Map::new();
            for p in [Platform::Linux, Platform::Mac, Platform::Windows] {
                out.insert(platform_name(p).to_owned(), Value::Object(f(p)));
            }
            Value::Object(out)
        });
        let actual = json!({
            "type": kind,
            "title": def.title,
            "category": def.category,
            "description": def.description,
            "phase": def.phase,
            "inputs": ports(&def.inputs),
            "outputs": ports(&def.outputs),
            "virtual": def.is_virtual,
            "action": def.action,
            "executes": def.executes,
            "secretParams": def.secret_params,
            "usesProjectSize": def.uses_project_size,
            "element": def.element,
            "hidden": def.hidden,
            "cardHeight": def.card_height,
            "compiles": def.has_compile(),
            "hasCard": def.has_card(),
            "defaults": def.defaults(),
            "mutates": def.parse(&def.defaults(), None).ok().and_then(|v| def.mutation(&v)),
            "hostDefaults": host,
        });
        let mut wanted = node.clone();
        wanted.as_object_mut().unwrap().remove("params");
        let blank_targets: Vec<String> = def
            .params
            .keys()
            .iter()
            .filter(|k| {
                def.params.field(k).is_some_and(|f| {
                    f["x-widget"] == "target" && f["x-check"]["message"] == "Pick an element"
                })
            })
            .filter(|k| wanted["defaults"].get(k.as_str()).is_none())
            .cloned()
            .collect();
        for key in blank_targets {
            wanted["defaults"][key] = Value::from("");
        }
        if let Some(d) = first_difference(&wanted, &actual) {
            problems.push(format!("{kind}{d}"));
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

fn property_defaults(schema: &Value) -> Vec<(String, Value)> {
    schema["properties"]
        .as_object()
        .map(|p| {
            p.iter()
                .map(|(k, v)| (k.clone(), v.get("default").cloned().unwrap_or(Value::Null)))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn catalogue_matches() {
    let expected = fixture("catalog.json");
    let mut problems = Vec::new();
    let list = serde_json::to_value(opsis_compile::catalog::node_catalog()).unwrap();
    if let Some(d) = first_difference(&expected["list"], &list) {
        problems.push(format!("list{d}"));
    }
    for entry in expected["entries"].as_array().unwrap() {
        let kind = entry["type"].as_str().unwrap();
        let Some(actual) = opsis_compile::catalog::catalog_entry(kind) else {
            problems.push(format!("{kind}: missing"));
            continue;
        };
        let mut actual = serde_json::to_value(actual).unwrap();
        let params = actual.as_object_mut().unwrap().remove("params").unwrap();
        let mut wanted = entry.clone();
        let wanted_params = wanted.as_object_mut().unwrap().remove("params").unwrap();
        if let Some(d) = first_difference(&wanted, &actual) {
            problems.push(format!("{kind}{d}"));
        }
        let names = |s: &Value| {
            property_defaults(s)
                .into_iter()
                .map(|(k, _)| k)
                .collect::<Vec<_>>()
        };
        if names(&wanted_params) != names(&params) {
            problems.push(format!(
                "{kind}: settings {:?} vs {:?}",
                names(&wanted_params),
                names(&params)
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
