use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opsis_capture::live::FrameSink;
use opsis_stream::host::{ViewEnv, ViewHost};
use opsis_stream::picker::{PickRequest, PickResult, pick};
use opsis_stream::view::{StreamView, ViewEvent, ViewInput, ViewSink, ViewSize};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::engine::checked_engine_files;
use crate::error::{CommandError, CommandResult};
use crate::events::Emit;
use crate::frames::{FrameChannel, send};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ViewEventFired {
    pub id: String,
    #[specta(type = ViewEvent)]
    pub event: serde_json::Value,
}

#[derive(Debug)]
struct Entry {
    view: Arc<StreamView>,
    closed: CancellationToken,
}

#[derive(Debug, Default)]
pub struct Views {
    open: Mutex<HashMap<String, Entry>>,
}

impl Views {
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
    app: Session,
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
        let event = match serde_json::to_value(&event) {
            Ok(event) => event,
            Err(error) => {
                log::warn!("writing a view event: {error}");
                return;
            }
        };
        if let Err(error) = (ViewEventFired {
            id: self.id.clone(),
            event,
        })
        .emit(&self.app)
        {
            log::warn!("sending a view event: {error}");
        }
    }
}

#[derive(Debug)]
pub struct AppViewHost {
    pub app: Session,
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
        let on_close = closed.clone();
        let opened =
            self.app
                .platform()
                .open_view_window(&id, title, Box::new(move || on_close.cancel()));
        if let Err(error) = opened {
            log::error!("opening a view window: {error}");
            closed.cancel();
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
        if let Some(id) = id {
            self.app.platform().close_view_window(&id);
        }
    }
}

async fn view_env(state: &AppState) -> CommandResult<ViewEnv> {
    let engine = checked_engine_files(state, &CancellationToken::new()).await;
    Ok(ViewEnv {
        executable: engine.browser()?.to_path_buf(),
        temp_dir: state.paths.views_temp.clone(),
    })
}
pub async fn pick_element(app: Session, request: PickRequest) -> CommandResult<Option<PickResult>> {
    let state = app.state::<AppState>();
    let env = view_env(state).await?;
    let host = AppViewHost { app: app.clone() };
    Ok(pick(&env, &host, &request, &CancellationToken::new()).await?)
}
pub fn attach_view(app: Session, id: String, frames: FrameChannel) -> CommandResult<()> {
    let views = app.state::<Views>();
    let view = views.view(&id)?;
    view.attach(Arc::new(ChannelSink { app, id, frames }));
    Ok(())
}
pub async fn view_input(app: &Session, id: String, input: ViewInput) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.input(&input).await?)
}
pub async fn view_resize(app: &Session, id: String, size: ViewSize) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.resize(size).await?)
}
pub async fn view_navigate(app: &Session, id: String, url: String) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.navigate(&url).await?)
}
pub async fn view_go(app: &Session, id: String, steps: i64) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.go(steps).await?)
}
pub async fn view_reload(app: &Session, id: String) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.reload().await?)
}
pub async fn view_switch_tab(app: &Session, id: String, tab: String) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.switch_tab(&tab).await?)
}
pub async fn view_answer_dialog(
    app: &Session,
    id: String,
    accept: bool,
    prompt: String,
) -> CommandResult<()> {
    let views = app.state::<Views>();
    Ok(views.view(&id)?.answer_dialog(accept, &prompt).await?)
}
pub async fn view_choose_files(app: Session, id: String) -> CommandResult<()> {
    let views = app.state::<Views>();
    let view = views.view(&id)?;
    let files = crate::platform::pick_files(app.platform(), "Choose files", None).await;
    Ok(view.choose_files(&files).await?)
}
pub fn view_close(app: &Session, id: &str) {
    let views = app.state::<Views>();
    if let Some(entry) = views.open.lock().expect("views").get(id) {
        entry.closed.cancel();
    }
}
