use std::path::PathBuf;
use std::sync::Arc;

use opsis_capture::executors::SignInStore;
use opsis_capture::held::{BrowserClosed, HeldBrowser, HoldFailed, hold};
use opsis_capture::take::{SignInError, SignIns, TakeEnv};
use opsis_compile::compile::compile_workflow;
use opsis_compile::node::SESSION_PORT;
use opsis_types::values::{ActionResult, ActionState, BrowserSessionValue, CompileValue};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::settings::ProjectSignIns;
use crate::state::AppState;

pub const BROWSER_TASK: &str = "A browser open for the agent";

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("No project is open.")]
    NoProject,
    #[error("No browser is open.")]
    NotOpen,
    #[error("{0} isn't a browser action or an Open browser node in this project.")]
    NotAnAction(String),
    #[error(
        "{0} doesn't come after the open browser's actions. Open the browser again on its flow."
    )]
    NotAhead(String),
    #[error("{0}")]
    Engine(String),
    #[error(transparent)]
    Failed(#[from] HoldFailed),
    #[error(transparent)]
    Closed(#[from] BrowserClosed),
}

#[derive(Debug)]
struct Held {
    browser: HeldBrowser,
    flow: String,
    played: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AgentBrowser {
    held: Mutex<Option<Held>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserState {
    pub address: String,
    pub flow: String,
    pub at: String,
    pub page: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checked {
    pub actions: Vec<ActionResult>,
    pub at: String,
}

struct SignInsOf {
    store: ProjectSignIns,
    project_dir: PathBuf,
}

impl SignIns for SignInsOf {
    fn read(&self, name: &str) -> Result<Option<String>, SignInError> {
        self.store.read(&self.project_dir, name)
    }
}

fn session_for(app: &Session, action: &str) -> Result<BrowserSessionValue, BrowserError> {
    let state = app.state::<AppState>();
    let (_, workflow) = crate::document::document_workflow(state).ok_or(BrowserError::NoProject)?;
    let results = state.editor().results.clone();
    let compiled = compile_workflow(&workflow, Some(&results));
    match compiled
        .values
        .get(action)
        .and_then(|ports| ports.get(SESSION_PORT))
    {
        Some(CompileValue::BrowserSession(session)) => Ok(session.clone()),
        _ => Err(BrowserError::NotAnAction(action.to_owned())),
    }
}

fn take_env(app: &Session) -> Result<TakeEnv, BrowserError> {
    let state = app.state::<AppState>();
    let project_dir = crate::document::document_dir(state).ok_or(BrowserError::NoProject)?;
    let executable = crate::engine::engine_files(state)
        .browser()
        .map_err(|missing| BrowserError::Engine(missing.to_string()))?
        .to_path_buf();
    Ok(TakeEnv {
        sign_ins: Arc::new(SignInsOf {
            store: ProjectSignIns { app: app.clone() },
            project_dir: project_dir.clone(),
        }),
        project_dir,
        executable,
        temp_dir: state.paths.views_temp.clone(),
        live: None,
        trace: false,
        cancel: CancellationToken::new(),
        log: Arc::new(|level, message| log::info!("agent browser ({level:?}): {message}")),
        step: Arc::new(|_, _, _| {}),
    })
}

fn last_of(held: &Held) -> String {
    held.played
        .last()
        .cloned()
        .unwrap_or_else(|| held.flow.clone())
}

async fn state_of(held: &Held) -> Result<BrowserState, BrowserError> {
    Ok(BrowserState {
        address: held.browser.address().to_owned(),
        flow: held.flow.clone(),
        at: last_of(held),
        page: held.browser.page().await?,
    })
}

pub async fn open(app: &Session, up_to: &str) -> Result<BrowserState, BrowserError> {
    let session = session_for(app, up_to)?;
    let env = take_env(app)?;
    close(app).await;
    let browser = hold(env, session.clone()).await?;
    let held = Held {
        browser,
        flow: session.browser.node_id.clone(),
        played: session.steps.iter().map(|step| step.id.clone()).collect(),
    };
    let shown = state_of(&held).await?;
    *app.state::<AgentBrowser>().held.lock().await = Some(held);
    app.state::<AppState>().tasks.begin(BROWSER_TASK);
    Ok(shown)
}

pub async fn state(app: &Session) -> Result<BrowserState, BrowserError> {
    let held = app.state::<AgentBrowser>().held.lock().await;
    state_of(held.as_ref().ok_or(BrowserError::NotOpen)?).await
}

pub async fn check(app: &Session, up_to: &str) -> Result<Checked, BrowserError> {
    let mut guard = app.state::<AgentBrowser>().held.lock().await;
    let held = guard.as_mut().ok_or(BrowserError::NotOpen)?;
    let session = session_for(app, up_to)?;
    let ahead = session.browser.node_id == held.flow
        && session.steps.len() >= held.played.len()
        && session
            .steps
            .iter()
            .zip(&held.played)
            .all(|(step, played)| step.id == *played);
    if !ahead {
        return Err(BrowserError::NotAhead(up_to.to_owned()));
    }
    let remaining = session.steps[held.played.len()..].to_vec();
    let actions = held.browser.check(remaining).await?;
    held.played.extend(
        actions
            .iter()
            .take_while(|action| action.state == ActionState::Passed)
            .map(|action| action.node_id.clone()),
    );
    Ok(Checked {
        actions,
        at: last_of(held),
    })
}

pub async fn close(app: &Session) {
    if app
        .state::<AgentBrowser>()
        .held
        .lock()
        .await
        .take()
        .is_some()
    {
        app.state::<AppState>().tasks.end(BROWSER_TASK);
    }
}
