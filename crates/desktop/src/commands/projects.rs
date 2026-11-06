use opsis_session::error::CommandResult;
use opsis_store::projects::RecentProject;
use opsis_types::project::{OpenProject, ProjectLocations};
use opsis_types::values::LastRun;
use tauri::{AppHandle, WebviewWindow};

use crate::session_of;
use crate::windows::{MAIN_WINDOW, apply_project_window};

#[tauri::command]
#[specta::specta]
pub async fn list_recent_projects(app: AppHandle) -> CommandResult<Vec<RecentProject>> {
    let session = session_of(&app);
    opsis_session::projects::list_recent_projects(&session).await
}

#[tauri::command]
#[specta::specta]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the editor's bindings type this command's answer as a result, which the cut-over drops with the web editor"
)]
pub fn project_locations(app: AppHandle) -> CommandResult<ProjectLocations> {
    let session = session_of(&app);
    Ok(opsis_session::projects::project_locations(&session))
}

#[tauri::command]
#[specta::specta]
pub async fn choose_folder(
    app: AppHandle,
    title: String,
    default_path: Option<String>,
) -> CommandResult<Option<String>> {
    let session = session_of(&app);
    opsis_session::projects::choose_folder(session.clone(), title, default_path).await
}

#[tauri::command]
#[specta::specta]
pub async fn pick_project_dir(app: AppHandle) -> CommandResult<Option<String>> {
    let session = session_of(&app);
    opsis_session::projects::pick_project_dir(session.clone()).await
}

#[tauri::command]
#[specta::specta]
pub async fn open_project(
    app: AppHandle,
    window: WebviewWindow,
    dir: String,
) -> CommandResult<OpenProject> {
    let session = session_of(&app);
    let project = opsis_session::projects::open_project(session.clone(), dir).await?;
    if window.label() == MAIN_WINDOW {
        apply_project_window(&window, &project.view);
    }
    Ok(project)
}

#[tauri::command]
#[specta::specta]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the editor's bindings type this command's answer as a result, which the cut-over drops with the web editor"
)]
pub fn project_folder(app: AppHandle, parent: String, name: String) -> CommandResult<String> {
    let session = session_of(&app);
    Ok(opsis_session::projects::project_folder(
        &session, &parent, &name,
    ))
}

#[tauri::command]
#[specta::specta]
pub async fn create_project(
    app: AppHandle,
    name: String,
    template: String,
    dir: Option<String>,
) -> CommandResult<Option<OpenProject>> {
    let session = session_of(&app);
    opsis_session::projects::create_project(session.clone(), name, template, dir).await
}

#[tauri::command]
#[specta::specta]
pub fn read_last_run(dir: String) -> CommandResult<Option<LastRun>> {
    opsis_session::projects::read_last_run(&dir)
}

#[tauri::command]
#[specta::specta]
pub async fn import_assets(
    app: AppHandle,
    dir: String,
    paths: Option<Vec<String>>,
) -> CommandResult<Vec<String>> {
    let session = session_of(&app);
    opsis_session::projects::import_assets(session.clone(), dir, paths).await
}

#[tauri::command]
#[specta::specta]
pub fn clear_cache(app: AppHandle, dir: String) -> CommandResult<()> {
    let session = session_of(&app);
    opsis_session::projects::clear_cache(&session, &dir)
}

#[tauri::command]
#[specta::specta]
pub fn list_templates() -> Vec<opsis_project::templates::TemplateInfo> {
    opsis_session::projects::list_templates()
}
