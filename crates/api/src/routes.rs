use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path as UrlPath, State};
use axum::http::{Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures::StreamExt;
use futures::future::BoxFuture;
use opsis_compile::catalog::{catalog_entry, node_catalog};
use opsis_compile::check::{ApiProblem, check_workflow, workflow_problems};
use opsis_compile::graph::{IssueLevel, connection_problem};
use opsis_compile::registry::node_def;
use opsis_types::workflow::{WORKFLOW_FILE, Workflow, WorkflowSettings};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use tokio_stream::wrappers::BroadcastStream;

use crate::bodies::{
    AddEdgeBody, AddNodeBody, EditorRequest, OpenBody, RunBody, UpdateNodeBody, VariablesBody,
    parse_body,
};
use crate::openapi::{OpenProjectName, ProjectState, RunState, Status, openapi_document};

pub const MAX_BODY: usize = 20 * 1024 * 1024;

pub trait ApiHost: Send + Sync + 'static {
    fn version(&self) -> String;
    fn ask_editor(&self, request: Value) -> BoxFuture<'static, Value>;
    fn run_events(&self) -> tokio::sync::broadcast::Receiver<String>;
}

type Host = Arc<dyn ApiHost>;

fn json_reply(status: StatusCode, payload: &impl Serialize) -> Response {
    let text = serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_owned());
    (
        status,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        text,
    )
        .into_response()
}

fn failure(status: StatusCode, error: &str) -> Response {
    json_reply(status, &json!({ "error": error }))
}

fn editor_reply(reply: &Value) -> Response {
    if reply["ok"] == json!(true) {
        return json_reply(StatusCode::OK, &reply["result"]);
    }
    let status = reply["status"]
        .as_u64()
        .and_then(|s| u16::try_from(s).ok())
        .and_then(|s| StatusCode::from_u16(s).ok())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut payload = Map::new();
    payload.insert("error".into(), reply["error"].clone());
    if let Some(problems) = reply.get("problems") {
        payload.insert("problems".into(), problems.clone());
    }
    json_reply(status, &payload)
}

fn read_json(bytes: &Bytes, required: bool) -> Result<Value, Box<Response>> {
    let Ok(text) = std::str::from_utf8(bytes).map(str::trim) else {
        return Err(Box::new(failure(
            StatusCode::BAD_REQUEST,
            "The request body isn't UTF-8 text.",
        )));
    };
    if text.is_empty() {
        return if required {
            Err(Box::new(failure(
                StatusCode::BAD_REQUEST,
                "This request needs a JSON body.",
            )))
        } else {
            Ok(json!({}))
        };
    }
    serde_json::from_str(text).map_err(|_| {
        Box::new(failure(
            StatusCode::BAD_REQUEST,
            "The request body isn't valid JSON.",
        ))
    })
}

fn body_of<T: DeserializeOwned>(bytes: &Bytes, required: bool) -> Result<T, Box<Response>> {
    parse_body(&read_json(bytes, required)?)
        .map_err(|e| Box::new(failure(StatusCode::BAD_REQUEST, &e)))
}

fn problems_in(workflow: &Value) -> Option<Vec<ApiProblem>> {
    let workflow: Workflow = serde_json::from_value(workflow.clone()).ok()?;
    Some(
        workflow_problems(&workflow)
            .into_iter()
            .map(|issue| ApiProblem { issue, path: None })
            .collect(),
    )
}

fn with_problems(mut reply: Value, check: bool) -> Value {
    let Some(result) = reply.get_mut("result").and_then(Value::as_object_mut) else {
        return reply;
    };
    let Some(problems) = result.remove("workflow").as_ref().and_then(problems_in) else {
        return reply;
    };
    if check {
        let ok = !problems.iter().any(|p| p.issue.level == IssueLevel::Error);
        result.insert("ok".into(), json!(ok));
    }
    result.insert("problems".into(), json!(problems));
    reply
}

async fn ask(host: &Host, request: EditorRequest) -> Response {
    let check = matches!(request, EditorRequest::Problems);
    let keeps_workflow = matches!(request, EditorRequest::Open { .. } | EditorRequest::Read);
    match serde_json::to_value(&request) {
        Ok(request) => {
            let reply = host.ask_editor(request).await;
            editor_reply(&if keeps_workflow {
                reply
            } else {
                with_problems(reply, check)
            })
        }
        Err(error) => failure(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
    }
}

async fn read_project(host: &Host) -> Result<ProjectState, Box<Response>> {
    let reply = host.ask_editor(json!({ "op": "read" })).await;
    if reply["ok"] != json!(true) {
        return Err(Box::new(editor_reply(&reply)));
    }
    serde_json::from_value(reply["result"].clone()).map_err(|_| {
        Box::new(failure(
            StatusCode::BAD_GATEWAY,
            "The editor's answer couldn't be read.",
        ))
    })
}

async fn openapi(State(host): State<Host>) -> Response {
    json_reply(StatusCode::OK, &openapi_document(&host.version()))
}

async fn status(State(host): State<Host>) -> Response {
    let project = read_project(&host).await.ok();
    let answer = Status {
        app: "Opsis".to_owned(),
        version: host.version(),
        project: project.as_ref().and_then(|p| {
            p.dir.clone().map(|dir| OpenProjectName {
                dir,
                name: p.workflow.name.clone(),
            })
        }),
        running: project.is_some_and(|p| p.running),
    };
    json_reply(StatusCode::OK, &answer)
}

async fn nodes() -> Response {
    json_reply(StatusCode::OK, &node_catalog())
}

async fn node(UrlPath(kind): UrlPath<String>) -> Response {
    match catalog_entry(&kind) {
        Some(entry) => json_reply(StatusCode::OK, &entry),
        None => failure(StatusCode::NOT_FOUND, "No such node type."),
    }
}

async fn validate(bytes: Bytes) -> Response {
    let body = match read_json(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let checked = check_workflow(&body);
    json_reply(
        StatusCode::OK,
        &json!({ "ok": checked.ok, "problems": checked.problems }),
    )
}

async fn open(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match read_json(&bytes, true).and_then(|b| {
        parse_body::<OpenBody>(&b).map_err(|e| Box::new(failure(StatusCode::BAD_REQUEST, &e)))
    }) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let dir = Path::new(&body.dir);
    if !dir.is_absolute() {
        return failure(
            StatusCode::BAD_REQUEST,
            "dir must be an absolute folder path.",
        );
    }
    if !dir.join(WORKFLOW_FILE).exists() {
        return failure(
            StatusCode::NOT_FOUND,
            &format!("There's no Opsis project in {}.", body.dir),
        );
    }
    ask(&host, EditorRequest::Open { dir: body.dir }).await
}

async fn project(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Read).await
}

async fn replace(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match read_json(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let checked = check_workflow(&body);
    match checked.workflow {
        Some(workflow) => {
            ask(
                &host,
                EditorRequest::Replace {
                    workflow: Box::new(workflow),
                },
            )
            .await
        }
        None => json_reply(
            StatusCode::BAD_REQUEST,
            &json!({ "error": "That isn't a valid project.", "problems": checked.problems }),
        ),
    }
}

async fn add_node(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match body_of::<AddNodeBody>(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    if node_def(&body.kind).is_none() {
        return failure(
            StatusCode::BAD_REQUEST,
            &format!("There's no node type “{}”. See GET /nodes.", body.kind),
        );
    }
    ask(&host, EditorRequest::AddNode(body)).await
}

async fn update_node(
    State(host): State<Host>,
    UrlPath(id): UrlPath<String>,
    bytes: Bytes,
) -> Response {
    match body_of::<UpdateNodeBody>(&bytes, true) {
        Ok(patch) => ask(&host, EditorRequest::UpdateNode { id, patch }).await,
        Err(response) => *response,
    }
}

async fn remove_node(State(host): State<Host>, UrlPath(id): UrlPath<String>) -> Response {
    ask(&host, EditorRequest::RemoveNodes { ids: vec![id] }).await
}

async fn add_edge(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match body_of::<AddEdgeBody>(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let project = match read_project(&host).await {
        Ok(project) => project,
        Err(response) => return *response,
    };
    if let Some(problem) = connection_problem(
        &project.workflow,
        &body.source,
        &body.source_handle,
        &body.target,
        &body.target_handle,
    ) {
        let status = if problem.not_found {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        return failure(status, &problem.message);
    }
    ask(&host, EditorRequest::AddEdge(body)).await
}

async fn remove_edge(State(host): State<Host>, UrlPath(id): UrlPath<String>) -> Response {
    ask(&host, EditorRequest::RemoveEdges { ids: vec![id] }).await
}

fn settings_problem(body: &Map<String, Value>) -> Option<String> {
    let Ok(Value::Object(mut whole)) = serde_json::to_value(WorkflowSettings::default()) else {
        return Some("The project settings couldn't be read.".to_owned());
    };
    if let Some(unknown) = body
        .keys()
        .find(|key| !whole.contains_key(*key) && *key != "background")
    {
        return Some(format!("{unknown}: isn't a project setting."));
    }
    whole.extend(body.clone());
    serde_path_to_error::deserialize::<_, WorkflowSettings>(Value::Object(whole))
        .err()
        .map(|e| format!("{}: {}", e.path(), e.inner()))
}

async fn settings(State(host): State<Host>, bytes: Bytes) -> Response {
    match read_json(&bytes, true) {
        Ok(Value::Object(body)) => match settings_problem(&body) {
            Some(problem) => failure(StatusCode::BAD_REQUEST, &problem),
            None => ask(&host, EditorRequest::SetSettings { settings: body }).await,
        },
        Ok(_) => failure(
            StatusCode::BAD_REQUEST,
            "The request body must be a JSON object.",
        ),
        Err(response) => *response,
    }
}

async fn variables(State(host): State<Host>, bytes: Bytes) -> Response {
    match body_of::<VariablesBody>(&bytes, true) {
        Ok(variables) => ask(&host, EditorRequest::SetVariables { variables }).await,
        Err(response) => *response,
    }
}

async fn problems(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Problems).await
}

async fn undo(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Undo).await
}

async fn redo(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Redo).await
}

async fn save(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Save).await
}

async fn run_state(State(host): State<Host>) -> Response {
    match read_project(&host).await {
        Ok(project) => json_reply(
            StatusCode::OK,
            &RunState {
                running: project.running,
                run_id: project.run_id,
            },
        ),
        Err(response) => *response,
    }
}

async fn run(State(host): State<Host>, bytes: Bytes) -> Response {
    match body_of::<RunBody>(&bytes, false) {
        Ok(body) => ask(&host, EditorRequest::Run(body)).await,
        Err(response) => *response,
    }
}

async fn stop(State(host): State<Host>) -> Response {
    ask(&host, EditorRequest::Stop).await
}

async fn run_events(State(host): State<Host>) -> Response {
    let opening = futures::stream::once(async {
        Ok::<_, std::io::Error>(Bytes::from_static(b": following run events\n\n"))
    });
    let events = BroadcastStream::new(host.run_events()).filter_map(|event| async move {
        event
            .ok()
            .map(|json| Ok(Bytes::from(format!("data: {json}\n\n"))))
    });
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(opening.chain(events)))
        .unwrap_or_else(|e| failure(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))
}

async fn not_found(method: Method, uri: Uri) -> Response {
    failure(
        StatusCode::NOT_FOUND,
        &format!(
            "No endpoint for {method} {}. See GET /openapi.json.",
            uri.path()
        ),
    )
}

pub fn routes(host: Host) -> Router {
    Router::new()
        .route("/openapi.json", get(openapi))
        .route("/status", get(status))
        .route("/nodes", get(nodes))
        .route("/nodes/{type}", get(node))
        .route("/validate", post(validate))
        .route("/open", post(open))
        .route("/project", get(project).put(replace))
        .route("/project/nodes", post(add_node))
        .route(
            "/project/nodes/{id}",
            axum::routing::patch(update_node).delete(remove_node),
        )
        .route("/project/edges", post(add_edge))
        .route("/project/edges/{id}", axum::routing::delete(remove_edge))
        .route("/project/settings", axum::routing::patch(settings))
        .route("/project/variables", axum::routing::put(variables))
        .route("/project/problems", get(problems))
        .route("/project/undo", post(undo))
        .route("/project/redo", post(redo))
        .route("/project/save", post(save))
        .route("/project/run", get(run_state).post(run))
        .route("/project/run/stop", post(stop))
        .route("/project/run/events", get(run_events))
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY))
        .with_state(host)
}
