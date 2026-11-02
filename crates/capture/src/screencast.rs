use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chromiumoxide::cdp::browser_protocol::page::EventScreencastFrame;
use futures::StreamExt;
use opsis_types::actions::Viewport;
use serde_json::json;
use tokio::task::JoinHandle;

use crate::cdp::{Result, Tab};
use crate::clock::{as_ms, wall_time};
use crate::encoder::FrameFeed;
use crate::live::FrameSink;
use crate::trace::Trace;

pub const DEFAULT_FRAME_QUALITY: u32 = 80;

#[derive(Default)]
pub struct FrameSinks {
    pub encoder: Option<FrameFeed>,
    pub view: Option<Arc<dyn FrameSink>>,
    pub trace: Option<Arc<Trace>>,
}

impl fmt::Debug for FrameSinks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameSinks")
            .field("encoder", &self.encoder)
            .field("trace", &self.trace)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Screencast {
    task: JoinHandle<()>,
}

const fn even(px: u32) -> u32 {
    px & !1
}

impl Screencast {
    pub async fn start(
        tab: &Arc<Tab>,
        size: Viewport,
        quality: u32,
        sinks: FrameSinks,
    ) -> Result<Self> {
        let mut frames = tab.page.event_listener::<EventScreencastFrame>().await?;
        let acks = Arc::clone(tab);
        let task = tokio::spawn(async move {
            while let Some(frame) = frames.next().await {
                if let Err(error) = acks
                    .send(
                        "Page.screencastFrameAck",
                        json!({ "sessionId": frame.session_id }),
                    )
                    .await
                {
                    log::debug!("acknowledging a frame: {error}");
                }
                let encoded: &str = frame.data.as_ref();
                let Ok(jpeg) = STANDARD.decode(encoded) else {
                    continue;
                };
                let at = frame
                    .metadata
                    .timestamp
                    .as_ref()
                    .and_then(|s| Duration::try_from_secs_f64(*s.inner()).ok())
                    .unwrap_or_else(wall_time);
                let t = as_ms(at);
                if let Some(view) = &sinks.view {
                    view.frame(
                        &jpeg,
                        frame.metadata.device_width,
                        frame.metadata.device_height,
                        t,
                    );
                }
                if let Some(trace) = &sinks.trace {
                    trace.frame(&jpeg, t);
                }
                if let Some(encoder) = &sinks.encoder {
                    encoder.offer(jpeg, at);
                }
            }
        });
        let started = tab.send("Page.startScreencast", json!({ "format": "jpeg", "quality": quality, "maxWidth": even(size.width), "maxHeight": even(size.height) })).await;
        if let Err(error) = started {
            task.abort();
            return Err(error);
        }
        Ok(Self { task })
    }

    pub async fn stop(self, tab: &Tab) -> Result<()> {
        let stopped = tab.send("Page.stopScreencast", json!({})).await;
        self.task.abort();
        if let Err(error) = self.task.await
            && !error.is_cancelled()
        {
            log::debug!("the frame listener stopped: {error}");
        }
        stopped.map(drop)
    }
}
