use std::path::{Path, PathBuf};

use opsis_project::files::{clear_cache as clear_project_cache, expand_home};
use opsis_project::import::import_assets as import_project_assets;
use opsis_project::last_run::read_last_run as read_project_last_run;
use opsis_project::paths::project_locations as locations_for;
use opsis_runner::context::now_ms;
use opsis_store::projects::RecentProject;
use opsis_types::project::{OpenProject, ProjectLocations};
use opsis_types::values::LastRun;
use serde_json::json;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

use crate::error::CommandResult;
use crate::state::AppState;
use crate::windows::{MAIN_WINDOW, apply_project_window};

fn shown(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn locations(app: &AppHandle, state: &AppState) -> CommandResult<ProjectLocations> {
    let settings = state.settings();
    let home = app.path().home_dir()?;
    Ok(locations_for(&settings.projects, &home))
}

pub async fn choose_folder_in(
    app: &AppHandle,
    title: &str,
    default_path: Option<&str>,
) -> Option<PathBuf> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut dialog = app.dialog().file().set_title(title);
    if let Some(dir) = default_path.map(Path::new).filter(|d| d.is_dir()) {
        dialog = dialog.set_directory(dir);
    }
    dialog.pick_folder(move |folder| {
        if sender.send(folder).is_err() {
            log::debug!("the folder dialog closed after its caller left");
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
pub async fn list_recent_projects(state: State<'_, AppState>) -> CommandResult<Vec<RecentProject>> {
    Ok(state.db().list_recent().await?)
}

#[tauri::command]
#[specta::specta]
pub fn project_locations(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<ProjectLocations> {
    locations(&app, &state)
}

#[tauri::command]
#[specta::specta]
pub async fn choose_folder(
    app: AppHandle,
    title: String,
    default_path: Option<String>,
) -> CommandResult<Option<String>> {
    Ok(choose_folder_in(&app, &title, default_path.as_deref())
        .await
        .map(|f| shown(&f)))
}

#[tauri::command]
#[specta::specta]
pub async fn pick_project_dir(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<String>> {
    let default_dir = locations(&app, &state)?.default_dir;
    Ok(
        choose_folder_in(&app, "Open Opsis project", Some(&default_dir))
            .await
            .map(|f| shown(&f)),
    )
}

#[tauri::command]
#[specta::specta]
pub async fn open_project(
    window: WebviewWindow,
    app: AppHandle,
    dir: String,
) -> CommandResult<OpenProject> {
    let project = crate::document::open(&app, Path::new(&dir)).await?;
    if window.label() == MAIN_WINDOW {
        apply_project_window(&window, &project.view);
    }
    Ok(project)
}

#[tauri::command]
#[specta::specta]
pub fn project_folder(app: AppHandle, parent: String, name: String) -> CommandResult<String> {
    let home = app.path().home_dir()?;
    let name = match name.trim() {
        "" => "Demo",
        given => given,
    };
    Ok(shown(
        &expand_home(&parent, &home).join(opsis_compile::text::project_folder_name(name)),
    ))
}

#[tauri::command]
#[specta::specta]
pub async fn create_project(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    template: String,
    dir: Option<String>,
) -> CommandResult<Option<OpenProject>> {
    let chosen = match dir {
        Some(dir) => PathBuf::from(dir),
        None => {
            let suggested = locations(&app, &state)?.suggested_dir;
            match choose_folder_in(&app, "Choose where to create the project", Some(&suggested))
                .await
            {
                Some(folder) => folder,
                None => return Ok(None),
            }
        }
    };
    let home = app.path().home_dir()?;
    let parent = expand_home(&chosen.to_string_lossy(), &home);
    let name = match name.trim() {
        "" => "Demo".to_owned(),
        given => given.to_owned(),
    };
    let created = state.projects.create(&parent, &name, &template)?;
    let shown_dir = shown(&created.dir);
    state.db().forget_view_state(&shown_dir).await?;
    state
        .db()
        .touch_recent(&shown_dir, &created.workflow.name, now_ms())
        .await?;
    let last_dir = serde_json::from_value(json!({ "projects": { "lastDir": shown(&parent) } }))?;
    let settings = state.patch_settings(&last_dir).await?;
    crate::settings::announce(&app, settings);
    let project =
        crate::document::adopt(&app, &created.dir, created.workflow, created.version).await?;
    Ok(Some(project))
}

#[tauri::command]
#[specta::specta]
pub fn read_last_run(dir: String) -> CommandResult<Option<LastRun>> {
    Ok(read_project_last_run(Path::new(&dir))?)
}

#[tauri::command]
#[specta::specta]
pub async fn import_assets(
    app: AppHandle,
    dir: String,
    paths: Option<Vec<String>>,
) -> CommandResult<Vec<String>> {
    let files: Vec<PathBuf> = match paths {
        Some(paths) => paths.into_iter().map(PathBuf::from).collect(),
        None => {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            app.dialog()
                .file()
                .set_title("Import files into the project")
                .pick_files(move |files| {
                    if sender.send(files).is_err() {
                        log::debug!("the import dialog closed after its caller left");
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
        return Ok(Vec::new());
    }
    Ok(
        tokio::task::spawn_blocking(move || import_project_assets(Path::new(&dir), &files))
            .await??,
    )
}

#[tauri::command]
#[specta::specta]
pub fn clear_cache(app: AppHandle, dir: String) -> CommandResult<()> {
    clear_project_cache(Path::new(&dir))?;
    crate::run::reseed(&app, &dir, None);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_templates() -> Vec<opsis_project::templates::TemplateInfo> {
    opsis_project::templates::templates()
}
