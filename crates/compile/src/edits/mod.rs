pub mod keyframes;
pub mod mix;
pub mod timeline;
pub mod workflow;

use indexmap::IndexSet;
use opsis_types::edits::{MarkerEdit, Removal, WorkflowEdit};
use opsis_types::workflow::{Marker, Workflow, WorkflowNode};

use crate::graph::downstream_of;
use crate::ids::{Ids, node_id_prefix};
use crate::node::Platform;
use crate::registry::node_def;
use timeline::apply_removal;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
    #[error("There's no node “{0}”.")]
    NoNode(String),
    #[error("A node with the id “{0}” already exists.")]
    NodeExists(String),
    #[error("There's no node type “{0}”.")]
    NoNodeType(String),
    #[error("There's no connection “{0}”.")]
    NoEdge(String),
    #[error("There's no bus “{0}”.")]
    NoBus(String),
    #[error("There's no marker “{0}”.")]
    NoMarker(String),
    #[error("The keyframes of “{0}” can't be read. Check them in the Inspector.")]
    UnreadableKeys(String),
}

pub struct EditContext<'a> {
    pub platform: Platform,
    pub ids: &'a mut dyn Ids,
}

fn check_nodes(wf: &Workflow, ids: &[String]) -> Result<(), EditError> {
    match ids.iter().find(|id| wf.node(id).is_none()) {
        Some(missing) => Err(EditError::NoNode(missing.clone())),
        None => Ok(()),
    }
}

fn check_edges(wf: &Workflow, ids: &[String]) -> Result<(), EditError> {
    match ids.iter().find(|id| !wf.edges.iter().any(|e| &e.id == *id)) {
        Some(missing) => Err(EditError::NoEdge(missing.clone())),
        None => Ok(()),
    }
}

fn marker_mut<'a>(wf: &'a mut Workflow, id: &str) -> Result<&'a mut Marker, EditError> {
    wf.markers
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| EditError::NoMarker(id.to_owned()))
}

fn apply_marker_edit(
    wf: &mut Workflow,
    edit: MarkerEdit,
    ids: &mut dyn Ids,
) -> Result<Option<String>, EditError> {
    match edit {
        MarkerEdit::Add { at_ms } => {
            let id = ids.id("marker");
            let label = format!("Marker {}", wf.markers.len() + 1);
            wf.markers.push(Marker {
                id: id.clone(),
                at_ms,
                label,
            });
            return Ok(Some(id));
        }
        MarkerEdit::Update { id, at_ms, label } => {
            let marker = marker_mut(wf, &id)?;
            if let Some(at_ms) = at_ms {
                marker.at_ms = at_ms;
            }
            if let Some(label) = label {
                marker.label = label;
            }
        }
        MarkerEdit::Remove { id } => {
            marker_mut(wf, &id)?;
            wf.markers.retain(|m| m.id != id);
        }
    }
    Ok(None)
}

pub fn apply_edit(
    wf: &mut Workflow,
    edit: WorkflowEdit,
    ctx: &mut EditContext<'_>,
) -> Result<Vec<String>, EditError> {
    use workflow::*;
    let one = |id: Option<String>| id.into_iter().collect();
    Ok(match edit {
        WorkflowEdit::AddNode {
            kind,
            id,
            label,
            position,
            params,
        } => {
            if node_def(&kind).is_none() {
                return Err(EditError::NoNodeType(kind));
            }
            let node = WorkflowNode {
                id: id.unwrap_or_else(|| ctx.ids.id(node_id_prefix(&kind))),
                position: position.unwrap_or_else(|| next_position(&wf.nodes)),
                params: new_node_params(&kind, params, ctx.platform),
                label: label.filter(|l| !l.is_empty()),
                disabled: None,
                parent_id: None,
                size: None,
                kind,
            };
            vec![add_node(wf, node)?]
        }
        WorkflowEdit::UpdateNode { id, patch } => {
            patch_node(node_mut(wf, &id)?, patch);
            Vec::new()
        }
        WorkflowEdit::MoveNodes { positions } => {
            move_nodes(wf, &positions);
            Vec::new()
        }
        WorkflowEdit::RemoveNodes { ids } => {
            check_nodes(wf, &ids)?;
            remove_nodes(wf, &ids);
            Vec::new()
        }
        WorkflowEdit::AddEdge { ends } => vec![connect(wf, ends)],
        WorkflowEdit::RemoveEdges { ids } => {
            check_edges(wf, &ids)?;
            remove_edges(wf, &ids);
            Vec::new()
        }
        WorkflowEdit::RemoveParts { node_ids, edge_ids } => {
            apply_removal(&Removal { node_ids, edge_ids }, wf);
            Vec::new()
        }
        WorkflowEdit::SetSettings { patch } => {
            set_settings(wf, patch);
            Vec::new()
        }
        WorkflowEdit::AddProjectFont { font } => {
            add_project_font(wf, font);
            Vec::new()
        }
        WorkflowEdit::SetFontRole { role, family } => {
            set_font_role(wf, role, family);
            Vec::new()
        }
        WorkflowEdit::SetVariables { variables } => {
            wf.variables = variables;
            Vec::new()
        }
        WorkflowEdit::RenameVariable { from, to } => {
            rename_workflow_variable(wf, &from, &to);
            Vec::new()
        }
        WorkflowEdit::Rename { name } => {
            wf.name = name;
            Vec::new()
        }
        WorkflowEdit::Paste {
            nodes,
            edges,
            offset,
        } => paste_nodes(wf, nodes, edges, offset, ctx.ids),
        WorkflowEdit::Group { ids, title, sizes } => {
            let group_id = ctx.ids.id("group");
            one(group_nodes(wf, &ids, &title, &sizes, group_id))
        }
        WorkflowEdit::Ungroup { id } => {
            ungroup(wf, &id)?;
            Vec::new()
        }
        WorkflowEdit::Mix { edit } => {
            let sound = wf.mix.get_or_insert_with(Default::default);
            one(mix::apply_mix_edit(sound, edit, ctx.ids)?)
        }
        WorkflowEdit::Markers { edit } => one(apply_marker_edit(wf, edit, ctx.ids)?),
        WorkflowEdit::Keyframes { edit } => {
            keyframes::apply_keyframe_edit(wf, edit)?;
            Vec::new()
        }
        WorkflowEdit::Replace { workflow } => {
            *wf = workflow.normalized();
            Vec::new()
        }
    })
}

fn incoming(wf: &Workflow, id: &str) -> Vec<(String, String, String)> {
    let mut ends: Vec<(String, String, String)> = wf
        .edges
        .iter()
        .filter(|e| e.target == id)
        .map(|e| {
            (
                e.source.clone(),
                e.source_handle.clone(),
                e.target_handle.clone(),
            )
        })
        .collect();
    ends.sort();
    ends
}

fn same_node(a: &WorkflowNode, b: &WorkflowNode) -> bool {
    a.kind == b.kind && a.is_disabled() == b.is_disabled() && a.params == b.params
}

pub fn edited_nodes(before: &Workflow, after: &Workflow) -> Vec<String> {
    let mut edited: IndexSet<String> = IndexSet::new();
    for node in &after.nodes {
        let changed = before.node(&node.id).is_none_or(|was| {
            !same_node(was, node) || incoming(before, &node.id) != incoming(after, &node.id)
        });
        if changed {
            edited.insert(node.id.clone());
            edited.extend(downstream_of(after, &node.id));
        }
    }
    edited.into_iter().collect()
}
