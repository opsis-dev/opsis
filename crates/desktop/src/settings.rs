use std::path::{Path, PathBuf};

use opsis_engine::licences::{
    LicenceKind, LicenceSource, is_licence_file, list_licences as licence_entries, shipped_credits,
};
use opsis_store::secrets::{list_secrets as project_secrets, read_secret, write_secret};
use opsis_store::settings::SettingsPatch;
use opsis_traces::read_trace as read_project_trace;
use opsis_types::engine::LicenceEntry;
use opsis_types::settings::{AppSettings, RUST_ONLY_SECTIONS};
use opsis_types::shell::{AppVersions, EnvStatus, MenuSection, ViewPrefChange};
use opsis_types::trace::TraceModel;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tauri_specta::Event;

use crate::engine::{engine_status, ffmpeg_licence};
use crate::error::{CommandError, CommandResult};
use crate::events::{SettingsChanged, ViewPrefChanged};
use crate::menu::app_menu;
use crate::state::AppState;

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
    if let Some(section) = RUST_ONLY_SECTIONS.iter().find(|s| patch.contains_key(**s)) {
        return Err(CommandError(format!(
            "The {section} settings are only changed by Opsis itself."
        )));
    }
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

#[tauri::command]
#[specta::specta]
pub fn get_environment(app: AppHandle, state: State<'_, AppState>) -> EnvStatus {
    EnvStatus {
        platform: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        is_packaged: state.paths.packaged,
        user_data: state.paths.user_data.to_string_lossy().into_owned(),
        versions: AppVersions {
            app: app.package_info().version.to_string(),
            tauri: tauri::VERSION.to_owned(),
            webview: tauri::webview_version().unwrap_or_default(),
        },
        engine: engine_status(&state),
    }
}

fn credit_roots(state: &AppState) -> Vec<PathBuf> {
    vec![
        state.paths.resources.join("resources"),
        state.paths.resources.clone(),
    ]
}

fn licence_sources(state: &AppState) -> Vec<LicenceSource> {
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
            path: shipped_credits(&credit_roots(state)).unwrap_or_default(),
        },
        LicenceSource {
            id: "ffmpeg",
            title: "FFmpeg",
            summary: "GNU General Public License v3. Separate programs that encode and mix the video.",
            kind: LicenceKind::Text,
            path: ffmpeg_licence(state).unwrap_or_default(),
        },
        LicenceSource {
            id: "compositor-ffmpeg",
            title: "FFmpeg in the video renderer",
            summary: "GNU General Public License v2 or later, and where to get its source",
            kind: LicenceKind::Text,
            path: resources.join("compositor").join("SOURCES.md"),
        },
        LicenceSource {
            id: "compositor-gpl",
            title: "GNU General Public License v2",
            summary: "The licence of the video renderer's FFmpeg",
            kind: LicenceKind::File,
            path: resources.join("compositor").join("COPYING.GPLv2"),
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
