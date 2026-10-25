use std::path::PathBuf;

use opsis_compile::mix::plan::build_mix_plan;
use opsis_compile::timeline::chapters::film_chapters;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_types::json::first_difference;
use opsis_types::timeline::TimelineSpec;
use serde_json::Value;

fn workflow_fixtures() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/compile/workflows");
    let mut out: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .expect("fixtures exist")
        .map(|entry| {
            let path = entry.expect("entry").path();
            let text = std::fs::read_to_string(&path).expect("readable");
            (
                path.file_stem().unwrap().to_string_lossy().into_owned(),
                serde_json::from_str(&text).expect("json"),
            )
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn flattened_specs_match() {
    let mut problems = Vec::new();
    let mut checked = 0;
    for (name, fixture) in workflow_fixtures() {
        let mix: Option<opsis_types::mix::Mix> =
            serde_json::from_value(fixture["workflow"]["mix"].clone()).unwrap();
        for side in [&fixture, &fixture["run"]] {
            for (id, spec) in side["compile"]["specs"].as_object().unwrap() {
                let spec: TimelineSpec = serde_json::from_value(spec.clone()).unwrap();
                let flattened = flatten_spec(&spec);
                let flat = serde_json::to_value(&flattened).unwrap();
                let mut plan =
                    serde_json::to_value(build_mix_plan(&spec, mix.as_ref(), &flattened)).unwrap();
                let mut wanted_plan = side["timelines"][id]["mix"].clone();
                plan.as_object_mut().unwrap().remove("key");
                wanted_plan.as_object_mut().unwrap().remove("key");
                if let Some(d) = first_difference(&wanted_plan, &plan) {
                    problems.push(format!("{name} {id} mix{d}"));
                }
                checked += 1;
                if let Some(d) = first_difference(&side["timelines"][id]["flat"], &flat) {
                    problems.push(format!("{name} {id}{d}"));
                }
                let chapters = serde_json::to_value(film_chapters(&spec)).unwrap();
                if let Some(d) = first_difference(&side["timelines"][id]["chapters"], &chapters) {
                    problems.push(format!("{name} {id} chapters{d}"));
                }
            }
        }
    }
    println!("{checked} specs flattened");
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
