use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_capture::browser::check_browser;
use opsis_engine::install::install_engine as download_engine;
use opsis_engine::paths::{EngineMissing, download_dir, engine_paths, find_headless_shell};
use opsis_engine::tools::{
    EngineFiles, ToolProblem, browser_warning, file_sha256, file_stamp, same_stamp, shown,
};
use opsis_media::executors::Tools;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_types::engine::{
    CheckedTool, EngineInstallEvent, EngineInstallUpdate, EngineStatus, ToolStatus,
};
use opsis_types::settings::EngineSection;
use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::error::{CommandError, CommandResult};
use crate::events::{Emit, EngineChanged, EngineInstallProgress};
use crate::settings::announce;
use crate::state::AppState;

const ENGINE_TASK: &str = "The engine is being set up.";
const BROWSER: &str = "recording browser";

#[derive(Debug, Default)]
pub struct EngineChecks {
    problem: Option<String>,
    checking: bool,
}

fn browser_status(checks: &EngineChecks, saved: Option<&CheckedTool>) -> ToolStatus {
    let version = saved.map(|c| c.version.clone()).filter(|v| !v.is_empty());
    let problem = checks.problem.clone();
    let warning = match (&version, &problem) {
        (Some(v), None) => browser_warning(v),
        _ => None,
    };
    ToolStatus {
        path: saved.map(|c| c.path.clone()),
        version,
        problem,
        warning,
        checking: checks.checking,
    }
}

fn saved_engine(state: &AppState) -> EngineSection {
    state.settings().engine
}

pub fn engine_status(state: &AppState) -> EngineStatus {
    let engine = saved_engine(state);
    let browser = browser_status(&state.engine_checks(), engine.browser.as_ref());
    EngineStatus {
        download_dir: shown(&download_dir(&state.paths.user_data)),
        complete: browser.ready(),
        browser,
    }
}

fn unusable(checks: &EngineChecks, saved: Option<&CheckedTool>) -> Option<EngineMissing> {
    if saved.is_none() {
        return Some(EngineMissing::NotSetUp);
    }
    if checks.checking {
        return Some(EngineMissing::Checking);
    }
    checks.problem.clone().map(EngineMissing::Failed)
}

pub fn engine_files(state: &AppState) -> EngineFiles {
    let engine = saved_engine(state);
    let unusable = unusable(&state.engine_checks(), engine.browser.as_ref());
    EngineFiles::from_checked(&engine, unusable)
}

async fn wait_for_check(
    mut checking: tokio::sync::watch::Receiver<bool>,
    cancel: &CancellationToken,
) {
    tokio::select! {
        waited = checking.wait_for(|busy| !*busy) => {
            if waited.is_err() {
                log::warn!("the engine check's channel closed");
            }
        }
        () = cancel.cancelled() => {}
    }
}

pub async fn checked_engine_files(state: &AppState, cancel: &CancellationToken) -> EngineFiles {
    wait_for_check(state.engine_checking.subscribe(), cancel).await;
    engine_files(state)
}

pub fn engine_checking(state: &AppState) -> bool {
    *state.engine_checking.borrow()
}

pub fn engine_tools(state: &AppState) -> Tools {
    Arc::new(Ffmpeg::new(&engine_files(state)))
}

fn send_status(app: &Session) {
    let status = engine_status(app.state::<AppState>());
    if let Err(error) = EngineChanged(status).emit(app) {
        log::warn!("sending the engine status: {error}");
    }
    crate::compile::request(app);
}

async fn save(app: &Session, checked: Option<CheckedTool>) -> CommandResult<()> {
    let state = app.state::<AppState>();
    let fields = Map::from_iter([("browser".to_owned(), serde_json::to_value(&checked)?)]);
    let patch = Map::from_iter([("engine".to_owned(), Value::Object(fields))]);
    let settings = state.patch_settings(&patch).await?;
    announce(app, settings);
    Ok(())
}

fn set_check(app: &Session, checking: bool, problem: Option<String>) {
    let state = app.state::<AppState>();
    {
        let mut checks = state.engine_checks();
        checks.checking = checking;
        checks.problem = problem;
    }
    state.engine_checking.send_replace(checking);
}

async fn hash_of(path: &Path) -> std::io::Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || file_sha256(&path))
        .await
        .map_err(std::io::Error::other)?
}

async fn hashed(
    path: &Path,
    version: String,
    known: Option<String>,
) -> Result<CheckedTool, ToolProblem> {
    let failed = |e: std::io::Error| ToolProblem::Failed(e.to_string());
    let sha256 = match known {
        Some(sha256) => sha256,
        None => hash_of(path).await.map_err(failed)?,
    };
    let stamp = file_stamp(path).map_err(failed)?;
    Ok(CheckedTool {
        path: shown(path),
        version,
        sha256,
        size: stamp.size,
        modified_ms: stamp.modified_ms,
    })
}

async fn check_and_save(
    app: &Session,
    path: &Path,
    known: Option<String>,
) -> Result<(), ToolProblem> {
    let previous = app.state::<AppState>().engine_checks().problem.clone();
    set_check(app, true, previous.clone());
    send_status(app);
    let temp = app.state::<AppState>().paths.views_temp.clone();
    let checked = match check_browser(path, &temp).await {
        Ok(version) => hashed(path, version, known).await,
        Err(problem) => Err(problem),
    };
    let result = match checked {
        Ok(checked) => {
            set_check(app, false, None);
            save(app, Some(checked))
                .await
                .map_err(|e| ToolProblem::Failed(e.0))
        }
        Err(problem) => {
            set_check(app, false, previous);
            Err(problem)
        }
    };
    send_status(app);
    result
}

async fn verify_changed(app: &Session, saved: CheckedTool) {
    let path = PathBuf::from(&saved.path);
    let hash = hash_of(&path).await.ok();
    if let (Some(hash), Ok(stamp)) = (&hash, file_stamp(&path))
        && *hash == saved.sha256
    {
        let refreshed = CheckedTool {
            size: stamp.size,
            modified_ms: stamp.modified_ms,
            ..saved
        };
        set_check(app, false, None);
        if let Err(error) = save(app, Some(refreshed)).await {
            log::warn!("saving the {BROWSER}'s file stamp: {}", error.0);
        }
        return;
    }
    if let Err(problem) = check_and_save(app, &path, hash).await {
        log::warn!(
            "the {BROWSER} at {} failed its check: {problem}",
            path.display()
        );
        set_check(app, false, Some(problem.to_string()));
    }
}

pub fn verify_on_start(app: &Session) {
    let Some(checked) = saved_engine(app.state::<AppState>()).browser else {
        return;
    };
    match file_stamp(Path::new(&checked.path)) {
        Err(_) => {
            let missing = ToolProblem::NotFound(checked.path.clone());
            set_check(app, false, Some(missing.to_string()));
        }
        Ok(stamp) if same_stamp(&checked, stamp) => {}
        Ok(_) => {
            set_check(app, true, None);
            let app = app.clone();
            crate::runtime::spawn(async move {
                verify_changed(&app, checked).await;
                send_status(&app);
            });
        }
    }
}

const fn progress_event(
    update: EngineInstallUpdate,
    done: Option<bool>,
    error: Option<String>,
) -> EngineInstallEvent {
    EngineInstallEvent {
        update,
        done,
        error,
    }
}

#[derive(Debug, thiserror::Error)]
enum SetupError {
    #[error(transparent)]
    Install(#[from] opsis_engine::install::InstallError),
    #[error("The recording browser wasn't in the download.")]
    Missing,
    #[error(transparent)]
    Check(#[from] ToolProblem),
}

async fn download_and_check(app: &Session) -> Result<(), SetupError> {
    let state = app.state::<AppState>();
    let dir = download_dir(&state.paths.user_data);
    let emitter = app.clone();
    let mut report = move |update: EngineInstallUpdate| {
        if let Err(error) = EngineInstallProgress(progress_event(update, None, None)).emit(&emitter)
        {
            log::warn!("sending engine progress: {error}");
        }
    };
    download_engine(
        &dir,
        state.paths.platform,
        &mut report,
        &CancellationToken::new(),
    )
    .await?;
    let paths = engine_paths(&dir, state.paths.platform);
    let browser = find_headless_shell(&paths).ok_or(SetupError::Missing)?;
    check_and_save(app, browser, None).await?;
    Ok(())
}
pub async fn install_engine(app: Session) -> CommandResult<()> {
    let state = app.state::<AppState>();
    state.tasks.begin(ENGINE_TASK);
    let result = download_and_check(&app).await;
    state.tasks.end(ENGINE_TASK);
    let error = result.as_ref().err().map(ToString::to_string);
    let finished = EngineInstallUpdate {
        step: "done".to_owned(),
        progress: if error.is_some() { None } else { Some(1.0) },
        message: error.clone().unwrap_or_else(|| "Engine ready".to_owned()),
    };
    if let Err(problem) =
        EngineInstallProgress(progress_event(finished, Some(true), error)).emit(&app)
    {
        log::warn!("sending engine progress: {problem}");
    }
    Ok(result?)
}

async fn pick_file(app: &Session) -> Option<PathBuf> {
    crate::platform::pick_file(app.platform(), &format!("Choose the {BROWSER}")).await
}
pub async fn choose_browser(app: Session) -> CommandResult<bool> {
    let Some(path) = pick_file(&app).await else {
        return Ok(false);
    };
    check_and_save(&app, &path, None)
        .await
        .map_err(|e| CommandError(e.to_string()))?;
    Ok(true)
}
pub fn get_engine_status(app: &Session) -> EngineStatus {
    let state = app.state::<AppState>();
    engine_status(state)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use opsis_types::engine::CheckedTool;
    use tokio::sync::watch;
    use tokio_util::sync::CancellationToken;

    use super::{EngineChecks, EngineMissing, unusable, wait_for_check};

    #[tokio::test]
    async fn a_run_waits_until_the_check_ends() {
        let checking = watch::Sender::new(true);
        let flip = checking.clone();
        let started = tokio::time::Instant::now();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(80)).await;
            flip.send_replace(false);
        });
        wait_for_check(checking.subscribe(), &CancellationToken::new()).await;
        assert!(started.elapsed() >= Duration::from_millis(80));
        assert!(!*checking.borrow());
    }

    #[tokio::test]
    async fn a_stopped_run_stops_waiting() {
        let checking = watch::Sender::new(true);
        let cancel = CancellationToken::new();
        cancel.cancel();
        wait_for_check(checking.subscribe(), &cancel).await;
        assert!(*checking.borrow());
    }

    #[test]
    fn says_why_the_browser_cannot_be_used() {
        let saved = CheckedTool {
            path: "/engine/chrome-headless-shell".to_owned(),
            version: "157.0.8080.0".to_owned(),
            sha256: String::new(),
            size: 0,
            modified_ms: 0,
        };
        let checks = |checking: bool, problem: Option<&str>| EngineChecks {
            problem: problem.map(str::to_owned),
            checking,
        };
        assert_eq!(
            unusable(&checks(false, None), None),
            Some(EngineMissing::NotSetUp)
        );
        assert_eq!(
            unusable(&checks(true, None), Some(&saved)),
            Some(EngineMissing::Checking)
        );
        assert_eq!(
            unusable(&checks(false, Some("It crashed.")), Some(&saved)),
            Some(EngineMissing::Failed("It crashed.".to_owned()))
        );
        assert_eq!(unusable(&checks(false, None), Some(&saved)), None);
        assert_eq!(
            EngineMissing::Checking.to_string(),
            "The recording browser is still being checked. Try again in a moment."
        );
    }
}
