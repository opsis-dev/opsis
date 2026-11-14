use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, capture_executors};
use opsis_capture::live::{LiveFeed, LiveFrame, LiveMessage, LiveNote};
use opsis_compile::terminal::default_look;
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
use opsis_types::events::{LogLevel, RunEvent, RunRequest, RunState, RunTargets, RunView};
use opsis_types::live::LiveEvent;
use opsis_types::values::LastRun;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::engine::{engine_files, engine_tools};
use crate::error::{CommandError, CommandResult};
use crate::events::{Emit, LiveNoted, RunEventFired, RunLogged, RunStateReset, RunViewChanged};
use crate::frames::{FrameChannel, send};
use crate::settings::ProjectSignIns;
use crate::state::AppState;
use crate::terminal_paint::PaintQueue;
use crate::views::AppViewHost;

#[derive(Debug)]
struct ActiveRun {
    run_id: String,
    project_dir: String,
    cancel: CancellationToken,
}

#[derive(Debug, Default)]
pub struct RunHost {
    active: Mutex<Option<ActiveRun>>,
    view: Mutex<RunView>,
    log: Mutex<RunLog>,
    changes: AtomicU64,
    pub live: Mutex<HashMap<String, FrameChannel>>,
    live_notes: Mutex<Vec<LiveNoted>>,
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

fn with_exports(app: &Session, mut view: RunView) -> RunView {
    view.seq = app.state::<RunHost>().changes.load(Ordering::Relaxed);
    let state = app.state::<AppState>();
    let editor = state.editor();
    view.exports = editor.document.as_ref().map_or_default(|doc| {
        exported_artifacts(
            doc.workflow()
                .nodes
                .iter()
                .map(|n| (n.id.as_str(), n.kind.as_str())),
            &view.progress.results,
        )
    });
    drop(editor);
    view
}

fn emit_view(app: &Session) {
    let view = app.state::<RunHost>().view().clone();
    if let Err(error) = RunViewChanged(with_exports(app, view)).emit(app) {
        log::warn!("sending the run state: {error}");
    }
}

fn emit_reset(app: &Session) {
    let mut state = app.state::<RunHost>().state();
    state.view = with_exports(app, state.view);
    if let Err(error) = RunStateReset(state).emit(app) {
        log::warn!("sending the run state: {error}");
    }
}

pub fn reseed(app: &Session, dir: &str, last_run: Option<&LastRun>) {
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

pub fn forget_errors(app: &Session, ids: &[String]) {
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

#[derive(Clone)]
struct AppLive {
    app: Session,
}

struct TerminalLive {
    live: AppLive,
    painting: Mutex<HashMap<String, PaintQueue<f64>>>,
}

impl TerminalLive {
    fn painter(&self, capture_id: &str, node_id: &str) -> PaintQueue<f64> {
        let live = self.live.clone();
        let (capture, node) = (capture_id.to_owned(), node_id.to_owned());
        PaintQueue::start(&self.live.app, default_look(None), move |size, jpeg, t| {
            live.send(LiveMessage::Frame(LiveFrame {
                capture_id: capture.clone(),
                node_id: node.clone(),
                jpeg,
                width: f64::from(size.width),
                height: f64::from(size.height),
                t,
            }));
        })
    }
}

impl opsis_terminal::live_feed::LiveFeed for TerminalLive {
    fn frame(&self, capture_id: &str, node_id: &str, frames: Vec<u8>, t: f64) {
        self.painting
            .lock()
            .expect("live painting")
            .entry(capture_id.to_owned())
            .or_insert_with(|| self.painter(capture_id, node_id))
            .offer(frames, t);
    }

    fn event(&self, capture_id: &str, node_id: &str, event: LiveEvent, t: f64) {
        if matches!(event, LiveEvent::End { .. }) {
            self.painting
                .lock()
                .expect("live painting")
                .remove(capture_id);
        }
        self.live.send(LiveMessage::Event(LiveNote {
            capture_id: capture_id.to_owned(),
            node_id: node_id.to_owned(),
            t,
            event,
        }));
    }
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
                if matches!(
                    noted.event,
                    LiveEvent::Start { .. } | LiveEvent::Step { .. } | LiveEvent::End { .. }
                ) {
                    self.app
                        .state::<RunHost>()
                        .live_notes
                        .lock()
                        .expect("live notes")
                        .push(noted.clone());
                }
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

pub fn executors(app: &Session, temp: &Path) -> Executors {
    let state = app.state::<AppState>();
    let tools = engine_tools(state, temp);
    let engine = Arc::new(RenderEngine {
        tools: Arc::clone(&tools),
    });
    let capture = Arc::new(CaptureDeps {
        tools: Arc::clone(&tools),
        sign_ins: Arc::new(ProjectSignIns { app: app.clone() }),
        login: Arc::new(StreamLogin {
            engine: engine_files(state),
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
        opsis_terminal::executors::terminal_executors(&opsis_terminal::executors::TerminalDeps {
            live_tapes: Some(Arc::new(crate::terminal_live::AppLiveTapes {
                app: app.clone(),
            })),
            live_frames: Some(Arc::new(TerminalLive {
                live: AppLive { app: app.clone() },
                painting: Mutex::new(HashMap::new()),
            })),
            previews: Some(Arc::new(opsis_render::preview::TakePreviews::new(
                Arc::clone(&engine),
            ))),
        }),
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

fn notify_finished(app: &Session, event: &RunEvent) {
    if app.platform().editor_focused() {
        return;
    }
    if let Some((title, body)) = finished_message(event) {
        app.platform().notify(&title, &body);
    }
}

fn on_event(app: &Session, run_id: &str, event: RunEvent) {
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

pub async fn start(app: &Session, targets: RunTargets) -> Result<StartedRun, StartError> {
    if !crate::document::save(app).await? {
        return Err(StartError::VersionUnchosen);
    }
    let state = app.state::<AppState>();
    let (dir, workflow) = crate::document::document_workflow(state).ok_or(StartError::NoProject)?;
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
    host.live_notes.lock().expect("live notes").clear();
    host.changed();
    emit_view(app);
    let temp_dir = make_child_temp(
        &state.paths.temp_root,
        &child_temp_name(ChildKind::Run, &run_id, std::process::id()),
    )
    .map_err(|e| StartError::Failed(e.to_string()))?;
    if crate::engine::engine_checking(state) {
        on_event(
            app,
            &run_id,
            RunEvent::NodeLog {
                run_id: run_id.clone(),
                node_id: None,
                level: LogLevel::Info,
                message: "Waiting for the recording browser's check to finish.".to_owned(),
                at: now_ms(),
            },
        );
    }
    let engine = crate::engine::checked_engine_files(state, &cancel).await;
    let settings = state.settings();
    let env = Arc::new(RunEnv {
        app_version: crate::APP_VERSION.to_owned(),
        engine,
        public_dir: state.paths.public_dir.clone(),
        temp_dir: temp_dir.clone(),
        settings,
    });
    let emitter = app.clone();
    let id = run_id.clone();
    let deps = RunDeps {
        env,
        executors: Arc::new(executors(app, &temp_dir)),
        emit: Arc::new(move |event| on_event(&emitter, &id, event)),
        cancel,
    };
    let id = run_id.clone();
    crate::runtime::spawn(async move {
        run_runner(&id, request, deps).await;
        remove_child_temp(&temp_dir);
    });
    Ok(StartedRun { run_id })
}

pub fn active_run(app: &Session) -> Option<String> {
    app.state::<RunHost>()
        .active
        .lock()
        .expect("active run")
        .as_ref()
        .map(|r| r.run_id.clone())
}
pub async fn run_workflow(app: Session, targets: RunTargets) -> CommandResult<StartedRun> {
    Ok(start(&app, targets).await?)
}
pub fn cancel_run(app: &Session) {
    let host = app.state::<RunHost>();
    host.cancel();
}
pub fn run_state(app: &Session) -> RunState {
    let mut state = app.state::<RunHost>().state();
    state.view = with_exports(app, state.view);
    state
}
pub fn clear_run_log(app: &Session) {
    let host = app.state::<RunHost>();
    host.log().clear();
    host.changed();
    emit_reset(app);
}
pub fn watch_live(app: &Session, window: &str, frames: FrameChannel) -> Vec<LiveNoted> {
    let host = app.state::<RunHost>();
    host.live
        .lock()
        .expect("live channels")
        .insert(window.to_owned(), frames);
    host.live_notes.lock().expect("live notes").clone()
}
