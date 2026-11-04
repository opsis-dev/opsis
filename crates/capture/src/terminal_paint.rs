use std::fmt;
use std::path::Path;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use opsis_compile::nodes::output::Gl;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_types::fonts::FontFamilyDef;
use opsis_types::terminal::TerminalLook;
use serde::Deserialize;
use serde_json::json;

use crate::browser::{Launched, launch_with};
use crate::cdp::{CaptureError, Result, Tab};
use crate::film::{Capture, film_switches};

const TERMINAL_FOLDER: &str = ".opsis/terminal";
const PAGE_TIMEOUT: Duration = Duration::from_secs(60);
const PAGE_POLL: Duration = Duration::from_millis(50);
const START_SIZE: u32 = 800;
const JPEG_QUALITY: u8 = 85;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Painted {
    pub width: u32,
    pub height: u32,
}

pub struct TerminalPainter {
    launched: Launched,
    tab: Tab,
    _server: FileServer,
}

impl fmt::Debug for TerminalPainter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalPainter").finish_non_exhaustive()
    }
}

async fn wait_for_page(tab: &Tab) -> Result<()> {
    tokio::time::timeout(PAGE_TIMEOUT, async {
        loop {
            let ready: bool = tab
                .evaluate_main("typeof window.opsisTerminal?.paint === 'function'")
                .await
                .unwrap_or(false);
            if ready {
                return;
            }
            tokio::time::sleep(PAGE_POLL).await;
        }
    })
    .await
    .map_err(|_| CaptureError::failed("The terminal page didn't start."))
}

impl TerminalPainter {
    pub async fn open(
        executable: &Path,
        temp_dir: &Path,
        terminal_bundle: &Path,
        public_dir: &Path,
        fonts: &[FontFamilyDef],
    ) -> Result<Self> {
        let routes = folder_routes(TERMINAL_FOLDER, terminal_bundle);
        let server = FileServer::start_with(public_dir, routes)
            .await
            .map_err(|e| CaptureError::failed(e.to_string()))?;
        let switches = film_switches(Gl::Angle, Capture::Screenshot, START_SIZE, START_SIZE);
        let launched = launch_with(executable, temp_dir, switches).await?;
        let opened = async {
            let page = launched.browser.new_page("about:blank").await?;
            let tab = Tab::new(page);
            let url = format!("{}{TERMINAL_FOLDER}/index.html", server.url);
            tab.send("Page.navigate", json!({ "url": url })).await?;
            wait_for_page(&tab).await?;
            let defs =
                serde_json::to_string(fonts).map_err(|e| CaptureError::failed(e.to_string()))?;
            tab.evaluate_main::<bool>(&format!("window.opsisTerminal.fonts({defs})"))
                .await?;
            Ok::<_, CaptureError>(tab)
        }
        .await;
        match opened {
            Ok(tab) => Ok(Self {
                launched,
                tab,
                _server: server,
            }),
            Err(error) => {
                launched.close().await;
                Err(error)
            }
        }
    }

    pub async fn paint(&self, frames: &[u8], look: &TerminalLook) -> Result<(Painted, Vec<u8>)> {
        let look = serde_json::to_string(look).map_err(|e| CaptureError::failed(e.to_string()))?;
        let painted: Painted = self
            .tab
            .evaluate_main(&format!(
                "window.opsisTerminal.paint({}, {look})",
                serde_json::Value::from(STANDARD.encode(frames))
            ))
            .await?;
        let shot = self
            .tab
            .send(
                "Page.captureScreenshot",
                json!({
                    "format": "jpeg",
                    "quality": JPEG_QUALITY,
                    "captureBeyondViewport": true,
                    "clip": { "x": 0, "y": 0, "width": painted.width, "height": painted.height, "scale": 1 }
                }),
            )
            .await?;
        let data = shot["data"]
            .as_str()
            .ok_or_else(|| CaptureError::failed("The terminal picture came back empty."))?;
        let jpeg = STANDARD.decode(data).map_err(|e| {
            CaptureError::failed(format!("The terminal picture came back unreadable: {e}"))
        })?;
        Ok((painted, jpeg))
    }

    pub async fn close(self) {
        if let Err(error) = self.tab.page.clone().close().await {
            log::debug!("closing the terminal page: {error}");
        }
        self.launched.close().await;
    }
}
