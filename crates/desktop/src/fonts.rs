use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::fonts::{font_choice, font_choices as choices_for, is_known_family};
use opsis_fonts::library::{InspectedFonts, InstalledFamily, installed_families, system_fonts};
use opsis_fonts::online::FontCatalogEntry;
use opsis_types::fonts::{FontChoice, FontChoices, FontFamilyDef};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

const DOWNLOAD_TASK: &str = "A font is being downloaded.";
const FONT_FILES: [&str; 6] = ["ttf", "otf", "woff", "woff2", "ttc", "zip"];

#[tauri::command]
#[specta::specta]
pub async fn list_fonts(state: State<'_, AppState>) -> CommandResult<Vec<FontChoice>> {
    let library = state.fonts.list(state.db()).await?;
    Ok(library.iter().map(|def| font_choice(def, "")).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn font_choices(
    state: State<'_, AppState>,
    value: String,
    project: Vec<FontFamilyDef>,
) -> CommandResult<FontChoices> {
    let library = state.fonts.list(state.db()).await?;
    Ok(choices_for(&value, &project, &library))
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_font_files(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Option<Vec<String>>,
) -> CommandResult<Option<InspectedFonts>> {
    let files: Vec<PathBuf> = match paths {
        Some(paths) => paths.into_iter().map(PathBuf::from).collect(),
        None => {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            app.dialog()
                .file()
                .set_title("Add fonts")
                .add_filter("Fonts", &FONT_FILES)
                .pick_files(move |files| {
                    if sender.send(files).is_err() {
                        log::debug!("the font dialog closed after its caller left");
                    }
                });
            receiver
                .await
                .ok()
                .flatten()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|f| f.into_path().ok())
                .collect()
        }
    };
    if files.is_empty() {
        return Ok(None);
    }
    Ok(Some(state.fonts.inspect_files(&files)?))
}

#[tauri::command]
#[specta::specta]
pub async fn installed_font_families() -> CommandResult<Vec<InstalledFamily>> {
    Ok(tokio::task::spawn_blocking(|| installed_families(&system_fonts())).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_installed_fonts(
    state: State<'_, AppState>,
    families: Vec<String>,
) -> CommandResult<InspectedFonts> {
    let system = tokio::task::spawn_blocking(system_fonts).await?;
    Ok(state.fonts.inspect_installed(&system, &families))
}

#[tauri::command]
#[specta::specta]
pub async fn add_inspected_fonts(
    state: State<'_, AppState>,
    token: String,
    families: Vec<String>,
) -> CommandResult<Vec<FontFamilyDef>> {
    Ok(state
        .fonts
        .add_inspected(state.db(), &token, &families)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub fn release_inspected_fonts(state: State<'_, AppState>, token: String) {
    state.fonts.release_inspected(&token);
}

#[tauri::command]
#[specta::specta]
pub async fn remove_font(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    Ok(state.fonts.remove(state.db(), &id).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn font_catalog(state: State<'_, AppState>) -> CommandResult<Vec<FontCatalogEntry>> {
    let library = state.fonts.list(state.db()).await?;
    let mut entries = Arc::unwrap_or_clone(state.fonts_online.catalog().await?);
    for entry in &mut entries {
        entry.added = is_known_family(&entry.family, &library);
    }
    Ok(entries)
}

#[tauri::command]
#[specta::specta]
pub async fn download_font(state: State<'_, AppState>, id: String) -> CommandResult<FontFamilyDef> {
    state.tasks.begin(DOWNLOAD_TASK);
    let downloaded = state
        .fonts_online
        .download(&state.fonts, state.db(), &id)
        .await;
    state.tasks.end(DOWNLOAD_TASK);
    Ok(downloaded?)
}

#[tauri::command]
#[specta::specta]
pub async fn use_font_in_project(
    state: State<'_, AppState>,
    dir: String,
    font_id: String,
) -> CommandResult<FontFamilyDef> {
    let library = state.fonts.list(state.db()).await?;
    let def = library
        .into_iter()
        .find(|f| f.id == font_id)
        .ok_or_else(|| CommandError("That font isn't in your library any more.".into()))?;
    Ok(state.fonts.copy_to_project(Path::new(&dir), &def)?)
}
