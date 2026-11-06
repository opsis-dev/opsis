use std::path::{Path, PathBuf};

use opsis_capture::executors::SignInStore;
use opsis_capture::take::SignInError;
use opsis_compile::asset_ref::{RefTarget, ref_target};
use opsis_engine::ffmpeg::FFMPEG_LICENCE_FILE;
use opsis_engine::licences::{
    LicenceKind, LicenceSource, is_licence_file, list_licences as licence_entries, shipped,
    shipped_credits,
};
use opsis_store::secrets::{read_secret, write_secret};
use opsis_store::settings::SettingsPatch;
use opsis_traces::read_trace as read_project_trace;
use opsis_types::engine::LicenceEntry;
use opsis_types::settings::{AppSettings, RUST_ONLY_SECTIONS};
use opsis_types::shell::{AppVersions, EnvStatus, MenuSection, ViewPrefChange};
use opsis_types::trace::TraceModel;
use serde_json::Value;

use crate::Session;
use crate::engine::engine_status;
use crate::error::{CommandError, CommandResult};
use crate::events::{Emit, SettingsChanged, ViewPrefChanged};
use crate::menu::app_menu;
use crate::state::AppState;

pub fn announce(app: &Session, settings: AppSettings) {
    if let Err(error) = SettingsChanged(settings).emit(app) {
        log::warn!("sending the settings to the windows: {error}");
    }
    crate::api::follow_setting(app);
}
pub fn get_settings(app: &Session) -> AppSettings {
    let state = app.state::<AppState>();
    state.settings()
}
pub async fn patch_settings(app: Session, patch: SettingsPatch) -> CommandResult<AppSettings> {
    let state = app.state::<AppState>();
    if let Some(section) = RUST_ONLY_SECTIONS.iter().find(|s| patch.contains_key(**s)) {
        return Err(CommandError(format!(
            "The {section} settings are only changed by Opsis itself."
        )));
    }
    let settings = state.patch_settings(&patch).await?;
    announce(&app, settings.clone());
    Ok(settings)
}
pub async fn view_state(
    app: &Session,
    dir: String,
) -> CommandResult<serde_json::Map<String, Value>> {
    let state = app.state::<AppState>();
    Ok(state.db().read_view_state(&dir).await?)
}
pub async fn set_view_pref(
    app: Session,
    dir: String,
    key: String,
    value: Value,
) -> CommandResult<()> {
    let state = app.state::<AppState>();
    if state.db().write_view_state(&dir, &key, &value).await?
        && let Err(error) = ViewPrefChanged(ViewPrefChange { dir, key, value }).emit(&app)
    {
        log::warn!("sending a view setting to the windows: {error}");
    }
    Ok(())
}
pub fn app_menu_sections() -> Vec<MenuSection> {
    app_menu(cfg!(target_os = "macos"))
}
pub fn get_environment(app: &Session) -> EnvStatus {
    let state = app.state::<AppState>();
    let versions = app.platform().versions();
    EnvStatus {
        platform: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        is_packaged: state.paths.packaged,
        user_data: state.paths.user_data.to_string_lossy().into_owned(),
        versions: AppVersions {
            app: crate::APP_VERSION.to_owned(),
            tauri: versions.shell,
            webview: versions.view,
        },
        engine: engine_status(state),
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
            summary: "GNU General Public License version 3 or later",
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
            summary: "GNU General Public License version 3. The separate program that encodes and mixes the video.",
            kind: LicenceKind::Text,
            path: shipped(&credit_roots(state), Path::new(FFMPEG_LICENCE_FILE)).unwrap_or_default(),
        },
    ]
}
pub fn list_licences(app: &Session) -> CommandResult<Vec<LicenceEntry>> {
    let state = app.state::<AppState>();
    Ok(licence_entries(&licence_sources(state))?)
}
pub fn open_licence_file(app: &Session, file: &str) -> CommandResult<()> {
    let state = app.state::<AppState>();
    if !is_licence_file(&licence_sources(state), Path::new(&file)) {
        return Err(CommandError("That isn't one of the licence files.".into()));
    }
    Ok(app.platform().open_path(Path::new(&file))?)
}
pub fn reveal_in_folder(app: &Session, dir: &str, file: Option<String>) -> CommandResult<()> {
    let Some(file) = file else {
        return Ok(app.platform().open_path(Path::new(&dir))?);
    };
    let path = match ref_target(&file) {
        RefTarget::Project(rel) => Path::new(&dir).join(rel),
        RefTarget::Absolute(path) => PathBuf::from(path),
        RefTarget::App(_) | RefTarget::External(_) => {
            return Err(CommandError("That file isn't on this computer.".into()));
        }
    };
    Ok(app.platform().reveal_path(&path)?)
}
pub fn read_trace(dir: &str, trace_id: &str) -> CommandResult<TraceModel> {
    Ok(read_project_trace(Path::new(&dir), trace_id)?)
}

#[derive(Debug)]
pub struct ProjectSignIns {
    pub app: Session,
}

impl SignInStore for ProjectSignIns {
    fn read(&self, project_dir: &Path, name: &str) -> Result<Option<String>, SignInError> {
        let state = self.app.state::<AppState>();
        Ok(read_secret(
            state.keys().map_err(Clone::clone)?,
            project_dir,
            name,
        )?)
    }

    fn write(&self, project_dir: &Path, name: &str, value: &str) -> Result<(), SignInError> {
        let state = self.app.state::<AppState>();
        Ok(write_secret(
            state.keys().map_err(Clone::clone)?,
            project_dir,
            name,
            value,
        )?)
    }
}
