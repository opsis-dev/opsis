use indexmap::IndexMap;
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::{AudioValue, CompileValue, NodeResult, PortValues};
use opsis_types::workflow::{Workflow, WorkflowEdge};
use serde::{Deserialize, Serialize};

use crate::clip::{clip_of, clip_to_spec};
use crate::errors::{CompileError, SettingsError};
use crate::graph::topo_sort;
use crate::node::{CLIP_PORT, Ctx, NodeDef};
use crate::params::Values;
use crate::ports::convert_input;
use crate::registry::{is_output_type, node_def};
use crate::vars::{Vars, interpolate_map, workflow_vars};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    pub values: IndexMap<String, PortValues>,
    pub errors: IndexMap<String, String>,
    pub order: Vec<String>,
}

pub const UNKNOWN_TYPE: &str = "This node type isn't available";

fn merge_values(prev: &CompileValue, next: CompileValue) -> CompileValue {
    match (prev, next) {
        (CompileValue::Audio(a), CompileValue::Audio(b)) => CompileValue::Audio(AudioValue {
            items: a.items.iter().cloned().chain(b.items).collect(),
            ..a.clone()
        }),
        (_, next) => next,
    }
}

pub fn gather_inputs<'a>(
    wf: &Workflow,
    def: &NodeDef,
    id: &str,
    value_of: impl Fn(&WorkflowEdge) -> Option<&'a CompileValue>,
) -> PortValues {
    let mut inputs = PortValues::new();
    for e in wf.edges.iter().filter(|e| e.target == id) {
        let spec = def.input(&e.target_handle);
        let Some(value) = convert_input(value_of(e), spec) else {
            continue;
        };
        let merged = match inputs.get(&e.target_handle) {
            Some(prev) if spec.is_some_and(|s| s.many) => merge_values(prev, value),
            _ => value,
        };
        inputs.insert(e.target_handle.clone(), merged);
    }
    inputs
}

fn bypass(def: &NodeDef, inputs: &PortValues) -> PortValues {
    let order = def.input_ports(inputs.keys().map(String::as_str));
    def.outputs
        .iter()
        .filter_map(|(port, spec)| {
            order
                .iter()
                .filter_map(|(input, _, _)| inputs.get(input))
                .find(|v| v.port_type() == spec.kind)
                .map(|v| ((*port).to_owned(), v.clone()))
        })
        .collect()
}

pub fn interpolated_params(def: Option<&NodeDef>, params: &Values, vars: &Vars) -> Values {
    match def {
        Some(def) => interpolate_map(&def.params.with_defaults(params), vars),
        None => interpolate_map(params, vars),
    }
}

pub fn parse_node_params(
    def: &NodeDef,
    node: &opsis_types::workflow::WorkflowNode,
    vars: &Vars,
) -> Result<Values, SettingsError> {
    def.parse(
        &interpolated_params(Some(def), &node.params, vars),
        Some(&node.params),
    )
}

struct RunContext<'a> {
    vars: &'a Vars,
    result: Option<&'a NodeResult>,
}

fn run_compile(
    wf: &Workflow,
    node: &opsis_types::workflow::WorkflowNode,
    def: &NodeDef,
    inputs: &PortValues,
    run: RunContext<'_>,
) -> Result<PortValues, CompileError> {
    let values = def.parse(
        &interpolated_params(Some(def), &node.params, run.vars),
        Some(&node.params),
    )?;
    let ctx = Ctx {
        node_id: &node.id,
        kind: &node.kind,
        label: node.label.as_deref(),
        params: (),
        values: &values,
        inputs,
        settings: &wf.settings,
        result: run.result,
        vars: run.vars,
        workflow: wf,
    };
    Ok(match def.run_compile(&ctx)? {
        Some(out) => out,
        None => run.result.map(|r| r.outputs.clone()).unwrap_or_default(),
    })
}

pub fn compile_workflow(
    wf: &Workflow,
    results: Option<&IndexMap<String, NodeResult>>,
) -> CompileResult {
    let sorted = topo_sort(wf).sorted;
    let vars = workflow_vars(&wf.name, &wf.variables);
    let mut out = CompileResult {
        order: sorted.iter().map(|n| n.id.clone()).collect(),
        ..CompileResult::default()
    };
    for node in sorted {
        let id = &node.id;
        let Some(def) = node_def(&node.kind) else {
            out.errors.insert(id.clone(), UNKNOWN_TYPE.to_owned());
            continue;
        };
        let inputs = gather_inputs(wf, def, id, |e| {
            out.values
                .get(&e.source)
                .and_then(|v| v.get(&e.source_handle))
        });
        if node.is_disabled() {
            out.values.insert(id.clone(), bypass(def, &inputs));
            continue;
        }
        let run = RunContext {
            vars: &vars,
            result: results.and_then(|r| r.get(id)),
        };
        match run_compile(wf, node, def, &inputs, run) {
            Ok(values) => {
                out.values.insert(id.clone(), values);
            }
            Err(error) => {
                out.values.insert(id.clone(), PortValues::new());
                out.errors.insert(id.clone(), error.to_string());
            }
        }
    }
    out
}

pub fn film_of(wf: &Workflow, result: &CompileResult, id: &str) -> Option<TimelineSpec> {
    let node = wf
        .nodes
        .iter()
        .find(|n| n.id == id && !n.is_disabled() && !result.errors.contains_key(id))?;
    let def = node_def(&node.kind).filter(|_| is_output_type(&node.kind))?;
    let inputs = gather_inputs(wf, def, id, |e| {
        result
            .values
            .get(&e.source)
            .and_then(|v| v.get(&e.source_handle))
    });
    clip_of(inputs.get(CLIP_PORT)).map(|clip| clip_to_spec(clip, &wf.settings))
}

pub fn films(wf: &Workflow, result: &CompileResult) -> IndexMap<String, TimelineSpec> {
    result
        .order
        .iter()
        .filter_map(|id| film_of(wf, result, id).map(|spec| (id.clone(), spec)))
        .collect()
}

fn pick_spec(wf: &Workflow, result: &CompileResult, id: &str) -> Option<TimelineSpec> {
    let out = result.values.get(id)?;
    film_of(wf, result, id).or_else(|| {
        out.values()
            .find_map(CompileValue::as_clip)
            .map(|clip| clip_to_spec(clip, &wf.settings))
    })
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FilmTarget {
    pub spec: TimelineSpec,
    pub spans: IndexMap<String, crate::timeline::flatten::FrameRange>,
    pub source_node_id: String,
}

pub fn picked_film(
    wf: &Workflow,
    result: &CompileResult,
    selected: Option<&str>,
) -> Option<FilmTarget> {
    let candidates = selected
        .into_iter()
        .map(str::to_owned)
        .chain(films(wf, result).into_keys())
        .chain(result.order.iter().rev().cloned());
    for id in candidates {
        if let Some(spec) = pick_spec(wf, result, &id) {
            let spans = crate::timeline::flatten::node_spans(&spec);
            return Some(FilmTarget {
                spec,
                spans,
                source_node_id: id,
            });
        }
    }
    None
}
