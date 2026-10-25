use std::path::PathBuf;

use indexmap::IndexMap;
use opsis_compile::node::{CardInputs, Ctx};
use opsis_compile::params::Values;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_types::json::first_difference;
use opsis_types::values::{CompileValue, NodeResult, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/compile")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).expect("fixture exists")).expect("json")
}

fn ts_defaults() -> IndexMap<String, Values> {
    fixture("nodes.json")
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["type"].as_str().unwrap().to_owned(),
                n["defaults"].as_object().unwrap().clone(),
            )
        })
        .collect()
}

fn applied(base: &Values, case: &Value) -> Values {
    let (Some(key), Some(change)) = (case["key"].as_str(), case["change"].as_object()) else {
        return if case["label"] == "empty" {
            Values::new()
        } else {
            base.clone()
        };
    };
    let mut values = base.clone();
    match change.get("set") {
        Some(set) => {
            values.insert(key.to_owned(), set.clone());
        }
        None => {
            values.shift_remove(key);
        }
    }
    values
}

fn rust_outcome(kind: &str, values: &Values, key: Option<&str>) -> Value {
    let def = node_def(kind).unwrap();
    match def.parse(values, Some(values)) {
        Err(problem) => json!({ "ok": false, "problem": problem }),
        Ok(data) => match key {
            None => json!({ "ok": true, "data": data }),
            Some(key) => {
                json!({ "ok": true, "value": data.get(key).cloned().unwrap_or(Value::Null), "keys": data.len() })
            }
        },
    }
}

fn check_messages(field: &Value) -> Vec<&str> {
    match field.get("x-check") {
        Some(Value::Array(list)) => list.iter().filter_map(|c| c["message"].as_str()).collect(),
        Some(one) => one["message"].as_str().into_iter().collect(),
        None => Vec::new(),
    }
}

fn is_blank_target_check(kind: &str, key: &str) -> bool {
    node_def(kind)
        .and_then(|d| d.params.field(key))
        .is_some_and(|f| {
            f.get("x-widget") == Some(&json!("target"))
                && check_messages(f).contains(&"Pick an element")
        })
}

fn blank_targets_are_refused(kind: &str, case: &Value, actual: &Value) -> bool {
    let picks = actual["problem"] == "Pick an element";
    let whole_empty = case["label"] == "empty" && case["key"].is_null();
    let blank_target = case["key"]
        .as_str()
        .is_some_and(|k| is_blank_target_check(kind, k))
        && ["missing", "empty", "spaces", "null"].contains(&case["label"].as_str().unwrap_or(""));
    picks && (whole_empty || blank_target)
}

fn optional_fields_take_null(kind: &str, case: &Value) -> bool {
    let nullable = case["key"]
        .as_str()
        .and_then(|k| node_def(kind)?.params.field(k).cloned())
        .is_some_and(|f| {
            f["type"]
                .as_array()
                .is_some_and(|t| t.contains(&json!("null")))
        });
    case["label"] == "null" && nullable
}

fn defaults_are_checked_too(kind: &str, case: &Value, actual: &Value) -> bool {
    let Some(problem) = actual["problem"].as_str() else {
        return false;
    };
    let refused_default = node_def(kind).is_some_and(|d| {
        d.params.keys().iter().any(|k| {
            d.params
                .field(k)
                .is_some_and(|f| check_messages(f).contains(&problem))
        })
    });
    let defaulted =
        case["label"] == "missing" || (case["label"] == "empty" && case["key"].is_null());
    case["outcome"]["ok"] == true && defaulted && refused_default
}

fn colours_are_worded_in_english(case: &Value, actual: &Value) -> bool {
    case["outcome"]["problem"] == "Invalid color" && actual["problem"] == "Pick a colour"
}

fn incomplete_settings_are_not_valid(case: &Value, actual: &Value) -> bool {
    let label_of = |text: &str| {
        text.split_once(" has a value that isn't allowed")
            .map(|(label, _)| label.to_owned())
    };
    let expected = case["outcome"]["problem"]
        .as_str()
        .and_then(label_of)
        .filter(|_| case["change"]["set"].is_object());
    expected.is_some_and(|label| actual["problem"] == format!("{label} isn't valid"))
}

fn accepted(kind: &str, case: &Value, actual: &Value) -> bool {
    colours_are_worded_in_english(case, actual)
        || incomplete_settings_are_not_valid(case, actual)
        || blank_targets_are_refused(kind, case, actual)
        || optional_fields_take_null(kind, case)
        || defaults_are_checked_too(kind, case, actual)
}

#[test]
fn param_cases_match() {
    let fixture = fixture("param-cases.json");
    let bases: IndexMap<String, Values> = serde_json::from_value(fixture["bases"].clone()).unwrap();
    let mut problems = Vec::new();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let kind = case["type"].as_str().unwrap();
        if node_def(kind).is_none() {
            continue;
        }
        checked += 1;
        let values = applied(&bases[kind], case);
        let actual = rust_outcome(kind, &values, case["key"].as_str());
        if let Some(d) =
            first_difference(&case["outcome"], &actual).filter(|_| !accepted(kind, case, &actual))
        {
            problems.push(format!("{kind} {} {}: {d}", case["label"], case["key"]));
        }
    }
    println!("{checked} param cases checked");
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

fn empty_workflow() -> Workflow {
    serde_json::from_value(
        json!({ "version": 1, "id": "case", "name": "Case", "nodes": [], "edges": [] }),
    )
    .unwrap()
}

#[test]
fn node_cases_match() {
    let defaults = ts_defaults();
    let wf = empty_workflow();
    let settings = WorkflowSettings::default();
    let vars = Vars::new();
    let mut problems = Vec::new();
    let mut checked = 0;
    for case in fixture("node-cases.json").as_array().unwrap() {
        let kind = case["type"].as_str().unwrap();
        let Some(def) = node_def(kind) else { continue };
        checked += 1;
        let at = format!("{kind} filled={}", case["filled"]);
        let inputs: PortValues = serde_json::from_value(case["inputs"].clone()).unwrap();
        let result: Option<NodeResult> = serde_json::from_value(case["result"].clone()).unwrap();
        let params = &defaults[kind];
        match def.parse(params, None) {
            Err(problem) => {
                if case["error"].is_null() {
                    problems.push(format!("{at}: defaults fail: {problem}"));
                }
            }
            Ok(values) => {
                let ctx = Ctx {
                    node_id: "n",
                    kind,
                    label: None,
                    params: (),
                    values: &values,
                    inputs: &inputs,
                    settings: &settings,
                    result: result.as_ref(),
                    vars: &vars,
                    workflow: &wf,
                };
                match def.run_compile(&ctx) {
                    Err(e) if case["error"] == e.as_str() => {}
                    Err(e) => problems.push(format!(
                        "{at}: compile error {e}, expected {}",
                        case["error"]
                    )),
                    Ok(out) => {
                        let out = out.unwrap_or_else(|| {
                            result
                                .as_ref()
                                .map(|r| r.outputs.clone())
                                .unwrap_or_default()
                        });
                        let actual = serde_json::to_value(&out).unwrap();
                        if let Some(d) = first_difference(&case["outputs"], &actual) {
                            problems.push(format!("{at} outputs{d}"));
                        }
                    }
                }
            }
        }
        let card_inputs = CardInputs {
            secrets: serde_json::from_value(case["secrets"].clone()).unwrap(),
            connected: serde_json::from_value(case["connected"].clone()).unwrap(),
            frames: serde_json::from_value(case["inputFrames"].clone()).unwrap(),
            ..CardInputs::default()
        };
        let rows = def.card_for(params, &vars, &settings, result.as_ref(), &card_inputs);
        let actual = json!({ "rows": rows });
        if let Some(d) = first_difference(&case["card"], &actual) {
            problems.push(format!("{at} card{d}"));
        }
    }
    let _ = CompileValue::as_clip;
    println!("{checked} node cases checked");
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
