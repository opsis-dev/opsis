use std::time::Duration;

use hyper::Method;
use rmcp::model::{CallToolResult, ContentBlock, JsonObject, Tool};
use serde_json::{Value, json};

use crate::local_api::{Answer, LocalApi, LocalApiError};

pub const RUN_LUA: &str = "run_lua";
pub const UNDO: &str = "undo";
pub const REDO: &str = "redo";
pub const WAIT_RUN: &str = "wait_run";
pub const DEFAULT_WAIT_SECONDS: u64 = 60;
pub const MAX_WAIT_SECONDS: u64 = 600;

fn object(schema: Value) -> JsonObject {
    match schema {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    }
}

fn no_arguments() -> JsonObject {
    object(json!({ "type": "object", "properties": {} }))
}

pub fn assistant_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            RUN_LUA,
            "Run a Lua script on the project open in Opsis and answer when it ends: whether it ended well, the value it returned, its printed lines, its failure and the editor's problems. The script gets the opsis module and reaches only the project folder. Its changes to the project are one undo step, undone by themselves when it fails.",
            object(json!({
                "type": "object",
                "properties": { "source": { "type": "string", "description": "The Lua to run." } },
                "required": ["source"],
            })),
        ),
        Tool::new(UNDO, "Undo the project's last change.", no_arguments()),
        Tool::new(REDO, "Redo the change undo took back.", no_arguments()),
        Tool::new(
            WAIT_RUN,
            "Wait for a run to end, for at most the given seconds, and answer how it ended: whether it went well, whether it was cancelled, its error, the nodes that failed or were skipped, and the files it made. Start runs from Lua with opsis.run.start() and return its id.",
            object(json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The run's id." },
                    "seconds": { "type": "integer", "minimum": 1, "maximum": MAX_WAIT_SECONDS, "default": DEFAULT_WAIT_SECONDS },
                },
                "required": ["id"],
            })),
        ),
    ]
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error(transparent)]
    LocalApi(#[from] LocalApiError),
    #[error("{0}")]
    Refused(String),
}

pub fn answered(answer: Answer) -> Result<Value, ToolError> {
    if answer.ok() {
        return Ok(answer.body);
    }
    let error = answer
        .error()
        .map_or_else(|| answer.body.to_string(), str::to_owned);
    Err(ToolError::Refused(error))
}

fn text_argument<'a>(
    arguments: &'a JsonObject,
    name: &str,
    tool: &str,
) -> Result<&'a str, ToolError> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::Refused(format!("{tool} needs {name}, a string.")))
}

pub fn wait_seconds(arguments: &JsonObject) -> u64 {
    arguments
        .get("seconds")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_WAIT_SECONDS)
        .clamp(1, MAX_WAIT_SECONDS)
}

async fn run_lua(api: &LocalApi, arguments: &JsonObject) -> Result<Value, ToolError> {
    let source = text_argument(arguments, "source", RUN_LUA)?;
    let body = json!({ "source": source });
    answered(
        api.call(Method::POST, "/project/script", Some(&body))
            .await?,
    )
}

async fn post(api: &LocalApi, path: &str) -> Result<Value, ToolError> {
    answered(api.call(Method::POST, path, None).await?)
}

fn ends(event: &Value, id: &str) -> bool {
    event.get("type").and_then(Value::as_str) == Some("run:done")
        && event.get("runId").and_then(Value::as_str) == Some(id)
}

async fn wait_run(api: &LocalApi, arguments: &JsonObject) -> Result<Value, ToolError> {
    let id = text_argument(arguments, "id", WAIT_RUN)?;
    let seconds = wait_seconds(arguments);
    let mut events = api.follow("/project/run/events").await?;
    let state = answered(api.call(Method::GET, "/project/run", None).await?)?;
    let current = state.get("runId").and_then(Value::as_str) == Some(id)
        && state.get("running").and_then(Value::as_bool) == Some(true);
    if !current {
        return Ok(json!({
            "runId": id,
            "running": false,
            "note": "This run isn't going. It ended before waiting started, or the id is wrong; read its results with opsis.run.results().",
        }));
    }
    let waited = tokio::time::timeout(Duration::from_secs(seconds), async {
        while let Some(event) = events.next().await {
            if ends(&event, id) {
                return Some(event);
            }
        }
        None
    })
    .await;
    match waited {
        Ok(Some(done)) => Ok(done),
        Ok(None) => Err(ToolError::LocalApi(LocalApiError::Unreachable)),
        Err(_) => Ok(json!({ "runId": id, "running": true, "waitedSeconds": seconds })),
    }
}

pub fn shown(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn script_failed(name: &str, value: &Value) -> bool {
    name == RUN_LUA && value.get("ok").and_then(Value::as_bool) == Some(false)
}

pub fn failure(error: &ToolError) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(error.to_string())])
}

pub async fn call(api: &LocalApi, name: &str, arguments: &JsonObject) -> CallToolResult {
    let outcome = match name {
        RUN_LUA => run_lua(api, arguments).await,
        UNDO => post(api, "/project/undo").await,
        REDO => post(api, "/project/redo").await,
        WAIT_RUN => wait_run(api, arguments).await,
        _ => Err(ToolError::Refused(format!("There is no tool {name}."))),
    };
    match outcome {
        Ok(value) if script_failed(name, &value) => {
            CallToolResult::error(vec![ContentBlock::text(shown(&value))])
        }
        Ok(value) => CallToolResult::success(vec![ContentBlock::text(shown(&value))]),
        Err(error) => failure(&error),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_wait_stays_within_its_bounds(seconds in proptest::option::of(any::<u64>())) {
            let mut arguments = JsonObject::new();
            if let Some(seconds) = seconds {
                arguments.insert("seconds".into(), json!(seconds));
            }
            let waited = wait_seconds(&arguments);
            prop_assert!((1..=MAX_WAIT_SECONDS).contains(&waited));
        }
    }
}
