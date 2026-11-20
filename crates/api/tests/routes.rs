#![cfg(unix)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use opsis_api::openapi::{ProjectState, RunState};
use opsis_api::routes::{ApiHost, HostError, HostResult};
use opsis_api::server::ApiServer;
use opsis_compile::check::workflow_problems;
use opsis_compile::graph::{GraphIssue, IssueLevel};
use opsis_folders::address::ApiAddress;
use opsis_types::edits::{Edited, WorkflowEdit};
use opsis_types::events::RunTargets;
use opsis_types::script::{ScriptOutcome, ScriptRequest, ScriptStatus};
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};
use tokio::sync::broadcast;

struct FakeEditor {
    ran: Mutex<Vec<ScriptRequest>>,
    asked: Mutex<Vec<WorkflowEdit>>,
    events: broadcast::Sender<String>,
    noted: Mutex<Vec<GraphIssue>>,
}

fn demo() -> ProjectState {
    ProjectState {
        running: false,
        run_id: None,
        dir: Some("/p".into()),
        workflow: serde_json::from_value(json!({ "version": 1, "id": "w", "name": "Demo", "settings": {}, "variables": {}, "nodes": [], "edges": [], "markers": [] })).unwrap(),
        revision: 3,
        saved: true,
    }
}

impl ApiHost for FakeEditor {
    fn version(&self) -> String {
        "9.9.9".into()
    }

    fn project(&self) -> HostResult<ProjectState> {
        Ok(demo())
    }

    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited> {
        let missing = matches!(&edit, WorkflowEdit::RemoveNodes { ids } if ids == &["x"]);
        self.asked.lock().unwrap().push(edit);
        if missing {
            return Err(HostError::new(404, "No node x."));
        }
        Ok(Edited {
            revision: 4,
            ids: Vec::new(),
        })
    }

    fn undo(&self) -> HostResult<u64> {
        Ok(4)
    }

    fn redo(&self) -> HostResult<u64> {
        Ok(4)
    }

    fn save(&self) -> BoxFuture<'static, HostResult<u64>> {
        Box::pin(async { Ok(3) })
    }

    fn open(&self, _dir: String) -> BoxFuture<'static, HostResult<ProjectState>> {
        Box::pin(async { Ok(demo()) })
    }

    fn run(&self, _targets: RunTargets) -> BoxFuture<'static, HostResult<RunState>> {
        Box::pin(async {
            Ok(RunState {
                running: true,
                run_id: Some("r".into()),
            })
        })
    }

    fn stop(&self) -> RunState {
        RunState {
            running: false,
            run_id: None,
        }
    }

    fn run_events(&self) -> broadcast::Receiver<String> {
        self.events.subscribe()
    }

    fn run_script(&self, request: ScriptRequest) -> BoxFuture<'static, HostResult<ScriptOutcome>> {
        self.ran.lock().unwrap().push(request);
        Box::pin(async move {
            Ok(ScriptOutcome {
                ok: false,
                value: None,
                output: Vec::new(),
                failure: Some(opsis_types::script::ScriptFailure {
                    message: "request.lua:1: boom".into(),
                    file: "request.lua".into(),
                    line: Some(1),
                }),
                stopped: false,
                rolled_back: true,
            })
        })
    }

    fn stop_script(&self) -> HostResult<ScriptStatus> {
        self.script_status()
    }

    fn script_status(&self) -> HostResult<ScriptStatus> {
        Ok(ScriptStatus {
            running: false,
            file: None,
        })
    }

    fn problems(&self, workflow: &Workflow) -> Vec<GraphIssue> {
        let mut problems = workflow_problems(workflow);
        problems.extend(self.noted.lock().unwrap().clone());
        problems
    }
}

struct Served {
    socket: String,
    editor: Arc<FakeEditor>,
    server: ApiServer,
    _dir: tempfile::TempDir,
}

async fn serve() -> Served {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("api.sock").to_string_lossy().into_owned();
    let editor = Arc::new(FakeEditor {
        ran: Mutex::default(),
        asked: Mutex::default(),
        events: broadcast::Sender::new(8),
        noted: Mutex::default(),
    });
    let server = ApiServer::new(ApiAddress {
        address: socket.clone(),
        error: None,
    });
    let status = server.set_enabled(true, editor.clone()).await;
    assert!(status.listening, "{status:?}");
    Served {
        socket,
        editor,
        server,
        _dir: dir,
    }
}

async fn call(socket: &str, method: &str, path: &str, body: Option<&str>) -> (u16, Value) {
    let stream = tokio::net::UnixStream::connect(socket).await.unwrap();
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
        .await
        .unwrap();
    tokio::spawn(connection);
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .body(Full::new(Bytes::from(body.unwrap_or_default().to_owned())))
        .unwrap();
    let response = sender.send_request(request).await.unwrap();
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn answers_status_nodes_and_the_document_without_an_editor_edit() {
    let served = serve().await;
    let (code, project) = call(&served.socket, "GET", "/project", None).await;
    assert_eq!((code, &project["workflow"]["name"]), (200, &json!("Demo")));
    let (code, status) = call(&served.socket, "GET", "/status", None).await;
    assert_eq!(
        (code, &status["version"], &status["project"]["name"]),
        (200, &json!("9.9.9"), &json!("Demo"))
    );
    let (code, nodes) = call(&served.socket, "GET", "/nodes", None).await;
    assert_eq!(code, 200);
    assert!(
        nodes
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["type"] == "browser.click")
    );
    let (code, click) = call(&served.socket, "GET", "/nodes/browser.click", None).await;
    assert_eq!(
        (code, click["params"]["type"].as_str()),
        (200, Some("object"))
    );
    let (code, missing) = call(&served.socket, "GET", "/nodes/nope", None).await;
    assert_eq!(
        (code, missing["error"].as_str()),
        (404, Some("No such node type."))
    );
    let (code, doc) = call(&served.socket, "GET", "/openapi.json", None).await;
    assert_eq!(
        (code, doc["info"]["version"].as_str()),
        (200, Some("9.9.9"))
    );
    let text = serde_json::to_string(&doc).unwrap();
    assert!(text.len() < 100_000, "the document is {} bytes", text.len());
    assert!(!text.contains("#core"));
    served.server.stop();
}

#[tokio::test]
async fn documents_every_route_it_serves() {
    let served = serve().await;
    let (_, doc) = call(&served.socket, "GET", "/openapi.json", None).await;
    let paths = doc["paths"].as_object().unwrap();
    for (method, path) in [
        ("get", "/openapi.json"),
        ("get", "/status"),
        ("get", "/nodes"),
        ("get", "/nodes/{type}"),
        ("get", "/project"),
        ("put", "/project"),
        ("post", "/project/nodes"),
        ("patch", "/project/nodes/{id}"),
        ("delete", "/project/nodes/{id}"),
        ("post", "/project/edges"),
        ("delete", "/project/edges/{id}"),
        ("patch", "/project/settings"),
        ("put", "/project/variables"),
        ("get", "/project/problems"),
        ("post", "/project/undo"),
        ("post", "/project/redo"),
        ("post", "/project/save"),
        ("get", "/project/run"),
        ("post", "/project/run"),
        ("post", "/project/run/stop"),
        ("get", "/project/run/events"),
        ("get", "/project/script"),
        ("post", "/project/script"),
        ("post", "/project/script/stop"),
        ("post", "/open"),
        ("post", "/validate"),
    ] {
        assert!(
            paths[path].get(method).is_some(),
            "{method} {path} isn't documented"
        );
    }
    let schemas = doc["components"]["schemas"].as_object().unwrap();
    for name in [
        "Workflow",
        "ProjectSettings",
        "Problem",
        "Check",
        "Edited",
        "Project",
        "RunStatus",
        "Status",
        "Port",
        "NodeType",
        "Error",
        "ScriptAnswer",
        "ScriptStatus",
    ] {
        assert!(schemas.contains_key(name), "{name} is missing");
    }
    served.server.stop();
}

#[tokio::test]
async fn runs_scripts_and_answers_their_outcome() {
    let served = serve().await;
    let (status, _) = call(&served.socket, "GET", "/project/script", None).await;
    assert_eq!(status, 200);
    for body in [
        "{}",
        r#"{"source":"x","file":"scripts/a.lua"}"#,
        r#"{"code":"x"}"#,
    ] {
        let (status, _) = call(&served.socket, "POST", "/project/script", Some(body)).await;
        assert_eq!(status, 400, "{body}");
    }
    let (status, answer) = call(
        &served.socket,
        "POST",
        "/project/script",
        Some(r#"{"file":"scripts/a.lua"}"#),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["rolledBack"], true);
    assert_eq!(answer["failure"]["line"], 1);
    assert!(answer["problems"].is_array());
    assert_eq!(
        served
            .editor
            .ran
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .file
            .as_deref(),
        Some("scripts/a.lua")
    );
    let (status, answer) = call(&served.socket, "POST", "/project/script/stop", None).await;
    assert_eq!(
        (status, answer["running"].clone()),
        (200, Value::Bool(false))
    );
    served.server.stop();
}

#[tokio::test]
async fn sends_edits_to_the_document_with_their_bodies_checked() {
    let served = serve().await;
    let (code, error) = call(&served.socket, "POST", "/project/nodes", Some("{}")).await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("type: is required."))
    );
    let (code, error) = call(
        &served.socket,
        "POST",
        "/project/nodes",
        Some(r#"{ "type": "t", "position": { "x": "a", "y": 1 } }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("position.x: must be a number."))
    );
    let (code, error) = call(
        &served.socket,
        "POST",
        "/project/edges",
        Some(r#"{ "target": "a" }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("source: is required."))
    );
    let (code, edited) = call(
        &served.socket,
        "POST",
        "/project/nodes",
        Some(r#"{ "type": "browser.click", "params": { "target": "Save" } }"#),
    )
    .await;
    assert_eq!(
        (code, &edited["revision"], &edited["problems"]),
        (200, &json!(4), &json!([]))
    );
    assert!(edited.get("workflow").is_none());
    let (code, error) = call(
        &served.socket,
        "POST",
        "/project/nodes",
        Some(r#"{ "type": "browser.nothing" }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (
            400,
            Some("There's no node type “browser.nothing”. See GET /nodes.")
        )
    );
    let (code, error) = call(
        &served.socket,
        "POST",
        "/project/edges",
        Some(r#"{ "source": "a", "sourceHandle": "out", "target": "b", "targetHandle": "in" }"#),
    )
    .await;
    assert_eq!(code, 404, "{error}");
    let (code, error) = call(
        &served.socket,
        "PATCH",
        "/project/settings",
        Some(r#"{ "fps": "fast" }"#),
    )
    .await;
    assert_eq!(code, 400, "{error}");
    assert!(error["error"].as_str().unwrap().starts_with("fps: "));
    let (code, error) = call(
        &served.socket,
        "PATCH",
        "/project/settings",
        Some(r#"{ "colour": "red" }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("colour: isn't a project setting."))
    );
    let (code, check) = call(&served.socket, "GET", "/project/problems", None).await;
    assert_eq!(
        (code, &check["ok"], &check["problems"]),
        (200, &json!(true), &json!([]))
    );
    let (code, error) = call(&served.socket, "DELETE", "/project/nodes/x", None).await;
    assert_eq!((code, error["error"].as_str()), (404, Some("No node x.")));
    let (code, run) = call(&served.socket, "GET", "/project/run", None).await;
    assert_eq!(
        (code, run),
        (200, json!({ "running": false, "runId": null }))
    );
    let asked = served.editor.asked.lock().unwrap().clone();
    assert!(asked.iter().any(|edit| matches!(
        edit,
        WorkflowEdit::AddNode { kind, params: Some(params), .. }
            if kind == "browser.click" && params["target"] == "Save"
    )));
    assert!(
        asked
            .iter()
            .any(|edit| matches!(edit, WorkflowEdit::RemoveNodes { ids } if ids == &["x"]))
    );
    served.server.stop();
}

#[tokio::test]
async fn checks_projects_and_explains_bad_requests() {
    let served = serve().await;
    let (code, check) = call(
        &served.socket,
        "POST",
        "/validate",
        Some(r#"{ "nodes": 3 }"#),
    )
    .await;
    assert_eq!((code, &check["ok"]), (200, &json!(false)));
    let (code, error) = call(&served.socket, "POST", "/validate", Some("{")).await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("The request body isn't valid JSON."))
    );
    let (code, error) = call(&served.socket, "PUT", "/project", None).await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("This request needs a JSON body."))
    );
    let (code, error) = call(
        &served.socket,
        "POST",
        "/open",
        Some(r#"{ "dir": "relative" }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (400, Some("dir must be an absolute folder path."))
    );
    let (code, error) = call(
        &served.socket,
        "POST",
        "/open",
        Some(r#"{ "dir": "/nowhere" }"#),
    )
    .await;
    assert_eq!(
        (code, error["error"].as_str()),
        (404, Some("There's no Opsis project in /nowhere."))
    );
    let (code, error) = call(&served.socket, "GET", "/nothing", None).await;
    assert_eq!(
        (code, error["error"].as_str()),
        (
            404,
            Some("No endpoint for GET /nothing. See GET /openapi.json.")
        )
    );
    served.server.stop();
}

#[tokio::test]
async fn keeps_the_socket_to_its_owner_and_leaves_a_live_one_alone() {
    use std::os::unix::fs::PermissionsExt;
    let served = serve().await;
    let mode = std::fs::metadata(&served.socket)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    let second = ApiServer::new(ApiAddress {
        address: served.socket.clone(),
        error: None,
    });
    let status = second.set_enabled(true, served.editor.clone()).await;
    assert_eq!(
        status.error.as_deref(),
        Some("Another copy of Opsis is already using this address.")
    );
    served.server.stop();
}

#[tokio::test]
async fn never_deletes_a_file_that_isnt_a_socket() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("api.sock");
    std::fs::write(&file, "mine").unwrap();
    let editor = Arc::new(FakeEditor {
        ran: Mutex::default(),
        asked: Mutex::default(),
        events: broadcast::Sender::new(8),
        noted: Mutex::default(),
    });
    let server = ApiServer::new(ApiAddress {
        address: file.to_string_lossy().into_owned(),
        error: None,
    });
    let status = server.set_enabled(true, editor).await;
    assert!(status.error.unwrap().contains("isn't Opsis's API"));
    assert_eq!(std::fs::read_to_string(Path::new(&file)).unwrap(), "mine");
}

#[tokio::test]
async fn answers_with_the_problems_the_app_knows_from_runs_and_file_checks() {
    let served = serve().await;
    served.editor.noted.lock().unwrap().push(GraphIssue {
        level: IssueLevel::Warning,
        message: "This file can't be read as a video, an image or a sound.".into(),
        node_id: Some("clip".into()),
        edge_id: None,
    });
    let (code, check) = call(&served.socket, "GET", "/project/problems", None).await;
    assert_eq!(
        (code, &check["ok"], &check["problems"]),
        (
            200,
            &json!(true),
            &json!([{ "level": "warning", "message": "This file can't be read as a video, an image or a sound.", "nodeId": "clip" }])
        )
    );
    let (_, edited) = call(&served.socket, "POST", "/project/undo", None).await;
    assert_eq!(edited["problems"][0]["nodeId"], "clip", "{edited}");
}
