use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;

use opsis_compile::edits::EditError;
use opsis_compile::node::Platform;
use opsis_document::Document;
use opsis_project::last_run::read_last_run;
use opsis_project::watch::{LoadedFilesWatch, watch_project};
use opsis_runner::context::now_ms;
use opsis_types::edits::{
    DocumentState, Edited, Removal, TimelineEdit, TimelineReply, WorkflowEdit,
};
use opsis_types::project::{OpenProject, ProjectFileChange, SaveResult};
use opsis_types::workflow::Workflow;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::error::{CommandError, CommandResult};
use crate::events::{DiskChangeAsked, DocumentChanged, ProjectFilesChanged, ProjectReloaded};
use crate::state::{AppState, DiskChange};
use crate::windows::MAIN_WINDOW;

const AUTOSAVE_DELAY: Duration = Duration::from_millis(800);
pub const VERSION_UNCHOSEN: &str =
    "The project file changed on disk. Choose which version to keep first.";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("No project is open.")]
    NoProject,
    #[error(transparent)]
    Edit(#[from] EditError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DiskChoice {
    Load,
    KeepMine,
    Wait,
}

fn shown(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn announce(app: &AppHandle) {
    let changed = app
        .state::<AppState>()
        .editor()
        .document
        .as_mut()
        .map(|doc| DocumentChanged {
            document: doc.state(),
            edited: doc.take_edited(),
        });
    let Some(changed) = changed else {
        return;
    };
    crate::run::forget_errors(app, &changed.edited);
    if let Err(error) = changed.emit(app) {
        log::warn!("sending the document: {error}");
    }
}

fn schedule_autosave(app: &AppHandle) {
    let state = app.state::<AppState>();
    let turn = state.autosave.fetch_add(1, Ordering::Relaxed) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(AUTOSAVE_DELAY).await;
        if app.state::<AppState>().autosave.load(Ordering::Relaxed) != turn {
            return;
        }
        if let Err(error) = save(&app).await {
            log::warn!("saving the project: {}", error.0);
        }
    });
}

pub fn changed(app: &AppHandle) {
    announce(app);
    schedule_autosave(app);
    crate::compile::request(app);
}

pub fn document_dir(state: &AppState) -> Option<PathBuf> {
    state.editor().dir()
}

pub fn document_workflow(state: &AppState) -> Option<(PathBuf, Workflow)> {
    let editor = state.editor();
    let doc = editor.document.as_ref()?;
    Some((doc.dir().to_path_buf(), doc.workflow().clone()))
}

fn ask_about_disk(app: &AppHandle, workflow: Option<Workflow>, version: String) {
    let readable = workflow.is_some();
    app.state::<AppState>().editor().disk_change = Some(DiskChange { workflow, version });
    if let Err(error) = (DiskChangeAsked { readable }).emit_to(app, MAIN_WINDOW) {
        log::warn!("asking about the changed project file: {error}");
    }
}

pub async fn save(app: &AppHandle) -> CommandResult<bool> {
    let state = app.state::<AppState>();
    let _turn = state.saving.lock().await;
    let (dir, workflow, revision, base) = {
        let editor = state.editor();
        let Some(doc) = editor.document.as_ref() else {
            return Ok(true);
        };
        if editor.disk_change.is_some() {
            return Ok(false);
        }
        if !doc.is_dirty() {
            return Ok(true);
        }
        (
            doc.dir().to_path_buf(),
            doc.workflow().clone(),
            doc.revision(),
            doc.file_version().to_owned(),
        )
    };
    match state.projects.save(&dir, &workflow, Some(&base))? {
        SaveResult::Saved { version } => {
            if let Some(doc) = state.editor().document.as_mut().filter(|d| d.dir() == dir) {
                doc.mark_saved(revision, version);
            }
            state
                .db()
                .touch_recent(&shown(&dir), &workflow.name, now_ms())
                .await?;
            announce(app);
            Ok(true)
        }
        SaveResult::Changed { version, workflow } => {
            ask_about_disk(app, workflow.map(|w| *w), version);
            Ok(false)
        }
    }
}

fn outside_change(app: &AppHandle, change: ProjectFileChange) {
    let state = app.state::<AppState>();
    let load = {
        let editor = state.editor();
        let Some(doc) = editor.document.as_ref() else {
            return;
        };
        if shown(doc.dir()) != change.dir || doc.file_version() == change.version {
            return;
        }
        change.workflow.is_some() && !doc.is_dirty() && editor.disk_change.is_none()
    };
    match (load, change.workflow) {
        (true, Some(workflow)) => {
            if let Some(doc) = state.editor().document.as_mut() {
                doc.load_from_disk(workflow, change.version);
            }
            if let Err(error) = ProjectReloaded.emit_to(app, MAIN_WINDOW) {
                log::warn!("telling the editor the project reloaded: {error}");
            }
            changed(app);
        }
        (_, workflow) => ask_about_disk(app, workflow, change.version),
    }
}

fn watch(app: &AppHandle, dir: &Path) -> CommandResult<()> {
    let state = app.state::<AppState>();
    let mut watching = state.watching.lock().expect("watching");
    if watching.dir.as_deref() == Some(dir) {
        return Ok(());
    }
    watching.project = None;
    watching.files = None;
    watching.dir = Some(dir.to_path_buf());
    let on_project = app.clone();
    watching.project = Some(watch_project(&state.projects, dir, move |change| {
        outside_change(&on_project, change);
    })?);
    let on_files = app.clone();
    let mut files = LoadedFilesWatch::start(dir, move |change| {
        if let Err(error) = ProjectFilesChanged(change).emit(&on_files) {
            log::warn!("telling windows files changed: {error}");
        }
        crate::compile::request(&on_files);
    })?;
    if let Some((refs_dir, refs)) = &watching.refs
        && refs_dir == dir
    {
        files.follow(refs);
    }
    watching.files = Some(files);
    Ok(())
}

async fn opened_project(state: &AppState, dir: &Path) -> CommandResult<OpenProject> {
    let shown_dir = shown(dir);
    let document = state
        .editor()
        .document
        .as_ref()
        .map(Document::state)
        .ok_or(DocumentError::NoProject)?;
    Ok(OpenProject {
        last_run: read_last_run(dir)?,
        view: state.db().read_view_state(&shown_dir).await?,
        dir: shown_dir,
        document,
    })
}

pub async fn open(app: &AppHandle, dir: &Path) -> CommandResult<OpenProject> {
    let state = app.state::<AppState>();
    if document_dir(&state).as_deref() != Some(dir) {
        save(app).await?;
        let project = state.projects.open(dir)?;
        state
            .db()
            .touch_recent(&shown(dir), &project.workflow.name, now_ms())
            .await?;
        {
            let mut editor = state.editor();
            editor.results = project
                .last_run
                .as_ref()
                .map(|r| r.nodes.clone())
                .unwrap_or_default();
            editor.disk_change = None;
            editor.document = Some(Document::open(
                dir,
                project.workflow,
                project.version,
                Platform::current(),
            ));
        }
        crate::run::reseed(app, &shown(dir), project.last_run.as_ref());
        crate::shared::forget_project(app);
    }
    watch(app, dir)?;
    changed(app);
    opened_project(&state, dir).await
}

pub async fn adopt(
    app: &AppHandle,
    dir: &Path,
    workflow: Workflow,
    version: String,
) -> CommandResult<OpenProject> {
    save(app).await?;
    let state = app.state::<AppState>();
    {
        let mut editor = state.editor();
        editor.results.clear();
        editor.disk_change = None;
        editor.document = Some(Document::open(dir, workflow, version, Platform::current()));
    }
    crate::run::reseed(app, &shown(dir), None);
    crate::shared::forget_project(app);
    watch(app, dir)?;
    changed(app);
    opened_project(&state, dir).await
}

pub fn follow_loaded_files(app: &AppHandle, dir: PathBuf, refs: Vec<String>) {
    let state = app.state::<AppState>();
    let mut watching = state.watching.lock().expect("watching");
    if watching.refs.as_ref() == Some(&(dir.clone(), refs.clone())) {
        return;
    }
    if watching.dir.as_ref() == Some(&dir)
        && let Some(files) = watching.files.as_mut()
    {
        files.follow(&refs);
    }
    watching.refs = Some((dir, refs));
}

pub fn edit(app: &AppHandle, edit: WorkflowEdit) -> Result<Edited, DocumentError> {
    let edited = {
        let state = app.state::<AppState>();
        let mut editor = state.editor();
        let doc = editor.document.as_mut().ok_or(DocumentError::NoProject)?;
        doc.apply(edit)?
    };
    changed(app);
    Ok(edited)
}

pub fn step(app: &AppHandle, redo: bool) -> Result<u64, DocumentError> {
    let revision = {
        let state = app.state::<AppState>();
        let mut editor = state.editor();
        let doc = editor.document.as_mut().ok_or(DocumentError::NoProject)?;
        if redo {
            doc.redo()
        } else {
            doc.undo()
        };
        doc.revision()
    };
    changed(app);
    Ok(revision)
}

fn with_document(state: &AppState, change: impl FnOnce(&mut Document)) {
    if let Some(doc) = state.editor().document.as_mut() {
        change(doc);
    }
}

#[tauri::command]
#[specta::specta]
pub fn document_state(state: State<'_, AppState>) -> Option<DocumentState> {
    state.editor().document.as_ref().map(Document::state)
}

#[tauri::command]
#[specta::specta]
pub fn edit_workflow(app: AppHandle, edit: WorkflowEdit) -> CommandResult<Edited> {
    Ok(self::edit(&app, edit)?)
}

#[tauri::command]
#[specta::specta]
pub fn undo(app: AppHandle) -> CommandResult<()> {
    step(&app, false)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn redo(app: AppHandle) -> CommandResult<()> {
    step(&app, true)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn begin_gesture(state: State<'_, AppState>) {
    let mut editor = state.editor();
    let starting = editor
        .document
        .as_ref()
        .is_some_and(|d| d.gesture_start().is_none());
    if starting {
        editor.gesture_film = editor.film.clone();
    }
    if let Some(doc) = editor.document.as_mut() {
        doc.begin_gesture();
    }
}

#[tauri::command]
#[specta::specta]
pub fn end_gesture(app: AppHandle, state: State<'_, AppState>) {
    {
        let mut editor = state.editor();
        if let Some(doc) = editor.document.as_mut() {
            doc.end_gesture();
            if doc.gesture_start().is_none() {
                editor.gesture_film = None;
            }
        }
    }
    announce(&app);
}

#[tauri::command]
#[specta::specta]
pub fn edit_timeline(app: AppHandle, edit: TimelineEdit) -> CommandResult<TimelineReply> {
    let reply = {
        let state = app.state::<AppState>();
        let mut editor = state.editor();
        let film = editor
            .gesture_film
            .clone()
            .or_else(|| editor.film.clone())
            .ok_or_else(|| CommandError("There's no film on the timeline yet.".into()))?;
        let doc = editor.document.as_mut().ok_or(DocumentError::NoProject)?;
        doc.edit_timeline(edit, &film)
    };
    if reply.error.is_none() {
        changed(&app);
    }
    Ok(reply)
}

#[tauri::command]
#[specta::specta]
pub fn timeline_removal(state: State<'_, AppState>, selected: Vec<String>) -> Option<Removal> {
    let editor = state.editor();
    let film = editor.film.as_ref()?;
    editor.document.as_ref()?.timeline_removal(film, &selected)
}

#[tauri::command]
#[specta::specta]
pub async fn save_project(app: AppHandle) -> CommandResult<bool> {
    save(&app).await
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_disk_change(app: AppHandle, choice: DiskChoice) -> CommandResult<bool> {
    let pending = app.state::<AppState>().editor().disk_change.take();
    let Some(DiskChange { workflow, version }) = pending else {
        return Ok(true);
    };
    let state = app.state::<AppState>();
    match (choice, workflow) {
        (DiskChoice::Load, Some(workflow)) => {
            with_document(&state, |doc| doc.load_from_disk(workflow, version));
            changed(&app);
            Ok(true)
        }
        (DiskChoice::KeepMine, _) => {
            with_document(&state, |doc| doc.keep_file_version(version));
            save(&app).await
        }
        _ => Ok(false),
    }
}
