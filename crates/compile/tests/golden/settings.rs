use opsis_compile::node::{CardInputs, Ctx, NodeDef};
use opsis_compile::params::Values;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_types::values::{NodeResult, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};

const PARAM_CASES: &str = "compile/param-cases.json";
const NODE_CASES: &str = "compile/node-cases.json";

fn changed(defaults: &Values, case: &Value) -> Values {
    let (Some(key), Some(change)) = (case["key"].as_str(), case["change"].as_object()) else {
        return if case["label"] == "empty" {
            Values::new()
        } else {
            defaults.clone()
        };
    };
    let mut values = defaults.clone();
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

fn outcome(def: &NodeDef, values: &Values, key: Option<&str>) -> Value {
    match (def.parse(values, Some(values)), key) {
        (Err(problem), _) => json!({ "ok": false, "problem": problem }),
        (Ok(data), None) => json!({ "ok": true, "data": data }),
        (Ok(data), Some(key)) => {
            json!({ "ok": true, "value": data.get(key).cloned().unwrap_or(Value::Null), "keys": data.len() })
        }
    }
}

#[test]
fn settings_parse_as_recorded() {
    let stored = opsis_golden::read(PARAM_CASES);
    let cases: Vec<Value> = stored["cases"]
        .as_array()
        .expect("a case list")
        .iter()
        .filter_map(|case| {
            let def = node_def(case["type"].as_str()?)?;
            let values = changed(&def.defaults(), case);
            let mut case = case.clone();
            case["outcome"] = outcome(def, &values, case["key"].as_str());
            Some(case)
        })
        .collect();
    opsis_golden::check(PARAM_CASES, &json!({ "cases": cases }));
}

fn empty_workflow() -> Workflow {
    serde_json::from_value(
        json!({ "version": 1, "id": "case", "name": "Case", "nodes": [], "edges": [] }),
    )
    .expect("an empty workflow")
}

fn compiled(def: &NodeDef, case: &Value) -> (Value, Value) {
    let workflow = empty_workflow();
    let settings = WorkflowSettings::default();
    let vars = Vars::new();
    let inputs: PortValues = serde_json::from_value(case["inputs"].clone()).expect("inputs");
    let result: Option<NodeResult> =
        serde_json::from_value(case["result"].clone()).expect("result");
    let values = match def.parse(&def.defaults(), None) {
        Ok(values) => values,
        Err(problem) => return (Value::Null, Value::from(problem)),
    };
    let ctx = Ctx {
        node_id: "n",
        kind: &def.kind,
        label: None,
        params: (),
        values: &values,
        inputs: &inputs,
        settings: &settings,
        result: result.as_ref(),
        vars: &vars,
        workflow: &workflow,
    };
    match def.run_compile(&ctx) {
        Err(error) => (Value::Null, Value::from(error)),
        Ok(outputs) => {
            let outputs = outputs.unwrap_or_else(|| {
                result
                    .as_ref()
                    .map(|r| r.outputs.clone())
                    .unwrap_or_default()
            });
            (
                serde_json::to_value(outputs).expect("outputs serialize"),
                Value::Null,
            )
        }
    }
}

fn card(def: &NodeDef, case: &Value) -> Value {
    let result: Option<NodeResult> =
        serde_json::from_value(case["result"].clone()).expect("result");
    let inputs = CardInputs {
        secrets: serde_json::from_value(case["secrets"].clone()).expect("secrets"),
        connected: serde_json::from_value(case["connected"].clone()).expect("connected"),
        frames: serde_json::from_value(case["inputFrames"].clone()).expect("input frames"),
        ..CardInputs::default()
    };
    let rows = def.card_for(
        &def.defaults(),
        &Vars::new(),
        &WorkflowSettings::default(),
        result.as_ref(),
        &inputs,
    );
    json!({ "rows": rows })
}

#[test]
fn nodes_compile_and_draw_their_cards_as_recorded() {
    let stored = opsis_golden::read(NODE_CASES);
    let cases: Vec<Value> = stored
        .as_array()
        .expect("a case list")
        .iter()
        .filter_map(|case| {
            let def = node_def(case["type"].as_str()?)?;
            let (outputs, error) = compiled(def, case);
            let rows = card(def, case);
            let mut case = case.clone();
            case["outputs"] = outputs;
            case["error"] = error;
            case["card"] = rows;
            Some(case)
        })
        .collect();
    opsis_golden::check(NODE_CASES, &Value::Array(cases));
}
