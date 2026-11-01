use std::collections::HashMap;
use std::path::Path;

use opsis_compile::compile::interpolated_params;
use opsis_compile::node::Phase;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::hash::versioned_hash;
use opsis_engine::versions::{PICTURE_VERSION, PREPARED_VERSION};
use opsis_project::stamps::file_stamps;
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value, json};

use crate::context::Executors;

pub const ENGINE_VERSION: &str = "opsis-engine/1";
const KEY_LENGTH: usize = 24;

#[derive(Debug, Clone, Copy, Default)]
pub struct Release<'a> {
    pub app: &'a str,
    pub engine: &'a str,
}

pub fn node_hash(
    kind: &str,
    params: &Value,
    upstream: &[(String, String)],
    release: &Release<'_>,
) -> String {
    let mut ports: Vec<&(String, String)> = upstream.iter().collect();
    ports.sort_unstable();
    let ports: Vec<[&str; 2]> = ports
        .into_iter()
        .map(|(p, f)| [p.as_str(), f.as_str()])
        .collect();
    versioned_hash(
        ENGINE_VERSION,
        &json!({ "type": kind, "params": params, "ports": ports, "app": release.app, "engine": release.engine }),
        KEY_LENGTH,
    )
}

#[derive(Debug)]
pub struct KeyInputs<'a> {
    pub workflow: &'a Workflow,
    pub vars: &'a Vars,
    pub project_dir: &'a Path,
    pub executors: &'a Executors,
    pub release: Release<'a>,
}

pub fn hash_nodes(order: &[&WorkflowNode], inputs: &KeyInputs<'_>) -> HashMap<String, String> {
    let wf = inputs.workflow;
    let mut hashes: HashMap<String, String> = HashMap::new();
    for node in order {
        let upstream: Vec<(String, String)> = wf
            .edges
            .iter()
            .filter(|e| e.target == node.id)
            .filter_map(|e| {
                hashes
                    .get(&e.source)
                    .map(|h| (e.target_handle.clone(), format!("{h}:{}", e.source_handle)))
            })
            .collect();
        if node.is_disabled() {
            hashes.insert(
                node.id.clone(),
                node_hash(
                    &format!("{}:skipped", node.kind),
                    &Value::Null,
                    &upstream,
                    &inputs.release,
                ),
            );
            continue;
        }
        let def = node_def(&node.kind);
        let params = interpolated_params(def, &node.params, inputs.vars);
        let mut stamps = file_stamps(inputs.project_dir, &Value::Object(params.clone()));
        if let Some(executor) = inputs.executors.get(&node.kind) {
            stamps.extend(executor.stamps(inputs.project_dir, &params));
        }
        let files: Map<String, Value> = stamps
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect();
        let keyed = match def {
            Some(d) if d.phase == Phase::Render => json!({
                "params": params,
                "settings": wf.settings,
                "mix": wf.mix,
                "picture": PICTURE_VERSION,
                "prepared": PREPARED_VERSION,
                "vars": inputs.vars,
                "files": files,
            }),
            Some(d) if d.uses_project_size || d.uses_project_fonts => {
                let mut keyed = json!({ "params": params, "files": files });
                if d.uses_project_size {
                    keyed["size"] = json!({ "width": wf.settings.width, "height": wf.settings.height, "fps": wf.settings.fps });
                }
                if d.uses_project_fonts {
                    keyed["fonts"] = json!(wf.settings.fonts);
                }
                keyed
            }
            _ if !files.is_empty() => json!({ "params": params, "files": files }),
            _ => Value::Object(params),
        };
        hashes.insert(
            node.id.clone(),
            node_hash(&node.kind, &keyed, &upstream, &inputs.release),
        );
    }
    hashes
}
