use std::collections::HashSet;

use indexmap::IndexMap;
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;
use serde::Serialize;
use serde_json::Value;

use crate::compile::compile_workflow;
use crate::graph::{GraphIssue, IssueLevel, validate_workflow};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Problem")]
pub struct ApiProblem {
    #[serde(flatten)]
    pub issue: GraphIssue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
pub struct CheckResult {
    pub ok: bool,
    pub problems: Vec<ApiProblem>,
    pub workflow: Option<Workflow>,
}

pub fn workflow_problems(wf: &Workflow) -> Vec<GraphIssue> {
    workflow_problems_with(wf, None)
}

pub fn workflow_problems_with(
    wf: &Workflow,
    results: Option<&IndexMap<String, NodeResult>>,
) -> Vec<GraphIssue> {
    let compiled = compile_workflow(wf, results);
    let mut problems = validate_workflow(wf, Some(&compiled));
    let flagged: HashSet<Option<String>> = problems
        .iter()
        .filter(|p| p.level == IssueLevel::Error)
        .map(|p| p.node_id.clone())
        .collect();
    for (node_id, message) in &compiled.errors {
        if !flagged.contains(&Some(node_id.clone())) {
            problems.push(GraphIssue {
                level: IssueLevel::Error,
                message: message.clone(),
                node_id: Some(node_id.clone()),
                edge_id: None,
            });
        }
    }
    problems
}

fn checked(problems: Vec<ApiProblem>, workflow: Option<Workflow>) -> CheckResult {
    CheckResult {
        ok: !problems.iter().any(|p| p.issue.level == IssueLevel::Error),
        problems,
        workflow,
    }
}

pub fn check_workflow(input: &Value) -> CheckResult {
    match serde_path_to_error::deserialize::<_, Workflow>(input) {
        Err(error) => {
            let path = error.path().to_string();
            let issue = GraphIssue {
                level: IssueLevel::Error,
                message: error.inner().to_string(),
                node_id: None,
                edge_id: None,
            };
            checked(
                vec![ApiProblem {
                    issue,
                    path: Some(path),
                }],
                None,
            )
        }
        Ok(workflow) => {
            let workflow = workflow.normalized();
            let problems = workflow_problems(&workflow)
                .into_iter()
                .map(|issue| ApiProblem { issue, path: None })
                .collect();
            checked(problems, Some(workflow))
        }
    }
}

fn is_file_field(field: &Value) -> bool {
    let inner = crate::elements::introspect::branch(field);
    crate::elements::introspect::widget_of(inner) == Some("asset")
        || inner.get("x-accept").is_some()
        || field.get("x-accept").is_some()
}

pub fn absolute_file_settings(kind: &str, params: &serde_json::Map<String, Value>) -> Vec<String> {
    let Some(def) = crate::registry::node_def(kind) else {
        return Vec::new();
    };
    let found = std::cell::RefCell::new(Vec::new());
    for (key, value) in params {
        let Some(field) = def.params.field(key) else {
            continue;
        };
        crate::elements::introspect::map_leaves(field, value, &|schema, leaf| {
            let absolute = leaf.as_str().is_some_and(|text| {
                matches!(
                    crate::asset_ref::ref_target(text.trim()),
                    crate::asset_ref::RefTarget::Absolute(_)
                )
            });
            if absolute && is_file_field(schema) && !found.borrow().contains(key) {
                found.borrow_mut().push(key.clone());
            }
            Some(leaf.clone())
        });
    }
    found.into_inner()
}
