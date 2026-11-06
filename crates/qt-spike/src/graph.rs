use std::collections::HashMap;

use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, workflow_cards};
use opsis_compile::catalog::{CatalogPort, EditorNode, node_inputs};
use opsis_compile::compile::compile_workflow;
use opsis_compile::edits::workflow::absolute_position;
use opsis_types::cards::CardRow;
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::Value;

use crate::bridge::ffi::{CardKind, CardView, EdgeView, PortView, RowView};
use crate::look;

pub const CARD_WIDTH: f64 = 256.0;
const HEADER_HEIGHT: f64 = 44.0;
const BODY_PADDING: f64 = 8.0;
const PORT_ROW: f64 = 20.0;
const PORT_GAP: f64 = 4.0;
const CARD_ROW: f64 = 22.0;
const NOTE_HEIGHT: f64 = 120.0;
const GROUP_WIDTH: f64 = 240.0;
const GROUP_HEIGHT: f64 = 160.0;
const LANE_GAP: f64 = 14.0;
const STEP_OFFSET: f64 = 20.0;
const GROUP_TYPE: &str = "core.group";
const NOTE_TYPE: &str = "core.note";

#[derive(Debug, Clone, Copy)]
struct LaneBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug)]
struct Run<'a> {
    id: &'a str,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

pub fn cards(wf: &Workflow, catalog: &IndexMap<String, EditorNode>) -> Vec<CardView> {
    let compiled = compile_workflow(wf, None);
    let gone = |_: &str| false;
    let sources = CardSources {
        compiled: &compiled,
        results: None,
        secrets: None,
        gone: &gone,
    };
    let rows = workflow_cards(wf, &sources);
    let inputs = node_inputs(wf);
    let mut cards: Vec<CardView> = wf
        .nodes
        .iter()
        .map(|node| {
            card(
                wf,
                node,
                catalog.get(&node.kind),
                inputs.get(&node.id),
                rows.get(&node.id),
                compiled.errors.get(&node.id),
            )
        })
        .collect();
    cards.sort_by_key(|c| c.kind != CardKind::Group);
    cards
}

fn card(
    wf: &Workflow,
    node: &WorkflowNode,
    def: Option<&EditorNode>,
    inputs: Option<&IndexMap<String, CatalogPort>>,
    rows: Option<&Vec<CardRow>>,
    error: Option<&String>,
) -> CardView {
    let at = absolute_position(wf, node);
    let category = def.map_or_default(category_name);
    let mut view = CardView {
        id: node.id.clone(),
        node_type: node.kind.clone(),
        kind: CardKind::Card,
        title: node
            .label
            .clone()
            .or_else(|| def.map(|d| d.title.clone()))
            .unwrap_or_else(|| node.kind.clone()),
        category: def.map_or_default(|d| d.category_label.clone()),
        color: look::category_color(&category).to_owned(),
        glyph: def
            .and_then(|d| d.look.as_ref())
            .map_or_default(|l| l.glyph.clone()),
        x: at.x,
        y: at.y,
        width: CARD_WIDTH,
        height: HEADER_HEIGHT,
        inputs: Vec::new(),
        outputs: Vec::new(),
        rows: Vec::new(),
        disabled: node.disabled.unwrap_or(false),
        problem: error.cloned().unwrap_or_default(),
        text: String::new(),
    };
    match node.kind.as_str() {
        GROUP_TYPE => {
            view.kind = CardKind::Group;
            view.title = text_param(node, def, "title");
            view.text = text_param(node, def, "color");
            let size = node.size.as_ref();
            view.width = size.map_or(GROUP_WIDTH, |s| f64::from(s.width));
            view.height = size.map_or(GROUP_HEIGHT, |s| f64::from(s.height));
        }
        NOTE_TYPE => {
            view.kind = CardKind::Note;
            view.text = text_param(node, def, "text");
            let size = node.size.as_ref();
            view.width = size.map_or(CARD_WIDTH, |s| f64::from(s.width));
            view.height = size.map_or(NOTE_HEIGHT, |s| f64::from(s.height));
        }
        _ => lay_out_card(&mut view, def, inputs, rows),
    }
    view
}

fn lay_out_card(
    view: &mut CardView,
    def: Option<&EditorNode>,
    inputs: Option<&IndexMap<String, CatalogPort>>,
    rows: Option<&Vec<CardRow>>,
) {
    let top = HEADER_HEIGHT + BODY_PADDING;
    let inputs = inputs.or_else(|| def.map(|d| &d.inputs));
    view.inputs = ports(inputs, top);
    view.outputs = ports(def.map(|d| &d.outputs), top);
    let port_rows = view.inputs.len().max(view.outputs.len());
    let mut y = top + rows_height(port_rows, PORT_ROW + PORT_GAP);
    if port_rows > 0 {
        y += PORT_GAP;
    }
    view.rows = rows
        .into_iter()
        .flatten()
        .map(|row| RowView {
            icon: plain(&row.icon),
            text: row.text.clone(),
            muted: row.tone.is_some(),
        })
        .collect();
    y += rows_height(view.rows.len(), CARD_ROW);
    if !view.problem.is_empty() {
        y = CARD_ROW.mul_add(2.0, y);
    }
    view.height = y + BODY_PADDING;
}

fn rows_height(count: usize, each: f64) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX)) * each
}

fn ports(ports: Option<&IndexMap<String, CatalogPort>>, top: f64) -> Vec<PortView> {
    ports
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, (id, port))| {
            let kind = plain(&port.kind);
            PortView {
                id: id.clone(),
                label: port.label.clone(),
                color: look::port_color(&kind).to_owned(),
                kind,
                optional: port.optional,
                y: top + rows_height(index, PORT_ROW + PORT_GAP) + PORT_ROW / 2.0,
            }
        })
        .collect()
}

fn plain(value: &impl serde::Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(text)) => text,
        _ => String::new(),
    }
}

fn category_name(def: &EditorNode) -> String {
    plain(&def.category)
}

fn text_param(node: &WorkflowNode, def: Option<&EditorNode>, key: &str) -> String {
    node.params
        .get(key)
        .or_else(|| def.and_then(|d| d.defaults.get(key)))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

pub fn edges(wf: &Workflow, cards: &[CardView]) -> Vec<EdgeView> {
    let boxes: HashMap<&str, LaneBox> = cards
        .iter()
        .map(|c| {
            (
                c.id.as_str(),
                LaneBox {
                    x: c.x,
                    y: c.y,
                    width: c.width,
                    height: c.height,
                },
            )
        })
        .collect();
    let steps = step_positions(&boxes, wf);
    wf.edges
        .iter()
        .map(|edge| {
            let color = cards
                .iter()
                .find(|c| c.id == edge.source)
                .and_then(|c| c.outputs.iter().find(|p| p.id == edge.source_handle))
                .map_or_else(|| look::NEUTRAL.to_owned(), |p| p.color.clone());
            EdgeView {
                id: edge.id.clone(),
                source: edge.source.clone(),
                source_handle: edge.source_handle.clone(),
                target: edge.target.clone(),
                target_handle: edge.target_handle.clone(),
                color,
                step: steps.get(edge.id.as_str()).copied().unwrap_or(0.5),
            }
        })
        .collect()
}

fn run_of<'a>(
    id: &'a str,
    source: &str,
    target: &str,
    boxes: &HashMap<&str, LaneBox>,
) -> Option<(i64, Run<'a>)> {
    let from = boxes.get(source)?;
    let to = boxes.get(target)?;
    let left = from.x + from.width + STEP_OFFSET;
    let right = to.x - STEP_OFFSET;
    (right > left).then(|| {
        (
            opsis_types::numbers::truncate(to.x.round()),
            Run {
                id,
                left,
                right,
                top: from.y.min(to.y),
                bottom: (from.y + from.height).max(to.y + to.height),
            },
        )
    })
}

fn lanes_of(runs: &mut [Run<'_>]) -> Vec<usize> {
    runs.sort_by(|a, b| {
        a.top
            .total_cmp(&b.top)
            .then(a.bottom.total_cmp(&b.bottom))
            .then(a.id.cmp(b.id))
    });
    let mut ends: Vec<f64> = Vec::new();
    runs.iter()
        .map(|run| {
            let lane = ends
                .iter()
                .position(|end| *end < run.top)
                .unwrap_or(ends.len());
            if lane == ends.len() {
                ends.push(run.bottom);
            } else {
                ends[lane] = run.bottom;
            }
            lane
        })
        .collect()
}

fn step_positions<'a>(boxes: &HashMap<&str, LaneBox>, wf: &'a Workflow) -> HashMap<&'a str, f64> {
    let mut columns: IndexMap<i64, Vec<Run<'a>>> = IndexMap::new();
    for edge in &wf.edges {
        if let Some((column, run)) = run_of(&edge.id, &edge.source, &edge.target, boxes) {
            columns.entry(column).or_default().push(run);
        }
    }
    let mut steps = HashMap::new();
    for runs in columns.values_mut() {
        let lanes = lanes_of(runs);
        let count = lanes.iter().max().map_or(1, |m| m + 1);
        let left = runs.iter().map(|r| r.left).fold(f64::MIN, f64::max);
        let right = runs.iter().map(|r| r.right).fold(f64::MAX, f64::min);
        if right <= left {
            continue;
        }
        let count_f = rows_height(count, 1.0);
        let spacing = LANE_GAP.min((right - left) / (count_f + 1.0));
        let middle = f64::midpoint(left, right);
        for (run, lane) in runs.iter().zip(lanes) {
            let offset = rows_height(lane, 1.0) - (count_f - 1.0) / 2.0;
            let x = offset.mul_add(spacing, middle);
            steps.insert(run.id, (x - run.left) / (run.right - run.left));
        }
    }
    steps
}
