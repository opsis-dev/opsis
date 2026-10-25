use std::path::Path;

use opsis_project::files::workflow_text;
use opsis_types::workflow::Workflow;
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn saves_projects_with_the_text_the_ts_app_writes() {
    let saved = fixture("project/saved.json");
    for case in saved.as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let workflow: Workflow = serde_json::from_value(
            fixture(&format!("compile/workflows/{name}.json"))["workflow"].clone(),
        )
        .unwrap();
        let text = case["text"].as_str().unwrap();
        assert!(
            workflow_text(&workflow) == text,
            "{name} is saved differently"
        );
        let reread: Workflow = serde_json::from_str(text).unwrap();
        assert!(
            workflow_text(&reread) == text,
            "{name} changes when saved again"
        );
    }
}
