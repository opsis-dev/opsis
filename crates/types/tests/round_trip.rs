use std::path::PathBuf;

use opsis_types::json::first_difference;
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::{CompileValue, NodeResult};
use opsis_types::workflow::Workflow;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn workflow_fixtures() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/compile/workflows");
    let mut out: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .expect("fixtures/compile/workflows exists")
        .map(|entry| {
            let path = entry.expect("entry").path();
            let text = std::fs::read_to_string(&path).expect("readable fixture");
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                serde_json::from_str(&text).expect("json"),
            )
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn round_trip<T: DeserializeOwned + Serialize>(what: &str, value: &Value) -> Vec<String> {
    match serde_json::from_value::<T>(value.clone()) {
        Err(error) => vec![format!("{what}: {error}")],
        Ok(parsed) => {
            let back = serde_json::to_value(&parsed).expect("serializes");
            first_difference(value, &back)
                .map(|d| format!("{what}{d}"))
                .into_iter()
                .collect()
        }
    }
}

#[test]
fn fixture_values_round_trip() {
    let mut problems = Vec::new();
    for (name, fixture) in workflow_fixtures() {
        problems.extend(round_trip::<Workflow>(
            &format!("{name} workflow"),
            &fixture["workflow"],
        ));
        for compile in [&fixture["compile"], &fixture["run"]["compile"]] {
            for (id, spec) in compile["specs"].as_object().unwrap() {
                problems.extend(round_trip::<TimelineSpec>(
                    &format!("{name} spec {id}"),
                    spec,
                ));
            }
            for (id, ports) in compile["values"].as_object().unwrap() {
                for (port, value) in ports.as_object().unwrap() {
                    problems.extend(round_trip::<CompileValue>(
                        &format!("{name} value {id}.{port}"),
                        value,
                    ));
                }
            }
        }
        for (id, result) in fixture["run"]["results"].as_object().unwrap() {
            problems.extend(round_trip::<NodeResult>(
                &format!("{name} result {id}"),
                result,
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
