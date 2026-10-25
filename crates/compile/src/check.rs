use std::collections::HashSet;

use opsis_types::workflow::Workflow;
use serde::Serialize;
use serde_json::Value;

use crate::compile::compile_workflow;
use crate::graph::{GraphIssue, IssueLevel, validate_workflow};

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
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
    let compiled = compile_workflow(wf, None);
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
