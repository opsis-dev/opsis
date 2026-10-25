use std::collections::HashMap;
use std::path::Path;

use opsis_compile::compile::interpolated_params;
use opsis_compile::node::Phase;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::hash::versioned_hash;
use opsis_engine::versions::PICTURE_VERSION;
use opsis_project::stamps::{file_stamps, sign_in_stamp};
use opsis_types::secrets::secret_name;
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value, json};

pub const ENGINE_VERSION: &str = "opsis-engine/1";
const KEY_LENGTH: usize = 24;

pub fn node_hash(
    kind: &str,
    params: &Value,
    upstream: &[(String, String)],
    app_version: &str,
) -> String {
    let mut ports: Vec<&(String, String)> = upstream.iter().collect();
    ports.sort_unstable();
    let ports: Vec<[&str; 2]> = ports
        .into_iter()
        .map(|(p, f)| [p.as_str(), f.as_str()])
        .collect();
    versioned_hash(
        ENGINE_VERSION,
        &json!({ "type": kind, "params": params, "ports": ports, "app": app_version }),
        KEY_LENGTH,
    )
}

fn secret_stamps(
    node: &WorkflowNode,
    params: &Map<String, Value>,
    project_dir: &Path,
) -> Map<String, Value> {
    let keys = node_def(&node.kind)
        .map(|d| d.secret_params)
        .unwrap_or_default();
    keys.iter()
        .filter_map(|key| {
            let name = secret_name(params.get(*key).and_then(Value::as_str).unwrap_or(""));
            sign_in_stamp(project_dir, &name)
                .map(|stamp| (format!("secret:{key}"), Value::String(stamp)))
        })
        .collect()
}

pub struct KeyInputs<'a> {
    pub workflow: &'a Workflow,
    pub vars: &'a Vars,
    pub project_dir: &'a Path,
    pub app_version: &'a str,
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
                    inputs.app_version,
                ),
            );
            continue;
        }
        let def = node_def(&node.kind);
        let params = interpolated_params(def, &node.params, inputs.vars);
        let mut files: Map<String, Value> =
            file_stamps(inputs.project_dir, &Value::Object(params.clone()))
                .into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect();
        files.extend(secret_stamps(node, &params, inputs.project_dir));
        let keyed = match def {
            Some(d) if d.phase == Phase::Render => json!({
                "params": params,
                "settings": wf.settings,
                "mix": wf.mix,
                "picture": PICTURE_VERSION,
                "vars": inputs.vars,
                "files": files,
            }),
            Some(d) if d.uses_project_size => json!({
                "params": params,
                "files": files,
                "size": { "width": wf.settings.width, "height": wf.settings.height, "fps": wf.settings.fps },
            }),
            _ if !files.is_empty() => json!({ "params": params, "files": files }),
            _ => Value::Object(params),
        };
        hashes.insert(
            node.id.clone(),
            node_hash(&node.kind, &keyed, &upstream, inputs.app_version),
        );
    }
    hashes
}
