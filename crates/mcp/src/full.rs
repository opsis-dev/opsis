use std::collections::BTreeSet;

use hyper::Method;
use rmcp::model::{CallToolResult, ContentBlock, JsonObject, Tool};
use serde_json::{Map, Value, json};

use crate::local_api::LocalApi;
use crate::tools::{ToolError, answered, shown};

pub const DOCUMENT_PATH: &str = "/openapi.json";
const SCHEMAS: &str = "#/components/schemas/";
const DEFS: &str = "#/$defs/";
const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

#[derive(Debug, Clone, PartialEq)]
pub struct RouteTool {
    pub tool: Tool,
    pub method: String,
    pub path: String,
    pub params: Vec<String>,
    pub takes_body: bool,
}

pub fn tool_name(method: &str, path: &str) -> String {
    let mut name = method.to_owned();
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        name.push('_');
        name.extend(
            segment
                .trim_matches(|c| c == '{' || c == '}')
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }),
        );
    }
    name
}

fn refs_in(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get("$ref")
                && let Some(name) = target.strip_prefix(SCHEMAS)
            {
                found.insert(name.to_owned());
            }
            map.values().for_each(|v| refs_in(v, found));
        }
        Value::Array(list) => list.iter().for_each(|v| refs_in(v, found)),
        _ => {}
    }
}

fn point_at_defs(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(target)) = map.get_mut("$ref")
                && let Some(name) = target.strip_prefix(SCHEMAS)
            {
                *target = format!("{DEFS}{name}");
            }
            map.values_mut().for_each(point_at_defs);
        }
        Value::Array(list) => list.iter_mut().for_each(point_at_defs),
        _ => {}
    }
}

fn defs_for(schemas: &Map<String, Value>, roots: &Value) -> Map<String, Value> {
    let mut wanted = BTreeSet::new();
    refs_in(roots, &mut wanted);
    let mut defs = Map::new();
    let mut waiting: Vec<String> = wanted.into_iter().collect();
    while let Some(name) = waiting.pop() {
        if defs.contains_key(&name) {
            continue;
        }
        let Some(schema) = schemas.get(&name) else {
            continue;
        };
        let mut more = BTreeSet::new();
        refs_in(schema, &mut more);
        waiting.extend(more.into_iter().filter(|n| !defs.contains_key(n)));
        defs.insert(name, schema.clone());
    }
    defs
}

fn answers_json(op: &Value) -> bool {
    op.pointer("/responses/200/content/application~1json")
        .is_some()
}

fn route_tool(method: &str, path: &str, op: &Value, schemas: &Map<String, Value>) -> RouteTool {
    let params: Vec<String> = op
        .get("parameters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|p| p.get("in").and_then(Value::as_str) == Some("path"))
        .filter_map(|p| p.get("name").and_then(Value::as_str).map(str::to_owned))
        .collect();
    let mut properties = Map::new();
    let mut required: Vec<Value> = Vec::new();
    for param in &params {
        properties.insert(param.clone(), json!({ "type": "string" }));
        required.push(json!(param));
    }
    let body = op
        .pointer("/requestBody/content/application~1json/schema")
        .cloned();
    let takes_body = body.is_some();
    if let Some(body) = body {
        properties.insert("body".into(), body);
        if op.pointer("/requestBody/required").and_then(Value::as_bool) == Some(true) {
            required.push(json!("body"));
        }
    }
    let mut schema = json!({ "type": "object", "properties": properties, "required": required });
    let defs = defs_for(schemas, &schema);
    if !defs.is_empty() {
        schema["$defs"] = Value::Object(defs);
    }
    point_at_defs(&mut schema);
    let summary = op
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let description = op
        .get("description")
        .and_then(Value::as_str)
        .map_or_else(|| summary.to_owned(), |more| format!("{summary}. {more}"));
    let input = match schema {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    };
    RouteTool {
        tool: Tool::new(tool_name(method, path), description, input),
        method: method.to_owned(),
        path: path.to_owned(),
        params,
        takes_body,
    }
}

pub fn route_tools(document: &Value) -> Vec<RouteTool> {
    let empty = Map::new();
    let schemas = document
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut tools = Vec::new();
    for (path, item) in paths {
        if path == DOCUMENT_PATH {
            continue;
        }
        for method in METHODS {
            if let Some(op) = item.get(method).filter(|op| answers_json(op)) {
                tools.push(route_tool(method, path, op, schemas));
            }
        }
    }
    tools
}

pub async fn fetch_route_tools(api: &LocalApi) -> Result<Vec<RouteTool>, ToolError> {
    let document = answered(api.call(Method::GET, DOCUMENT_PATH, None).await?)?;
    Ok(route_tools(&document))
}

fn filled_path(route: &RouteTool, arguments: &JsonObject) -> Result<String, ToolError> {
    let mut path = route.path.clone();
    for param in &route.params {
        let value = arguments
            .get(param)
            .and_then(Value::as_str)
            .filter(|v| {
                !v.is_empty()
                    && v.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c))
            })
            .ok_or_else(|| {
                ToolError::Refused(format!(
                    "{} needs {param}: letters, digits, '-', '_', '.' or ':'.",
                    route.tool.name
                ))
            })?;
        path = path.replace(&format!("{{{param}}}"), value);
    }
    Ok(path)
}

async fn call_route_inner(
    api: &LocalApi,
    route: &RouteTool,
    arguments: &JsonObject,
) -> Result<Value, ToolError> {
    let path = filled_path(route, arguments)?;
    let method = Method::from_bytes(route.method.to_uppercase().as_bytes())
        .map_err(|_| ToolError::Refused(format!("{} has no method.", route.tool.name)))?;
    let body = route.takes_body.then(|| arguments.get("body")).flatten();
    answered(api.call(method, &path, body).await?)
}

pub async fn call_route(
    api: &LocalApi,
    route: &RouteTool,
    arguments: &JsonObject,
) -> CallToolResult {
    match call_route_inner(api, route, arguments).await {
        Ok(value) => CallToolResult::success(vec![ContentBlock::text(shown(&value))]),
        Err(error) => CallToolResult::error(vec![ContentBlock::text(error.to_string())]),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_tool_name_is_a_plain_identifier(method in "(get|post|put|patch|delete)", path in "(/[a-z{}.]{1,8}){1,4}") {
            let name = tool_name(&method, &path);
            prop_assert!(name.starts_with(&method));
            prop_assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }
}
