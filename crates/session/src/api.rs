use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::future::BoxFuture;
use opsis_api::bodies::CreateProjectBody;
use opsis_api::openapi::{
    AboutReply, BrowserReply, CheckReply, FontsReply, PathsReply, ProjectPaths, ProjectState,
    RecentProject, RunState, TemplateReply,
};
use opsis_api::routes::{ApiHost, HostError, HostResult};
use opsis_api::server::ApiServer;
use opsis_compile::check::workflow_problems_with;
use opsis_compile::edits::EditError;
use opsis_compile::graph::GraphIssue;
use opsis_document::Document;
use opsis_engine::paths::download_dir;
use opsis_engine::tools::shown;
use opsis_folders::address::local_api_address;
use opsis_persistence::logs::assistant_logs;
use opsis_project::last_run::read_last_run as read_project_last_run;
use opsis_types::edits::{Edited, WorkflowEdit};
use opsis_types::engine::EngineStatus;
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::project::{CACHE_DIR, LAST_RUN_FILE, OUT_DIR, TRACES_DIR};
use opsis_types::script::{ScriptOutcome, ScriptRequest, ScriptStatus};
use opsis_types::settings::AppSettings;
use opsis_types::shell::ApiServerStatus;
use opsis_types::trace::TraceHeader;
use opsis_types::values::LastRun;
use opsis_types::workflow::Workflow;
use tokio::sync::broadcast;

use crate::Session;
use crate::document::DocumentError;
use crate::events::{ApiStatusChanged, Emit, MAIN_WINDOW, OpenRequested};
use crate::run::{RunHost, StartError};
use crate::state::AppState;

const RUN_EVENTS_KEPT: usize = 256;

#[derive(Debug)]
pub struct Api {
    server: ApiServer,
    events: broadcast::Sender<String>,
}

impl Api {
    pub fn new(state: &AppState) -> Self {
        let address = local_api_address(&state.paths.user_data, &state.paths.home);
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
fn script_failure(error: &crate::script::ScriptStartError) -> HostError {
    use crate::script::ScriptStartError;
    let status = match error {
        ScriptStartError::NoProject | ScriptStartError::Busy => CONFLICT,
        ScriptStartError::NotAScript(_) => BAD_REQUEST,
        ScriptStartError::Unreadable(_) => NOT_FOUND,
    };
    HostError::new(status, error.to_string())
}

fn document_failure(error: &DocumentError) -> HostError {
    let status = match error {
        DocumentError::NoProject | DocumentError::ScriptRunning => CONFLICT,
        DocumentError::Edit(
            EditError::NoNode(_)
            | EditError::NoEdge(_)
            | EditError::NoBus(_)
            | EditError::NoMarker(_),
        ) => NOT_FOUND,
        DocumentError::Edit(EditError::NodeExists(_) | EditError::NoNodeType(_))
        | DocumentError::Refused(_) => BAD_REQUEST,
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

fn run_state(app: &Session) -> RunState {
    let run_id = crate::run::active_run(app);
    RunState {
        running: run_id.is_some(),
        run_id,
    }
}

fn project_state(app: &Session) -> HostResult<ProjectState> {
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
    app: Session,
}

impl ApiHost for AppApi {
    fn version(&self) -> String {
        crate::APP_VERSION.to_owned()
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
            open_and_show(&app, Path::new(&dir))
                .await
                .map_err(|e| HostError::new(UNREADABLE, e))?;
            project_state(&app)
        })
    }

    fn run(
        &self,
        targets: RunTargets,
        session: Option<String>,
    ) -> BoxFuture<'static, HostResult<RunState>> {
        let app = self.app.clone();
        Box::pin(async move {
            let started = crate::run::start(&app, targets, session)
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

    fn run_script(
        &self,
        request: ScriptRequest,
        session: Option<String>,
    ) -> BoxFuture<'static, HostResult<ScriptOutcome>> {
        let app = self.app.clone();
        Box::pin(async move {
            let source = match (request.file, request.source) {
                (Some(file), _) => crate::script::ScriptSource::File(file),
                (None, source) => crate::script::request_source(source.unwrap_or_default()),
            };
            let outcome =
                crate::script::run_script(&app, source, session).map_err(|e| script_failure(&e))?;
            outcome
                .await
                .map_err(|_| HostError::new(500, "The script ended without an answer."))
        })
    }

    fn stop_script(&self) -> HostResult<ScriptStatus> {
        Ok(crate::script::stop_script(&self.app))
    }

    fn script_status(&self) -> HostResult<ScriptStatus> {
        project_state(&self.app)?;
        Ok(crate::script::script_status(&self.app))
    }

    fn settings(&self) -> AppSettings {
        self.app.state::<AppState>().settings()
    }

    fn engine(&self) -> EngineStatus {
        crate::engine::get_engine_status(&self.app)
    }

    fn fonts(&self) -> BoxFuture<'static, HostResult<FontsReply>> {
        let app = self.app.clone();
        Box::pin(async move {
            let state = app.state::<AppState>();
            let library = state
                .fonts
                .list(state.db())
                .await
                .map_err(|e| HostError::new(500, e.to_string()))?;
            let computer = crate::fonts::installed_font_families()
                .await
                .map_err(|e| HostError::new(500, e.0))?
                .into_iter()
                .map(|family| family.family)
                .collect();
            Ok(FontsReply { library, computer })
        })
    }

    fn paths(&self) -> PathsReply {
        let state = self.app.state::<AppState>();
        let user_data = &state.paths.user_data;
        let project = crate::document::document_dir(state).map(|dir| ProjectPaths {
            dir: shown(&dir),
            out: shown(&dir.join(OUT_DIR)),
            traces: shown(&dir.join(TRACES_DIR)),
            cache: shown(&dir.join(CACHE_DIR)),
            last_run: shown(&dir.join(LAST_RUN_FILE)),
        });
        PathsReply {
            data: shown(user_data),
            engine: shown(&download_dir(user_data)),
            assistant_logs: shown(&assistant_logs(user_data)),
            project,
        }
    }

    fn about(&self) -> AboutReply {
        let state = self.app.state::<AppState>();
        AboutReply {
            version: crate::APP_VERSION.to_owned(),
            commit: opsis_update::COMMIT.to_owned(),
            platform: format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
            packaged: state.paths.packaged,
            browser: crate::engine::get_engine_status(&self.app).browser.version,
        }
    }

    fn templates(&self) -> Vec<TemplateReply> {
        crate::projects::list_templates()
            .into_iter()
            .map(|template| TemplateReply {
                id: template.id,
                title: template.title,
                description: template.description,
            })
            .collect()
    }

    fn projects(&self) -> BoxFuture<'static, HostResult<Vec<RecentProject>>> {
        let app = self.app.clone();
        Box::pin(async move {
            Ok(app
                .state::<AppState>()
                .db()
                .list_recent()
                .await
                .map_err(|e| HostError::new(500, e.to_string()))?
                .into_iter()
                .map(|recent| RecentProject {
                    dir: recent.dir,
                    name: recent.name,
                    opened_at: recent.opened_at,
                })
                .collect())
        })
    }

    fn create_project(
        &self,
        body: CreateProjectBody,
    ) -> BoxFuture<'static, HostResult<ProjectState>> {
        let app = self.app.clone();
        Box::pin(async move {
            let created = crate::projects::create_project(
                app.clone(),
                body.name,
                body.template,
                Some(body.dir),
            )
            .await
            .map_err(|e| HostError::new(UNREADABLE, e.0))?
            .ok_or_else(|| HostError::new(UNREADABLE, "The project wasn't created."))?;
            if let Err(error) = (OpenRequested { dir: created.dir }).emit_to(&app, MAIN_WINDOW) {
                log::warn!("showing the project that was created: {error}");
            }
            project_state(&app)
        })
    }

    fn last_run(&self) -> HostResult<Option<LastRun>> {
        let dir = open_dir(&self.app)?;
        read_project_last_run(&dir).map_err(|e| HostError::new(UNREADABLE, e.to_string()))
    }

    fn traces(&self) -> HostResult<Vec<TraceHeader>> {
        let dir = open_dir(&self.app)?;
        opsis_traces::list_traces(&dir).map_err(|e| HostError::new(UNREADABLE, e.to_string()))
    }

    fn open_browser(&self, up_to: String) -> BoxFuture<'static, HostResult<BrowserReply>> {
        let app = self.app.clone();
        Box::pin(async move {
            crate::agent_browser::open(&app, &up_to)
                .await
                .map(browser_reply)
                .map_err(|e| browser_failure(&e))
        })
    }

    fn browser(&self) -> BoxFuture<'static, HostResult<BrowserReply>> {
        let app = self.app.clone();
        Box::pin(async move {
            crate::agent_browser::state(&app)
                .await
                .map(browser_reply)
                .map_err(|e| browser_failure(&e))
        })
    }

    fn check_browser(&self, up_to: String) -> BoxFuture<'static, HostResult<CheckReply>> {
        let app = self.app.clone();
        Box::pin(async move {
            let checked = crate::agent_browser::check(&app, &up_to)
                .await
                .map_err(|e| {
                    let failure = browser_failure(&e);
                    if matches!(e, crate::agent_browser::BrowserError::NotOpen) {
                        HostError::new(CONFLICT, failure.message)
                    } else {
                        failure
                    }
                })?;
            Ok(CheckReply {
                actions: checked.actions,
                at: checked.at,
            })
        })
    }

    fn close_browser(&self) -> BoxFuture<'static, ()> {
        let app = self.app.clone();
        Box::pin(async move { crate::agent_browser::close(&app).await })
    }
}

fn browser_failure(error: &crate::agent_browser::BrowserError) -> HostError {
    use crate::agent_browser::BrowserError;
    let status = match error {
        BrowserError::NoProject | BrowserError::Engine(_) | BrowserError::Closed(_) => CONFLICT,
        BrowserError::NotOpen => NOT_FOUND,
        BrowserError::NotAnAction(_) | BrowserError::NotAhead(_) => BAD_REQUEST,
        BrowserError::Failed(_) => UNREADABLE,
    };
    HostError::new(status, error.to_string())
}

fn browser_reply(state: crate::agent_browser::BrowserState) -> BrowserReply {
    BrowserReply {
        address: state.address,
        flow: state.flow,
        at: state.at,
        page: state.page,
    }
}

fn open_dir(app: &Session) -> HostResult<PathBuf> {
    crate::document::document_dir(app.state::<AppState>())
        .ok_or_else(|| document_failure(&DocumentError::NoProject))
}

pub async fn open_and_show(app: &Session, dir: &Path) -> Result<(), String> {
    crate::document::open(app, dir)
        .await
        .map_err(|e| format!("Couldn't open the project. {}", e.0))?;
    let dir = dir.to_string_lossy().into_owned();
    if let Err(error) = (OpenRequested { dir }).emit_to(app, MAIN_WINDOW) {
        log::warn!("showing the project that was opened: {error}");
    }
    Ok(())
}

fn announce(app: &Session, status: ApiServerStatus) {
    if let Err(error) = ApiStatusChanged(status).emit(app) {
        log::warn!("sending the API status: {error}");
    }
}

pub fn follow_setting(app: &Session) {
    let enabled = app.state::<AppState>().settings().assistant.enabled || app.platform().headless();
    let app = app.clone();
    crate::runtime::spawn(async move {
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

pub fn stop(app: &Session) {
    app.state::<Api>().server.stop();
}

pub fn run_event(app: &Session, event: &RunEvent) {
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
pub fn subscribe_run_events(app: &Session) -> broadcast::Receiver<String> {
    app.state::<Api>().events.subscribe()
}

pub fn api_status(app: &Session) -> ApiServerStatus {
    let api = app.state::<Api>();
    api.server.status()
}
