use std::io::Read;

use opsis_compile::catalog::editor_bundle;
use opsis_compile::compile::{compile_workflow, preview_spec};
use opsis_compile::node::Platform;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_types::workflow::Workflow;
use serde_json::json;

fn film() -> serde_json::Value {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .expect("a workflow on standard input");
    let workflow: Workflow = serde_json::from_str(&text).expect("a workflow");
    let compiled = compile_workflow(&workflow, None);
    let preview = preview_spec(&workflow, &compiled, None);
    let flat = preview.as_ref().map(|p| flatten_spec(&p.spec));
    let local_frames: serde_json::Map<String, serde_json::Value> = flat
        .iter()
        .flat_map(|f| f.bars.iter())
        .map(|b| (b.key.clone(), json!(b.local_frames())))
        .collect();
    json!({ "spec": preview.map(|p| p.spec), "flat": flat, "localFrames": local_frames })
}

fn main() {
    let answer = match std::env::args().nth(1).as_deref() {
        Some("catalog") => json!(editor_bundle(Platform::Linux)),
        Some("film") => film(),
        _ => panic!("say catalog or film"),
    };
    println!("{answer}");
}
