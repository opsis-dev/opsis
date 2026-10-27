use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use opsis_capture::live::FrameSink;
use opsis_compile::nodes::output::{Variant, shaped_preview};
use opsis_stream::host::{ViewEnv, ViewHost};
use opsis_stream::picker::{PickRequest, PickResult, pick};
use opsis_stream::preview::{PlayerState, Preview, PreviewDirs};
use opsis_stream::view::{StreamView, ViewEvent, ViewInput, ViewSink, ViewSize};
use opsis_types::timeline::TimelineSpec;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;
use tokio_util::sync::CancellationToken;

use crate::engine::engine_files;
use crate::error::{CommandError, CommandResult};
use crate::frames::{FrameChannel, send};
use crate::state::AppState;

const VIEW_PREFIX: &str = "view-";
const PREVIEW_VIEW: &str = "preview";
const PREVIEW_QUALITY: u32 = 80;

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct ViewEventFired {
    pub id: String,
    #[specta(type = ViewEvent)]
    pub event: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct PreviewReported {
    pub frame: f64,
    pub playing: bool,
}

struct Entry {
    view: Arc<StreamView>,
    closed: CancellationToken,
}

#[derive(Default)]
pub struct Views {
    open: Mutex<HashMap<String, Entry>>,
    preview: tokio::sync::Mutex<Option<(PathBuf, Preview)>>,
}

impl Views {
    pub async fn close_preview(&self) {
        if let Some((_, preview)) = self.preview.lock().await.take() {
            preview.close().await;
        }
    }

    fn view(&self, id: &str) -> CommandResult<Arc<StreamView>> {
        self.open
            .lock()
            .expect("views")
            .get(id)
            .map(|e| Arc::clone(&e.view))
            .ok_or_else(|| CommandError("That view is closed.".into()))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameHeader {
    width: f64,
    height: f64,
    t: f64,
}

struct ChannelSink {
    app: AppHandle,
    id: String,
    frames: FrameChannel,
}

impl FrameSink for ChannelSink {
    fn frame(&self, jpeg: &[u8], width: f64, height: f64, t: f64) {
        send(&self.frames, &FrameHeader { width, height, t }, jpeg);
    }
}

impl ViewSink for ChannelSink {
    fn event(&self, event: ViewEvent) {
        let event = serde_json::to_value(&event).map_err(tauri::Error::from);
        if let Err(error) = event.and_then(|event| {
            (ViewEventFired {
                id: self.id.clone(),
                event,
            })
            .emit(&self.app)
        }) {
            log::warn!("sending a view event: {error}");
        }
    }
}

pub struct AppViewHost {
    pub app: AppHandle,
}

impl ViewHost for AppViewHost {
    fn show(&self, title: &str, view: Arc<StreamView>, closed: CancellationToken) {
        let id = uuid::Uuid::new_v4().simple().to_string();
        self.app
            .state::<Views>()
            .open
            .lock()
            .expect("views")
            .insert(
                id.clone(),
                Entry {
                    view,
                    closed: closed.clone(),
                },
            );
        let label = format!("{VIEW_PREFIX}{id}");
        let url = WebviewUrl::App(format!("index.html#/view/{id}").into());
        let built = WebviewWindowBuilder::new(&self.app, &label, url)
            .title(title)
            .inner_size(1280.0, 900.0)
            .build();
        match built {
            Ok(window) => {
                window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Destroyed) {
                        closed.cancel();
                    }
                });
            }
            Err(error) => {
                log::error!("opening a view window: {error}");
                closed.cancel();
            }
        }
    }

    fn hide(&self, view: &Arc<StreamView>) {
        let views = self.app.state::<Views>();
        let id = {
            let mut open = views.open.lock().expect("views");
            let id = open
                .iter()
                .find(|(_, e)| Arc::ptr_eq(&e.view, view))
                .map(|(id, _)| id.clone());
            if let Some(id) = &id {
                open.remove(id);
            }
            id
        };
        if let Some(window) =
            id.and_then(|id| self.app.get_webview_window(&format!("{VIEW_PREFIX}{id}")))
            && let Err(error) = window.destroy()
        {
            log::warn!("closing a view window: {error}");
        }
    }
}

fn view_env(state: &AppState) -> CommandResult<ViewEnv> {
    Ok(ViewEnv {
        executable: engine_files(state).browser()?.to_path_buf(),
        temp_dir: state.paths.views_temp.clone(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn pick_element(
    app: AppHandle,
    state: State<'_, AppState>,
    request: PickRequest,
) -> CommandResult<Option<PickResult>> {
    let env = view_env(&state)?;
    let host = AppViewHost { app: app.clone() };
    Ok(pick(&env, &host, &request, &CancellationToken::new()).await?)
}

#[tauri::command]
#[specta::specta]
pub fn attach_view(
    app: AppHandle,
    views: State<'_, Views>,
    id: String,
    frames: FrameChannel,
) -> CommandResult<()> {
    let view = views.view(&id)?;
    view.attach(Arc::new(ChannelSink { app, id, frames }));
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn view_input(
    views: State<'_, Views>,
    id: String,
    input: ViewInput,
) -> CommandResult<()> {
    Ok(views.view(&id)?.input(&input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_resize(views: State<'_, Views>, id: String, size: ViewSize) -> CommandResult<()> {
    Ok(views.view(&id)?.resize(size).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_navigate(views: State<'_, Views>, id: String, url: String) -> CommandResult<()> {
    Ok(views.view(&id)?.navigate(&url).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_go(views: State<'_, Views>, id: String, steps: i64) -> CommandResult<()> {
    Ok(views.view(&id)?.go(steps).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_reload(views: State<'_, Views>, id: String) -> CommandResult<()> {
    Ok(views.view(&id)?.reload().await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_switch_tab(
    views: State<'_, Views>,
    id: String,
    tab: String,
) -> CommandResult<()> {
    Ok(views.view(&id)?.switch_tab(&tab).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_answer_dialog(
    views: State<'_, Views>,
    id: String,
    accept: bool,
    prompt: String,
) -> CommandResult<()> {
    Ok(views.view(&id)?.answer_dialog(accept, &prompt).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn view_choose_files(
    app: AppHandle,
    views: State<'_, Views>,
    id: String,
) -> CommandResult<()> {
    let view = views.view(&id)?;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_files(move |files| {
        if sender.send(files).is_err() {
            log::debug!("the file dialog closed after its caller left");
        }
    });
    let files: Vec<PathBuf> = receiver
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| f.into_path().ok())
        .collect();
    Ok(view.choose_files(&files).await?)
}

#[tauri::command]
#[specta::specta]
pub fn view_close(views: State<'_, Views>, id: String) {
    if let Some(entry) = views.open.lock().expect("views").get(&id) {
        entry.closed.cancel();
    }
}

async fn with_preview<T>(
    views: &Views,
    act: impl AsyncFnOnce(&Preview) -> opsis_capture::cdp::Result<T>,
) -> CommandResult<T> {
    let held = views.preview.lock().await;
    let (_, preview) = held
        .as_ref()
        .ok_or_else(|| CommandError("The preview isn't open.".into()))?;
    Ok(act(preview).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn preview_open(
    app: AppHandle,
    state: State<'_, AppState>,
    views: State<'_, Views>,
    project_dir: String,
    size: ViewSize,
    frames: FrameChannel,
) -> CommandResult<String> {
    let project = PathBuf::from(project_dir);
    let mut held = views.preview.lock().await;
    let reuse = held.as_ref().is_some_and(|(dir, _)| *dir == project);
    if !reuse {
        if let Some((_, old)) = held.take() {
            old.close().await;
        }
        let env = view_env(&state)?;
        let dirs = PreviewDirs {
            page: &state.paths.preview_page,
            public: &state.paths.public_dir,
            project: &project,
        };
        let preview = Preview::open(&env, &dirs, size, PREVIEW_QUALITY).await?;
        let mut frames = preview.frames();
        let mut states = preview.states();
        let emitter = app.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let changed = tokio::select! {
                    changed = frames.changed() => changed,
                    changed = states.changed() => changed,
                };
                if changed.is_err() {
                    break;
                }
                let reported = PreviewReported {
                    frame: *frames.borrow(),
                    playing: *states.borrow() == PlayerState::Playing,
                };
                if let Err(error) = reported.emit(&emitter) {
                    log::warn!("sending the preview frame: {error}");
                }
            }
        });
        *held = Some((project, preview));
    }
    let (_, preview) = held.as_ref().expect("the preview was just opened");
    if reuse {
        preview.view().resize(size).await?;
    }
    preview.view().attach(Arc::new(ChannelSink {
        app,
        id: PREVIEW_VIEW.to_owned(),
        frames,
    }));
    Ok(PREVIEW_VIEW.to_owned())
}

#[tauri::command]
#[specta::specta]
pub async fn preview_load(views: State<'_, Views>, spec: TimelineSpec) -> CommandResult<()> {
    with_preview(&views, async |p| p.load(&spec).await).await
}

#[tauri::command]
#[specta::specta]
pub fn preview_spec(
    state: State<'_, AppState>,
    spec: TimelineSpec,
    shape: Variant,
) -> TimelineSpec {
    let editor = state.editor();
    match editor.document.as_ref() {
        Some(doc) => shaped_preview(&spec, shape, doc.workflow()),
        None => spec,
    }
}

#[tauri::command]
#[specta::specta]
pub async fn preview_play(views: State<'_, Views>) -> CommandResult<()> {
    with_preview(&views, async |p| p.play().await).await
}

#[tauri::command]
#[specta::specta]
pub async fn preview_pause(views: State<'_, Views>) -> CommandResult<()> {
    with_preview(&views, async |p| p.pause().await).await
}

#[tauri::command]
#[specta::specta]
pub async fn preview_seek(views: State<'_, Views>, frame: f64) -> CommandResult<()> {
    with_preview(&views, async |p| p.seek(frame).await).await
}

#[tauri::command]
#[specta::specta]
pub async fn preview_resize(views: State<'_, Views>, size: ViewSize) -> CommandResult<()> {
    with_preview(&views, async |p| p.view().resize(size).await).await
}

#[tauri::command]
#[specta::specta]
pub async fn preview_close(views: State<'_, Views>) -> CommandResult<()> {
    views.close_preview().await;
    Ok(())
}
