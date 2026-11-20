#![cfg(unix)]

use std::path::Path;
use std::time::Duration;

use axum::Router;
use axum::extract::Json;
use axum::routing::{get, post};
use opsis_mcp::local_api::LocalApi;
use opsis_mcp::server::GUIDE;
use opsis_mcp::tools::{self, REDO, RUN_LUA, UNDO, WAIT_RUN};
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, JsonObject};
use rmcp::transport::TokioChildProcess;
use serde_json::{Value, json};

async fn script(Json(body): Json<Value>) -> Json<Value> {
    let source = body["source"].as_str().unwrap_or_default();
    if source.contains("error") {
        Json(
            json!({ "ok": false, "failure": { "message": "request.lua:1: boom" }, "rolledBack": true, "problems": [] }),
        )
    } else {
        Json(
            json!({ "ok": true, "value": source.len(), "output": [], "rolledBack": false, "problems": [] }),
        )
    }
}

async fn edited() -> Json<Value> {
    Json(json!({ "revision": 7, "problems": [] }))
}

async fn run_state() -> Json<Value> {
    Json(json!({ "running": true, "runId": "r1" }))
}

async fn run_events() -> String {
    [
        ": following run events\n\n",
        "data: {\"type\":\"run:done\",\"runId\":\"other\",\"ok\":true}\n\n",
        "data: {\"type\":\"run:done\",\"runId\":\"r1\",\"ok\":true,\"cancelled\":false,\"outputs\":[\".opsis/out/film.mp4\"]}\n\n",
    ]
    .concat()
}

fn serve_fake(socket: &Path) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(socket).unwrap();
    let router = Router::new()
        .route("/project/script", post(script))
        .route("/project/undo", post(edited))
        .route("/project/redo", post(edited))
        .route("/project/run", get(run_state))
        .route("/project/run/events", get(run_events));
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    })
}

fn arguments(value: Value) -> JsonObject {
    match value {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    }
}

fn text(result: &CallToolResult) -> String {
    result.content[0].as_text().expect("text").text.clone()
}

fn answer(result: &CallToolResult) -> Value {
    serde_json::from_str(&text(result)).unwrap()
}

#[tokio::test]
async fn each_tool_asks_the_local_api_and_returns_its_answer() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("api.sock");
    let _server = serve_fake(&socket);
    let api = LocalApi::new(socket.to_string_lossy().into_owned(), None);

    let ran = tools::call(&api, RUN_LUA, &arguments(json!({ "source": "return 1" }))).await;
    assert_eq!(ran.is_error, Some(false));
    assert_eq!(answer(&ran)["value"], 8);

    let failed = tools::call(&api, RUN_LUA, &arguments(json!({ "source": "error()" }))).await;
    assert_eq!(failed.is_error, Some(true));
    assert_eq!(answer(&failed)["rolledBack"], true);

    let missing = tools::call(&api, RUN_LUA, &JsonObject::new()).await;
    assert_eq!(missing.is_error, Some(true));
    assert_eq!(text(&missing), "run_lua needs source, a string.");

    for name in [UNDO, REDO] {
        let result = tools::call(&api, name, &JsonObject::new()).await;
        assert_eq!(answer(&result)["revision"], 7, "{name}");
    }

    let waited = tools::call(
        &api,
        WAIT_RUN,
        &arguments(json!({ "id": "r1", "seconds": 5 })),
    )
    .await;
    assert_eq!(answer(&waited)["outputs"][0], ".opsis/out/film.mp4");

    let gone = tools::call(&api, WAIT_RUN, &arguments(json!({ "id": "r2" }))).await;
    assert_eq!(answer(&gone)["running"], false);

    let unknown = tools::call(&api, "export", &JsonObject::new()).await;
    assert_eq!(text(&unknown), "There is no tool export.");
}

#[tokio::test]
async fn every_call_says_so_plainly_when_opsis_isnt_running() {
    let dir = tempfile::tempdir().unwrap();
    let api = LocalApi::new(
        dir.path().join("api.sock").to_string_lossy().into_owned(),
        None,
    );
    for name in [RUN_LUA, UNDO, REDO, WAIT_RUN] {
        let result = tools::call(
            &api,
            name,
            &arguments(json!({ "source": "return 1", "id": "r1" })),
        )
        .await;
        assert_eq!(result.is_error, Some(true), "{name}");
        assert_eq!(
            text(&result),
            "Opsis isn't running, or the assistant is off.",
            "{name}"
        );
    }
}

#[tokio::test]
async fn the_program_serves_the_tools_and_the_guide_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("api.sock");
    let _server = serve_fake(&socket);
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_opsis-mcp"));
    command.arg("stdio").env("OPSIS_API_SOCKET", &socket);
    let client = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
    let info = client.peer_info().unwrap();
    assert_eq!(info.instructions.as_deref(), Some(GUIDE));
    let mut names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|tool| tool.name.into_owned())
        .collect();
    names.sort();
    assert_eq!(names, [REDO, RUN_LUA, UNDO, WAIT_RUN]);
    let ran = tokio::time::timeout(
        Duration::from_secs(10),
        client.call_tool(
            CallToolRequestParams::new(RUN_LUA)
                .with_arguments(arguments(json!({ "source": "return 2" }))),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(answer(&ran)["ok"], true);
    client.cancel().await.unwrap();
}
