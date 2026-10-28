use indexmap::IndexMap;
use opsis_types::actions::Viewport;
use opsis_types::values::{BrowserSessionValue, CompileValue, MediaKind, NodeResult};
use opsis_types::workflow::Workflow;
use serde::Serialize;
use serde_json::Value;

use crate::asset_ref::kind_from_path;
use crate::color::project_swatches;
use crate::compile::CompileResult;
use crate::elements::introspect::widget_keys;
use crate::graph::{downstream_of, session_into, upstream_of};
use crate::node::{Category, SESSION_PORT};
use crate::nodes::media::APPLY_GRADE_TYPE;
use crate::params::{ParamSchema, Values};
use crate::ports::convert_input;
use crate::registry::{node_def, node_name};
use crate::vars::{PROJECT_NAME_VAR, SITE_ADDRESS_VAR, interpolate_text, workflow_vars};

const NO_PAGE: &str = "Add a Go to page action before this one, so there's a page to pick from.";
const MAX_SWATCHES: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
pub struct VarInfo {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
pub struct ActionRef {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SpotBackground {
    pub src: String,
    pub time_ms: f64,
    pub duration_ms: f64,
    pub still: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
pub struct PickerPage {
    pub url: String,
    pub viewport: Option<Viewport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PickerTarget {
    Page(PickerPage),
    Problem { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InspectorContext {
    pub node_id: String,
    pub vars: Vec<VarInfo>,
    pub layers: usize,
    pub actions: Vec<ActionRef>,
    pub spot_background: Option<SpotBackground>,
    pub picker: PickerTarget,
    pub swatches: Vec<String>,
    pub connected_inputs: Vec<String>,
    pub ports: Vec<PortChoice>,
    pub file_kinds: IndexMap<String, MediaKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
pub struct PortChoice {
    pub port: String,
    pub label: String,
}

fn file_kinds(params: &ParamSchema, values: &Values) -> IndexMap<String, MediaKind> {
    let values = params.with_defaults(values);
    widget_keys(params, "asset")
        .into_iter()
        .filter_map(|key| {
            let path = values.get(&key)?.as_str()?.trim();
            (!path.is_empty()).then(|| (key, kind_from_path(path)))
        })
        .collect()
}

fn variable_list(wf: &Workflow) -> Vec<VarInfo> {
    let vars = workflow_vars(&wf.name, &wf.variables);
    wf.variables
        .iter()
        .map(|(name, value)| VarInfo {
            name: name.clone(),
            value: value.clone(),
        })
        .chain(std::iter::once(VarInfo {
            name: PROJECT_NAME_VAR.to_owned(),
            value: vars.get(PROJECT_NAME_VAR).cloned().unwrap_or_default(),
        }))
        .collect()
}

fn recording_before(
    wf: &Workflow,
    node_id: &str,
    results: &IndexMap<String, NodeResult>,
) -> Option<SpotBackground> {
    upstream_of(wf, node_id).iter().find_map(|id| {
        match results.get(id)?.outputs.get("recording")? {
            CompileValue::Recording(rec) => rec.src.clone().map(|src| SpotBackground {
                src,
                time_ms: 0.0,
                duration_ms: rec.duration_ms,
                still: false,
            }),
            _ => None,
        }
    })
}

fn graded_file(wf: &Workflow, node_id: &str, compiled: &CompileResult) -> Option<SpotBackground> {
    let later = downstream_of(wf, node_id);
    let media_port = node_def(APPLY_GRADE_TYPE).and_then(|d| {
        d.inputs
            .iter()
            .find(|(name, _)| *name == "media")
            .map(|(_, spec)| spec)
    });
    wf.nodes
        .iter()
        .filter(|n| n.kind == APPLY_GRADE_TYPE && later.contains(&n.id))
        .find_map(|apply| {
            let edge = wf
                .edges
                .iter()
                .find(|e| e.target == apply.id && e.target_handle == "media")?;
            let value = compiled
                .values
                .get(&edge.source)
                .and_then(|v| v.get(&edge.source_handle));
            match convert_input(value, media_port)? {
                CompileValue::Media(media) if media.kind != MediaKind::Audio => {
                    media.src.map(|src| SpotBackground {
                        src,
                        time_ms: 0.0,
                        duration_ms: media.duration_ms.map_or(0.0, f64::from),
                        still: media.kind == MediaKind::Image,
                    })
                }
                _ => None,
            }
        })
}

fn input_picture(wf: &Workflow, node_id: &str, compiled: &CompileResult) -> Option<SpotBackground> {
    let edge = wf
        .edges
        .iter()
        .find(|e| e.target == node_id && e.target_handle == crate::nodes::image::IMAGE_PORT)?;
    match compiled
        .values
        .get(&edge.source)?
        .get(&edge.source_handle)?
    {
        CompileValue::Image(image) => image.src.clone(),
        CompileValue::Media(media) if media.kind == MediaKind::Image => media.src.clone(),
        _ => None,
    }
    .map(|src| SpotBackground {
        src,
        time_ms: 0.0,
        duration_ms: 0.0,
        still: true,
    })
}

fn session_at<'a>(
    wf: &Workflow,
    compiled: &'a CompileResult,
    node_id: &str,
) -> Option<&'a BrowserSessionValue> {
    match compiled
        .values
        .get(node_id)
        .and_then(|v| v.get(SESSION_PORT))
    {
        Some(CompileValue::BrowserSession(session)) => Some(session),
        _ => session_into(wf, compiled, node_id),
    }
}

fn pixels(params: &serde_json::Map<String, Value>, key: &str) -> Option<u32> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
}

fn picker_target(wf: &Workflow, compiled: &CompileResult, node_id: &str) -> PickerTarget {
    let session = session_at(wf, compiled, node_id);
    let Some(url) = session
        .and_then(|s| s.address.clone())
        .or_else(|| wf.variables.get(SITE_ADDRESS_VAR).cloned())
    else {
        return PickerTarget::Problem {
            message: NO_PAGE.to_owned(),
        };
    };
    let viewport = session.and_then(|s| {
        Some(Viewport {
            width: pixels(&s.browser.params, "viewportWidth")?,
            height: pixels(&s.browser.params, "viewportHeight")?,
        })
    });
    PickerTarget::Page(PickerPage {
        url: interpolate_text(&url, &workflow_vars(&wf.name, &wf.variables)),
        viewport,
    })
}

fn has_boxes(params: &Value) -> bool {
    params["properties"].as_object().is_some_and(|fields| {
        fields
            .values()
            .any(|f| f["x-widget"] == "boxes" || f["x-widget"] == "marks")
    })
}

pub fn inspector_context(
    wf: &Workflow,
    compiled: &CompileResult,
    results: &IndexMap<String, NodeResult>,
    node_id: &str,
) -> Option<InspectorContext> {
    let node = wf.nodes.iter().find(|n| n.id == node_id)?;
    let def = node_def(&node.kind)?;
    let connected_inputs: Vec<String> = wf
        .edges
        .iter()
        .filter(|e| e.target == node_id)
        .map(|e| e.target_handle.clone())
        .collect();
    let family = def.family.as_ref();
    let connected = family.map_or_else(Vec::new, |f| {
        f.connected(connected_inputs.iter().map(String::as_str))
    });
    let layers = connected.len().max(1);
    let ports = family.map_or_else(Vec::new, |f| {
        connected
            .iter()
            .map(|&i| PortChoice {
                port: f.port(i),
                label: f.label(i, connected.last().copied()),
            })
            .collect()
    });
    let spot_background = match def.category {
        Category::Grade => graded_file(wf, node_id, compiled),
        Category::Image => input_picture(wf, node_id, compiled),
        Category::Camera => recording_before(wf, node_id, results),
        _ if has_boxes(def.params.json()) => recording_before(wf, node_id, results),
        _ => None,
    };
    let params: Vec<Value> = wf
        .nodes
        .iter()
        .map(|n| Value::Object(n.params.clone()))
        .collect();
    let params: Vec<&Value> = params.iter().collect();
    Some(InspectorContext {
        node_id: node_id.to_owned(),
        vars: variable_list(wf),
        layers,
        actions: wf
            .nodes
            .iter()
            .filter(|n| node_def(&n.kind).is_some_and(|d| d.action))
            .map(|n| ActionRef {
                id: n.id.clone(),
                label: node_name(n),
            })
            .collect(),
        spot_background,
        picker: picker_target(wf, compiled, node_id),
        swatches: project_swatches(&params, MAX_SWATCHES),
        file_kinds: file_kinds(&def.params, &node.params),
        connected_inputs,
        ports,
    })
}
