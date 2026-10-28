use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, named_project_files, workflow_cards};
use opsis_compile::check::workflow_problems;
use opsis_compile::compile::{CompileResult, compile_workflow, films, preview_spec};
use opsis_compile::graph::{connection_problem, run_targets, validate_workflow};
use opsis_compile::mix::plan::build_mix_plan;
use opsis_compile::registry::node_def;
use opsis_compile::timeline::chapters::film_chapters;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_compile::vars::workflow_vars;
use opsis_types::values::NodeResult;
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value, json};

use crate::known::known_types;

const WORKFLOWS: &str = "compile/workflows";
const PROBLEM_CASES: &str = "compile/problem-cases.json";

fn recorded_workflows() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(opsis_golden::path(WORKFLOWS))
        .expect("the workflow folder")
        .map(|entry| {
            let name = entry.expect("an entry").file_name();
            format!("{WORKFLOWS}/{}", name.to_string_lossy())
        })
        .collect();
    names.sort();
    names
}

fn shown_settings(wf: &Workflow, node: &WorkflowNode) -> Value {
    let vars = workflow_vars(&wf.name, &wf.variables);
    node_def(&node.kind)
        .map(|d| Value::Object(d.params.card_values(&node.params, &vars)))
        .unwrap_or(Value::Null)
}

fn cards(
    wf: &Workflow,
    compiled: &CompileResult,
    results: Option<&IndexMap<String, NodeResult>>,
) -> Value {
    let gone = |file: &str| {
        wf.nodes.iter().any(|n| {
            named_project_files(&shown_settings(wf, n))
                .iter()
                .enumerate()
                .any(|(i, f)| i % 2 == 1 && f == file)
        })
    };
    let secrets = ["Main account".to_owned()];
    let sources = CardSources {
        compiled,
        results,
        secrets: Some(&secrets),
        gone: &gone,
    };
    let cards: Map<String, Value> = workflow_cards(wf, &sources)
        .into_iter()
        .map(|(id, rows)| (id, json!({ "rows": rows })))
        .collect();
    Value::Object(cards)
}

fn timelines(wf: &Workflow, compiled: &CompileResult) -> Value {
    let films: Map<String, Value> = films(wf, compiled)
        .iter()
        .map(|(id, spec)| {
            let flat = flatten_spec(spec);
            let mut mix = serde_json::to_value(build_mix_plan(spec, wf.mix.as_ref(), &flat))
                .expect("a mix plan");
            mix.as_object_mut().expect("a plan").remove("key");
            let film = json!({ "flat": flat, "mix": mix, "chapters": film_chapters(spec) });
            (id.clone(), film)
        })
        .collect();
    Value::Object(films)
}

fn outcome(wf: &Workflow, results: Option<&IndexMap<String, NodeResult>>) -> Map<String, Value> {
    let compiled = compile_workflow(wf, results);
    let mut out = Map::new();
    out.insert(
        "compile".into(),
        serde_json::to_value(&compiled).expect("compiled"),
    );
    out.insert(
        "problems".into(),
        serde_json::to_value(validate_workflow(wf, Some(&compiled))).expect("problems"),
    );
    out.insert("cards".into(), cards(wf, &compiled, results));
    out.insert(
        "preview".into(),
        serde_json::to_value(preview_spec(wf, &compiled, None)).expect("preview"),
    );
    out.insert(
        "films".into(),
        serde_json::to_value(films(wf, &compiled)).expect("films"),
    );
    out.insert("timelines".into(), timelines(wf, &compiled));
    out
}

#[test]
fn workflows_compile_check_and_flatten_as_recorded() {
    for name in recorded_workflows() {
        let stored = opsis_golden::read(&name);
        let wf: Workflow = serde_json::from_value(stored["workflow"].clone()).expect("a workflow");
        known_types(&name, wf.nodes.iter().map(|n| n.kind.as_str()));
        let results: IndexMap<String, NodeResult> =
            serde_json::from_value(stored["run"]["results"].clone()).expect("run results");
        let mut run = Map::new();
        run.insert("results".into(), stored["run"]["results"].clone());
        run.extend(outcome(&wf, Some(&results)));
        let mut made = Map::new();
        made.insert("workflow".into(), stored["workflow"].clone());
        made.extend(outcome(&wf, None));
        made.insert("run".into(), Value::Object(run));
        opsis_golden::check(&name, &Value::Object(made));
    }
}

#[test]
fn problems_and_connections_are_reported_as_recorded() {
    let stored = opsis_golden::read(PROBLEM_CASES);
    let workflows: Map<String, Value> = stored["workflows"]
        .as_object()
        .expect("problem workflows")
        .iter()
        .map(|(name, case)| {
            let wf: Workflow =
                serde_json::from_value(case["workflow"].clone()).expect("a workflow");
            let recorded = json!({
                "workflow": case["workflow"],
                "problems": validate_workflow(&wf, None),
                "checked": workflow_problems(&wf),
                "runTargets": run_targets(&wf),
            });
            (name.clone(), recorded)
        })
        .collect();
    let wf: Workflow =
        serde_json::from_value(stored["workflows"]["exportClash"]["workflow"].clone())
            .expect("the export clash workflow");
    let connections: Vec<Value> = stored["connections"]
        .as_array()
        .expect("connection cases")
        .iter()
        .map(|case| {
            let end = |k: &str| case["ends"][k].as_str().expect("an end");
            let problem = connection_problem(
                &wf,
                end("source"),
                end("sourceHandle"),
                end("target"),
                end("targetHandle"),
            );
            json!({ "ends": case["ends"], "problem": problem })
        })
        .collect();
    opsis_golden::check(
        PROBLEM_CASES,
        &json!({ "workflows": workflows, "connections": connections }),
    );
}
