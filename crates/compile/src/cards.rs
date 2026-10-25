use std::sync::LazyLock;

use indexmap::IndexMap;
use opsis_types::cards::CardRow;
use opsis_types::values::{CompileValue, NodeResult};
use opsis_types::workflow::{Workflow, WorkflowNode};
use regex::Regex;
use serde_json::Value;

use crate::asset_ref::project_rel_path;
use crate::clip::{input_take, recording_take, scene_item_count};
use crate::compile::CompileResult;
use crate::node::{CLIP_PORT, CardInputs, InputMedia};
use crate::registry::node_def;
use crate::vars::{Vars, strings_in, workflow_vars};

static FILE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\.[a-z0-9]{2,6}$").expect("pattern"));

pub fn named_files(params: &Value) -> Vec<String> {
    let mut found = Vec::new();
    strings_in(params, &|text| FILE_NAME.is_match(text.trim()), &mut found);
    found.into_iter().map(str::to_owned).collect()
}

pub fn named_project_files(params: &Value) -> Vec<String> {
    named_files(params)
        .into_iter()
        .filter(|v| project_rel_path(v.trim()).is_some())
        .collect()
}

pub fn card_inputs(
    wf: &Workflow,
    compiled: &CompileResult,
    node: &WorkflowNode,
    secrets: Option<&[String]>,
) -> CardInputs {
    let mut inputs = CardInputs {
        secrets: secrets.map(<[String]>::to_vec),
        clip_frames: compiled
            .values
            .get(&node.id)
            .and_then(|v| v.get(CLIP_PORT))
            .and_then(CompileValue::as_clip)
            .map(|c| c.duration_in_frames),
        ..CardInputs::default()
    };
    for e in wf.edges.iter().filter(|e| e.target == node.id) {
        inputs.connected.push(e.target_handle.clone());
        let port = e.target_handle.clone();
        let Some(v) = compiled
            .values
            .get(&e.source)
            .and_then(|v| v.get(&e.source_handle))
        else {
            continue;
        };
        match v {
            CompileValue::Media(m) => {
                inputs.media.insert(
                    port,
                    InputMedia {
                        kind: m.kind,
                        has_audio: m.has_audio,
                    },
                );
            }
            CompileValue::Captions(c) => {
                inputs
                    .captions_ms
                    .insert(port, c.captions.last().map_or(0.0, |c| c.end_ms));
            }
            CompileValue::BrowserSession(s) => {
                inputs
                    .steps
                    .insert(port, s.steps.iter().map(|s| s.kind.clone()).collect());
            }
            CompileValue::RecordingStyle(s) => {
                inputs
                    .style
                    .insert(port, s.entries.iter().map(|s| s.kind.clone()).collect());
            }
            CompileValue::Grade(g) => {
                inputs.grade.insert(port, g.entries.clone());
            }
            CompileValue::Recording(r) => {
                if let Some(take) = recording_take(r) {
                    inputs.takes.insert(port, Some(take));
                }
            }
            CompileValue::Clip(c) => {
                inputs.frames.insert(port.clone(), c.duration_in_frames);
                inputs
                    .items
                    .insert(port.clone(), scene_item_count(c) as f64);
                if let Some(take) = input_take(c) {
                    inputs.takes.insert(port, take);
                }
            }
            _ => {}
        }
    }
    inputs
}

pub struct CardSources<'a> {
    pub compiled: &'a CompileResult,
    pub results: Option<&'a IndexMap<String, NodeResult>>,
    pub secrets: Option<&'a [String]>,
    pub gone: &'a dyn Fn(&str) -> bool,
}

pub fn node_card(
    wf: &Workflow,
    node: &WorkflowNode,
    vars: &Vars,
    sources: &CardSources<'_>,
) -> Option<Vec<CardRow>> {
    let def = node_def(&node.kind)?;
    let mut inputs = card_inputs(wf, sources.compiled, node, sources.secrets);
    let shown = def.params.card_values(&node.params, vars);
    inputs.missing_files = named_project_files(&Value::Object(shown))
        .into_iter()
        .filter(|f| (sources.gone)(f))
        .collect();
    let result = sources.results.and_then(|r| r.get(&node.id));
    Some(def.card_for(&node.params, vars, &wf.settings, result, &inputs))
}

pub fn workflow_cards(wf: &Workflow, sources: &CardSources<'_>) -> IndexMap<String, Vec<CardRow>> {
    let vars = workflow_vars(&wf.name, &wf.variables);
    wf.nodes
        .iter()
        .filter_map(|node| node_card(wf, node, &vars, sources).map(|rows| (node.id.clone(), rows)))
        .collect()
}
