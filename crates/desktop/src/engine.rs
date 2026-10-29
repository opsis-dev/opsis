use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_capture::browser::check_browser;
use opsis_engine::install::install_engine as download_engine;
use opsis_engine::paths::{ShellPlatform, download_dir, engine_paths, find_headless_shell};
use opsis_engine::tools::{
    EngineFiles, ToolProblem, browser_warning, file_sha256, file_stamp, same_stamp, shown,
};
use opsis_media::capabilities::check_ffmpeg;
use opsis_media::executors::Tools;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_types::engine::{
    CheckedTool, EngineInstallEvent, EngineInstallUpdate, EnginePart, EngineStatus, EngineTool,
    ToolStatus,
};
use opsis_types::settings::EngineSection;
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;
use tokio_util::sync::CancellationToken;

use crate::error::{CommandError, CommandResult};
use crate::events::{EngineChanged, EngineInstallProgress};
use crate::settings::announce;
use crate::state::AppState;

const ENGINE_TASK: &str = "The engine is being set up.";

#[derive(Default)]
pub struct EngineChecks {
    problems: [Option<String>; 2],
    checking: [bool; 2],
}

fn slot(tool: EngineTool) -> usize {
    match tool {
        EngineTool::Browser => 0,
        EngineTool::Ffmpeg => 1,
    }
}

fn tool_status(checks: &EngineChecks, saved: Option<&CheckedTool>, tool: EngineTool) -> ToolStatus {
    let version = saved.map(|c| c.version.clone()).filter(|v| !v.is_empty());
    let problem = checks.problems[slot(tool)].clone();
    let warning = match (tool, &version, &problem) {
        (EngineTool::Browser, Some(v), None) => browser_warning(v),
        _ => None,
    };
    ToolStatus {
        path: saved.map(|c| c.path.clone()),
        version,
        problem,
        warning,
        checking: checks.checking[slot(tool)],
    }
}

fn saved_engine(state: &AppState) -> EngineSection {
    state.settings().engine
}

pub fn engine_status(state: &AppState) -> EngineStatus {
    let engine = saved_engine(state);
    let checks = state.engine_checks();
    let status = |tool| tool_status(&checks, engine.tool(tool), tool);
    let (browser, ffmpeg) = (status(EngineTool::Browser), status(EngineTool::Ffmpeg));
    EngineStatus {
        download_dir: shown(&download_dir(&state.paths.user_data)),
        complete: browser.ready() && ffmpeg.ready(),
        browser,
        ffmpeg,
    }
}

pub fn engine_files(state: &AppState) -> EngineFiles {
    let engine = saved_engine(state);
    let checks = state.engine_checks();
    EngineFiles::from_checked(&engine, |tool| {
        tool_status(&checks, engine.tool(tool), tool).ready()
    })
}

pub fn engine_tools(state: &AppState) -> Tools {
    Arc::new(Ffmpeg::new(&engine_files(state)))
}

fn send_status(app: &AppHandle) {
    let status = engine_status(&app.state::<AppState>());
    if let Err(error) = EngineChanged(status).emit(app) {
        log::warn!("sending the engine status: {error}");
    }
    crate::compile::request(app);
}

async fn save(
    app: &AppHandle,
    tool: EngineTool,
    checked: Option<CheckedTool>,
) -> CommandResult<()> {
    let state = app.state::<AppState>();
    let value = serde_json::to_value(&checked)?;
    let mut fields = Map::new();
    fields.insert(
        serde_json::to_value(tool)?
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        value,
    );
    let patch = Map::from_iter([("engine".to_owned(), Value::Object(fields))]);
    let settings = state.patch_settings(&patch).await?;
    announce(app, settings);
    Ok(())
}

fn set_check(app: &AppHandle, tool: EngineTool, checking: bool, problem: Option<String>) {
    let state = app.state::<AppState>();
    let mut checks = state.engine_checks();
    checks.checking[slot(tool)] = checking;
    checks.problems[slot(tool)] = problem;
}

async fn full_check(app: &AppHandle, tool: EngineTool, path: &Path) -> Result<String, ToolProblem> {
    let temp = app.state::<AppState>().paths.views_temp.clone();
    match tool {
        EngineTool::Browser => check_browser(path, &temp).await,
        EngineTool::Ffmpeg => check_ffmpeg(path).await,
    }
}

async fn hash_of(path: &Path) -> std::io::Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || file_sha256(&path))
        .await
        .map_err(std::io::Error::other)?
}

async fn hashed(path: &Path, version: String) -> Result<CheckedTool, ToolProblem> {
    let failed = |e: std::io::Error| ToolProblem::Failed(e.to_string());
    let sha256 = hash_of(path).await.map_err(failed)?;
    let stamp = file_stamp(path).map_err(failed)?;
    Ok(CheckedTool {
        path: shown(path),
        version,
        sha256,
        size: stamp.size,
        modified_ms: stamp.modified_ms,
    })
}

async fn check_and_save(app: &AppHandle, tool: EngineTool, path: &Path) -> Result<(), ToolProblem> {
    let previous = app.state::<AppState>().engine_checks().problems[slot(tool)].clone();
    set_check(app, tool, true, previous.clone());
    send_status(app);
    let checked = match full_check(app, tool, path).await {
        Ok(version) => hashed(path, version).await,
        Err(problem) => Err(problem),
    };
    let result = match checked {
        Ok(checked) => {
            set_check(app, tool, false, None);
            save(app, tool, Some(checked))
                .await
                .map_err(|e| ToolProblem::Failed(e.0))
        }
        Err(problem) => {
            set_check(app, tool, false, previous);
            Err(problem)
        }
    };
    send_status(app);
    result
}

async fn verify_changed(app: &AppHandle, tool: EngineTool, saved: CheckedTool) {
    let path = PathBuf::from(&saved.path);
    if let (Ok(hash), Ok(stamp)) = (hash_of(&path).await, file_stamp(&path))
        && hash == saved.sha256
    {
        let refreshed = CheckedTool {
            size: stamp.size,
            modified_ms: stamp.modified_ms,
            ..saved
        };
        set_check(app, tool, false, None);
        if let Err(error) = save(app, tool, Some(refreshed)).await {
            log::warn!("saving the {}'s file stamp: {}", tool.label(), error.0);
        }
        return;
    }
    if let Err(problem) = check_and_save(app, tool, &path).await {
        log::warn!(
            "the {} at {} failed its check: {problem}",
            tool.label(),
            path.display()
        );
        set_check(app, tool, false, Some(problem.to_string()));
    }
}

pub fn verify_on_start(app: &AppHandle) {
    let saved = saved_engine(&app.state::<AppState>());
    let mut changed = Vec::new();
    for tool in EngineTool::ALL {
        let Some(checked) = saved.tool(tool) else {
            continue;
        };
        match file_stamp(Path::new(&checked.path)) {
            Err(_) => {
                let missing = ToolProblem::NotFound(checked.path.clone());
                set_check(app, tool, false, Some(missing.to_string()));
            }
            Ok(stamp) if same_stamp(checked, stamp) => {}
            Ok(_) => {
                set_check(app, tool, true, None);
                changed.push((tool, checked.clone()));
            }
        }
    }
    if changed.is_empty() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        for (tool, checked) in changed {
            verify_changed(&app, tool, checked).await;
        }
        send_status(&app);
    });
}

fn progress_event(
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

fn downloaded_tools(dir: &Path, platform: ShellPlatform) -> [(EngineTool, Option<PathBuf>); 2] {
    let paths = engine_paths(dir, platform);
    [
        (
            EngineTool::Browser,
            find_headless_shell(&paths).map(Path::to_path_buf),
        ),
        (EngineTool::Ffmpeg, Some(paths.ffmpeg.clone())),
    ]
}

#[derive(Debug, thiserror::Error)]
enum SetupError {
    #[error(transparent)]
    Install(#[from] opsis_engine::install::InstallError),
    #[error("The {} wasn't in the download.", .0.label())]
    Missing(EngineTool),
    #[error(transparent)]
    Check(#[from] ToolProblem),
}

async fn download_and_check(app: &AppHandle) -> Result<(), SetupError> {
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
    for (tool, path) in downloaded_tools(&dir, state.paths.platform) {
        let path = path.ok_or(SetupError::Missing(tool))?;
        check_and_save(app, tool, &path).await?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn install_engine(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.tasks.begin(ENGINE_TASK);
    let result = download_and_check(&app).await;
    state.tasks.end(ENGINE_TASK);
    let error = result.as_ref().err().map(ToString::to_string);
    let finished = EngineInstallUpdate {
        part: EnginePart::HeadlessShell,
        step: "done".to_owned(),
        progress: if error.is_some() { None } else { Some(1.0) },
        message: error.clone().unwrap_or_else(|| "Engine ready".to_owned()),
    };
    if let Err(problem) =
        EngineInstallProgress(progress_event(finished, Some(true), error.clone())).emit(&app)
    {
        log::warn!("sending engine progress: {problem}");
    }
    Ok(result?)
}

async fn pick_file(app: &AppHandle, tool: EngineTool) -> Option<PathBuf> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(format!("Choose the {}", tool.label()))
        .pick_file(move |file| {
            if sender.send(file).is_err() {
                log::debug!("the file dialog closed after its caller left");
            }
        });
    receiver
        .await
        .ok()
        .flatten()
        .and_then(|f| f.into_path().ok())
}

#[tauri::command]
#[specta::specta]
pub async fn choose_engine_tool(app: AppHandle, tool: EngineTool) -> CommandResult<bool> {
    let Some(path) = pick_file(&app, tool).await else {
        return Ok(false);
    };
    check_and_save(&app, tool, &path)
        .await
        .map_err(|e| CommandError(e.to_string()))?;
    Ok(true)
}

#[tauri::command]
#[specta::specta]
pub fn get_engine_status(state: State<'_, AppState>) -> EngineStatus {
    engine_status(&state)
}

pub fn ffmpeg_licence(state: &AppState) -> Option<PathBuf> {
    let ffmpeg = saved_engine(state).ffmpeg?;
    Some(Path::new(&ffmpeg.path).parent()?.join("LICENSE.txt"))
}
