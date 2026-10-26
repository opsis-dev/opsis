use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chromiumoxide::cdp::js_protocol::runtime::EventBindingCalled;
use futures::StreamExt;
use opsis_capture::cdp::{CaptureError, Result};
use opsis_engine::http_files::FileServer;
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::host::ViewEnv;
use crate::view::{StreamView, ViewSize};

const FRAME_BINDING: &str = "opsisFrame";
const STATE_BINDING: &str = "opsisState";
const PAGE_READY: Duration = Duration::from_secs(30);
const PREVIEW_SWITCHES: [&str; 1] = ["--enable-features=CanvasDrawElement"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Loading,
    Waiting,
    Ready,
    Playing,
    Paused,
}

fn state_of(payload: &str) -> Option<PlayerState> {
    match payload {
        "waiting" => Some(PlayerState::Waiting),
        "ready" => Some(PlayerState::Ready),
        "playing" => Some(PlayerState::Playing),
        "paused" => Some(PlayerState::Paused),
        _ => None,
    }
}

struct PageServer {
    url: String,
    stop: CancellationToken,
}

impl PageServer {
    async fn start(page_dir: &Path, public_dir: &Path) -> std::io::Result<Self> {
        let listener =
            tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let url = format!("http://{}/", listener.local_addr()?);
        let routes = Router::new()
            .fallback_service(ServeDir::new(page_dir).fallback(ServeDir::new(public_dir)));
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, routes)
                .with_graceful_shutdown(stopped.cancelled_owned())
                .await
            {
                log::warn!("preview page server stopped: {error}");
            }
        });
        Ok(Self { url, stop })
    }
}

impl Drop for PageServer {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}

#[derive(Deserialize)]
struct Captured {
    data: String,
}

pub struct PreviewDirs<'a> {
    pub page: &'a Path,
    pub public: &'a Path,
    pub project: &'a Path,
}

pub struct Preview {
    view: Arc<StreamView>,
    frame: watch::Receiver<f64>,
    state: watch::Receiver<PlayerState>,
    listener: JoinHandle<()>,
    project: FileServer,
    _page: PageServer,
}

impl Preview {
    pub async fn open(
        env: &ViewEnv,
        dirs: &PreviewDirs<'_>,
        size: ViewSize,
        quality: u32,
    ) -> Result<Self> {
        let page = PageServer::start(dirs.page, dirs.public).await?;
        let prepared = PreparedDirs {
            project: dirs.project.to_path_buf(),
            public: dirs.public.to_path_buf(),
        };
        let project = FileServer::start_with(dirs.project, prepared_routes(prepared)).await?;
        let view = StreamView::open(
            &env.executable,
            &env.temp_dir,
            "about:blank",
            size,
            quality,
            &PREVIEW_SWITCHES,
        )
        .await?;
        let tab = view.active_tab().await;
        let started = async {
            let mut calls = tab.page.event_listener::<EventBindingCalled>().await?;
            for name in [FRAME_BINDING, STATE_BINDING] {
                tab.send("Runtime.addBinding", json!({ "name": name }))
                    .await?;
            }
            let (frame_sender, frame) = watch::channel(0.0);
            let (state_sender, state) = watch::channel(PlayerState::Loading);
            let listener = tokio::spawn(async move {
                while let Some(event) = calls.next().await {
                    match event.name.as_str() {
                        FRAME_BINDING => {
                            if let Ok(at) = event.payload.parse::<f64>() {
                                frame_sender.send_replace(at);
                            }
                        }
                        STATE_BINDING => {
                            if let Some(now) = state_of(&event.payload) {
                                state_sender.send_replace(now);
                            }
                        }
                        _ => {}
                    }
                }
            });
            view.navigate(&page.url).await?;
            Ok::<_, CaptureError>((frame, state, listener))
        }
        .await;
        drop(tab);
        match started {
            Ok((frame, state, listener)) => {
                let preview = Self {
                    view,
                    frame,
                    state,
                    listener,
                    project,
                    _page: page,
                };
                preview.until(|s| s != PlayerState::Loading).await?;
                Ok(preview)
            }
            Err(error) => {
                view.close().await;
                Err(error)
            }
        }
    }

    pub fn view(&self) -> &Arc<StreamView> {
        &self.view
    }

    pub fn frames(&self) -> watch::Receiver<f64> {
        self.frame.clone()
    }

    pub fn states(&self) -> watch::Receiver<PlayerState> {
        self.state.clone()
    }

    pub fn state(&self) -> PlayerState {
        *self.state.borrow()
    }

    pub async fn until(&self, done: impl Fn(PlayerState) -> bool) -> Result<()> {
        let mut state = self.state.clone();
        tokio::time::timeout(PAGE_READY, state.wait_for(|s| done(*s)))
            .await
            .map_err(|_| CaptureError::failed("The preview didn't start."))
            .map(drop)
    }

    async fn call(&self, script: &str) -> Result<()> {
        self.view
            .active_tab()
            .await
            .evaluate_main::<Value>(&format!("window.opsisPreview.{script}"))
            .await
            .map(drop)
    }

    pub async fn load(&self, spec: &TimelineSpec) -> Result<()> {
        let versions = spec.assets.as_ref().and_then(|a| a.versions.clone());
        let spec = TimelineSpec {
            assets: Some(SpecAssets {
                project_base: Some(self.project.url.clone()),
                versions,
            }),
            ..spec.clone()
        };
        let json = serde_json::to_string(&spec).map_err(|e| CaptureError::Failed(e.to_string()))?;
        self.call(&format!("load({json})")).await?;
        self.until(|s| s != PlayerState::Waiting).await
    }

    pub async fn play(&self) -> Result<()> {
        self.call("play()").await
    }

    pub async fn pause(&self) -> Result<()> {
        self.call("pause()").await
    }

    pub async fn seek(&self, frame: f64) -> Result<()> {
        self.call(&format!("seek({frame})")).await
    }

    pub async fn still(&self) -> Result<Vec<u8>> {
        let shot: Captured = self
            .view
            .active_tab()
            .await
            .send_as("Page.captureScreenshot", json!({ "format": "png" }))
            .await?;
        STANDARD
            .decode(shot.data)
            .map_err(|e| CaptureError::Failed(e.to_string()))
    }

    pub async fn close(self) {
        self.listener.abort();
        self.view.close().await;
    }
}
