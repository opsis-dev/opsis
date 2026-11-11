use std::time::Duration;

use opsis_types::edits::{Edited, TimelineEdit, TimelineReply, WorkflowEdit};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::shell::PlaybackCall;
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde::Serialize;
use serde_json::Value;

pub type HostResult<T> = Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub dir: String,
    pub revision: u64,
    pub saved: bool,
    pub settings: WorkflowSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunNext {
    Event(Box<RunEvent>),
    Missed(u64),
    Idle,
}

pub trait RunEvents {
    fn next(&mut self, wait: Duration) -> RunNext;
}

pub trait ScriptHost {
    fn version(&self) -> String;
    fn project(&self) -> HostResult<ProjectInfo>;
    fn workflow(&self) -> HostResult<Workflow>;
    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited>;
    fn edit_timeline(&self, edit: TimelineEdit) -> HostResult<TimelineReply>;
    fn save(&self) -> HostResult<bool>;
    fn problems(&self) -> HostResult<Value>;
    fn catalog(&self) -> Value;
    fn compile(&self) -> HostResult<Value>;
    fn validate(&self, workflow: &Value) -> Value;
    fn fonts(&self) -> HostResult<Value>;
    fn use_font(&self, id: &str) -> HostResult<String>;
    fn start_run(&self, targets: RunTargets) -> HostResult<(String, Box<dyn RunEvents>)>;
    fn stop_run(&self);
    fn active_run(&self) -> Option<String>;
    fn run_state(&self) -> Value;
    fn results(&self) -> Value;
    fn clear_results(&self) -> HostResult<()>;
    fn clear_log(&self);
    fn playback(&self, call: PlaybackCall) -> HostResult<()>;
    fn shared(&self) -> Value;
    fn select(&self, ids: Vec<String>) -> HostResult<()>;
}

pub fn run_of(event: &RunEvent) -> &str {
    match event {
        RunEvent::RunStart { run_id, .. }
        | RunEvent::NodeStart { run_id, .. }
        | RunEvent::NodeProgress { run_id, .. }
        | RunEvent::NodeLog { run_id, .. }
        | RunEvent::NodeDone { run_id, .. }
        | RunEvent::NodeError { run_id, .. }
        | RunEvent::NodeSkipped { run_id, .. }
        | RunEvent::RunDone { run_id, .. } => run_id,
    }
}
