use std::path::PathBuf;

use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, named_project_files, workflow_cards};
use opsis_compile::check::workflow_problems;
use opsis_compile::compile::{compile_workflow, preview_spec};
use opsis_compile::graph::{connection_problem, run_targets, validate_workflow};
use opsis_types::json::first_difference;
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;
use serde_json::Value;

fn workflow_fixtures() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/compile/workflows");
    let mut out: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .expect("fixtures exist")
        .map(|entry| {
            let path = entry.expect("entry").path();
            let text = std::fs::read_to_string(&path).expect("readable");
            (
                path.file_stem().unwrap().to_string_lossy().into_owned(),
                serde_json::from_str(&text).expect("json"),
            )
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn node_shown(wf: &Workflow, node: &opsis_types::workflow::WorkflowNode) -> Value {
    let vars = opsis_compile::vars::workflow_vars(&wf.name, &wf.variables);
    opsis_compile::registry::node_def(&node.kind)
        .map(|d| Value::Object(d.params.card_values(&node.params, &vars)))
        .unwrap_or(Value::Null)
}

#[test]
fn compiled_workflows_match() {
    let mut problems = Vec::new();
    for (name, fixture) in workflow_fixtures() {
        let wf: Workflow = serde_json::from_value(fixture["workflow"].clone()).unwrap();
        let results: IndexMap<String, NodeResult> =
            serde_json::from_value(fixture["run"]["results"].clone()).unwrap();
        for (side, expected, results) in [
            ("edit", &fixture, None),
            ("run", &fixture["run"], Some(&results)),
        ] {
            let compiled = compile_workflow(&wf, results);
            let actual = serde_json::to_value(&compiled).unwrap();
            if let Some(d) = first_difference(&expected["compile"], &actual) {
                problems.push(format!("{name} {side} compile{d}"));
            }
            let problems_found =
                serde_json::to_value(validate_workflow(&wf, Some(&compiled))).unwrap();
            if let Some(d) = first_difference(&expected["problems"], &problems_found) {
                problems.push(format!("{name} {side} problems{d}"));
            }
            let gone = |file: &str| {
                wf.nodes.iter().any(|n| {
                    let shown = node_shown(&wf, n);
                    named_project_files(&shown)
                        .iter()
                        .enumerate()
                        .any(|(i, f)| i % 2 == 1 && f == file)
                })
            };
            let secrets = ["Main account".to_owned()];
            let sources = CardSources {
                compiled: &compiled,
                results,
                secrets: Some(&secrets),
                gone: &gone,
            };
            let cards: IndexMap<String, Value> = workflow_cards(&wf, &sources)
                .into_iter()
                .map(|(id, rows)| (id, serde_json::json!({ "rows": rows })))
                .collect();
            if let Some(d) =
                first_difference(&expected["cards"], &serde_json::to_value(cards).unwrap())
            {
                problems.push(format!("{name} {side} cards{d}"));
            }
            let preview =
                serde_json::to_value(preview_spec(&wf.settings, &compiled, None)).unwrap();
            if let Some(d) = first_difference(&expected["preview"], &preview) {
                problems.push(format!("{name} {side} preview{d}"));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/compile")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).expect("fixture exists")).expect("json")
}

#[test]
fn problem_cases_match() {
    let cases = fixture("problem-cases.json");
    let mut problems = Vec::new();
    for (name, case) in cases["workflows"].as_object().unwrap() {
        let wf: Workflow = serde_json::from_value(case["workflow"].clone()).unwrap();
        let found = serde_json::to_value(validate_workflow(&wf, None)).unwrap();
        if let Some(d) = first_difference(&case["problems"], &found) {
            problems.push(format!("{name} problems{d}"));
        }
        let checked = serde_json::to_value(workflow_problems(&wf)).unwrap();
        if let Some(d) = first_difference(&case["checked"], &checked) {
            problems.push(format!("{name} checked{d}"));
        }
        let targets = serde_json::to_value(run_targets(&wf)).unwrap();
        if let Some(d) = first_difference(&case["runTargets"], &targets) {
            problems.push(format!("{name} run targets{d}"));
        }
    }
    let wf: Workflow =
        serde_json::from_value(cases["workflows"]["exportClash"]["workflow"].clone()).unwrap();
    for case in cases["connections"].as_array().unwrap() {
        let e = &case["ends"];
        let s = |k: &str| e[k].as_str().unwrap();
        let found = serde_json::to_value(connection_problem(
            &wf,
            s("source"),
            s("sourceHandle"),
            s("target"),
            s("targetHandle"),
        ))
        .unwrap();
        if let Some(d) = first_difference(&case["problem"], &found) {
            problems.push(format!("connection {e}{d}"));
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
