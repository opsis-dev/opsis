use std::collections::{HashMap, HashSet};

use indexmap::IndexSet;
use opsis_types::values::{BrowserAction, BrowserSessionValue};
use opsis_types::workflow::{PortType, Workflow, WorkflowEdge, WorkflowNode};
use serde::{Deserialize, Serialize};

use crate::compile::{CompileResult, compile_workflow};
use crate::node::{Claim, NodeDef, PortSpec, SESSION_PORT};
use crate::ports::{can_connect, convert_input};
use crate::registry::{is_output_type, node_def, node_name};
use crate::vars::{interpolate_map, unknown_variables, workflow_vars};

pub const GROUP_TYPE: &str = "core.group";
pub const NOTE_TYPE: &str = "core.note";

pub fn is_group_node(node: &WorkflowNode) -> bool {
    node.kind == GROUP_TYPE
}

pub fn is_note_node(node: &WorkflowNode) -> bool {
    node.kind == NOTE_TYPE
}

pub fn is_structural(node: &WorkflowNode) -> bool {
    is_group_node(node) || is_note_node(node)
}

fn by_position(a: &WorkflowNode, b: &WorkflowNode) -> std::cmp::Ordering {
    a.position
        .x
        .total_cmp(&b.position.x)
        .then(a.position.y.total_cmp(&b.position.y))
        .then_with(|| a.id.cmp(&b.id))
}

#[derive(Debug)]
pub struct Sorted<'a> {
    pub sorted: Vec<&'a WorkflowNode>,
    pub cyclic: Vec<String>,
}

pub fn topo_sort(wf: &Workflow) -> Sorted<'_> {
    let nodes: Vec<&WorkflowNode> = wf.nodes.iter().filter(|n| !is_structural(n)).collect();
    let index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut indegree = vec![0usize; nodes.len()];
    let mut next: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for e in &wf.edges {
        let (Some(&s), Some(&t)) = (index.get(e.source.as_str()), index.get(e.target.as_str()))
        else {
            continue;
        };
        next[s].push(t);
        indegree[t] += 1;
    }
    let order = |list: &mut Vec<usize>| list.sort_by(|a, b| by_position(nodes[*a], nodes[*b]));
    let mut ready: Vec<usize> = (0..nodes.len()).filter(|i| indegree[*i] == 0).collect();
    order(&mut ready);
    let mut sorted = Vec::new();
    while !ready.is_empty() {
        let vertex = ready.remove(0);
        sorted.push(vertex);
        for &after in &next[vertex] {
            indegree[after] -= 1;
            if indegree[after] == 0 {
                ready.push(after);
            }
        }
        order(&mut ready);
    }
    let done: HashSet<usize> = sorted.iter().copied().collect();
    Sorted {
        sorted: sorted.into_iter().map(|i| nodes[i]).collect(),
        cyclic: (0..nodes.len())
            .filter(|i| !done.contains(i))
            .map(|i| nodes[i].id.clone())
            .collect(),
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
}

fn reach(wf: &Workflow, node_id: &str, direction: Direction) -> IndexSet<String> {
    let mut links: HashMap<&str, Vec<&str>> = HashMap::new();
    for e in &wf.edges {
        let (from, to) = match direction {
            Direction::Up => (e.target.as_str(), e.source.as_str()),
            Direction::Down => (e.source.as_str(), e.target.as_str()),
        };
        links.entry(from).or_default().push(to);
    }
    let mut out = IndexSet::new();
    let mut stack = vec![node_id];
    while let Some(id) = stack.pop() {
        for linked in links.get(id).into_iter().flatten() {
            if out.insert((*linked).to_owned()) {
                stack.push(linked);
            }
        }
    }
    out
}

pub fn upstream_of(wf: &Workflow, node_id: &str) -> IndexSet<String> {
    reach(wf, node_id, Direction::Up)
}

pub fn downstream_of(wf: &Workflow, node_id: &str) -> IndexSet<String> {
    reach(wf, node_id, Direction::Down)
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum IssueLevel {
    Error,
    Warning,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct GraphIssue {
    pub level: IssueLevel,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_id: Option<String>,
}

fn error(node_id: &str, message: impl Into<String>) -> GraphIssue {
    GraphIssue {
        level: IssueLevel::Error,
        message: message.into(),
        node_id: Some(node_id.to_owned()),
        edge_id: None,
    }
}

fn warning(node_id: &str, message: impl Into<String>) -> GraphIssue {
    GraphIssue {
        level: IssueLevel::Warning,
        message: message.into(),
        node_id: Some(node_id.to_owned()),
        edge_id: None,
    }
}

fn nodes_by_id(wf: &Workflow) -> HashMap<&str, &WorkflowNode> {
    let mut map = HashMap::new();
    for n in &wf.nodes {
        map.entry(n.id.as_str()).or_insert(n);
    }
    map
}

enum EdgeFault<'a> {
    Node(&'a str),
    Output(&'a WorkflowNode, &'a str),
    Input(&'a WorkflowNode, &'a str),
    Types(PortType, PortType),
}

fn edge_fault<'a>(
    source: &'a str,
    source_handle: &'a str,
    target: &'a str,
    target_handle: &'a str,
    by_id: &HashMap<&str, &'a WorkflowNode>,
) -> Option<EdgeFault<'a>> {
    let Some(s) = by_id.get(source) else {
        return Some(EdgeFault::Node(source));
    };
    let Some(t) = by_id.get(target) else {
        return Some(EdgeFault::Node(target));
    };
    let Some(from) = node_def(&s.kind).and_then(|d| d.output(source_handle)) else {
        return Some(EdgeFault::Output(s, source_handle));
    };
    let Some(to) = node_def(&t.kind).and_then(|d| d.input(target_handle)) else {
        return Some(EdgeFault::Input(t, target_handle));
    };
    (!can_connect(from.kind, to.kind)).then_some(EdgeFault::Types(from.kind, to.kind))
}

fn port_mismatch(from: PortType, to: PortType) -> String {
    format!("A {} can't connect to a {} input", from.word(), to.word())
}

fn input_list(def: &NodeDef) -> String {
    let mut names: Vec<String> = def.inputs.iter().map(|(id, _)| (*id).to_owned()).collect();
    if let Some(f) = &def.family {
        names.push(format!("{}, {}, …", f.port(0), f.port(1)));
    }
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

fn port_list(ports: &[(&'static str, PortSpec)]) -> String {
    if ports.is_empty() {
        "none".to_owned()
    } else {
        ports
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProblem {
    pub not_found: bool,
    pub message: String,
}

pub fn connection_problem(
    wf: &Workflow,
    source: &str,
    source_handle: &str,
    target: &str,
    target_handle: &str,
) -> Option<ConnectionProblem> {
    let by_id = nodes_by_id(wf);
    let fault = edge_fault(source, source_handle, target, target_handle, &by_id);
    if let Some(EdgeFault::Node(id)) = fault {
        return Some(ConnectionProblem {
            not_found: true,
            message: format!("There's no node “{id}”."),
        });
    }
    if source == target {
        return Some(ConnectionProblem {
            not_found: false,
            message: "A node can't connect to itself.".to_owned(),
        });
    }
    let message = match fault? {
        EdgeFault::Types(from, to) => format!("{}.", port_mismatch(from, to)),
        EdgeFault::Output(node, handle) => {
            let ports = node_def(&node.kind)
                .map(|d| port_list(&d.outputs))
                .unwrap_or_else(|| "none".to_owned());
            format!(
                "“{}” has no output “{handle}”. Its outputs: {ports}.",
                node.id
            )
        }
        EdgeFault::Input(node, handle) => {
            let ports = node_def(&node.kind).map_or_else(|| "none".to_owned(), input_list);
            format!(
                "“{}” has no input “{handle}”. Its inputs: {ports}.",
                node.id
            )
        }
        EdgeFault::Node(_) => unreachable!("handled above"),
    };
    Some(ConnectionProblem {
        not_found: false,
        message,
    })
}

fn edge_ok(e: &WorkflowEdge, by_id: &HashMap<&str, &WorkflowNode>) -> bool {
    edge_fault(
        &e.source,
        &e.source_handle,
        &e.target,
        &e.target_handle,
        by_id,
    )
    .is_none()
}

pub fn run_targets(wf: &Workflow) -> Vec<String> {
    let executes = |n: Option<&WorkflowNode>| {
        n.is_some_and(|n| {
            !n.is_disabled() && !is_structural(n) && node_def(&n.kind).is_some_and(|d| d.executes)
        })
    };
    let by_id = nodes_by_id(wf);
    let outputs: Vec<&WorkflowNode> = wf
        .nodes
        .iter()
        .filter(|n| executes(Some(n)) && is_output_type(&n.kind))
        .collect();
    if outputs.is_empty() {
        return wf
            .nodes
            .iter()
            .filter(|n| executes(Some(n)))
            .map(|n| n.id.clone())
            .collect();
    }
    let mut targets: indexmap::IndexSet<String> = outputs.iter().map(|n| n.id.clone()).collect();
    for out in &outputs {
        for id in downstream_of(wf, &out.id) {
            if executes(by_id.get(id.as_str()).copied()) {
                targets.insert(id);
            }
        }
    }
    targets.into_iter().collect()
}

pub fn session_into<'a>(
    wf: &Workflow,
    values: &'a CompileResult,
    node_id: &str,
) -> Option<&'a BrowserSessionValue> {
    let edge = wf
        .edges
        .iter()
        .find(|e| e.target == node_id && e.target_handle == SESSION_PORT)?;
    values
        .values
        .get(&edge.source)?
        .get(&edge.source_handle)?
        .as_session()
}

fn port_issues(wf: &Workflow, node: &WorkflowNode, def: &NodeDef) -> Vec<GraphIssue> {
    let by_id = nodes_by_id(wf);
    let mut issues = Vec::new();
    let handles = wf
        .edges
        .iter()
        .filter(|e| e.target == node.id)
        .map(|e| e.target_handle.as_str());
    for (port_id, name, spec) in def.input_ports(handles) {
        let count = wf
            .edges
            .iter()
            .filter(|e| e.target == node.id && e.target_handle == port_id && edge_ok(e, &by_id))
            .count();
        if count == 0 && !spec.optional {
            issues.push(error(&node.id, format!("Connect something to “{name}”")));
        }
        if count > 1 && !spec.many {
            issues.push(error(
                &node.id,
                format!("“{name}” can only take one connection"),
            ));
        }
    }
    issues
}

fn definition_issues(wf: &Workflow, node: &WorkflowNode) -> Vec<GraphIssue> {
    let Some(def) = node_def(&node.kind) else {
        return vec![error(&node.id, crate::compile::UNKNOWN_TYPE)];
    };
    if node.is_disabled() {
        return Vec::new();
    }
    let mut issues = Vec::new();
    if let Err(problem) = def.parse(&node.params, None) {
        issues.push(error(&node.id, problem.to_string()));
    }
    let vars = workflow_vars(&wf.name, &wf.variables);
    if let Some(name) = unknown_variables(&serde_json::Value::Object(node.params.clone()), &vars)
        .into_iter()
        .next()
    {
        issues.push(error(
            &node.id,
            format!("“{name}” isn't one of the project's variables. Add it under Variables in the project settings, or change this setting."),
        ));
    }
    if !is_structural(node) {
        issues.extend(port_issues(wf, node, def));
    }
    issues
}

fn node_issues(wf: &Workflow) -> Vec<GraphIssue> {
    let mut seen = HashSet::new();
    let repeated: HashSet<usize> = wf
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !seen.insert(n.id.as_str()))
        .map(|(i, _)| i)
        .collect();
    wf.nodes
        .iter()
        .enumerate()
        .flat_map(|(i, node)| {
            let mut issues = Vec::new();
            if repeated.contains(&i) {
                issues.push(error(&node.id, "Two nodes share this id"));
            }
            issues.extend(definition_issues(wf, node));
            issues
        })
        .collect()
}

fn edge_issues(wf: &Workflow) -> Vec<GraphIssue> {
    let by_id = nodes_by_id(wf);
    wf.edges
        .iter()
        .filter_map(|e| {
            let fault = edge_fault(
                &e.source,
                &e.source_handle,
                &e.target,
                &e.target_handle,
                &by_id,
            )?;
            let edge_id = Some(e.id.clone());
            Some(match fault {
                EdgeFault::Node(_) => GraphIssue {
                    level: IssueLevel::Error,
                    message: "A connection points at a deleted node".into(),
                    node_id: None,
                    edge_id,
                },
                EdgeFault::Types(from, to) => GraphIssue {
                    level: IssueLevel::Error,
                    message: port_mismatch(from, to),
                    node_id: Some(e.target.clone()),
                    edge_id,
                },
                EdgeFault::Output(..) | EdgeFault::Input(..) => GraphIssue {
                    level: IssueLevel::Error,
                    message: "A connection uses a point that no longer exists".into(),
                    node_id: Some(e.target.clone()),
                    edge_id,
                },
            })
        })
        .collect()
}

fn cycle_issues(wf: &Workflow) -> Vec<GraphIssue> {
    topo_sort(wf)
        .cyclic
        .iter()
        .map(|id| error(id, "This node loops back into itself"))
        .collect()
}

fn anchor_issues(wf: &Workflow) -> Vec<GraphIssue> {
    let actions: HashSet<&str> = wf
        .nodes
        .iter()
        .filter(|n| node_def(&n.kind).is_some_and(|d| d.action))
        .map(|n| n.id.as_str())
        .collect();
    wf.nodes
        .iter()
        .filter(|n| !n.is_disabled())
        .filter(|n| {
            n.params
                .get("atAction")
                .and_then(|v| v.as_str())
                .is_some_and(|a| !a.is_empty() && !actions.contains(a))
        })
        .map(|n| {
            error(
                &n.id,
                "The action this sound plays on is gone. Pick another, or play it at a time.",
            )
        })
        .collect()
}

fn clash(claim: &Claim, first: &str) -> String {
    match claim {
        Claim::SignIn(name) => {
            format!("“{first}” also saves the sign-in “{name}”. Give one of them another name.")
        }
        Claim::File(file) => {
            format!("“{first}” also saves “{file}”. Give one of them another file name.")
        }
    }
}

fn node_claims(node: &WorkflowNode, vars: &crate::vars::Vars) -> Vec<Claim> {
    let Some(def) = node_def(&node.kind) else {
        return Vec::new();
    };
    let params = def
        .params
        .with_defaults(&interpolate_map(&node.params, vars));
    let values = def.params.parse(&params, None).unwrap_or(params);
    def.claimed(&values)
}

fn claim_clashes(wf: &Workflow) -> Vec<GraphIssue> {
    let vars = workflow_vars(&wf.name, &wf.variables);
    let mut owner: HashMap<String, &WorkflowNode> = HashMap::new();
    let mut issues = Vec::new();
    for node in wf.nodes.iter().filter(|n| !n.is_disabled()) {
        for claim in node_claims(node, &vars) {
            match owner.get(&claim.key()) {
                None => {
                    owner.insert(claim.key(), node);
                }
                Some(first) if first.id != node.id => {
                    issues.push(error(&node.id, clash(&claim, &node_name(first))));
                }
                Some(_) => {}
            }
        }
    }
    issues
}

fn passes_nothing(wf: &Workflow, skipped: &WorkflowNode, handle: &str) -> bool {
    let Some(def) = node_def(&skipped.kind) else {
        return false;
    };
    let Some(kind) = def.output(handle).map(|p| p.kind) else {
        return false;
    };
    let by_id = nodes_by_id(wf);
    !wf.edges.iter().any(|e| {
        e.target == skipped.id
            && def.input(&e.target_handle).is_some_and(|p| p.kind == kind)
            && edge_ok(e, &by_id)
    })
}

fn skipped_source_issues(wf: &Workflow) -> Vec<GraphIssue> {
    let by_id = nodes_by_id(wf);
    wf.edges
        .iter()
        .filter_map(|e| {
            let source = by_id.get(e.source.as_str()).filter(|s| s.is_disabled())?;
            let target = by_id.get(e.target.as_str()).filter(|t| !t.is_disabled())?;
            passes_nothing(wf, source, &e.source_handle).then(|| {
                warning(
                    &target.id,
                    format!(
                        "“{}” is skipped and has nothing to pass on, so this gets nothing from it.",
                        node_name(source)
                    ),
                )
            })
        })
        .collect()
}

fn reload_issues(wf: &Workflow, compiled: &CompileResult) -> Vec<GraphIssue> {
    wf.nodes
        .iter()
        .filter(|n| n.kind == BrowserAction::SetState.kind() && !n.is_disabled() && n.params.get("reload") == Some(&serde_json::Value::Bool(true)))
        .filter(|n| session_into(wf, compiled, &n.id).is_some_and(|s| s.address.is_none()))
        .map(|n| {
            warning(
                &n.id,
                "Before Go to page the script runs as the site first opens, so there's nothing to reload. Turn off Reload, or move this after Go to page.",
            )
        })
        .collect()
}

fn file_need_issues(wf: &Workflow, compiled: &CompileResult) -> Vec<GraphIssue> {
    let by_id = nodes_by_id(wf);
    let mut seen: HashSet<(&str, &str)> = HashSet::new();
    wf.edges
        .iter()
        .filter_map(|e| {
            let target = by_id.get(e.target.as_str()).filter(|t| !t.is_disabled())?;
            let spec = node_def(&target.kind)?.input(&e.target_handle)?;
            let need = spec.need?;
            let value = compiled.values.get(&e.source)?.get(&e.source_handle)?;
            let media = convert_input(Some(value), Some(spec))?;
            let refused = media.as_media().is_some_and(|m| need.refuses(m));
            (refused && seen.insert((target.id.as_str(), need.message())))
                .then(|| warning(&target.id, need.message()))
        })
        .collect()
}

pub fn validate_workflow(wf: &Workflow, compiled: Option<&CompileResult>) -> Vec<GraphIssue> {
    let owned;
    let compiled = if let Some(c) = compiled {
        c
    } else {
        owned = compile_workflow(wf, None);
        &owned
    };
    [
        node_issues(wf),
        edge_issues(wf),
        cycle_issues(wf),
        reload_issues(wf, compiled),
        anchor_issues(wf),
        claim_clashes(wf),
        skipped_source_issues(wf),
        file_need_issues(wf, compiled),
    ]
    .into_iter()
    .flatten()
    .collect()
}
