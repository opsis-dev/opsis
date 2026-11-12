use std::path::Path;
use std::time::Duration;

use opsis_capture::browser::{Launched, launch_with};
use opsis_capture::cdp::{CaptureError, Tab};
use opsis_capture::film::{Capture, film_switches};
use opsis_types::timeline::TimelineSpec;
use serde_json::{Value, json};

use crate::pictures::ORACLE_GL;

const READY_SCRIPT: &str = "new Promise((ready) => { const check = () => (window.opsis && window.oracleBoxes ? ready(true) : setTimeout(check, 20)); check(); })";
const PAGE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum BoxesError {
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error("the case page didn't start within {0:?}")]
    Timeout(Duration),
}

#[derive(Debug)]
pub struct BoxReader {
    launched: Launched,
    width: u32,
    height: u32,
}

impl BoxReader {
    pub async fn open(shell: &Path, temp: &Path, width: u32, height: u32) -> Result<Self, BoxesError> {
        let switches = film_switches(ORACLE_GL, Capture::Screenshot, width, height);
        Ok(Self {
            launched: launch_with(shell, temp, switches).await?,
            width,
            height,
        })
    }

    pub async fn boxes(&self, url: &str, spec: &TimelineSpec) -> Result<Value, BoxesError> {
        let page = self
            .launched
            .browser
            .new_page(url)
            .await
            .map_err(|e| CaptureError::Cdp(e.to_string()))?;
        let tab = Tab::new(page);
        let read = async {
            tab.send(
                "Emulation.setDeviceMetricsOverride",
                json!({ "width": self.width, "height": self.height, "deviceScaleFactor": 1, "mobile": false }),
            )
            .await?;
            tokio::time::timeout(PAGE_TIMEOUT, tab.evaluate_main::<bool>(READY_SCRIPT))
                .await
                .map_err(|_| BoxesError::Timeout(PAGE_TIMEOUT))??;
            let spec = serde_json::to_string(spec).expect("a spec as JSON");
            Ok::<Value, BoxesError>(
                tab.evaluate_main::<Value>(&format!(
                    "window.opsis.load({spec}).then(() => window.oracleBoxes())"
                ))
                .await?,
            )
        };
        let boxes = read.await;
        if let Err(error) = tab.page.clone().close().await {
            eprintln!("closing a case page: {error}");
        }
        boxes
    }

    pub async fn close(self) {
        self.launched.close().await;
    }
}
