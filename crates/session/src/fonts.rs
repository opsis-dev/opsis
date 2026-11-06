use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::fonts::{font_choice, font_choices as choices_for, is_known_family};
use opsis_fonts::library::{InspectedFonts, InstalledFamily, installed_families, system_fonts};
use opsis_fonts::online::FontCatalogEntry;
use opsis_types::fonts::{FontChoice, FontChoices, FontFamilyDef};

use crate::Session;
use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

const DOWNLOAD_TASK: &str = "A font is being downloaded.";
const FONT_FILES: [&str; 6] = ["ttf", "otf", "woff", "woff2", "ttc", "zip"];
pub async fn list_fonts(app: &Session) -> CommandResult<Vec<FontChoice>> {
    let state = app.state::<AppState>();
    let library = state.fonts.list(state.db()).await?;
    Ok(library.iter().map(|def| font_choice(def, "")).collect())
}
pub async fn font_choices(
    app: &Session,
    value: String,
    project: Vec<FontFamilyDef>,
) -> CommandResult<FontChoices> {
    let state = app.state::<AppState>();
    let library = state.fonts.list(state.db()).await?;
    Ok(choices_for(&value, &project, &library))
}
pub async fn inspect_font_files(
    app: Session,
    paths: Option<Vec<String>>,
) -> CommandResult<Option<InspectedFonts>> {
    let state = app.state::<AppState>();
    let files: Vec<PathBuf> = if let Some(paths) = paths {
        paths.into_iter().map(PathBuf::from).collect()
    } else {
        let filter = crate::platform::FileFilter {
            name: "Fonts".to_owned(),
            extensions: FONT_FILES.iter().map(|e| (*e).to_owned()).collect(),
        };
        crate::platform::pick_files(app.platform(), "Add fonts", Some(filter)).await
    };
    if files.is_empty() {
        return Ok(None);
    }
    Ok(Some(state.fonts.inspect_files(&files)?))
}
pub async fn installed_font_families() -> CommandResult<Vec<InstalledFamily>> {
    Ok(tokio::task::spawn_blocking(|| installed_families(&system_fonts())).await?)
}
pub async fn inspect_installed_fonts(
    app: &Session,
    families: Vec<String>,
) -> CommandResult<InspectedFonts> {
    let state = app.state::<AppState>();
    let system = tokio::task::spawn_blocking(system_fonts).await?;
    Ok(state.fonts.inspect_installed(&system, &families))
}
pub async fn add_inspected_fonts(
    app: &Session,
    token: String,
    families: Vec<String>,
) -> CommandResult<Vec<FontFamilyDef>> {
    let state = app.state::<AppState>();
    Ok(state
        .fonts
        .add_inspected(state.db(), &token, &families)
        .await?)
}
pub fn release_inspected_fonts(app: &Session, token: &str) {
    let state = app.state::<AppState>();
    state.fonts.release_inspected(token);
}
pub async fn remove_font(app: &Session, id: String) -> CommandResult<()> {
    let state = app.state::<AppState>();
    Ok(state.fonts.remove(state.db(), &id).await?)
}
pub async fn font_catalog(app: &Session) -> CommandResult<Vec<FontCatalogEntry>> {
    let state = app.state::<AppState>();
    let library = state.fonts.list(state.db()).await?;
    let mut entries = Arc::unwrap_or_clone(state.fonts_online.catalog().await?);
    for entry in &mut entries {
        entry.added = is_known_family(&entry.family, &library);
    }
    Ok(entries)
}
pub async fn download_font(app: &Session, id: String) -> CommandResult<FontFamilyDef> {
    let state = app.state::<AppState>();
    state.tasks.begin(DOWNLOAD_TASK);
    let downloaded = state
        .fonts_online
        .download(&state.fonts, state.db(), &id)
        .await;
    state.tasks.end(DOWNLOAD_TASK);
    Ok(downloaded?)
}
pub async fn use_font_in_project(
    app: &Session,
    dir: String,
    font_id: String,
) -> CommandResult<FontFamilyDef> {
    let state = app.state::<AppState>();
    let library = state.fonts.list(state.db()).await?;
    let def = library
        .into_iter()
        .find(|f| f.id == font_id)
        .ok_or_else(|| CommandError("That font isn't in your library any more.".into()))?;
    Ok(state.fonts.copy_to_project(Path::new(&dir), &def)?)
}
