use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use opsis_types::actions::Viewport;
use opsis_types::edits::{EdgeEnds, NodePatch, SettingsPatch};
use opsis_types::fonts::{FontFamilyDef, FontRole};
use opsis_types::numbers::whole;
use opsis_types::workflow::{Position, Workflow, WorkflowEdge, WorkflowNode};
use serde_json::{Map, Value};

use crate::edits::EditError;
use crate::fonts::same_family;
use crate::graph::{GROUP_TYPE, is_group_node};
use crate::ids::{Ids, node_id_prefix};
use crate::node::{NodeDef, Platform};
use crate::registry::node_def;
use crate::vars::rename_variable;

pub const GROUP_PAD: f64 = 40.0;
pub const GROUP_TITLE_HEIGHT: f64 = 24.0;
const GROUP_TITLE: &str = "title";

pub fn node_mut<'a>(wf: &'a mut Workflow, id: &str) -> Result<&'a mut WorkflowNode, EditError> {
    wf.nodes
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or_else(|| EditError::NoNode(id.to_owned()))
}

pub fn new_node_params(
    kind: &str,
    params: Option<Map<String, Value>>,
    platform: Platform,
) -> Map<String, Value> {
    let mut out = Map::new();
    if let Some(def) = node_def(kind) {
        out.extend(def.defaults());
        if let Some(host) = def.host_defaults {
            out.extend(host(platform));
        }
    }
    out.extend(params.unwrap_or_default());
    out
}

pub fn add_node(wf: &mut Workflow, node: WorkflowNode) -> Result<String, EditError> {
    if wf.node(&node.id).is_some() {
        return Err(EditError::NodeExists(node.id));
    }
    let id = node.id.clone();
    wf.nodes.push(node);
    Ok(id)
}

pub fn next_position(nodes: &[WorkflowNode]) -> Position {
    let right = nodes.iter().map(|n| n.position.x).reduce(f64::max);
    right.map_or_else(Position::default, |x| Position {
        x: x + 300.0,
        y: 0.0,
    })
}

pub fn patch_node(node: &mut WorkflowNode, patch: NodePatch) {
    if let Some(label) = patch.label {
        node.label = Some(label).filter(|l| !l.is_empty());
    }
    match patch.disabled {
        Some(true) => node.disabled = Some(true),
        Some(false) => node.disabled = None,
        None => {}
    }
    if let Some(position) = patch.position {
        node.position = position;
    }
    if let Some(size) = patch.size {
        node.size = Some(size);
    }
    if let Some(params) = patch.params {
        node.params.extend(params);
    }
}

pub fn move_nodes(wf: &mut Workflow, positions: &IndexMap<String, Position>) {
    for node in &mut wf.nodes {
        if let Some(position) = positions.get(&node.id) {
            node.position = *position;
        }
    }
}

pub fn absolute_position(wf: &Workflow, node: &WorkflowNode) -> Position {
    let Some(parent) = node.parent_id.as_deref().and_then(|id| wf.node(id)) else {
        return node.position;
    };
    let at = absolute_position(wf, parent);
    Position {
        x: at.x + node.position.x,
        y: at.y + node.position.y,
    }
}

fn detach(node: &mut WorkflowNode, parent: Option<Position>) {
    node.parent_id = None;
    let parent = parent.unwrap_or_default();
    node.position = Position {
        x: node.position.x + parent.x,
        y: node.position.y + parent.y,
    };
}

pub fn remove_nodes(wf: &mut Workflow, ids: &[String]) {
    let gone: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let positions: HashMap<String, Position> = wf
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.position))
        .collect();
    wf.nodes.retain(|n| !gone.contains(n.id.as_str()));
    for node in &mut wf.nodes {
        if let Some(parent) = node.parent_id.clone()
            && gone.contains(parent.as_str())
        {
            detach(node, positions.get(&parent).copied());
        }
    }
    wf.edges
        .retain(|e| !gone.contains(e.source.as_str()) && !gone.contains(e.target.as_str()));
    let kept: HashSet<&str> = wf.nodes.iter().map(|n| n.id.as_str()).collect();
    if let Some(mix) = wf.mix.as_mut() {
        mix.channels.retain(|id, _| kept.contains(id.as_str()));
    }
}

pub fn remove_edges(wf: &mut Workflow, ids: &[String]) {
    let gone: HashSet<&str> = ids.iter().map(String::as_str).collect();
    wf.edges.retain(|e| !gone.contains(e.id.as_str()));
}

fn same_ends(edge: &WorkflowEdge, ends: &EdgeEnds) -> bool {
    edge.source == ends.source
        && edge.source_handle == ends.source_handle
        && edge.target == ends.target
        && edge.target_handle == ends.target_handle
}

fn takes_many(wf: &Workflow, ends: &EdgeEnds) -> bool {
    wf.node(&ends.target)
        .and_then(|n| node_def(&n.kind))
        .and_then(|def| def.input(&ends.target_handle))
        .is_some_and(|port| port.many)
}

pub fn connect(wf: &mut Workflow, ends: EdgeEnds) -> String {
    let many = takes_many(wf, &ends);
    wf.edges.retain(|e| {
        if many {
            !same_ends(e, &ends)
        } else {
            !(e.target == ends.target && e.target_handle == ends.target_handle)
        }
    });
    let edge = ends.edge();
    let id = edge.id.clone();
    wf.edges.push(edge);
    id
}

pub fn set_settings(wf: &mut Workflow, patch: SettingsPatch) {
    let settings = &mut wf.settings;
    if let Some(width) = patch.width {
        settings.width = width;
    }
    if let Some(height) = patch.height {
        settings.height = height;
    }
    if let Some(fps) = patch.fps {
        settings.fps = fps;
    }
    if let Some(background) = patch.background {
        settings.background = Some(background).filter(|b| !b.is_empty());
    }
    if let Some(theme) = patch.theme {
        settings.theme = theme;
    }
    if let Some(fonts) = patch.fonts {
        settings.fonts = fonts;
    }
}

pub fn add_project_font(wf: &mut Workflow, font: FontFamilyDef) {
    let families = &mut wf.settings.fonts.families;
    families.retain(|f| !same_family(&f.family, &font.family));
    families.push(font);
}

pub fn set_font_role(wf: &mut Workflow, role: FontRole, family: String) {
    let fonts = &mut wf.settings.fonts;
    match role {
        FontRole::Main => fonts.main = family,
        FontRole::Heading => fonts.heading = family,
        FontRole::Code => fonts.code = family,
    }
}

pub fn rename_workflow_variable(wf: &mut Workflow, from: &str, to: &str) {
    let name = to.trim();
    if name.is_empty()
        || name == from
        || wf.variables.contains_key(name)
        || !wf.variables.contains_key(from)
    {
        return;
    }
    wf.variables = wf
        .variables
        .drain(..)
        .map(|(k, v)| (if k == from { name.to_owned() } else { k }, v))
        .collect();
    for node in &mut wf.nodes {
        let renamed = rename_variable(&Value::Object(std::mem::take(&mut node.params)), from, name);
        if let Value::Object(params) = renamed {
            node.params = params;
        }
    }
}

pub fn paste_nodes(
    wf: &mut Workflow,
    nodes: Vec<WorkflowNode>,
    edges: Vec<WorkflowEdge>,
    offset: Position,
    ids: &mut dyn Ids,
) -> Vec<String> {
    let fresh: HashMap<String, String> = nodes
        .iter()
        .map(|n| (n.id.clone(), ids.id(node_id_prefix(&n.kind))))
        .collect();
    let mut pasted = Vec::with_capacity(nodes.len());
    for node in &nodes {
        let mut copy = node.clone();
        copy.id.clone_from(&fresh[&node.id]);
        if let Some(parent) = node.parent_id.as_ref().and_then(|p| fresh.get(p)) {
            copy.parent_id = Some(parent.clone());
        } else {
            copy.parent_id = None;
            let at = absolute_position(wf, node);
            copy.position = Position {
                x: at.x + offset.x,
                y: at.y + offset.y,
            };
        }
        pasted.push(copy);
    }
    let pasted_edges: Vec<WorkflowEdge> = edges
        .into_iter()
        .filter_map(|e| {
            let source = fresh.get(&e.source)?.clone();
            let target = fresh.get(&e.target)?.clone();
            Some(
                EdgeEnds {
                    source,
                    source_handle: e.source_handle,
                    target,
                    target_handle: e.target_handle,
                }
                .edge(),
            )
        })
        .collect();
    let added = pasted.iter().map(|n| n.id.clone()).collect();
    wf.nodes.extend(pasted);
    wf.edges.extend(pasted_edges);
    added
}

pub fn group_nodes(
    wf: &mut Workflow,
    ids: &[String],
    title: &str,
    sizes: &IndexMap<String, Viewport>,
    group_id: String,
) -> Option<String> {
    let members: Vec<&WorkflowNode> = wf
        .nodes
        .iter()
        .filter(|n| ids.contains(&n.id) && n.parent_id.is_none() && !is_group_node(n))
        .collect();
    if members.len() < 2 {
        return None;
    }
    let size = |n: &WorkflowNode| sizes.get(&n.id).copied().unwrap_or_default();
    let x = members
        .iter()
        .map(|n| n.position.x)
        .fold(f64::INFINITY, f64::min)
        - GROUP_PAD;
    let y = members
        .iter()
        .map(|n| n.position.y)
        .fold(f64::INFINITY, f64::min)
        - GROUP_PAD
        - GROUP_TITLE_HEIGHT;
    let right = members
        .iter()
        .map(|n| n.position.x + f64::from(size(n).width))
        .fold(f64::NEG_INFINITY, f64::max);
    let bottom = members
        .iter()
        .map(|n| n.position.y + f64::from(size(n).height))
        .fold(f64::NEG_INFINITY, f64::max);
    let member_ids: HashSet<String> = members.iter().map(|n| n.id.clone()).collect();
    let mut params = node_def(GROUP_TYPE).map_or_default(NodeDef::defaults);
    params.insert(GROUP_TITLE.to_owned(), Value::String(title.to_owned()));
    let group = WorkflowNode {
        id: group_id.clone(),
        kind: GROUP_TYPE.to_owned(),
        position: Position { x, y },
        params,
        label: None,
        disabled: None,
        parent_id: None,
        size: Some(Viewport {
            width: whole(right - x + GROUP_PAD),
            height: whole(bottom - y + GROUP_PAD),
        }),
    };
    for node in &mut wf.nodes {
        if member_ids.contains(&node.id) {
            node.parent_id = Some(group_id.clone());
            node.position = Position {
                x: node.position.x - x,
                y: node.position.y - y,
            };
        }
    }
    wf.nodes.insert(0, group);
    Some(group_id)
}

pub fn ungroup(wf: &mut Workflow, group_id: &str) -> Result<(), EditError> {
    let group = wf
        .node(group_id)
        .map(|g| g.position)
        .ok_or_else(|| EditError::NoNode(group_id.to_owned()))?;
    wf.nodes.retain(|n| n.id != group_id);
    for node in &mut wf.nodes {
        if node.parent_id.as_deref() == Some(group_id) {
            detach(node, Some(group));
        }
    }
    Ok(())
}
