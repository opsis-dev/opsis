use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::future::BoxFuture;
use opsis_api::address::{API_SOCKET_ENV, Host, api_address};
use opsis_api::routes::ApiHost;
use opsis_api::server::ApiServer;
use opsis_types::events::RunEvent;
use opsis_types::shell::ApiServerStatus;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;
use tokio::sync::{broadcast, oneshot, watch};

use crate::events::{ApiRequestReceived, ApiStatusChanged};
use crate::state::AppState;
use crate::windows::MAIN_WINDOW;

const EDITOR_WAIT: Duration = Duration::from_secs(20);
const RUN_EVENTS_KEPT: usize = 256;

fn failure(status: u16, error: &str) -> Value {
    json!({ "ok": false, "status": status, "error": error })
}

pub struct Api {
    server: ApiServer,
    pending: Mutex<HashMap<String, oneshot::Sender<Value>>>,
    listening: watch::Sender<bool>,
    events: broadcast::Sender<String>,
}

impl Api {
    pub fn new(app: &AppHandle) -> Self {
        let state = app.state::<AppState>();
        let home = app.path().home_dir().unwrap_or_default();
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_default();
        let custom = std::env::var(API_SOCKET_ENV).ok();
        let address = api_address(&Host {
            windows: cfg!(windows),
            user_data: &state.paths.user_data,
            home: &home,
            username: &username,
            custom: custom.as_deref(),
        });
        Self {
            server: ApiServer::new(address),
            pending: Mutex::default(),
            listening: watch::Sender::new(false),
            events: broadcast::Sender::new(RUN_EVENTS_KEPT),
        }
    }

    fn settle_all(&self, reply: &Value) {
        for (_, waiting) in self.pending.lock().expect("API replies").drain() {
            if waiting.send(reply.clone()).is_err() {
                log::debug!("an API caller left before its answer");
            }
        }
    }

    pub fn editor_reloaded(&self) {
        self.listening.send_replace(false);
        self.settle_all(&failure(503, "The editor reloaded before it answered."));
    }

    pub fn editor_closed(&self) {
        self.listening.send_replace(false);
        self.settle_all(&failure(409, "No project is open."));
    }
}

struct AppApi {
    app: AppHandle,
}

impl ApiHost for AppApi {
    fn version(&self) -> String {
        self.app.package_info().version.to_string()
    }

    fn ask_editor(&self, request: Value) -> BoxFuture<'static, Value> {
        let app = self.app.clone();
        Box::pin(async move {
            if app.get_webview_window(MAIN_WINDOW).is_none() {
                return failure(409, "No project is open.");
            }
            let api = app.state::<Api>();
            let mut listening = api.listening.subscribe();
            if tokio::time::timeout(EDITOR_WAIT, listening.wait_for(|l| *l))
                .await
                .is_err()
            {
                return failure(504, "The editor didn't answer in time.");
            }
            let id = uuid::Uuid::new_v4().to_string();
            let (sender, receiver) = oneshot::channel();
            api.pending
                .lock()
                .expect("API replies")
                .insert(id.clone(), sender);
            if let Err(error) = (ApiRequestReceived {
                id: id.clone(),
                request,
            })
            .emit_to(&app, MAIN_WINDOW)
            {
                api.pending.lock().expect("API replies").remove(&id);
                return failure(500, &error.to_string());
            }
            match tokio::time::timeout(EDITOR_WAIT, receiver).await {
                Ok(Ok(reply)) => reply,
                _ => {
                    api.pending.lock().expect("API replies").remove(&id);
                    failure(504, "The editor didn't answer in time.")
                }
            }
        })
    }

    fn run_events(&self) -> broadcast::Receiver<String> {
        self.app.state::<Api>().events.subscribe()
    }
}

fn announce(app: &AppHandle, status: ApiServerStatus) {
    if let Err(error) = ApiStatusChanged(status).emit(app) {
        log::warn!("sending the API status: {error}");
    }
}

pub fn follow_setting(app: &AppHandle) {
    let enabled = app
        .state::<AppState>()
        .db()
        .read_settings()
        .map(|s| s.api.enabled)
        .unwrap_or(false);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let api = app.state::<Api>();
        if api.server.status().enabled == enabled && (api.server.status().listening || !enabled) {
            return;
        }
        let status = api
            .server
            .set_enabled(enabled, Arc::new(AppApi { app: app.clone() }))
            .await;
        announce(&app, status);
    });
}

pub fn stop(app: &AppHandle) {
    app.state::<Api>().server.stop();
}

pub fn run_event(app: &AppHandle, event: &RunEvent) {
    let api = app.state::<Api>();
    if api.events.receiver_count() == 0 {
        return;
    }
    match serde_json::to_string(event) {
        Ok(json) => {
            if api.events.send(json).is_err() {
                log::debug!("no one follows the run events");
            }
        }
        Err(error) => log::warn!("writing a run event for the API: {error}"),
    }
}

#[tauri::command]
#[specta::specta]
pub fn api_status(api: State<'_, Api>) -> ApiServerStatus {
    api.server.status()
}

#[tauri::command]
#[specta::specta]
pub fn api_listening(api: State<'_, Api>, listening: bool) {
    api.listening.send_replace(listening);
}

#[tauri::command]
#[specta::specta]
pub fn api_reply(api: State<'_, Api>, id: String, reply: Value) {
    if let Some(waiting) = api.pending.lock().expect("API replies").remove(&id)
        && waiting.send(reply).is_err()
    {
        log::debug!("an API caller left before its answer");
    }
}
