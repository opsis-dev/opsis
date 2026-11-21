use std::io::Read;

use opsis_compile::catalog::editor_bundle;
use opsis_compile::compile::{compile_workflow, picked_film};
use opsis_compile::node::Platform;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_types::workflow::Workflow;
use serde_json::{Map, Value, json};

use crate::error::ToolError;
use crate::output;

pub fn catalog() -> Result<(), ToolError> {
    output::json(&editor_bundle(Platform::Linux))
}

fn workflow_on_input() -> Result<Workflow, ToolError> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(ToolError::Input)?;
    serde_json::from_str(&text).map_err(ToolError::NotAWorkflow)
}

pub fn film() -> Result<(), ToolError> {
    let workflow = workflow_on_input()?;
    let compiled = compile_workflow(&workflow, None);
    let picked = picked_film(&workflow, &compiled, None);
    let flat = picked.as_ref().map(|p| flatten_spec(&p.spec));
    let local_frames: Map<String, Value> = flat
        .iter()
        .flat_map(|f| f.bars.iter())
        .map(|bar| (bar.key.clone(), json!(bar.local_frames())))
        .collect();
    output::json(
        &json!({ "spec": picked.map(|p| p.spec), "flat": flat, "localFrames": local_frames }),
    )
}
