use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

pub const SCRIPTS_DIR: &str = "scripts";
pub const SCRIPT_EXTENSION: &str = "lua";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ScriptLineKind {
    Print,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLine {
    pub id: u64,
    pub kind: ScriptLineKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScriptFailure {
    pub message: String,
    #[schemars(description = "The script the line is in")]
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ScriptOutcome")]
pub struct ScriptOutcome {
    #[schemars(description = "True when the script ended without an error")]
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<specta_typescript::Unknown>)]
    #[schemars(description = "What the script returned, as JSON")]
    pub value: Option<Value>,
    pub output: Vec<ScriptLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ScriptFailure>,
    #[schemars(description = "True when it was stopped before it ended")]
    pub stopped: bool,
    #[schemars(description = "True when its changes to the project were undone")]
    pub rolled_back: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScriptRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Lua source to run")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "A script in the project, such as scripts/name.lua")]
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ScriptStatus")]
pub struct ScriptStatus {
    pub running: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}
