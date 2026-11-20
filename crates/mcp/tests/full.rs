use std::collections::BTreeSet;

use opsis_api::openapi::openapi_document;
use opsis_mcp::full::{DOCUMENT_PATH, route_tools, tool_name};
use serde_json::Value;

fn defined_refs(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get("$ref") {
                found.insert(target.clone());
            }
            map.values().for_each(|v| defined_refs(v, found));
        }
        Value::Array(list) => list.iter().for_each(|v| defined_refs(v, found)),
        _ => {}
    }
}

#[test]
fn every_route_answering_json_becomes_one_tool_with_a_self_contained_schema() {
    let document = openapi_document("test");
    let tools = route_tools(&document);
    let names: BTreeSet<String> = tools.iter().map(|r| r.tool.name.to_string()).collect();
    assert_eq!(names.len(), tools.len(), "two routes share a tool name");
    for (path, item) in document["paths"].as_object().unwrap() {
        for (method, op) in item.as_object().unwrap() {
            let json = op
                .pointer("/responses/200/content/application~1json")
                .is_some();
            let listed = names.contains(&tool_name(method, path));
            assert_eq!(listed, json && path != DOCUMENT_PATH, "{method} {path}");
        }
    }
    assert!(!names.contains("get_project_run_events"));
    for route in &tools {
        let schema = Value::Object(route.tool.input_schema.as_ref().clone());
        let mut refs = BTreeSet::new();
        defined_refs(&schema, &mut refs);
        for target in refs {
            let name = target
                .strip_prefix("#/$defs/")
                .unwrap_or_else(|| panic!("{} points outside: {target}", route.tool.name));
            assert!(
                schema["$defs"].get(name).is_some(),
                "{} misses {name}",
                route.tool.name
            );
        }
    }
}
