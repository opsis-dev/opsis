use std::path::{Path, PathBuf};

use opsis_engine::tools::shown;
use opsis_project::files::{clear_cache as clear_project_cache, expand_home};
use opsis_project::import::import_assets as import_project_assets;
use opsis_project::last_run::read_last_run as read_project_last_run;
use opsis_project::paths::project_locations as locations_for;
use opsis_runner::context::now_ms;
use opsis_store::projects::RecentProject;
use opsis_types::project::{OpenProject, ProjectLocations};
use opsis_types::values::LastRun;
use serde_json::json;

use crate::Session;
use crate::error::CommandResult;
use crate::state::AppState;

fn locations(state: &AppState) -> ProjectLocations {
    let settings = state.settings();
    locations_for(&settings.projects, &state.paths.home)
}

pub async fn choose_folder_in(
    app: &Session,
    title: &str,
    default_path: Option<&str>,
) -> Option<PathBuf> {
    let start = default_path.map(Path::new).filter(|d| d.is_dir());
    crate::platform::pick_folder(app.platform(), title, start).await
}
pub async fn list_recent_projects(app: &Session) -> CommandResult<Vec<RecentProject>> {
    let state = app.state::<AppState>();
    Ok(state.db().list_recent().await?)
}
pub fn project_locations(app: &Session) -> ProjectLocations {
    locations(app.state::<AppState>())
}
pub async fn choose_folder(
    app: Session,
    title: String,
    default_path: Option<String>,
) -> CommandResult<Option<String>> {
    Ok(choose_folder_in(&app, &title, default_path.as_deref())
        .await
        .map(|f| shown(&f)))
}
pub async fn pick_project_dir(app: Session) -> CommandResult<Option<String>> {
    let state = app.state::<AppState>();
    let default_dir = locations(state).default_dir;
    Ok(
        choose_folder_in(&app, "Open Opsis project", Some(&default_dir))
            .await
            .map(|f| shown(&f)),
    )
}
pub async fn open_project(app: Session, dir: String) -> CommandResult<OpenProject> {
    crate::document::open(&app, Path::new(&dir)).await
}
pub fn project_folder(app: &Session, parent: &str, name: &str) -> String {
    let home = &app.state::<AppState>().paths.home;
    let name = match name.trim() {
        "" => "Demo",
        given => given,
    };
    shown(&expand_home(parent, home).join(opsis_compile::text::project_folder_name(name)))
}
pub async fn create_project(
    app: Session,
    name: String,
    template: String,
    dir: Option<String>,
) -> CommandResult<Option<OpenProject>> {
    let state = app.state::<AppState>();
    let chosen = if let Some(dir) = dir {
        PathBuf::from(dir)
    } else {
        let suggested = locations(state).suggested_dir;
        match choose_folder_in(&app, "Choose where to create the project", Some(&suggested)).await {
            Some(folder) => folder,
            None => return Ok(None),
        }
    };
    let parent = expand_home(&chosen.to_string_lossy(), &state.paths.home);
    let name = match name.trim() {
        "" => "Demo".to_owned(),
        given => given.to_owned(),
    };
    let created = state.projects.create(&parent, &name, &template)?;
    let dir = opsis_folders::real_folder(&created.dir);
    let shown_dir = shown(&dir);
    state.db().forget_view_state(&shown_dir).await?;
    state
        .db()
        .touch_recent(&shown_dir, &created.workflow.name, now_ms())
        .await?;
    let last_dir = serde_json::from_value(json!({ "projects": { "lastDir": shown(&parent) } }))?;
    let settings = state.patch_settings(&last_dir).await?;
    crate::settings::announce(&app, settings);
    let project = crate::document::adopt(&app, &dir, created.workflow, created.version).await?;
    Ok(Some(project))
}
pub fn read_last_run(dir: &str) -> CommandResult<Option<LastRun>> {
    Ok(read_project_last_run(Path::new(&dir))?)
}
pub async fn import_assets(
    app: Session,
    dir: String,
    paths: Option<Vec<String>>,
) -> CommandResult<Vec<String>> {
    let files: Vec<PathBuf> = if let Some(paths) = paths {
        paths.into_iter().map(PathBuf::from).collect()
    } else {
        crate::platform::pick_files(app.platform(), "Import files into the project", None).await
    };
    if files.is_empty() {
        return Ok(Vec::new());
    }
    Ok(
        tokio::task::spawn_blocking(move || import_project_assets(Path::new(&dir), &files))
            .await??,
    )
}
pub fn clear_cache(app: &Session, dir: &str) -> CommandResult<()> {
    clear_project_cache(Path::new(&dir))?;
    crate::run::reseed(app, dir, None);
    Ok(())
}
pub fn list_templates() -> Vec<opsis_project::templates::TemplateInfo> {
    opsis_project::templates::templates()
}
