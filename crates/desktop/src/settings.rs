use std::path::{Path, PathBuf};

use opsis_engine::install::install_engine as install_engine_files;
use opsis_engine::licences::{
    LicenceKind, LicenceSource, complete_engine_licences, is_licence_file,
    list_licences as licence_entries, shipped_credits_candidates,
};
use opsis_engine::paths::{CHROMIUM_CREDITS_FILE, engine_status};
use opsis_store::secrets::{list_secrets as project_secrets, read_secret, write_secret};
use opsis_store::settings::SettingsPatch;
use opsis_traces::read_trace as read_project_trace;
use opsis_types::engine::{EngineInstallEvent, EngineInstallUpdate, EnginePart, LicenceEntry};
use opsis_types::settings::AppSettings;
use opsis_types::shell::{AppVersions, Edition, EnvStatus, MenuSection, ViewPrefChange};
use opsis_types::trace::TraceModel;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tauri_specta::Event;
use tokio_util::sync::CancellationToken;

use crate::error::{CommandError, CommandResult};
use crate::events::{EngineInstallProgress, SettingsChanged, ViewPrefChanged};
use crate::menu::app_menu;
use crate::state::AppState;

const ENGINE_TASK: &str = "The engine is being set up.";

pub fn announce(app: &AppHandle, settings: AppSettings) {
    if let Err(error) = SettingsChanged(settings).emit(app) {
        log::warn!("sending the settings to the windows: {error}");
    }
    crate::api::follow_setting(app);
}

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    Ok(state.db().read_settings()?)
}

#[tauri::command]
#[specta::specta]
pub fn patch_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> CommandResult<AppSettings> {
    let (settings, _) = state.db().patch_settings(&patch)?;
    announce(&app, settings.clone());
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub fn set_view_pref(
    app: AppHandle,
    state: State<'_, AppState>,
    dir: String,
    key: String,
    value: Value,
) -> CommandResult<()> {
    if state.db().write_view_state(&dir, &key, &value)?
        && let Err(error) = ViewPrefChanged(ViewPrefChange { dir, key, value }).emit(&app)
    {
        log::warn!("sending a view setting to the windows: {error}");
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn app_menu_sections() -> Vec<MenuSection> {
    app_menu(cfg!(target_os = "macos"))
}

fn edition(state: &AppState) -> Edition {
    if !state.paths.packaged {
        return Edition::Dev;
    }
    if engine_status(
        &opsis_engine::paths::bundled_engine_dir(&state.paths.resources),
        state.paths.platform,
    )
    .complete
    {
        Edition::Full
    } else {
        Edition::Lite
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_environment(app: AppHandle, state: State<'_, AppState>) -> EnvStatus {
    EnvStatus {
        platform: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        is_packaged: state.paths.packaged,
        edition: edition(&state),
        user_data: state.paths.user_data.to_string_lossy().into_owned(),
        versions: AppVersions {
            app: app.package_info().version.to_string(),
            tauri: tauri::VERSION.to_owned(),
            webview: tauri::webview_version().unwrap_or_default(),
        },
        engine: engine_status(&state.paths.engine_dir(), state.paths.platform),
    }
}

fn credit_roots(state: &AppState) -> Vec<PathBuf> {
    vec![
        state.paths.resources.join("resources"),
        state.paths.resources.clone(),
    ]
}

pub fn complete_licences(state: &AppState) {
    let candidates = shipped_credits_candidates(&credit_roots(state));
    if let Err(error) =
        complete_engine_licences(&state.paths.engine_dir(), &candidates, state.paths.platform)
    {
        log::warn!("copying the recording browser's licences: {error}");
    }
}

fn engine_event(
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

#[tauri::command]
#[specta::specta]
pub async fn install_engine(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.tasks.begin(ENGINE_TASK);
    let emitter = app.clone();
    let mut report = move |update: EngineInstallUpdate| {
        if let Err(error) = EngineInstallProgress(engine_event(update, None, None)).emit(&emitter) {
            log::warn!("sending engine progress: {error}");
        }
    };
    let installed = install_engine_files(
        &state.paths.engine_dir(),
        state.paths.platform,
        &mut report,
        &CancellationToken::new(),
    )
    .await;
    state.tasks.end(ENGINE_TASK);
    let error = installed.as_ref().err().map(ToString::to_string);
    let finished = EngineInstallUpdate {
        part: EnginePart::HeadlessShell,
        step: "done".to_owned(),
        progress: if error.is_some() { None } else { Some(1.0) },
        message: error.clone().unwrap_or_else(|| "Engine ready".to_owned()),
    };
    if let Err(problem) =
        EngineInstallProgress(engine_event(finished, Some(true), error.clone())).emit(&app)
    {
        log::warn!("sending engine progress: {problem}");
    }
    match error {
        Some(error) => Err(CommandError(error)),
        None => {
            complete_licences(&state);
            Ok(())
        }
    }
}

fn licence_sources(state: &AppState) -> Vec<LicenceSource> {
    let engine = state.paths.engine_dir();
    let resources = &state.paths.resources;
    vec![
        LicenceSource {
            id: "opsis",
            title: "Opsis",
            summary: "Apache License 2.0",
            kind: LicenceKind::Text,
            path: resources.join("LICENSE"),
        },
        LicenceSource {
            id: "third-party",
            title: "Included source code",
            summary: "Notices for the components and code Opsis includes",
            kind: LicenceKind::Text,
            path: resources.join("THIRD_PARTY_NOTICES.md"),
        },
        LicenceSource {
            id: "engine-chromium",
            title: "Recording browser",
            summary: "Credits and licences of the Chromium build that records takes and exports videos",
            kind: LicenceKind::File,
            path: engine.join(CHROMIUM_CREDITS_FILE),
        },
        LicenceSource {
            id: "ffmpeg",
            title: "FFmpeg",
            summary: "GNU General Public License v3. Separate programs that encode and mix the video.",
            kind: LicenceKind::Text,
            path: engine.join("ffmpeg").join("LICENSE.txt"),
        },
    ]
}

#[tauri::command]
#[specta::specta]
pub fn list_licences(state: State<'_, AppState>) -> CommandResult<Vec<LicenceEntry>> {
    Ok(licence_entries(&licence_sources(&state))?)
}

#[tauri::command]
#[specta::specta]
pub fn open_licence_file(
    app: AppHandle,
    state: State<'_, AppState>,
    file: String,
) -> CommandResult<()> {
    if !is_licence_file(&licence_sources(&state), Path::new(&file)) {
        return Err(CommandError("That isn't one of the licence files.".into()));
    }
    Ok(app.opener().open_path(file, None::<&str>)?)
}

#[tauri::command]
#[specta::specta]
pub fn reveal_in_folder(
    app: AppHandle,
    dir: String,
    relative_path: Option<String>,
) -> CommandResult<()> {
    match relative_path {
        Some(rel) => Ok(app.opener().reveal_item_in_dir(Path::new(&dir).join(rel))?),
        None => Ok(app.opener().open_path(dir, None::<&str>)?),
    }
}

#[tauri::command]
#[specta::specta]
pub fn read_trace(dir: String, trace_id: String) -> CommandResult<TraceModel> {
    Ok(read_project_trace(Path::new(&dir), &trace_id)?)
}

#[tauri::command]
#[specta::specta]
pub fn list_secrets(dir: String) -> CommandResult<Vec<String>> {
    Ok(project_secrets(Path::new(&dir))?)
}

pub struct ProjectSignIns {
    pub app: AppHandle,
}

impl opsis_capture::executors::SignInStore for ProjectSignIns {
    fn read(&self, project_dir: &Path, name: &str) -> Result<Option<String>, String> {
        let state = self.app.state::<AppState>();
        let keys = state.keys().map_err(ToString::to_string)?;
        read_secret(keys, project_dir, name).map_err(|e| e.to_string())
    }

    fn write(&self, project_dir: &Path, name: &str, value: &str) -> Result<(), String> {
        let state = self.app.state::<AppState>();
        let keys = state.keys().map_err(ToString::to_string)?;
        write_secret(keys, project_dir, name, value).map_err(|e| e.to_string())
    }
}
