use serde::{Deserialize, Serialize};
use specta::Type;

use crate::values::NodeResult;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum NodeRunStatus {
    Idle,
    Queued,
    Running,
    Cached,
    Done,
    Error,
    Skipped,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum RunEvent {
    #[serde(rename = "run:start")]
    RunStart {
        run_id: String,
        node_ids: Vec<String>,
        at: i64,
    },
    #[serde(rename = "node:start")]
    NodeStart {
        run_id: String,
        node_id: String,
        at: i64,
    },
    #[serde(rename = "node:progress")]
    NodeProgress {
        run_id: String,
        node_id: String,
        progress: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    #[serde(rename = "node:log")]
    NodeLog {
        run_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
        level: LogLevel,
        message: String,
        at: i64,
    },
    #[serde(rename = "node:done")]
    NodeDone {
        run_id: String,
        node_id: String,
        cached: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<Box<NodeResult>>,
    },
    #[serde(rename = "node:error")]
    NodeError {
        run_id: String,
        node_id: String,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stack: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_id: Option<String>,
    },
    #[serde(rename = "node:skipped")]
    NodeSkipped {
        run_id: String,
        node_id: String,
        reason: String,
    },
    #[serde(rename = "run:done")]
    RunDone {
        run_id: String,
        ok: bool,
        cancelled: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        failed_node_ids: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skipped_node_ids: Option<Vec<String>>,
        outputs: Vec<String>,
        at: i64,
    },
}

pub const CANCELLED: &str = "Cancelled";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
pub struct NodeProgress {
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RunProgress {
    pub run_id: Option<String>,
    pub statuses: indexmap::IndexMap<String, NodeRunStatus>,
    pub progress: indexmap::IndexMap<String, NodeProgress>,
    pub cached: indexmap::IndexMap<String, bool>,
    pub errors: indexmap::IndexMap<String, String>,
    pub traces: indexmap::IndexMap<String, String>,
    pub results: indexmap::IndexMap<String, NodeResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub project_dir: String,
    pub workflow: crate::workflow::Workflow,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_node_id: Option<String>,
}
