use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use indexmap::IndexMap;
use opsis_types::workflow::{Position, Workflow, WorkflowNode};
use serde_json::{Value, json};

use crate::catalog::node_inputs;
use crate::graph::is_group_node;
use crate::lanes::LANE_GAP;

const LAYER_GAP: f64 = 80.0;
const NODE_GAP: f64 = 32.0;
const EDGE_NODE_GAP: f64 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapePort {
    pub handle: String,
    pub side: Side,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardShape {
    pub width: f64,
    pub height: f64,
    pub ports: Vec<ShapePort>,
}

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("couldn't lay out the graph: {0}")]
    Engine(String),
}

const fn side_name(side: Side) -> &'static str {
    match side {
        Side::In => "in",
        Side::Out => "out",
    }
}

fn port_id(node: &str, side: Side, handle: &str) -> String {
    format!("{node}::{}::{handle}", side_name(side))
}

fn listed_ports(wf: &Workflow, node: &WorkflowNode) -> Vec<(String, Side, Option<f64>)> {
    let inputs = node_inputs(wf);
    inputs
        .get(&node.id)
        .into_iter()
        .flat_map(|ports| ports.keys())
        .map(|handle| (handle.clone(), Side::In, None))
        .collect()
}

fn node_graph(wf: &Workflow, node: &WorkflowNode, shape: Option<&CardShape>) -> Value {
    let mut ports: Vec<(String, Side, Option<f64>)> = match shape {
        Some(shape) if !shape.ports.is_empty() => shape
            .ports
            .iter()
            .map(|p| (p.handle.clone(), p.side, Some(p.y)))
            .collect(),
        _ => listed_ports(wf, node),
    };
    let mut known: HashSet<String> = ports
        .iter()
        .map(|(handle, side, _)| port_id(&node.id, *side, handle))
        .collect();
    for edge in &wf.edges {
        let ends = [
            (edge.source == node.id).then(|| (edge.source_handle.clone(), Side::Out)),
            (edge.target == node.id).then(|| (edge.target_handle.clone(), Side::In)),
        ];
        for (handle, side) in ends.into_iter().flatten() {
            if known.insert(port_id(&node.id, side, &handle)) {
                ports.push((handle, side, None));
            }
        }
    }
    let width = shape.map_or(0.0, |s| s.width);
    let height = shape.map_or(0.0, |s| s.height);
    let placed = ports.iter().all(|(_, _, y)| y.is_some());
    let outs = ports.iter().filter(|(_, side, _)| *side == Side::Out);
    let ins = ports.iter().filter(|(_, side, _)| *side == Side::In).rev();
    let elk_ports: Vec<Value> = outs
        .chain(ins)
        .enumerate()
        .map(|(index, (handle, side, y))| {
            let mut port = json!({
                "id": port_id(&node.id, *side, handle),
                "width": 0,
                "height": 0,
                "layoutOptions": {
                    "elk.port.side": if *side == Side::In { "WEST" } else { "EAST" },
                    "elk.port.index": index.to_string(),
                },
            });
            if placed && let Some(y) = y {
                port["x"] = json!(if *side == Side::In { 0.0 } else { width });
                port["y"] = json!(y);
            }
            port
        })
        .collect();
    json!({
        "id": node.id,
        "width": width,
        "height": height,
        "ports": elk_ports,
        "layoutOptions": {
            "elk.portConstraints": if placed { "FIXED_POS" } else { "FIXED_ORDER" },
        },
    })
}

pub fn layout_graph<S: BuildHasher>(
    wf: &Workflow,
    shapes: &HashMap<String, CardShape, S>,
) -> Value {
    let nodes: Vec<&WorkflowNode> = wf
        .nodes
        .iter()
        .filter(|n| n.parent_id.is_none() && !is_group_node(n))
        .collect();
    let ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let children: Vec<Value> = nodes
        .iter()
        .map(|node| node_graph(wf, node, shapes.get(&node.id)))
        .collect();
    let edges: Vec<Value> = wf
        .edges
        .iter()
        .filter(|e| ids.contains(e.source.as_str()) && ids.contains(e.target.as_str()))
        .map(|e| {
            json!({
                "id": e.id,
                "sources": [port_id(&e.source, Side::Out, &e.source_handle)],
                "targets": [port_id(&e.target, Side::In, &e.target_handle)],
            })
        })
        .collect();
    json!({
        "id": "root",
        "layoutOptions": {
            "elk.algorithm": "layered",
            "elk.direction": "RIGHT",
            "elk.edgeRouting": "ORTHOGONAL",
            "elk.layered.spacing.nodeNodeBetweenLayers": LAYER_GAP.to_string(),
            "elk.layered.spacing.edgeEdgeBetweenLayers": LANE_GAP.to_string(),
            "elk.layered.spacing.edgeNodeBetweenLayers": EDGE_NODE_GAP.to_string(),
            "elk.spacing.nodeNode": NODE_GAP.to_string(),
            "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX",
        },
        "children": children,
        "edges": edges,
    })
}

pub fn auto_layout<S: BuildHasher>(
    wf: &Workflow,
    shapes: &HashMap<String, CardShape, S>,
) -> Result<IndexMap<String, Position>, LayoutError> {
    let laid_out =
        elk_layout::layout_value(layout_graph(wf, shapes), "{}").map_err(LayoutError::Engine)?;
    Ok(laid_out
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|child| {
            let id = child.get("id")?.as_str()?.to_owned();
            let x = child.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            let y = child.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            Some((id, Position { x, y }))
        })
        .collect())
}
