use opsis_compile::catalog::{CatalogNode, CatalogPort};
use opsis_compile::check::ApiProblem;
use opsis_types::events::RunTargets;
use opsis_types::script::{ScriptOutcome, ScriptRequest, ScriptStatus};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::bodies::{AddEdgeBody, AddNodeBody, OpenBody, UpdateNodeBody, VariablesBody};

const SCHEMAS: &str = "#/components/schemas/";

const DESCRIPTION: &str = "Reads, edits and runs the project open in Opsis. It listens only on a local socket (a named pipe on Windows) that only your user account can open, and only while it's switched on in Settings → API.

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
            "post": project_op(with(with(operation("Start a run", &[("200", answer(reference("RunStatus"))), ("400", bad())]), "description", "With no body, runs everything that needs running. targetNodeIds runs those nodes and what they need; fromNodeId runs again from that node. Runs open the websites in the flow.".into()), "requestBody", body(doc.schema::<RunTargets>(), false))),
        },
        "/project/run/stop": { "post": project_op(operation("Stop the run", &[("200", answer(reference("RunStatus")))])) },
        "/project/script": {
            "get": project_op(with(operation("Whether a script is running", &[("200", answer(reference("ScriptStatus"))), ("403", error("Scripts are switched off"))]), "description", "Needs Run Lua scripts switched on in Settings → API.".into())),
            "post": project_op(with(with(operation("Run a Lua script and answer when it ends", &[("200", answer(reference("ScriptAnswer"))), ("400", bad()), ("403", error("Scripts are switched off")), ("404", missing())]), "description", "Send source (Lua) or file (a script in the project's scripts folder). The script gets the opsis module and reaches only the project's files. A script that fails or is stopped undoes its changes to the project and answers ok: false; everything it changed is otherwise one undo step. Needs Run Lua scripts switched on in Settings → API.".into()), "requestBody", body(doc.schema::<ScriptRequest>(), true))),
        },
        "/project/script/stop": { "post": project_op(with(operation("Stop the running script", &[("200", answer(reference("ScriptStatus"))), ("403", error("Scripts are switched off"))]), "description", "Its changes to the project are undone.".into())) },
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
