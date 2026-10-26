use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opsis_capture::executors::{CaptureDeps, capture_executors};
use opsis_capture::live::{LiveFeed, LiveMessage};
use opsis_engine::temp::{ChildKind, child_temp_name, make_child_temp, remove_child_temp};
use opsis_image::executors::image_executors;
use opsis_media::executors::media_executors;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_render::export_project::ExportSources;
use opsis_runner::context::{Executors, RunEnv};
use opsis_runner::progress::apply_run_event;
use opsis_runner::runner::{RunDeps, run_workflow as run_runner};
use opsis_stream::sign_in::StreamLogin;
use opsis_types::events::{RunEvent, RunProgress, RunRequest};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;
use tokio_util::sync::CancellationToken;

use crate::error::{CommandError, CommandResult};
use crate::events::{LiveNoted, RunEventFired};
use crate::frames::{FrameChannel, send};
use crate::settings::ProjectSignIns;
use crate::state::AppState;
use crate::views::AppViewHost;
use crate::windows::MAIN_WINDOW;

struct ActiveRun {
    run_id: String,
    project_dir: String,
    cancel: CancellationToken,
}

#[derive(Default)]
pub struct RunHost {
    active: Mutex<Option<ActiveRun>>,
    progress: Mutex<Option<RunProgress>>,
    live: Mutex<HashMap<String, FrameChannel>>,
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

    fn record(&self, event: &RunEvent) {
        let mut progress = self.progress.lock().expect("run progress");
        match event {
            RunEvent::RunDone { .. } => *progress = None,
            RunEvent::RunStart { run_id, .. } => {
                let mut next = RunProgress {
                    run_id: Some(run_id.clone()),
                    ..RunProgress::default()
                };
                apply_run_event(&mut next, event);
                *progress = Some(next);
            }
            _ => {
                if let Some(current) = progress.as_mut() {
                    apply_run_event(current, event);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RunStatus {
    pub run_id: Option<String>,
    pub project_dir: Option<String>,
    pub progress: RunProgress,
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
                if let Err(error) = (LiveNoted {
                    capture_id: note.capture_id,
                    node_id: note.node_id,
                    t: note.t,
                    event: note.event,
                })
                .emit(&self.app)
                {
                    log::warn!("sending a live event: {error}");
                }
            }
        }
    }
}

fn executors(app: &AppHandle) -> Executors {
    let state = app.state::<AppState>();
    let tools = Arc::clone(&state.tools);
    let engine = Arc::new(RenderEngine {
        sidecar: Arc::clone(&state.sidecar),
        tools: Arc::clone(&tools),
        serve_url: state.paths.video_bundle.to_string_lossy().into_owned(),
        binaries_directory: state
            .paths
            .compositor()
            .map(|dir| dir.to_string_lossy().into_owned()),
        sources: ExportSources {
            root: state.paths.export_sources(),
            packaged: state.paths.packaged,
        },
    });
    let capture = Arc::new(CaptureDeps {
        tools: Arc::clone(&tools),
        sign_ins: Arc::new(ProjectSignIns { app: app.clone() }),
        login: Arc::new(StreamLogin {
            engine_dir: state.paths.engine_dir(),
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
    app.state::<RunHost>().record(&event);
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
        crate::compile::recompile(app);
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

#[tauri::command]
#[specta::specta]
pub async fn run_workflow(
    app: AppHandle,
    host: State<'_, RunHost>,
    state: State<'_, AppState>,
    request: RunRequest,
) -> CommandResult<StartedRun> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let cancel = CancellationToken::new();
    {
        let mut active = host.active.lock().expect("active run");
        if active.is_some() {
            return Err(CommandError("A run is already in progress.".into()));
        }
        *active = Some(ActiveRun {
            run_id: run_id.clone(),
            project_dir: request.project_dir.clone(),
            cancel: cancel.clone(),
        });
    }
    let temp_dir = make_child_temp(
        &state.paths.temp_root,
        &child_temp_name(ChildKind::Run, &run_id, std::process::id()),
    )?;
    let settings = state.db().read_settings()?;
    let env = Arc::new(RunEnv {
        app_version: app.package_info().version.to_string(),
        engine_dir: state.paths.engine_dir(),
        public_dir: state.paths.public_dir.clone(),
        temp_dir: temp_dir.clone(),
        settings,
    });
    let emitter = app.clone();
    let id = run_id.clone();
    let deps = RunDeps {
        env,
        executors: Arc::new(executors(&app)),
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

#[tauri::command]
#[specta::specta]
pub fn cancel_run(host: State<'_, RunHost>) {
    host.cancel();
}

#[tauri::command]
#[specta::specta]
pub fn run_status(host: State<'_, RunHost>) -> RunStatus {
    let active = host.active.lock().expect("active run");
    let run_id = active.as_ref().map(|r| r.run_id.clone());
    let project_dir = active.as_ref().map(|r| r.project_dir.clone());
    drop(active);
    let progress = host
        .progress
        .lock()
        .expect("run progress")
        .clone()
        .filter(|p| p.run_id == run_id)
        .unwrap_or_else(|| RunProgress {
            run_id: run_id.clone(),
            ..RunProgress::default()
        });
    RunStatus {
        run_id,
        project_dir,
        progress,
    }
}

#[tauri::command]
#[specta::specta]
pub fn watch_live(host: State<'_, RunHost>, window: tauri::WebviewWindow, frames: FrameChannel) {
    host.live
        .lock()
        .expect("live channels")
        .insert(window.label().to_owned(), frames);
}
