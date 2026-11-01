use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chromiumoxide::cdp::browser_protocol::page::EventScreencastFrame;
use futures::StreamExt;
use opsis_types::actions::Viewport;
use serde_json::json;
use tokio::task::JoinHandle;

use crate::cdp::{Result, Tab};
use crate::clock::wall_ms;
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

pub struct Screencast {
    task: JoinHandle<()>,
}

fn even(px: u32) -> u32 {
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
                let t = frame
                    .metadata
                    .timestamp
                    .as_ref()
                    .map_or_else(wall_ms, |s| *s.inner() * 1000.0);
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
                    encoder.offer(jpeg, t);
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
