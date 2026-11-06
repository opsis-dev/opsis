use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use indexmap::IndexMap;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;
use opsis_types::workflow::{Position, Workflow, WorkflowNode};

use crate::graph::is_group_node;

pub const LAYER_GAP: f64 = 80.0;
pub const NODE_GAP: f64 = 32.0;
const FALLBACK_WIDTH: f64 = 256.0;
const FALLBACK_HEIGHT: f64 = 80.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardShape {
    pub width: f64,
    pub height: f64,
}

fn laid_out(wf: &Workflow) -> Vec<&WorkflowNode> {
    wf.nodes
        .iter()
        .filter(|n| n.parent_id.is_none() && !is_group_node(n))
        .collect()
}

pub fn auto_layout<S: BuildHasher>(
    wf: &Workflow,
    shapes: &HashMap<String, CardShape, S>,
) -> IndexMap<String, Position> {
    let nodes = laid_out(wf);
    if nodes.is_empty() {
        return IndexMap::new();
    }
    let mut graph = VisualGraph::new(Orientation::LeftToRight);
    let sizes: Vec<CardShape> = nodes
        .iter()
        .map(|node| {
            shapes.get(&node.id).copied().unwrap_or(CardShape {
                width: FALLBACK_WIDTH,
                height: FALLBACK_HEIGHT,
            })
        })
        .collect();
    let handles: Vec<_> = sizes
        .iter()
        .map(|size| {
            graph.add_node(Element::create(
                ShapeKind::Box(String::new()),
                StyleAttr::simple(),
                Orientation::LeftToRight,
                Point::new(size.width + LAYER_GAP, size.height + NODE_GAP),
            ))
        })
        .collect();
    let index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut joined: HashSet<(usize, usize)> = HashSet::new();
    for edge in &wf.edges {
        let (Some(from), Some(to)) = (
            index.get(edge.source.as_str()),
            index.get(edge.target.as_str()),
        ) else {
            continue;
        };
        if from != to && joined.insert((*from, *to)) {
            graph.add_edge(Arrow::simple(""), handles[*from], handles[*to]);
        }
    }
    graph.do_it(false, false, false, &mut SVGWriter::new());
    let corners: Vec<(f64, f64)> = handles
        .iter()
        .zip(&sizes)
        .map(|(handle, size)| {
            let middle = graph.pos(*handle).middle();
            (
                middle.x - f64::midpoint(size.width, LAYER_GAP),
                middle.y - f64::midpoint(size.height, NODE_GAP),
            )
        })
        .collect();
    let left = corners.iter().map(|c| c.0).fold(f64::MAX, f64::min);
    let top = corners.iter().map(|c| c.1).fold(f64::MAX, f64::min);
    nodes
        .iter()
        .zip(corners)
        .map(|(node, (x, y))| {
            (
                node.id.clone(),
                Position {
                    x: (x - left).round(),
                    y: (y - top).round(),
                },
            )
        })
        .collect()
}
