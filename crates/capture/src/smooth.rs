use std::sync::{Arc, Weak};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_compile::nodes::capture::FramePicture;
use opsis_types::actions::Viewport;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::cdp::{Result, Tab};
use crate::clock::wall_ms;
use crate::encoder::FrameFeed;
use crate::live::FrameSink;
use crate::page_scripts::{PageNow, STEP_FRAME, SteppedPage, call, page_time_for_new_pages};
use crate::take_clock::{Stepper, TickFuture};

pub struct FramePictures {
    pub encoder: Option<FrameFeed>,
    pub view: Option<Arc<dyn FrameSink>>,
    pub picture: FramePicture,
    pub quality: u32,
    pub viewport: Viewport,
    pub dpr: f64,
}

pub struct Smooth {
    tab: Weak<Tab>,
    frame_ms: f64,
    pictures: FramePictures,
    new_page_time: Mutex<Option<String>>,
    scrolled: std::sync::Mutex<(f64, f64)>,
}

#[derive(Deserialize)]
struct Shot {
    data: String,
}

#[derive(Deserialize)]
struct Added {
    identifier: String,
}

pub fn page_ms(frame: u64, frame_ms: f64) -> f64 {
    let at = |n: u64| (n as f64 * frame_ms).round();
    at(frame) - at(frame.saturating_sub(1))
}

impl Smooth {
    pub async fn start(tab: &Arc<Tab>, fps: u32, pictures: FramePictures) -> Result<Arc<Self>> {
        tab.send("Animation.enable", json!({})).await?;
        tab.send("Animation.setPlaybackRate", json!({ "playbackRate": 0 }))
            .await?;
        let scrolled: (f64, f64) = tab
            .evaluate_main("[visualViewport.pageLeft, visualViewport.pageTop]")
            .await?;
        Ok(Arc::new(Self {
            tab: Arc::downgrade(tab),
            frame_ms: 1000.0 / f64::from(fps.max(1)),
            pictures,
            new_page_time: Mutex::new(None),
            scrolled: std::sync::Mutex::new(scrolled),
        }))
    }

    pub async fn picture(&self, at: f64) {
        let Some(tab) = self.tab.upgrade() else {
            return;
        };
        let shot = tab
            .send_as::<Shot>("Page.captureScreenshot", self.shot_params())
            .await;
        let jpeg = match shot.map(|s| STANDARD.decode(s.data)) {
            Ok(Ok(jpeg)) => jpeg,
            Ok(Err(error)) => {
                log::debug!("reading a frame: {error}");
                return;
            }
            Err(error) => {
                log::debug!("taking a frame: {error}");
                return;
            }
        };
        if let Some(view) = &self.pictures.view {
            view.frame(
                &jpeg,
                f64::from(self.pictures.viewport.width),
                f64::from(self.pictures.viewport.height),
                at,
            );
        }
        if let Some(encoder) = &self.pictures.encoder {
            encoder.offer(jpeg, at);
        }
    }

    fn shot_params(&self) -> Value {
        let (left, top) = *self.scrolled.lock().expect("scroll");
        let scale = match self.pictures.picture {
            FramePicture::Standard | FramePicture::Lossless => 1.0 / self.pictures.dpr.max(0.1),
            FramePicture::FullDetail => 1.0,
        };
        let clip = json!({
            "x": left,
            "y": top,
            "width": self.pictures.viewport.width,
            "height": self.pictures.viewport.height,
            "scale": scale,
        });
        match self.pictures.picture {
            FramePicture::Lossless => json!({ "format": "png", "clip": clip }),
            FramePicture::Standard | FramePicture::FullDetail => {
                json!({ "format": "jpeg", "quality": self.pictures.quality, "clip": clip })
            }
        }
    }

    async fn advance_page(&self, tab: &Tab, ms: f64) {
        let stepped = match tab
            .evaluate_main::<SteppedPage>(&call(STEP_FRAME, &json!(ms)))
            .await
        {
            Ok(stepped) => stepped,
            Err(error) => {
                log::debug!("moving the page on a frame: {error}");
                return;
            }
        };
        *self.scrolled.lock().expect("scroll") = (stepped.left, stepped.top);
        self.keep_page_time(tab, stepped.page).await;
    }

    async fn keep_page_time(&self, tab: &Tab, now: Option<PageNow>) {
        let Some(now) = now else {
            return;
        };
        let mut kept = self.new_page_time.lock().await;
        if let Some(previous) = kept.take()
            && let Err(error) = tab
                .send(
                    "Page.removeScriptToEvaluateOnNewDocument",
                    json!({ "identifier": previous }),
                )
                .await
        {
            log::debug!("replacing the page time for new pages: {error}");
        }
        match tab
            .send_as::<Added>(
                "Page.addScriptToEvaluateOnNewDocument",
                json!({ "source": page_time_for_new_pages(now, wall_ms()) }),
            )
            .await
        {
            Ok(added) => *kept = Some(added.identifier),
            Err(error) => log::debug!("keeping the page time for new pages: {error}"),
        }
    }
}

impl Stepper for Smooth {
    fn frame_ms(&self) -> f64 {
        self.frame_ms
    }

    fn tick(&self, frame: u64, at: f64) -> TickFuture<'_> {
        Box::pin(async move {
            let Some(tab) = self.tab.upgrade() else {
                return;
            };
            self.advance_page(&tab, page_ms(frame, self.frame_ms)).await;
            self.picture(at).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_the_page_by_whole_milliseconds_that_add_up_to_the_frame_times() {
        let frame_ms = 1000.0 / 30.0;
        let steps: Vec<f64> = (1..=6).map(|f| page_ms(f, frame_ms)).collect();
        assert_eq!(steps, [33.0, 34.0, 33.0, 33.0, 34.0, 33.0]);
        let total: f64 = (1..=30).map(|f| page_ms(f, frame_ms)).sum();
        assert_eq!(total, 1000.0);
        assert_eq!(page_ms(1, 1000.0 / 60.0), 17.0);
    }
}
