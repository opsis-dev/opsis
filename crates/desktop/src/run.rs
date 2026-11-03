use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, capture_executors};
use opsis_capture::live::{LiveFeed, LiveMessage};
use opsis_engine::temp::{ChildKind, child_temp_name, make_child_temp, remove_child_temp};
use opsis_image::executors::image_executors;
use opsis_media::executors::media_executors;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_runner::context::{Executors, RunEnv, now_ms};
use opsis_runner::progress::{
    RunLog, apply_to_view, begin_run, exported_artifacts, forget_errors as forget_run_errors,
    seeded_view,
};
use opsis_runner::runner::{RunDeps, run_workflow as run_runner};
use opsis_stream::sign_in::StreamLogin;
use opsis_types::events::{RunEvent, RunRequest, RunState, RunTargets, RunView};
use opsis_types::values::LastRun;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;
use tokio_util::sync::CancellationToken;

use crate::engine::{engine_files, engine_tools};
use crate::error::{CommandError, CommandResult};
use crate::events::{
    LiveNoted, LiveWindowWatching, RunEventFired, RunLogged, RunStateReset, RunViewChanged,
};
use crate::frames::{FrameChannel, send};
use crate::settings::ProjectSignIns;
use crate::state::AppState;
use crate::views::AppViewHost;
use crate::windows::{LIVE_WINDOW, MAIN_WINDOW};

struct ActiveRun {
    run_id: String,
    project_dir: String,
    cancel: CancellationToken,
}

#[derive(Default)]
pub struct RunHost {
    active: Mutex<Option<ActiveRun>>,
    view: Mutex<RunView>,
    log: Mutex<RunLog>,
    changes: AtomicU64,
    pub live: Mutex<HashMap<String, FrameChannel>>,
}

impl RunHost {
    pub fn is_running(&self) -> bool {
        self.active.lock().expect("active run").is_some()
    }

    pub fn cancel(&self) {
        if let Some(run) = self.active.lock().expect("active run").as_ref() {
            run.cancel.cancel();
        }
    }

    fn view(&self) -> std::sync::MutexGuard<'_, RunView> {
        self.view.lock().expect("run view")
    }

    fn log(&self) -> std::sync::MutexGuard<'_, RunLog> {
        self.log.lock().expect("run log")
    }

    fn changed(&self) {
        self.changes.fetch_add(1, Ordering::Relaxed);
    }

    fn state(&self) -> RunState {
        RunState {
            view: self.view().clone(),
            logs: self.log().lines(),
        }
    }
}

fn with_exports(app: &AppHandle, mut view: RunView) -> RunView {
    view.seq = app.state::<RunHost>().changes.load(Ordering::Relaxed);
    let nodes: Vec<(String, String)> = crate::document::document_workflow(&app.state::<AppState>())
        .map_or_default(|(_, wf)| wf.nodes.into_iter().map(|n| (n.id, n.kind)).collect());
    view.exports = exported_artifacts(
        nodes.iter().map(|(id, kind)| (id.as_str(), kind.as_str())),
        &view.progress.results,
    );
    view
}

fn emit_view(app: &AppHandle) {
    let view = app.state::<RunHost>().view().clone();
    if let Err(error) = RunViewChanged(with_exports(app, view)).emit(app) {
        log::warn!("sending the run state: {error}");
    }
}

fn emit_reset(app: &AppHandle) {
    let mut state = app.state::<RunHost>().state();
    state.view = with_exports(app, state.view);
    if let Err(error) = RunStateReset(state).emit(app) {
        log::warn!("sending the run state: {error}");
    }
}

pub fn reseed(app: &AppHandle, dir: &str, last_run: Option<&LastRun>) {
    let host = app.state::<RunHost>();
    let running_here = host
        .active
        .lock()
        .expect("active run")
        .as_ref()
        .is_some_and(|r| r.project_dir == dir);
    if running_here {
        return;
    }
    *host.view() = seeded_view(last_run);
    host.log().clear();
    host.changed();
    emit_reset(app);
}

pub fn forget_errors(app: &AppHandle, ids: &[String]) {
    let host = app.state::<RunHost>();
    if forget_run_errors(&mut host.view(), ids) {
        host.changed();
        emit_view(app);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartedRun {
    pub run_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LiveHeader<'a> {
    capture_id: &'a str,
    node_id: &'a str,
    width: f64,
    height: f64,
    t: f64,
}

struct AppLive {
    app: AppHandle,
}

impl LiveFeed for AppLive {
    fn send(&self, message: LiveMessage) {
        match message {
            LiveMessage::Frame(frame) => {
                let host = self.app.state::<RunHost>();
                let header = LiveHeader {
                    capture_id: &frame.capture_id,
                    node_id: &frame.node_id,
                    width: frame.width,
                    height: frame.height,
                    t: frame.t,
                };
                host.live
                    .lock()
                    .expect("live channels")
                    .retain(|_, channel| send(channel, &header, &frame.jpeg));
            }
            LiveMessage::Event(note) => {
                let noted = LiveNoted {
                    capture_id: note.capture_id,
                    node_id: note.node_id,
                    t: note.t,
                    event: note.event,
                };
                let watching: Vec<String> = self
                    .app
                    .state::<RunHost>()
                    .live
                    .lock()
                    .expect("live channels")
                    .keys()
                    .cloned()
                    .collect();
                for label in watching {
                    if let Err(error) = noted.clone().emit_to(&self.app, label.as_str()) {
                        log::warn!("sending a live event: {error}");
                    }
                }
            }
        }
    }
}

pub fn executors(app: &AppHandle) -> Executors {
    let state = app.state::<AppState>();
    let tools = engine_tools(&state);
    let engine = Arc::new(RenderEngine {
        tools: Arc::clone(&tools),
        film_bundle: state.paths.film_bundle.clone(),
    });
    let capture = Arc::new(CaptureDeps {
        tools: Arc::clone(&tools),
        sign_ins: Arc::new(ProjectSignIns { app: app.clone() }),
        login: Arc::new(StreamLogin {
            engine: engine_files(&state),
            temp_dir: state.paths.views_temp.clone(),
            host: Arc::new(AppViewHost { app: app.clone() }),
        }),
        live: Some(Arc::new(AppLive { app: app.clone() })),
    });
    let mut all = Executors::default();
    let lists = [
        media_executors(&tools),
        image_executors(),
        output_executors(&engine),
        capture_executors(&capture),
        opsis_terminal::executors::terminal_executors(),
    ];
    for list in lists {
        if let Err(error) = all.extend(list) {
            log::error!("{error}");
        }
    }
    all
}

fn finished_message(event: &RunEvent) -> Option<(String, String)> {
    let RunEvent::RunDone {
        ok,
        cancelled,
        error,
        outputs,
        ..
    } = event
    else {
        return None;
    };
    Some(if *ok {
        (
            "Export finished".to_owned(),
            if outputs.len() == 1 {
                "Your video is ready.".to_owned()
            } else {
                format!("{} files are ready.", outputs.len())
            },
        )
    } else if *cancelled {
        (
            "Run stopped".to_owned(),
            "The run was stopped before it finished.".to_owned(),
        )
    } else {
        (
            "Run failed".to_owned(),
            error
                .clone()
                .unwrap_or_else(|| "Something went wrong.".to_owned()),
        )
    })
}

fn notify_finished(app: &AppHandle, event: &RunEvent) {
    let focused = app
        .get_webview_window(MAIN_WINDOW)
        .and_then(|w| w.is_focused().ok())
        .unwrap_or(false);
    if focused {
        return;
    }
    if let Some((title, body)) = finished_message(event)
        && let Err(error) = app.notification().builder().title(title).body(body).show()
    {
        log::warn!("showing a notification: {error}");
    }
}

fn on_event(app: &AppHandle, run_id: &str, event: RunEvent) {
    let host = app.state::<RunHost>();
    let line = apply_to_view(&mut host.view(), &mut host.log(), &event, now_ms());
    host.changed();
    if let Some(line) = line
        && let Err(error) = RunLogged(line).emit(app)
    {
        log::warn!("sending a run log line: {error}");
    }
    if !matches!(event, RunEvent::NodeLog { .. }) {
        emit_view(app);
    }
    if let RunEvent::NodeDone {
        node_id,
        result: Some(result),
        ..
    } = &event
    {
        app.state::<AppState>()
            .editor()
            .results
            .insert(node_id.clone(), (**result).clone());
        crate::compile::request(app);
    }
    if matches!(event, RunEvent::RunDone { .. }) {
        let host = app.state::<RunHost>();
        let mut active = host.active.lock().expect("active run");
        if active.as_ref().is_some_and(|r| r.run_id == run_id) {
            *active = None;
        }
        drop(active);
        notify_finished(app, &event);
    }
    crate::api::run_event(app, &event);
    if let Err(error) = RunEventFired(event).emit(app) {
        log::warn!("sending a run event: {error}");
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    #[error("No project is open.")]
    NoProject,
    #[error("A run is already in progress.")]
    Busy,
    #[error("{}", crate::document::VERSION_UNCHOSEN)]
    VersionUnchosen,
    #[error("{0}")]
    Failed(String),
}

impl From<CommandError> for StartError {
    fn from(error: CommandError) -> Self {
        Self::Failed(error.0)
    }
}

pub async fn start(app: &AppHandle, targets: RunTargets) -> Result<StartedRun, StartError> {
    if !crate::document::save(app).await? {
        return Err(StartError::VersionUnchosen);
    }
    let state = app.state::<AppState>();
    let (dir, workflow) =
        crate::document::document_workflow(&state).ok_or(StartError::NoProject)?;
    let request = RunRequest::new(dir.to_string_lossy().into_owned(), workflow, targets);
    let host = app.state::<RunHost>();
    let run_id = uuid::Uuid::new_v4().to_string();
    let cancel = CancellationToken::new();
    {
        let mut active = host.active.lock().expect("active run");
        if active.is_some() {
            return Err(StartError::Busy);
        }
        *active = Some(ActiveRun {
            run_id: run_id.clone(),
            project_dir: request.project_dir.clone(),
            cancel: cancel.clone(),
        });
    }
    begin_run(&mut host.view(), &run_id);
    host.changed();
    emit_view(app);
    let temp_dir = make_child_temp(
        &state.paths.temp_root,
        &child_temp_name(ChildKind::Run, &run_id, std::process::id()),
    )
    .map_err(|e| StartError::Failed(e.to_string()))?;
    let settings = state.settings();
    let env = Arc::new(RunEnv {
        app_version: app.package_info().version.to_string(),
        engine: engine_files(&state),
        public_dir: state.paths.public_dir.clone(),
        temp_dir: temp_dir.clone(),
        settings,
    });
    let emitter = app.clone();
    let id = run_id.clone();
    let deps = RunDeps {
        env,
        executors: Arc::new(executors(app)),
        emit: Arc::new(move |event| on_event(&emitter, &id, event)),
        cancel,
    };
    let id = run_id.clone();
    tauri::async_runtime::spawn(async move {
        run_runner(&id, request, deps).await;
        remove_child_temp(&temp_dir);
    });
    Ok(StartedRun { run_id })
}

pub fn active_run(app: &AppHandle) -> Option<String> {
    app.state::<RunHost>()
        .active
        .lock()
        .expect("active run")
        .as_ref()
        .map(|r| r.run_id.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn run_workflow(app: AppHandle, targets: RunTargets) -> CommandResult<StartedRun> {
    Ok(start(&app, targets).await?)
}

#[tauri::command]
#[specta::specta]
pub fn cancel_run(host: State<'_, RunHost>) {
    host.cancel();
}

#[tauri::command]
#[specta::specta]
pub fn run_state(app: AppHandle) -> RunState {
    let mut state = app.state::<RunHost>().state();
    state.view = with_exports(&app, state.view);
    state
}

#[tauri::command]
#[specta::specta]
pub fn clear_run_log(app: AppHandle) {
    let host = app.state::<RunHost>();
    host.log().clear();
    host.changed();
    emit_reset(&app);
}

#[tauri::command]
#[specta::specta]
pub fn unwatch_live(host: State<'_, RunHost>, window: tauri::WebviewWindow, channel: u32) {
    let mut live = host.live.lock().expect("live channels");
    if live.get(window.label()).is_some_and(|c| c.id() == channel) {
        live.remove(window.label());
    }
}

#[tauri::command]
#[specta::specta]
pub fn watch_live(host: State<'_, RunHost>, window: tauri::WebviewWindow, frames: FrameChannel) {
    host.live
        .lock()
        .expect("live channels")
        .insert(window.label().to_owned(), frames);
    if window.label() == LIVE_WINDOW
        && let Err(error) = LiveWindowWatching(true).emit_to(window.app_handle(), MAIN_WINDOW)
    {
        log::warn!("telling the editor the live window is watching: {error}");
    }
}

#[tauri::command]
#[specta::specta]
pub fn live_window_watching(host: State<'_, RunHost>) -> bool {
    host.live
        .lock()
        .expect("live channels")
        .contains_key(LIVE_WINDOW)
}
