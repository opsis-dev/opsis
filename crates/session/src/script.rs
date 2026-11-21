use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use opsis_compile::catalog::{catalog_entry, node_catalog};
use opsis_compile::check::{absolute_file_settings, check_workflow, workflow_problems_with};
use opsis_runner::progress::MAX_LOG_LINES;
use opsis_script::host::{HostResult, ProjectInfo, RunEvents, RunNext, ScriptHost};
use opsis_script::{Script, run as run_lua};
use opsis_types::edits::{Edited, TimelineEdit, TimelineReply, WorkflowEdit};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::script::{
    SCRIPT_EXTENSION, SCRIPTS_DIR, ScriptLine, ScriptLineKind, ScriptOutcome, ScriptStatus,
};
use opsis_types::shell::{PlaybackCall, Selection};
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};
use tokio::runtime::Handle;
use tokio::sync::{broadcast, oneshot, watch};

use crate::Session;
use crate::document::DocumentError;
use crate::events::{Emit, ScriptChanged, ScriptLogged};
use crate::state::AppState;

pub const SCRIPT_TASK: &str = "Running a script";
const SCRIPT_WINDOW: &str = "script";
const REQUEST_NAME: &str = "request.lua";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptSource {
    File(String),
    Text { name: String, source: String },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScriptStartError {
    #[error("No project is open.")]
    NoProject,
    #[error("A script is already running.")]
    Busy,
    #[error("Scripts live in the project's scripts folder as .lua files; {0} isn't one.")]
    NotAScript(String),
    #[error("Couldn't read {0}.")]
    Unreadable(String),
}

#[derive(Debug)]
struct ActiveScript {
    file: String,
    stop: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct Scripts {
    active: Mutex<Option<ActiveScript>>,
    output: Mutex<VecDeque<ScriptLine>>,
    last: Mutex<Option<(String, ScriptOutcome)>>,
    lines: AtomicU64,
    running: watch::Sender<bool>,
}

impl Default for Scripts {
    fn default() -> Self {
        Self {
            active: Mutex::default(),
            output: Mutex::default(),
            last: Mutex::default(),
            lines: AtomicU64::new(0),
            running: watch::Sender::new(false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptState {
    pub running: Option<String>,
    pub output: Vec<ScriptLine>,
    pub last: Option<(String, ScriptOutcome)>,
}

pub fn script_path(file: &str) -> Option<String> {
    let path = Path::new(file);
    let mut parts = path.components();
    let first = parts.next()?;
    let inside = first.as_os_str() == SCRIPTS_DIR
        && parts.all(|c| matches!(c, std::path::Component::Normal(_)))
        && path.extension().is_some_and(|e| e == SCRIPT_EXTENSION)
        && path.components().count() > 1;
    inside.then(|| file.replace('\\', "/"))
}

fn note(app: &Session, kind: ScriptLineKind, text: &str) {
    let scripts = app.state::<Scripts>();
    let line = ScriptLine {
        id: scripts.lines.fetch_add(1, Ordering::Relaxed) + 1,
        kind,
        text: text.to_owned(),
    };
    {
        let mut output = scripts.output.lock().expect("script output");
        output.push_back(line.clone());
        while output.len() > MAX_LOG_LINES {
            output.pop_front();
        }
    }
    if let Err(error) = ScriptLogged(line).emit(app) {
        log::debug!("sending a script line: {error}");
    }
}

fn announce(app: &Session) {
    if let Err(error) = ScriptChanged.emit(app) {
        log::debug!("sending the script state: {error}");
    }
}

fn source_of(dir: &Path, source: ScriptSource) -> Result<Script, ScriptStartError> {
    match source {
        ScriptSource::File(file) => {
            let file = script_path(&file).ok_or(ScriptStartError::NotAScript(file))?;
            let text = std::fs::read_to_string(dir.join(&file))
                .map_err(|_| ScriptStartError::Unreadable(file.clone()))?;
            Ok(Script { file, source: text })
        }
        ScriptSource::Text { name, source } => Ok(Script { file: name, source }),
    }
}

pub fn request_source(source: String) -> ScriptSource {
    ScriptSource::Text {
        name: REQUEST_NAME.to_owned(),
        source,
    }
}

pub fn run_script(
    app: &Session,
    source: ScriptSource,
    session: Option<String>,
) -> Result<oneshot::Receiver<ScriptOutcome>, ScriptStartError> {
    let state = app.state::<AppState>();
    let dir = crate::document::document_dir(state).ok_or(ScriptStartError::NoProject)?;
    let script = source_of(&dir, source)?;
    let scripts = app.state::<Scripts>();
    let stop = Arc::new(AtomicBool::new(false));
    {
        let mut active = scripts.active.lock().expect("active script");
        if active.is_some() {
            return Err(ScriptStartError::Busy);
        }
        crate::document::begin_batch(app).map_err(|error| match error {
            DocumentError::NoProject => ScriptStartError::NoProject,
            _ => ScriptStartError::Busy,
        })?;
        *active = Some(ActiveScript {
            file: script.file.clone(),
            stop: Arc::clone(&stop),
        });
    }
    scripts.output.lock().expect("script output").clear();
    scripts.running.send_replace(true);
    state.tasks.begin(SCRIPT_TASK);
    announce(app);
    let (done, outcome) = oneshot::channel();
    let app = app.clone();
    let handle = crate::runtime::handle().clone();
    std::thread::spawn(move || {
        let outcome = run_on_thread(&app, &handle, &dir, &script, stop, session);
        let scripts = app.state::<Scripts>();
        *scripts.last.lock().expect("last script") = Some((script.file.clone(), outcome.clone()));
        *scripts.active.lock().expect("active script") = None;
        app.state::<AppState>().tasks.end(SCRIPT_TASK);
        scripts.running.send_replace(false);
        announce(&app);
        if done.send(outcome).is_err() {
            log::debug!("no one waits for the script's outcome");
        }
    });
    Ok(outcome)
}

fn run_on_thread(
    app: &Session,
    handle: &Handle,
    dir: &Path,
    script: &Script,
    stop: Arc<AtomicBool>,
    session: Option<String>,
) -> ScriptOutcome {
    let host = Rc::new(SessionHost {
        app: app.clone(),
        handle: handle.clone(),
        dir: dir.to_path_buf(),
        session,
    });
    let sink_app = app.clone();
    let ending = run_lua(
        script,
        dir,
        host,
        Rc::new(move |kind, text: &str| note(&sink_app, kind, text)),
        stop,
    );
    let keep = ending.failure.is_none();
    let rolled_back = crate::document::end_batch(app, dir, keep);
    let output = app
        .state::<Scripts>()
        .output
        .lock()
        .expect("script output")
        .iter()
        .cloned()
        .collect();
    ScriptOutcome {
        ok: keep,
        value: ending.value,
        output,
        failure: ending.failure,
        stopped: ending.stopped,
        rolled_back,
    }
}

pub fn stop_script(app: &Session) -> ScriptStatus {
    if let Some(active) = app
        .state::<Scripts>()
        .active
        .lock()
        .expect("active script")
        .as_ref()
    {
        active.stop.store(true, Ordering::Relaxed);
    }
    script_status(app)
}

pub async fn stop_and_wait(app: &Session) {
    stop_script(app);
    let mut running = app.state::<Scripts>().running.subscribe();
    if running.wait_for(|running| !running).await.is_err() {
        log::debug!("the script state closed while stopping");
    }
}

pub fn script_status(app: &Session) -> ScriptStatus {
    let file = app
        .state::<Scripts>()
        .active
        .lock()
        .expect("active script")
        .as_ref()
        .map(|a| a.file.clone());
    ScriptStatus {
        running: file.is_some(),
        file,
    }
}

pub fn script_state(app: &Session) -> ScriptState {
    let scripts = app.state::<Scripts>();
    ScriptState {
        running: script_status(app).file,
        output: scripts
            .output
            .lock()
            .expect("script output")
            .iter()
            .cloned()
            .collect(),
        last: scripts.last.lock().expect("last script").clone(),
    }
}

pub fn list_scripts(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut folders = vec![dir.join(SCRIPTS_DIR)];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                folders.push(path);
            } else if path.extension().is_some_and(|e| e == SCRIPT_EXTENSION)
                && let Ok(relative) = path.strip_prefix(dir)
            {
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    found
}

struct SessionHost {
    app: Session,
    handle: Handle,
    dir: PathBuf,
    session: Option<String>,
}

struct Following(broadcast::Receiver<String>, Handle);

impl RunEvents for Following {
    fn next(&mut self, wait: Duration) -> RunNext {
        let received = self
            .1
            .block_on(async { tokio::time::timeout(wait, self.0.recv()).await });
        match received {
            Ok(Ok(json)) => serde_json::from_str::<RunEvent>(&json)
                .map_or(RunNext::Idle, |event| RunNext::Event(Box::new(event))),
            Ok(Err(broadcast::error::RecvError::Lagged(missed))) => RunNext::Missed(missed),
            Ok(Err(broadcast::error::RecvError::Closed)) | Err(_) => RunNext::Idle,
        }
    }
}

fn file_check(edit: &WorkflowEdit) -> Result<(), String> {
    let nodes: Vec<(&str, &serde_json::Map<String, Value>)> = match edit {
        WorkflowEdit::AddNode {
            kind,
            params: Some(params),
            ..
        } => vec![(kind.as_str(), params)],
        WorkflowEdit::Paste { nodes, .. } => {
            nodes.iter().map(|n| (n.kind.as_str(), &n.params)).collect()
        }
        WorkflowEdit::Replace { workflow } => workflow
            .nodes
            .iter()
            .map(|n| (n.kind.as_str(), &n.params))
            .collect(),
        _ => Vec::new(),
    };
    for (kind, params) in nodes {
        if let Some(field) = absolute_file_settings(kind, params).first() {
            return Err(refused_file(field));
        }
    }
    Ok(())
}

fn refused_file(field: &str) -> String {
    format!(
        "{field}: scripts can't point a file setting at a path outside the project. Put the file in the project folder and use its path there."
    )
}

impl SessionHost {
    fn state(&self) -> &AppState {
        self.app.state::<AppState>()
    }

    fn checked_update(&self, edit: &WorkflowEdit) -> Result<(), String> {
        file_check(edit)?;
        if let WorkflowEdit::UpdateNode { id, patch } = edit
            && let Some(params) = &patch.params
        {
            let kind = self
                .state()
                .editor()
                .document
                .as_ref()
                .and_then(|d| d.workflow().node(id).map(|n| n.kind.clone()));
            if let Some(field) = kind
                .as_deref()
                .and_then(|kind| absolute_file_settings(kind, params).into_iter().next())
            {
                return Err(refused_file(&field));
            }
        }
        Ok(())
    }

    fn compiled(&self) -> HostResult<crate::compile::EditorCompile> {
        self.handle
            .block_on(crate::compile::latest_compile(self.app.clone()))
            .map_err(|e| e.0)?
            .ok_or_else(|| DocumentError::NoProject.to_string())
    }
}

impl ScriptHost for SessionHost {
    fn version(&self) -> String {
        crate::APP_VERSION.to_owned()
    }

    fn project(&self) -> HostResult<ProjectInfo> {
        let state = crate::document::document_state(&self.app)
            .ok_or_else(|| DocumentError::NoProject.to_string())?;
        Ok(ProjectInfo {
            name: state.workflow.name.clone(),
            dir: state.dir,
            revision: state.revision,
            saved: state.revision == state.saved_revision,
            settings: state.workflow.settings,
        })
    }

    fn workflow(&self) -> HostResult<Workflow> {
        crate::document::document_workflow(self.state())
            .map(|(_, workflow)| workflow)
            .ok_or_else(|| DocumentError::NoProject.to_string())
    }

    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited> {
        self.checked_update(&edit)?;
        crate::document::edit(&self.app, edit).map_err(|e| e.to_string())
    }

    fn edit_timeline(&self, edit: TimelineEdit) -> HostResult<TimelineReply> {
        let compiled = self.compiled()?;
        let film = crate::compile::film_of(&compiled)
            .ok_or_else(|| "There's no film on the timeline yet.".to_owned())?;
        crate::document::edit_timeline_on(&self.app, edit, &film).map_err(|e| e.to_string())
    }

    fn save(&self) -> HostResult<bool> {
        self.handle
            .block_on(crate::document::save(&self.app))
            .map_err(|e| e.0)
    }

    fn problems(&self) -> HostResult<Value> {
        let workflow = self.workflow()?;
        let results = self.state().editor().results.clone();
        let mut problems = workflow_problems_with(&workflow, Some(&results));
        problems.extend(crate::node_results::file_problems(&self.app));
        serde_json::to_value(problems).map_err(|e| e.to_string())
    }

    fn catalog(&self) -> Value {
        let entries: Vec<_> = node_catalog()
            .into_iter()
            .filter_map(|node| catalog_entry(&node.kind))
            .collect();
        serde_json::to_value(entries).unwrap_or(Value::Null)
    }

    fn compile(&self) -> HostResult<Value> {
        serde_json::to_value(self.compiled()?).map_err(|e| e.to_string())
    }

    fn validate(&self, workflow: &Value) -> Value {
        let checked = check_workflow(workflow);
        json!({ "ok": checked.ok, "problems": checked.problems })
    }

    fn fonts(&self) -> HostResult<Value> {
        let fonts = self
            .handle
            .block_on(crate::fonts::list_fonts(&self.app))
            .map_err(|e| e.0)?;
        serde_json::to_value(fonts).map_err(|e| e.to_string())
    }

    fn use_font(&self, id: &str) -> HostResult<String> {
        let copied = self
            .handle
            .block_on(crate::fonts::use_font_in_project(
                &self.app,
                self.dir.to_string_lossy().into_owned(),
                id.to_owned(),
            ))
            .map_err(|e| e.0)?;
        let family = copied.family.clone();
        self.edit(WorkflowEdit::AddProjectFont { font: copied })?;
        Ok(family)
    }

    fn start_run(&self, targets: RunTargets) -> HostResult<(String, Box<dyn RunEvents>)> {
        let events = crate::api::subscribe_run_events(&self.app);
        let started = self
            .handle
            .block_on(crate::run::start(&self.app, targets, self.session.clone()))
            .map_err(|e| e.to_string())?;
        Ok((
            started.run_id,
            Box::new(Following(events, self.handle.clone())),
        ))
    }

    fn stop_run(&self) {
        crate::run::cancel_run(&self.app);
    }

    fn active_run(&self) -> Option<String> {
        crate::run::active_run(&self.app)
    }

    fn run_state(&self) -> Value {
        serde_json::to_value(crate::run::run_state(&self.app)).unwrap_or(Value::Null)
    }

    fn results(&self) -> Value {
        serde_json::to_value(&self.state().editor().results).unwrap_or(Value::Null)
    }

    fn clear_results(&self) -> HostResult<()> {
        crate::projects::clear_cache(&self.app, &self.dir.to_string_lossy()).map_err(|e| e.0)
    }

    fn clear_log(&self) {
        crate::run::clear_run_log(&self.app);
    }

    fn playback(&self, call: PlaybackCall) -> HostResult<()> {
        crate::shared::control_playback(&self.app, call)
            .map_err(|_| "No editor window is playing.".to_owned())
    }

    fn shared(&self) -> Value {
        serde_json::to_value(crate::shared::shared_state(&self.app)).unwrap_or(Value::Null)
    }

    fn select(&self, ids: Vec<String>) -> HostResult<()> {
        let first = ids.first().cloned();
        crate::shared::share_selection(
            &self.app,
            SCRIPT_WINDOW,
            Selection {
                selected_ids: ids,
                ..Selection::default()
            },
        )
        .map_err(|e| e.0)?;
        crate::compile::set_compile_focus(&self.app, first.clone(), first);
        Ok(())
    }

    fn browser_open(&self, up_to: &str) -> HostResult<Value> {
        let opened = self
            .handle
            .block_on(crate::agent_browser::open(&self.app, up_to))
            .map_err(|e| e.to_string())?;
        serde_json::to_value(opened).map_err(|e| e.to_string())
    }

    fn browser_state(&self) -> HostResult<Value> {
        match self.handle.block_on(crate::agent_browser::state(&self.app)) {
            Ok(state) => serde_json::to_value(state).map_err(|e| e.to_string()),
            Err(crate::agent_browser::BrowserError::NotOpen) => Ok(Value::Null),
            Err(error) => Err(error.to_string()),
        }
    }

    fn browser_check(&self, up_to: &str) -> HostResult<Value> {
        let checked = self
            .handle
            .block_on(crate::agent_browser::check(&self.app, up_to))
            .map_err(|e| e.to_string())?;
        serde_json::to_value(checked).map_err(|e| e.to_string())
    }

    fn browser_close(&self) {
        self.handle.block_on(crate::agent_browser::close(&self.app));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_only_lua_files_inside_the_scripts_folder() {
        assert_eq!(
            script_path("scripts/a.lua").as_deref(),
            Some("scripts/a.lua")
        );
        assert_eq!(
            script_path("scripts/tools/b.lua").as_deref(),
            Some("scripts/tools/b.lua")
        );
        for refused in [
            "a.lua",
            "scripts/../a.lua",
            "scripts/a.txt",
            "/scripts/a.lua",
            "scripts",
        ] {
            assert_eq!(script_path(refused), None, "{refused}");
        }
    }
}
