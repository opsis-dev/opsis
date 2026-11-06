use opsis_session::error::CommandResult;
use opsis_store::settings::SettingsPatch;
use opsis_types::engine::LicenceEntry;
use opsis_types::settings::AppSettings;
use opsis_types::shell::{EnvStatus, MenuSection};
use opsis_types::trace::TraceModel;
use serde_json::Value;
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub fn get_settings(app: AppHandle) -> AppSettings {
    let session = session_of(&app);
    opsis_session::settings::get_settings(&session)
}

#[tauri::command]
#[specta::specta]
pub async fn patch_settings(app: AppHandle, patch: SettingsPatch) -> CommandResult<AppSettings> {
    let session = session_of(&app);
    opsis_session::settings::patch_settings(session.clone(), patch).await
}

#[tauri::command]
#[specta::specta]
pub async fn view_state(
    app: AppHandle,
    dir: String,
) -> CommandResult<serde_json::Map<String, Value>> {
    let session = session_of(&app);
    opsis_session::settings::view_state(&session, dir).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_view_pref(
    app: AppHandle,
    dir: String,
    key: String,
    value: Value,
) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::settings::set_view_pref(session.clone(), dir, key, value).await
}

#[tauri::command]
#[specta::specta]
pub fn app_menu_sections() -> Vec<MenuSection> {
    opsis_session::settings::app_menu_sections()
}

#[tauri::command]
#[specta::specta]
pub fn get_environment(app: AppHandle) -> EnvStatus {
    let session = session_of(&app);
    opsis_session::settings::get_environment(&session)
}

#[tauri::command]
#[specta::specta]
pub fn list_licences(app: AppHandle) -> CommandResult<Vec<LicenceEntry>> {
    let session = session_of(&app);
    opsis_session::settings::list_licences(&session)
}

#[tauri::command]
#[specta::specta]
pub fn open_licence_file(app: AppHandle, file: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::settings::open_licence_file(&session, &file)
}

#[tauri::command]
#[specta::specta]
pub fn reveal_in_folder(app: AppHandle, dir: String, file: Option<String>) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::settings::reveal_in_folder(&session, &dir, file)
}

#[tauri::command]
#[specta::specta]
pub fn read_trace(dir: String, trace_id: String) -> CommandResult<TraceModel> {
    opsis_session::settings::read_trace(&dir, &trace_id)
}
