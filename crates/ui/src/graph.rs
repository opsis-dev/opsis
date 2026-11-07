use std::collections::HashMap;

use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, workflow_cards};
use opsis_compile::catalog::{CatalogPort, EditorNode, node_inputs};
use opsis_compile::compile::compile_workflow;
use opsis_compile::edits::workflow::absolute_position;
use opsis_compile::graph::{GraphIssue, IssueLevel};
use opsis_compile::lanes::{LaneBox, LaneEdge, step_positions};
use opsis_compile::layout::CardShape;
use opsis_qt::{CardKind, CardView, EdgeView, PortView, RowView};
use opsis_types::cards::CardRow;
use opsis_types::events::{NodeRunStatus, RunProgress};
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde::Deserialize;
use serde_json::Value;

use crate::look;

pub const CARD_WIDTH: f64 = 256.0;
const HEADER_HEIGHT: f64 = 44.0;
const BODY_PADDING: f64 = 8.0;
const PORT_ROW: f64 = 20.0;
const PORT_GAP: f64 = 4.0;
const CARD_ROW: f64 = 22.0;
const NOTE_HEIGHT: f64 = 120.0;
const PROGRESS_HEIGHT: f64 = 28.0;
const GROUP_WIDTH: f64 = 240.0;
const GROUP_HEIGHT: f64 = 160.0;
const GROUP_TYPE: &str = "core.group";
const NOTE_TYPE: &str = "core.note";

#[derive(Debug, Default, Deserialize)]
pub struct CompiledErrors {
    #[serde(default)]
    pub errors: IndexMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Compiled {
    #[serde(default)]
    pub seq: u64,
    #[serde(default)]
    pub result: CompiledErrors,
    #[serde(default)]
    pub issues: Vec<GraphIssue>,
    #[serde(default)]
    pub cards: IndexMap<String, Vec<CardRow>>,
    #[serde(default)]
    pub inputs: IndexMap<String, IndexMap<String, CatalogPort>>,
    #[serde(default)]
    pub outdated: Vec<String>,
    #[serde(default)]
    pub inspector: Option<opsis_compile::inspector::InspectorContext>,
    #[serde(default)]
    pub flat: Option<FlatLinks>,
    #[serde(default)]
    pub plan: Option<opsis_compile::mix::plan::MixPlan>,
    #[serde(default)]
    pub mix: opsis_types::mix::Mix,
    #[serde(default)]
    pub solo: bool,
    #[serde(default)]
    pub film: Option<FilmSpans>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct FrameSpan {
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct FilmSpans {
    #[serde(default)]
    pub spans: std::collections::BTreeMap<String, FrameSpan>,
}

impl FilmSpans {
    pub fn active_at(&self, frame: f64) -> Vec<String> {
        let spans = self
            .spans
            .iter()
            .map(|(id, span)| (id.clone(), (span.from, span.to)))
            .collect();
        opsis_compile::timeline::view::active_at(&spans, frame)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FlatLinks {
    #[serde(default)]
    pub bars: Vec<FlatBarLinks>,
}

#[derive(Debug, Default, Deserialize)]
pub struct FlatBarLinks {
    #[serde(default)]
    pub chain: Vec<FlatLink>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatLink {
    #[serde(default)]
    pub node_id: Option<String>,
    pub from: f64,
    pub rate: f64,
}

pub fn playhead_in_node(flat: &FlatLinks, node_id: &str, frame: f64) -> Option<f64> {
    flat.bars
        .iter()
        .flat_map(|bar| &bar.chain)
        .find(|link| link.node_id.as_deref() == Some(node_id))
        .map(|link| ((frame - link.from) * link.rate).round().max(0.0))
}

#[derive(Debug, Default)]
struct Badge {
    label: &'static str,
    icon: &'static str,
    tone: &'static str,
    tip: &'static str,
}

const fn badge(status: Option<NodeRunStatus>, changed: bool) -> Option<Badge> {
    if changed {
        return Some(Badge {
            label: "Changed",
            icon: "refresh-cw",
            tone: "outline",
            tip: "Changed since the last run",
        });
    }
    let (label, icon, tone, tip) = match status {
        Some(NodeRunStatus::Queued) => ("Waiting", "clock", "outline", ""),
        Some(NodeRunStatus::Done) => ("Done", "circle-check", "outline", ""),
        Some(NodeRunStatus::Cached) => ("Up to date", "check-check", "secondary", ""),
        Some(NodeRunStatus::Skipped) => (
            "Skipped",
            "circle-slash",
            "outline",
            "Something before it failed",
        ),
        Some(NodeRunStatus::Error) => ("Failed", "circle-x", "destructive", ""),
        _ => return None,
    };
    Some(Badge {
        label,
        icon,
        tone,
        tip,
    })
}

const fn settled(status: Option<NodeRunStatus>) -> bool {
    matches!(
        status,
        None | Some(NodeRunStatus::Idle | NodeRunStatus::Done | NodeRunStatus::Cached)
    )
}

fn apply_run(view: &mut CardView, run: &RunProgress, outdated: bool) {
    let status = run.statuses.get(&view.id).copied();
    let changed = outdated && settled(status);
    if let Some(found) = badge(status, changed) {
        found.label.clone_into(&mut view.badge);
        found.icon.clone_into(&mut view.badge_icon);
        found.tone.clone_into(&mut view.badge_tone);
        found.tip.clone_into(&mut view.badge_tip);
    }
    let ring = match status {
        Some(NodeRunStatus::Running) => "loading",
        Some(NodeRunStatus::Error) => "error",
        Some(NodeRunStatus::Done | NodeRunStatus::Cached) if !changed => "done",
        _ => "",
    };
    ring.clone_into(&mut view.ring);
    if status == Some(NodeRunStatus::Running)
        && let Some(progress) = run.progress.get(&view.id)
    {
        view.progress = progress.value.clamp(0.0, 1.0);
        view.progress_message = progress.message.clone().unwrap_or_default();
    }
}

fn first_issue(issues: &[GraphIssue], id: &str, level: IssueLevel) -> Option<String> {
    issues
        .iter()
        .find(|issue| issue.level == level && issue.node_id.as_deref() == Some(id))
        .map(|issue| issue.message.clone())
}

pub fn cards(
    wf: &Workflow,
    catalog: &IndexMap<String, EditorNode>,
    compiled: &Compiled,
    run: &RunProgress,
) -> Vec<CardView> {
    let local = (compiled.seq == 0).then(|| compile_workflow(wf, None));
    let local_rows = local.as_ref().map(|result| {
        let gone = |_: &str| false;
        workflow_cards(
            wf,
            &CardSources {
                compiled: result,
                results: None,
                secrets: None,
                gone: &gone,
            },
        )
    });
    let rows = local_rows.as_ref().unwrap_or(&compiled.cards);
    let errors = local
        .as_ref()
        .map_or(&compiled.result.errors, |r| &r.errors);
    let fallback_inputs = node_inputs(wf);
    let mut cards: Vec<CardView> = wf
        .nodes
        .iter()
        .map(|node| {
            let inputs = compiled
                .inputs
                .get(&node.id)
                .or_else(|| fallback_inputs.get(&node.id));
            let error = errors
                .get(&node.id)
                .cloned()
                .or_else(|| first_issue(&compiled.issues, &node.id, IssueLevel::Error));
            let mut view = card(
                wf,
                node,
                catalog.get(&node.kind),
                inputs,
                rows.get(&node.id),
                error,
            );
            if !node.disabled.unwrap_or(false) && view.problem.is_empty() {
                view.warning = first_issue(&compiled.issues, &node.id, IssueLevel::Warning)
                    .unwrap_or_default();
            }
            apply_run(&mut view, run, compiled.outdated.contains(&node.id));
            finish_height(&mut view);
            view
        })
        .collect();
    cards.sort_by_key(|c| c.kind != CardKind::Group);
    cards
}

fn finish_height(view: &mut CardView) {
    if view.kind != CardKind::Card {
        return;
    }
    if view.progress >= 0.0 {
        view.height += PROGRESS_HEIGHT;
    }
    if !view.problem.is_empty() || !view.warning.is_empty() {
        view.height = CARD_ROW.mul_add(2.0, view.height);
    }
}

fn card(
    wf: &Workflow,
    node: &WorkflowNode,
    def: Option<&EditorNode>,
    inputs: Option<&IndexMap<String, CatalogPort>>,
    rows: Option<&Vec<CardRow>>,
    error: Option<String>,
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
        parent: node.parent_id.clone().unwrap_or_default(),
        motion: def
            .and_then(|d| d.look.as_ref())
            .and_then(|l| l.motion)
            .map_or_default(|m| plain(&m)),
        problem: error.unwrap_or_default(),
        warning: String::new(),
        text: String::new(),
        badge: String::new(),
        badge_icon: String::new(),
        badge_tone: String::new(),
        badge_tip: String::new(),
        ring: String::new(),
        progress: -1.0,
        progress_message: String::new(),
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
            let color = text_param(node, def, "color");
            if !color.is_empty() {
                view.color = color;
            }
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
    let lanes: Vec<LaneEdge<'_>> = wf
        .edges
        .iter()
        .map(|edge| LaneEdge {
            id: &edge.id,
            source: &edge.source,
            target: &edge.target,
        })
        .collect();
    let steps = step_positions(&boxes, &lanes);
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

pub fn shapes(cards: &[CardView]) -> HashMap<String, CardShape> {
    cards
        .iter()
        .filter(|card| card.kind == CardKind::Card)
        .map(|card| {
            (
                card.id.clone(),
                CardShape {
                    width: card.width,
                    height: card.height,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn links(nodes: &[(String, f64, f64)]) -> FlatLinks {
        FlatLinks {
            bars: vec![FlatBarLinks {
                chain: nodes
                    .iter()
                    .map(|(id, from, rate)| FlatLink {
                        node_id: Some(id.clone()),
                        from: *from,
                        rate: *rate,
                    })
                    .collect(),
            }],
        }
    }

    proptest! {
        #[test]
        fn the_playhead_in_a_node_is_a_whole_frame_from_its_start(
            from in 0f64..10_000.0,
            rate in 0.1f64..4.0,
            frame in 0f64..20_000.0,
        ) {
            let flat = links(&[("clip".to_owned(), from, rate)]);
            let local = playhead_in_node(&flat, "clip", frame);
            prop_assert!(local.is_some_and(|f| f >= 0.0 && f.fract() == 0.0));
            if frame >= from {
                prop_assert!((local.unwrap_or(-1.0) - ((frame - from) * rate).round()).abs() < f64::EPSILON);
            }
            prop_assert_eq!(playhead_in_node(&flat, "elsewhere", frame), None);
        }
    }
}
