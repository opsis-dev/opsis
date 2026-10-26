use std::path::{Path, PathBuf};

use opsis_project::files::{clear_cache as clear_project_cache, expand_home};
use opsis_project::import::import_assets as import_project_assets;
use opsis_project::last_run::read_last_run as read_project_last_run;
use opsis_project::paths::project_locations as locations_for;
use opsis_project::watch::{LoadedFilesWatch, watch_project as watch_project_file};
use opsis_runner::context::now_ms;
use opsis_store::projects::RecentProject;
use opsis_types::project::{OpenProject, ProjectLocations, SaveResult};
use opsis_types::values::LastRun;
use opsis_types::workflow::Workflow;
use serde_json::json;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::{ProjectFileChanged, ProjectFilesChanged};
use crate::state::AppState;
use crate::windows::{MAIN_WINDOW, apply_project_window};

fn shown(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn locations(app: &AppHandle, state: &AppState) -> CommandResult<ProjectLocations> {
    let settings = state.db().read_settings()?;
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
pub fn list_recent_projects(state: State<'_, AppState>) -> CommandResult<Vec<RecentProject>> {
    Ok(state.db().list_recent()?)
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

fn opened(state: &AppState, dir: &Path) -> CommandResult<OpenProject> {
    let project = state.projects.open(dir)?;
    let shown_dir = shown(dir);
    let db = state.db();
    db.touch_recent(&shown_dir, &project.workflow.name, now_ms())?;
    let view = db.read_view_state(&shown_dir)?;
    drop(db);
    let mut editor = state.editor();
    editor.dir = Some(dir.to_path_buf());
    editor.results = project
        .last_run
        .as_ref()
        .map(|r| r.nodes.clone())
        .unwrap_or_default();
    editor.workflow = Some(project.workflow.clone());
    Ok(OpenProject {
        dir: shown_dir,
        workflow: project.workflow,
        last_run: project.last_run,
        version: project.version,
        view,
    })
}

#[tauri::command]
#[specta::specta]
pub fn open_project(
    window: WebviewWindow,
    state: State<'_, AppState>,
    dir: String,
) -> CommandResult<OpenProject> {
    let project = opened(&state, Path::new(&dir))?;
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
    {
        let mut db = state.db();
        db.forget_view_state(&shown_dir)?;
        db.touch_recent(&shown_dir, &created.workflow.name, now_ms())?;
        let (settings, _) = db.patch_settings(&serde_json::from_value(
            json!({ "projects": { "lastDir": shown(&parent) } }),
        )?)?;
        drop(db);
        crate::settings::announce(&app, settings);
    }
    let mut editor = state.editor();
    editor.dir = Some(created.dir.clone());
    editor.results.clear();
    editor.workflow = Some(created.workflow.clone());
    Ok(Some(OpenProject {
        dir: shown_dir,
        workflow: created.workflow,
        last_run: None,
        version: created.version,
        view: serde_json::Map::new(),
    }))
}

#[tauri::command]
#[specta::specta]
pub fn save_workflow(
    state: State<'_, AppState>,
    dir: String,
    workflow: Workflow,
    base: Option<String>,
) -> CommandResult<SaveResult> {
    let result = state
        .projects
        .save(Path::new(&dir), &workflow, base.as_deref())?;
    if matches!(result, SaveResult::Saved { .. }) {
        state.db().touch_recent(&dir, &workflow.name, now_ms())?;
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub fn watch_project(
    app: AppHandle,
    state: State<'_, AppState>,
    dir: Option<String>,
) -> CommandResult<()> {
    let dir = dir.map(PathBuf::from);
    let mut watching = state.watching.lock().expect("watching");
    if watching.dir == dir {
        return Ok(());
    }
    watching.project = None;
    watching.files = None;
    watching.dir.clone_from(&dir);
    let Some(dir) = dir else {
        return Ok(());
    };
    let to_main = app.clone();
    watching.project = Some(watch_project_file(&state.projects, &dir, move |change| {
        if let Err(error) = ProjectFileChanged(change).emit_to(&to_main, MAIN_WINDOW) {
            log::warn!("telling the editor the project changed: {error}");
        }
    })?);
    let to_all = app.clone();
    let mut files = LoadedFilesWatch::start(&dir, move |change| {
        if let Err(error) = ProjectFilesChanged(change).emit(&to_all) {
            log::warn!("telling windows files changed: {error}");
        }
        crate::compile::recompile(&to_all);
    })?;
    if let Some((refs_dir, refs)) = &watching.refs
        && *refs_dir == dir
    {
        files.follow(refs);
    }
    watching.files = Some(files);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn watch_project_files(
    state: State<'_, AppState>,
    dir: String,
    refs: Vec<String>,
) -> CommandResult<()> {
    let dir = PathBuf::from(dir);
    let mut watching = state.watching.lock().expect("watching");
    if watching.dir.as_ref() == Some(&dir)
        && let Some(files) = watching.files.as_mut()
    {
        files.follow(&refs);
    }
    watching.refs = Some((dir, refs));
    Ok(())
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
pub fn clear_cache(dir: String) -> CommandResult<()> {
    Ok(clear_project_cache(Path::new(&dir))?)
}

#[tauri::command]
#[specta::specta]
pub fn list_templates() -> Vec<opsis_project::templates::TemplateInfo> {
    opsis_project::templates::templates()
}
