use opsis_compile::catalog::{CatalogNode, CatalogPort};
use opsis_compile::check::ApiProblem;
use opsis_types::engine::EngineStatus;
use opsis_types::events::RunTargets;
use opsis_types::fonts::FontFamilyDef;
use opsis_types::script::{SESSION_HEADER, ScriptOutcome, ScriptRequest, ScriptStatus};
use opsis_types::settings::AppSettings;
use opsis_types::trace::TraceHeader;
use opsis_types::values::{ActionResult, LastRun};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::bodies::{
    AddEdgeBody, AddNodeBody, BrowserBody, CreateProjectBody, OpenBody, UpdateNodeBody,
    VariablesBody,
};

const SCHEMAS: &str = "#/components/schemas/";

const DESCRIPTION: &str = "Reads, edits and runs the project open in Opsis. It listens only on a local socket (a named pipe on Windows), and only while the assistant is switched on in Settings → Assistant.

Each edit is one undo step, saves itself and shows in every window.

Start with GET /nodes for the node types, GET /nodes/{type} for one type's ports and settings, and GET /project for the open project.

A connection joins an output port (source, sourceHandle) to an input port (target, targetHandle). Ports have types; GET /nodes/{type} lists them.";

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "Check")]
pub struct Check {
    #[schemars(description = "True when there are no errors")]
    pub ok: bool,
    pub problems: Vec<ApiProblem>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "Edited")]
pub struct Edited {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "The new node's or connection's id")]
    pub id: Option<String>,
    pub revision: u64,
    pub problems: Vec<ApiProblem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectPaths")]
pub struct ProjectPaths {
    pub dir: String,
    #[schemars(description = "Where exports land")]
    pub out: String,
    pub traces: String,
    pub cache: String,
    #[schemars(description = "The last run's results, as JSON")]
    pub last_run: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Paths")]
pub struct PathsReply {
    #[schemars(description = "Opsis's data folder")]
    pub data: String,
    #[schemars(description = "Where the recording browser is downloaded to")]
    pub engine: String,
    #[schemars(description = "The assistant's request logs, newline-delimited JSON")]
    pub assistant_logs: String,
    #[schemars(description = "The open project's folders, when one is open")]
    pub project: Option<ProjectPaths>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "RecentProject")]
pub struct RecentProject {
    pub dir: String,
    pub name: String,
    #[schemars(description = "Milliseconds since the epoch")]
    pub opened_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Template")]
pub struct TemplateReply {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "About")]
pub struct AboutReply {
    pub version: String,
    #[schemars(description = "The commit a release was built from; empty in development")]
    pub commit: String,
    pub platform: String,
    pub packaged: bool,
    #[schemars(description = "The recording browser's version, when it's set up")]
    pub browser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Fonts")]
pub struct FontsReply {
    #[schemars(description = "The font library, which projects take fonts from")]
    pub library: Vec<FontFamilyDef>,
    #[schemars(description = "The font families installed on this computer")]
    pub computer: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Browser")]
pub struct BrowserReply {
    #[schemars(description = "The browser's DevTools address, for Playwright's connectOverCDP")]
    pub address: String,
    #[schemars(description = "The Open browser node whose flow it plays")]
    pub flow: String,
    #[schemars(description = "The last action it has played, or the Open browser node")]
    pub at: String,
    #[schemars(description = "The page it's on")]
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "BrowserCheck")]
pub struct CheckReply {
    #[schemars(description = "Each action played, in order, and the ones it didn't reach")]
    pub actions: Vec<ActionResult>,
    #[schemars(description = "The last action that passed, where the browser now is")]
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "RunStatus")]
pub struct RunState {
    pub running: bool,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "Project")]
pub struct ProjectState {
    pub running: bool,
    pub run_id: Option<String>,
    pub dir: Option<String>,
    pub workflow: Workflow,
    #[schemars(description = "Goes up with every edit")]
    pub revision: u64,
    pub saved: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "ScriptAnswer")]
pub struct ScriptReply {
    #[serde(flatten)]
    pub outcome: ScriptOutcome,
    pub problems: Vec<ApiProblem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenProjectName {
    pub dir: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Status")]
pub struct Status {
    pub app: String,
    pub version: String,
    pub project: Option<OpenProjectName>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "Error")]
pub struct ErrorBody {
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<ApiProblem>>,
}

fn reference(name: &str) -> Value {
    json!({ "$ref": format!("{SCHEMAS}{name}") })
}

fn content(schema: Value) -> Value {
    let mut content = json!({ "application/json": {} });
    content["application/json"]["schema"] = schema;
    content
}

fn answer(schema: Value) -> Value {
    json!({ "description": "OK", "content": content(schema) })
}

fn error(description: &str) -> Value {
    json!({ "description": description, "content": content(reference("Error")) })
}

fn body(schema: Value, required: bool) -> Value {
    json!({ "required": required, "content": content(schema) })
}

fn session_param() -> Value {
    json!([{ "name": SESSION_HEADER, "in": "header", "required": false, "schema": { "type": "string" }, "description": "The Agent panel session asking, which opsis-mcp sends; the run, or a run the script starts, reports its end to that session's agent" }])
}

fn path_param(name: &str) -> Value {
    json!([{ "name": name, "in": "path", "required": true, "schema": { "type": "string" } }])
}

struct Doc {
    generator: SchemaGenerator,
}

impl Doc {
    fn schema<T: JsonSchema>(&mut self) -> Value {
        self.generator.subschema_for::<T>().to_value()
    }
}

fn operation(summary: &str, responses: &[(&str, Value)]) -> Map<String, Value> {
    let mut op = Map::new();
    op.insert("summary".into(), summary.into());
    op.insert(
        "responses".into(),
        Value::Object(
            responses
                .iter()
                .map(|(code, v)| ((*code).to_owned(), v.clone()))
                .collect(),
        ),
    );
    op
}

fn with(mut op: Map<String, Value>, key: &str, value: Value) -> Map<String, Value> {
    op.insert(key.into(), value);
    op
}

fn project_op(op: Map<String, Value>) -> Value {
    let mut op = op;
    if let Some(Value::Object(responses)) = op.get_mut("responses") {
        responses.insert(
            "409".into(),
            error("No project is open, or a run is already going"),
        );
    }
    Value::Object(op)
}

fn paths(doc: &mut Doc) -> Value {
    let bad = || error("The request isn't valid");
    let missing = || error("Not found");
    let edited = || answer(reference("Edited"));
    let project = || answer(reference("Project"));
    json!({
        "/openapi.json": { "get": operation("This document", &[("200", answer(json!({ "type": "object" })))]) },
        "/status": { "get": operation("App version, the open project and whether a run is going", &[("200", answer(reference("Status")))]) },
        "/nodes": { "get": operation("Every node type, without its settings schema", &[("200", answer(json!({ "type": "array", "items": reference("NodeType") })))]) },
        "/nodes/{type}": { "get": with(operation("One node type with its ports and settings as JSON Schema", &[("200", answer(reference("NodeType"))), ("404", missing())]), "parameters", path_param("type")) },
        "/validate": { "post": with(with(operation("Check a project without opening it", &[("200", answer(reference("Check")))]), "description", "Checks the file format, node types, connections and every node's settings. Nothing is saved.".into()), "requestBody", body(reference("Workflow"), true)) },
        "/open": { "post": with(operation("Open a project folder in the editor", &[("200", project()), ("400", bad()), ("404", missing()), ("422", error("The project file can't be read"))]), "requestBody", body(doc.schema::<OpenBody>(), true)) },
        "/settings": { "get": operation("Every settings section, as Settings shows them", &[("200", answer(reference("AppSettings")))]) },
        "/engine": { "get": operation("Whether the recording browser is set up, its path and version, and why it can't be used", &[("200", answer(reference("EngineStatus")))]) },
        "/fonts": { "get": operation("The font library and the font families on this computer", &[("200", answer(reference("Fonts")))]) },
        "/paths": { "get": with(operation("Opsis's folders and the open project's", &[("200", answer(reference("Paths")))]), "description", "Read logs, exports, traces and the last run as files from these folders.".into()) },
        "/about": { "get": operation("The app's version, commit and platform, and the recording browser's version", &[("200", answer(reference("About")))]) },
        "/templates": { "get": operation("The templates a new project can start from", &[("200", answer(json!({ "type": "array", "items": reference("Template") })))]) },
        "/projects": {
            "get": operation("Recent projects, newest first", &[("200", answer(json!({ "type": "array", "items": reference("RecentProject") })))]),
            "post": with(with(operation("Create a project from a template and open it", &[("200", project()), ("400", bad()), ("422", error("The project can't be created there"))]), "description", "Makes a folder named after the project in dir, from the template, then opens it in the editor.".into()), "requestBody", body(doc.schema::<CreateProjectBody>(), true)),
        },
        "/project/run/last": { "get": project_op(operation("The last run's result for each node, and its failures", &[("200", answer(json!({ "oneOf": [reference("LastRun"), { "type": "null" }] })))])) },
        "/project/browser": {
            "get": with(operation("The browser open for agents: its DevTools address, its flow and where it is", &[("200", answer(reference("Browser"))), ("404", error("No browser is open"))]), "description", "Connect Playwright to address with connectOverCDP to read or drive the page; the browser keeps the project's sign-in and its flow's settings.".into()),
            "put": project_op(with(with(operation("Open the project's browser and play its flow up to an action", &[("200", answer(reference("Browser"))), ("400", bad()), ("422", error("An action failed while it played"))]), "description", "Opens the recording browser with the flow's Open browser settings, plays its actions as a take would, with no recording, and keeps it open. A browser already open closes first. It closes when the project closes or the app exits.".into()), "requestBody", body(doc.schema::<BrowserBody>(), true))),
            "delete": operation("Close the project's browser", &[("200", answer(json!({ "type": "object" })))]),
        },
        "/project/browser/check": { "post": project_op(with(with(operation("Play the flow on from where the open browser is, up to an action, and say how each action went", &[("200", answer(reference("BrowserCheck"))), ("400", bad())]), "description", "Each action plays with the take's own target matching and settings; the check stops at the first action that fails, and the browser stays where the last passing action left it.".into()), "requestBody", body(doc.schema::<BrowserBody>(), true))) },
        "/project/traces": { "get": project_op(with(operation("The recorded traces of browser takes, newest first", &[("200", answer(json!({ "type": "array", "items": reference("Trace") })))]), "description", "A take's actions, console errors and the moment a target failed are kept in its trace.".into())) },
        "/project": {
            "get": project_op(operation("The open project", &[("200", project())])),
            "put": project_op(with(operation("Replace the open project's flow", &[("200", edited()), ("400", bad())]), "requestBody", body(reference("Workflow"), true))),
        },
        "/project/nodes": { "post": project_op(with(with(operation("Add a node", &[("200", edited()), ("400", bad())]), "description", "Settings you leave out get their defaults.".into()), "requestBody", body(doc.schema::<AddNodeBody>(), true))) },
        "/project/nodes/{id}": {
            "patch": project_op(with(with(with(operation("Change a node's name, position, settings or whether it's skipped", &[("200", edited()), ("400", bad()), ("404", missing())]), "description", "params is merged into the node's settings.".into()), "parameters", path_param("id")), "requestBody", body(doc.schema::<UpdateNodeBody>(), true))),
            "delete": project_op(with(operation("Delete a node and its connections", &[("200", edited()), ("404", missing())]), "parameters", path_param("id"))),
        },
        "/project/edges": { "post": project_op(with(operation("Connect an output port to an input port", &[("200", edited()), ("400", bad()), ("404", missing())]), "requestBody", body(doc.schema::<AddEdgeBody>(), true))) },
        "/project/edges/{id}": { "delete": project_op(with(operation("Remove a connection", &[("200", edited()), ("404", missing())]), "parameters", path_param("id"))) },
        "/project/settings": { "patch": project_op(with(operation("Change the project's size, frame rate, background, theme or fonts", &[("200", edited()), ("400", bad())]), "requestBody", body(reference("ProjectSettings"), true))) },
        "/project/variables": { "put": project_op(with(operation("Replace the project's variables", &[("200", edited()), ("400", bad())]), "requestBody", body(doc.schema::<VariablesBody>(), true))) },
        "/project/problems": { "get": project_op(operation("Problems in the open project", &[("200", answer(reference("Check")))])) },
        "/project/undo": { "post": project_op(operation("Undo the last edit", &[("200", edited())])) },
        "/project/redo": { "post": project_op(operation("Redo", &[("200", edited())])) },
        "/project/save": { "post": project_op(operation("Save now", &[("200", edited())])) },
        "/project/run": {
            "get": project_op(operation("Whether a run is going", &[("200", answer(reference("RunStatus")))])),
            "post": project_op(with(with(with(operation("Start a run", &[("200", answer(reference("RunStatus"))), ("400", bad())]), "description", "With no body, runs everything that needs running. targetNodeIds runs those nodes and what they need; fromNodeId runs again from that node. Runs open the websites in the flow.".into()), "requestBody", body(doc.schema::<RunTargets>(), false)), "parameters", session_param())),
        },
        "/project/run/stop": { "post": project_op(operation("Stop the run", &[("200", answer(reference("RunStatus")))])) },
        "/project/script": {
            "get": project_op(operation("Whether a script is running", &[("200", answer(reference("ScriptStatus")))])),
            "post": project_op(with(with(with(operation("Run a Lua script and answer when it ends", &[("200", answer(reference("ScriptAnswer"))), ("400", bad()), ("404", missing())]), "description", "Send source (Lua) or file (a script in the project's scripts folder). The script gets the opsis module and reaches only the project's files. A script that fails or is stopped undoes its changes to the project and answers ok: false; everything it changed is otherwise one undo step.".into()), "requestBody", body(doc.schema::<ScriptRequest>(), true)), "parameters", session_param())),
        },
        "/project/script/stop": { "post": project_op(with(operation("Stop the running script", &[("200", answer(reference("ScriptStatus")))]), "description", "Its changes to the project are undone.".into())) },
        "/project/run/events": { "get": with(operation("Follow run progress", &[("200", json!({ "description": "text/event-stream" }))]), "description", "A server-sent event stream. Each event is a JSON run event (run:start, node:start, node:done, node:error, node:skipped, run:done, …).".into()) },
    })
}

pub fn openapi_document(version: &str) -> Value {
    let mut settings = SchemaSettings::openapi3();
    settings.definitions_path = "/components/schemas/".into();
    let mut doc = Doc {
        generator: settings.into_generator(),
    };
    for register in [
        Doc::schema::<Workflow>,
        Doc::schema::<WorkflowSettings>,
        Doc::schema::<Check>,
        Doc::schema::<Edited>,
        Doc::schema::<ProjectState>,
        Doc::schema::<RunState>,
        Doc::schema::<Status>,
        Doc::schema::<CatalogNode>,
        Doc::schema::<CatalogPort>,
        Doc::schema::<ErrorBody>,
        Doc::schema::<ScriptReply>,
        Doc::schema::<ScriptStatus>,
        Doc::schema::<AppSettings>,
        Doc::schema::<EngineStatus>,
        Doc::schema::<FontsReply>,
        Doc::schema::<PathsReply>,
        Doc::schema::<LastRun>,
        Doc::schema::<TraceHeader>,
        Doc::schema::<RecentProject>,
        Doc::schema::<TemplateReply>,
        Doc::schema::<AboutReply>,
        Doc::schema::<BrowserReply>,
        Doc::schema::<CheckReply>,
    ] {
        register(&mut doc);
    }
    let paths = paths(&mut doc);
    let mut schemas: Map<String, Value> =
        doc.generator.take_definitions(true).into_iter().collect();
    if let Some(Value::Object(settings)) = schemas.get("WorkflowSettings").cloned() {
        let mut partial = settings;
        partial.remove("required");
        schemas.insert("ProjectSettings".into(), Value::Object(partial));
    }
    json!({
        "openapi": "3.1.0",
        "info": { "title": "Opsis local API", "version": version, "description": DESCRIPTION },
        "servers": [{ "url": "http://localhost", "description": "Over the local socket or pipe" }],
        "paths": paths,
        "components": { "schemas": schemas },
    })
}
