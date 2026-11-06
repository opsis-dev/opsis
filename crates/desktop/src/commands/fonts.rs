use opsis_fonts::library::{InspectedFonts, InstalledFamily};
use opsis_fonts::online::FontCatalogEntry;
use opsis_session::error::CommandResult;
use opsis_types::fonts::{FontChoice, FontChoices, FontFamilyDef};
use tauri::AppHandle;

use crate::session_of;

#[tauri::command]
#[specta::specta]
pub async fn list_fonts(app: AppHandle) -> CommandResult<Vec<FontChoice>> {
    let session = session_of(&app);
    opsis_session::fonts::list_fonts(&session).await
}

#[tauri::command]
#[specta::specta]
pub async fn font_choices(
    app: AppHandle,
    value: String,
    project: Vec<FontFamilyDef>,
) -> CommandResult<FontChoices> {
    let session = session_of(&app);
    opsis_session::fonts::font_choices(&session, value, project).await
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_font_files(
    app: AppHandle,
    paths: Option<Vec<String>>,
) -> CommandResult<Option<InspectedFonts>> {
    let session = session_of(&app);
    opsis_session::fonts::inspect_font_files(session.clone(), paths).await
}

#[tauri::command]
#[specta::specta]
pub async fn installed_font_families() -> CommandResult<Vec<InstalledFamily>> {
    opsis_session::fonts::installed_font_families().await
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_installed_fonts(
    app: AppHandle,
    families: Vec<String>,
) -> CommandResult<InspectedFonts> {
    let session = session_of(&app);
    opsis_session::fonts::inspect_installed_fonts(&session, families).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_inspected_fonts(
    app: AppHandle,
    token: String,
    families: Vec<String>,
) -> CommandResult<Vec<FontFamilyDef>> {
    let session = session_of(&app);
    opsis_session::fonts::add_inspected_fonts(&session, token, families).await
}

#[tauri::command]
#[specta::specta]
pub fn release_inspected_fonts(app: AppHandle, token: String) {
    let session = session_of(&app);
    opsis_session::fonts::release_inspected_fonts(&session, &token);
}

#[tauri::command]
#[specta::specta]
pub async fn remove_font(app: AppHandle, id: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::fonts::remove_font(&session, id).await
}

#[tauri::command]
#[specta::specta]
pub async fn font_catalog(app: AppHandle) -> CommandResult<Vec<FontCatalogEntry>> {
    let session = session_of(&app);
    opsis_session::fonts::font_catalog(&session).await
}

#[tauri::command]
#[specta::specta]
pub async fn download_font(app: AppHandle, id: String) -> CommandResult<FontFamilyDef> {
    let session = session_of(&app);
    opsis_session::fonts::download_font(&session, id).await
}

#[tauri::command]
#[specta::specta]
pub async fn use_font_in_project(
    app: AppHandle,
    dir: String,
    font_id: String,
) -> CommandResult<FontFamilyDef> {
    let session = session_of(&app);
    opsis_session::fonts::use_font_in_project(&session, dir, font_id).await
}
