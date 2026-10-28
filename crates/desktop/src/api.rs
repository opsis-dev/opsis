use std::path::Path;
use std::sync::Arc;

use futures::future::BoxFuture;
use opsis_api::address::{API_SOCKET_ENV, Host, api_address};
use opsis_api::openapi::{ProjectState, RunState};
use opsis_api::routes::{ApiHost, HostError, HostResult};
use opsis_api::server::ApiServer;
use opsis_compile::check::workflow_problems_with;
use opsis_compile::edits::EditError;
use opsis_compile::graph::GraphIssue;
use opsis_document::Document;
use opsis_types::edits::{Edited, WorkflowEdit};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::shell::ApiServerStatus;
use opsis_types::workflow::Workflow;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;
use tokio::sync::broadcast;

use crate::document::DocumentError;
use crate::events::{ApiStatusChanged, OpenRequested};
use crate::run::{RunHost, StartError};
use crate::state::AppState;
use crate::windows::MAIN_WINDOW;

const RUN_EVENTS_KEPT: usize = 256;

pub struct Api {
    server: ApiServer,
    events: broadcast::Sender<String>,
}

impl Api {
    pub fn new(app: &AppHandle) -> Self {
        let state = app.state::<AppState>();
        let home = app.path().home_dir().unwrap_or_default();
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_default();
        let custom = std::env::var(API_SOCKET_ENV).ok();
        let address = api_address(&Host {
            windows: cfg!(windows),
            user_data: &state.paths.user_data,
            home: &home,
            username: &username,
            custom: custom.as_deref(),
        });
        Self {
            server: ApiServer::new(address),
            events: broadcast::Sender::new(RUN_EVENTS_KEPT),
        }
    }
}

const NOT_FOUND: u16 = 404;
const BAD_REQUEST: u16 = 400;
const CONFLICT: u16 = 409;
const UNREADABLE: u16 = 422;

fn document_failure(error: &DocumentError) -> HostError {
    let status = match error {
        DocumentError::NoProject => CONFLICT,
        DocumentError::Edit(
            EditError::NoNode(_)
            | EditError::NoEdge(_)
            | EditError::NoBus(_)
            | EditError::NoMarker(_),
        ) => NOT_FOUND,
        DocumentError::Edit(EditError::NodeExists(_) | EditError::NoNodeType(_)) => BAD_REQUEST,
        DocumentError::Edit(EditError::UnreadableKeys(_)) => UNREADABLE,
    };
    HostError::new(status, error.to_string())
}

fn start_failure(error: &StartError) -> HostError {
    let status = match error {
        StartError::NoProject | StartError::Busy | StartError::VersionUnchosen => CONFLICT,
        StartError::Failed(_) => 500,
    };
    HostError::new(status, error.to_string())
}

fn run_state(app: &AppHandle) -> RunState {
    let run_id = crate::run::active_run(app);
    RunState {
        running: run_id.is_some(),
        run_id,
    }
}

fn project_state(app: &AppHandle) -> HostResult<ProjectState> {
    let document = app
        .state::<AppState>()
        .editor()
        .document
        .as_ref()
        .map(Document::state)
        .ok_or_else(|| document_failure(&DocumentError::NoProject))?;
    let run = run_state(app);
    Ok(ProjectState {
        running: run.running,
        run_id: run.run_id,
        dir: Some(document.dir),
        workflow: document.workflow,
        revision: document.revision,
        saved: document.revision == document.saved_revision,
    })
}

struct AppApi {
    app: AppHandle,
}

impl ApiHost for AppApi {
    fn version(&self) -> String {
        self.app.package_info().version.to_string()
    }

    fn problems(&self, workflow: &Workflow) -> Vec<GraphIssue> {
        let results = self.app.state::<AppState>().editor().results.clone();
        let mut problems = workflow_problems_with(workflow, Some(&results));
        problems.extend(crate::node_results::file_problems(&self.app));
        problems
    }

    fn project(&self) -> HostResult<ProjectState> {
        project_state(&self.app)
    }

    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited> {
        crate::document::edit(&self.app, edit).map_err(|e| document_failure(&e))
    }

    fn undo(&self) -> HostResult<u64> {
        crate::document::step(&self.app, false).map_err(|e| document_failure(&e))
    }

    fn redo(&self) -> HostResult<u64> {
        crate::document::step(&self.app, true).map_err(|e| document_failure(&e))
    }

    fn save(&self) -> BoxFuture<'static, HostResult<u64>> {
        let app = self.app.clone();
        Box::pin(async move {
            let revision = project_state(&app)?.revision;
            match crate::document::save(&app).await {
                Ok(true) => Ok(revision),
                Ok(false) => Err(HostError::new(CONFLICT, crate::document::VERSION_UNCHOSEN)),
                Err(error) => Err(HostError::new(500, error.0)),
            }
        })
    }

    fn open(&self, dir: String) -> BoxFuture<'static, HostResult<ProjectState>> {
        let app = self.app.clone();
        Box::pin(async move {
            crate::document::open(&app, Path::new(&dir))
                .await
                .map_err(|e| {
                    HostError::new(UNREADABLE, format!("Couldn't open the project. {}", e.0))
                })?;
            if let Err(error) = (OpenRequested { dir }).emit_to(&app, MAIN_WINDOW) {
                log::warn!("showing the project the API opened: {error}");
            }
            project_state(&app)
        })
    }

    fn run(&self, targets: RunTargets) -> BoxFuture<'static, HostResult<RunState>> {
        let app = self.app.clone();
        Box::pin(async move {
            let started = crate::run::start(&app, targets)
                .await
                .map_err(|e| start_failure(&e))?;
            Ok(RunState {
                running: true,
                run_id: Some(started.run_id),
            })
        })
    }

    fn stop(&self) -> RunState {
        self.app.state::<RunHost>().cancel();
        run_state(&self.app)
    }

    fn run_events(&self) -> broadcast::Receiver<String> {
        self.app.state::<Api>().events.subscribe()
    }
}

fn announce(app: &AppHandle, status: ApiServerStatus) {
    if let Err(error) = ApiStatusChanged(status).emit(app) {
        log::warn!("sending the API status: {error}");
    }
}

pub fn follow_setting(app: &AppHandle) {
    let enabled = app.state::<AppState>().settings().api.enabled;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let api = app.state::<Api>();
        if api.server.status().enabled == enabled && (api.server.status().listening || !enabled) {
            return;
        }
        let status = api
            .server
            .set_enabled(enabled, Arc::new(AppApi { app: app.clone() }))
            .await;
        announce(&app, status);
    });
}

pub fn stop(app: &AppHandle) {
    app.state::<Api>().server.stop();
}

pub fn run_event(app: &AppHandle, event: &RunEvent) {
    let api = app.state::<Api>();
    if api.events.receiver_count() == 0 {
        return;
    }
    match serde_json::to_string(event) {
        Ok(json) => {
            if api.events.send(json).is_err() {
                log::debug!("no one follows the run events");
            }
        }
        Err(error) => log::warn!("writing a run event for the API: {error}"),
    }
}

#[tauri::command]
#[specta::specta]
pub fn api_status(api: State<'_, Api>) -> ApiServerStatus {
    api.server.status()
}
