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
use opsis_compile::graph::{GraphIssue, IssueLevel, connection_problem};
use opsis_compile::registry::node_def;
use opsis_types::edits::{Edited, WorkflowEdit};
use opsis_types::events::RunTargets;
use opsis_types::workflow::{WORKFLOW_FILE, Workflow, WorkflowSettings};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use tokio_stream::wrappers::BroadcastStream;

use crate::bodies::{
    AddEdgeBody, AddNodeBody, OpenBody, UpdateNodeBody, VariablesBody, parse_body,
};
use crate::openapi::{
    Check, Edited as EditedReply, OpenProjectName, ProjectState, RunState, Status, openapi_document,
};

pub const MAX_BODY: usize = 20 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostError {
    pub status: u16,
    pub message: String,
}

impl HostError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

pub type HostResult<T> = Result<T, HostError>;

pub trait ApiHost: Send + Sync + 'static {
    fn version(&self) -> String;
    fn project(&self) -> HostResult<ProjectState>;
    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited>;
    fn undo(&self) -> HostResult<u64>;
    fn redo(&self) -> HostResult<u64>;
    fn save(&self) -> BoxFuture<'static, HostResult<u64>>;
    fn open(&self, dir: String) -> BoxFuture<'static, HostResult<ProjectState>>;
    fn run(&self, targets: RunTargets) -> BoxFuture<'static, HostResult<RunState>>;
    fn stop(&self) -> RunState;
    fn run_events(&self) -> tokio::sync::broadcast::Receiver<String>;
    fn problems(&self, workflow: &Workflow) -> Vec<GraphIssue> {
        workflow_problems(workflow)
    }
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

fn host_failure(error: &HostError) -> Response {
    failure(
        StatusCode::from_u16(error.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        &error.message,
    )
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
        .map_err(|e| Box::new(failure(StatusCode::BAD_REQUEST, &e.to_string())))
}

fn problems_of(host: &Host, workflow: &Workflow) -> Vec<ApiProblem> {
    host.problems(workflow)
        .into_iter()
        .map(|issue| ApiProblem { issue, path: None })
        .collect()
}

fn edited_reply(host: &Host, id: Option<String>, revision: u64) -> Response {
    match host.project() {
        Ok(project) => json_reply(
            StatusCode::OK,
            &EditedReply {
                id,
                revision,
                problems: problems_of(host, &project.workflow),
            },
        ),
        Err(error) => host_failure(&error),
    }
}

fn edit(host: &Host, edit: WorkflowEdit) -> Response {
    match host.edit(edit) {
        Ok(edited) => edited_reply(host, edited.ids.into_iter().next(), edited.revision),
        Err(error) => host_failure(&error),
    }
}

fn after(host: &Host, revision: HostResult<u64>) -> Response {
    match revision {
        Ok(revision) => edited_reply(host, None, revision),
        Err(error) => host_failure(&error),
    }
}

async fn openapi(State(host): State<Host>) -> Response {
    json_reply(StatusCode::OK, &openapi_document(&host.version()))
}

async fn status(State(host): State<Host>) -> Response {
    let project = host.project().ok();
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
        parse_body::<OpenBody>(&b)
            .map_err(|e| Box::new(failure(StatusCode::BAD_REQUEST, &e.to_string())))
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
    match host.open(body.dir).await {
        Ok(project) => json_reply(StatusCode::OK, &project),
        Err(error) => host_failure(&error),
    }
}

async fn project(State(host): State<Host>) -> Response {
    match host.project() {
        Ok(project) => json_reply(StatusCode::OK, &project),
        Err(error) => host_failure(&error),
    }
}

async fn replace(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match read_json(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let checked = check_workflow(&body);
    match checked.workflow {
        Some(workflow) => edit(
            &host,
            WorkflowEdit::Replace {
                workflow: Box::new(workflow),
            },
        ),
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
    edit(
        &host,
        WorkflowEdit::AddNode {
            kind: body.kind,
            id: body.id,
            label: body.label,
            position: body.position,
            params: body.params,
        },
    )
}

async fn update_node(
    State(host): State<Host>,
    UrlPath(id): UrlPath<String>,
    bytes: Bytes,
) -> Response {
    match body_of::<UpdateNodeBody>(&bytes, true) {
        Ok(patch) => edit(
            &host,
            WorkflowEdit::UpdateNode {
                id,
                patch: patch.into(),
            },
        ),
        Err(response) => *response,
    }
}

async fn remove_node(State(host): State<Host>, UrlPath(id): UrlPath<String>) -> Response {
    edit(&host, WorkflowEdit::RemoveNodes { ids: vec![id] })
}

async fn add_edge(State(host): State<Host>, bytes: Bytes) -> Response {
    let body = match body_of::<AddEdgeBody>(&bytes, true) {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let project = match host.project().map_err(|e| Box::new(host_failure(&e))) {
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
    edit(&host, WorkflowEdit::AddEdge { ends: body.into() })
}

async fn remove_edge(State(host): State<Host>, UrlPath(id): UrlPath<String>) -> Response {
    edit(&host, WorkflowEdit::RemoveEdges { ids: vec![id] })
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
        Ok(Value::Object(mut body)) => {
            if let Some(problem) = settings_problem(&body) {
                failure(StatusCode::BAD_REQUEST, &problem)
            } else {
                if body.get("background") == Some(&Value::Null) {
                    body.insert("background".into(), Value::String(String::new()));
                }
                match serde_json::from_value(Value::Object(body)) {
                    Ok(patch) => edit(&host, WorkflowEdit::SetSettings { patch }),
                    Err(error) => failure(StatusCode::BAD_REQUEST, &error.to_string()),
                }
            }
        }
        Ok(_) => failure(
            StatusCode::BAD_REQUEST,
            "The request body must be a JSON object.",
        ),
        Err(response) => *response,
    }
}

async fn variables(State(host): State<Host>, bytes: Bytes) -> Response {
    match body_of::<VariablesBody>(&bytes, true) {
        Ok(variables) => edit(&host, WorkflowEdit::SetVariables { variables }),
        Err(response) => *response,
    }
}

async fn problems(State(host): State<Host>) -> Response {
    match host.project() {
        Ok(project) => {
            let problems = problems_of(&host, &project.workflow);
            let ok = !problems.iter().any(|p| p.issue.level == IssueLevel::Error);
            json_reply(StatusCode::OK, &Check { ok, problems })
        }
        Err(error) => host_failure(&error),
    }
}

async fn undo(State(host): State<Host>) -> Response {
    after(&host, host.undo())
}

async fn redo(State(host): State<Host>) -> Response {
    after(&host, host.redo())
}

async fn save(State(host): State<Host>) -> Response {
    let saved = host.save().await;
    after(&host, saved)
}

async fn run_state(State(host): State<Host>) -> Response {
    match host.project() {
        Ok(project) => json_reply(
            StatusCode::OK,
            &RunState {
                running: project.running,
                run_id: project.run_id,
            },
        ),
        Err(error) => host_failure(&error),
    }
}

async fn run(State(host): State<Host>, bytes: Bytes) -> Response {
    let targets = match body_of::<RunTargets>(&bytes, false) {
        Ok(targets) => targets,
        Err(response) => return *response,
    };
    match host.run(targets).await {
        Ok(state) => json_reply(StatusCode::OK, &state),
        Err(error) => host_failure(&error),
    }
}

async fn stop(State(host): State<Host>) -> Response {
    json_reply(StatusCode::OK, &host.stop())
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
