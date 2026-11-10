use std::fmt;
use std::sync::LazyLock;

use indexmap::IndexMap;
use opsis_types::cards::CardRow;
use opsis_types::values::{CompileValue, NodeResult};
use opsis_types::workflow::{PortType, Workflow, WorkflowNode};
use regex::Regex;
use serde_json::Value;

use crate::asset_ref::project_rel_path;
use crate::clip::{input_take, recording_take, scene_item_count};
use crate::compile::CompileResult;
use crate::node::{CLIP_PORT, CardInputs, InputMedia};
use crate::ports::convert_input;
use crate::registry::{node_def, node_name};
use crate::text::format_duration;
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

fn converted_to_file(
    node: &WorkflowNode,
    port: &str,
    value: &CompileValue,
) -> Option<CompileValue> {
    if matches!(value, CompileValue::Media(_)) {
        return None;
    }
    let spec = node_def(&node.kind)?
        .input(port)
        .filter(|s| s.kind == PortType::Media)?;
    convert_input(Some(value), Some(spec))
}

fn picked_names(wf: &Workflow, node: &WorkflowNode) -> IndexMap<String, String> {
    let Some(def) = node_def(&node.kind) else {
        return IndexMap::new();
    };
    node.params
        .iter()
        .filter_map(|(key, value)| {
            let id = value.as_str().filter(|v| !v.is_empty())?;
            let field = def.params.field(key)?;
            let name = match field["x-widget"].as_str()? {
                "action" => node_name(wf.node(id)?),
                "marker" => {
                    let marker = wf.markers.iter().find(|m| m.id == id)?;
                    if marker.label.is_empty() {
                        format!("the marker at {}", format_duration(marker.at_ms))
                    } else {
                        marker.label.clone()
                    }
                }
                _ => return None,
            };
            Some((id.to_owned(), name))
        })
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
        hits: compiled
            .values
            .get(&node.id)
            .and_then(|v| v.values().find_map(CompileValue::as_audio))
            .and_then(|a| a.hits),
        ..CardInputs::default()
    };
    inputs.names = picked_names(wf, node);
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
        if let Some(CompileValue::Media(m)) = converted_to_file(node, &port, v) {
            inputs.media.insert(
                port.clone(),
                InputMedia {
                    kind: m.kind,
                    has_audio: m.has_audio,
                },
            );
        }
        match v {
            CompileValue::Media(m) => {
                if let (Some(w), Some(h)) = (m.width, m.height) {
                    inputs.pixels.insert(port.clone(), (w, h));
                }
                inputs.media.insert(
                    port,
                    InputMedia {
                        kind: m.kind,
                        has_audio: m.has_audio,
                    },
                );
            }
            CompileValue::Image(image) => {
                if let (Some(w), Some(h)) = (image.width, image.height) {
                    inputs.pixels.insert(port, (w, h));
                }
            }
            CompileValue::Transition(t) => {
                inputs.enters.insert(port, t.clone());
            }
            CompileValue::Captions(c) => {
                inputs
                    .captions_ms
                    .insert(port, c.captions.last().map_or(0.0, |c| c.end_ms));
            }
            CompileValue::BrowserSession(s) => {
                inputs
                    .steps
                    .insert(port, s.steps.iter().map(|s| s.kind).collect());
            }
            CompileValue::TerminalSession(s) => {
                inputs.terminal_steps.insert(port, s.steps.len());
            }
            CompileValue::RecordingStyle(s) => {
                inputs.style.insert(port, s.entries.clone());
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
                if let Some(enter) = &c.enter {
                    inputs.enters.insert(port.clone(), enter.clone());
                }
                inputs.frames.insert(port.clone(), c.duration_in_frames);
                inputs.items.insert(port.clone(), scene_item_count(c));
                if let Some(take) = input_take(c) {
                    inputs.takes.insert(port, take);
                }
            }
            CompileValue::Audio(_) => {}
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

impl fmt::Debug for CardSources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardSources")
            .field("compiled", &self.compiled)
            .field("results", &self.results)
            .field("secrets", &self.secrets)
            .finish_non_exhaustive()
    }
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
