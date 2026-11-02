use std::fmt;
use std::sync::{Arc, Weak};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_compile::nodes::capture::FramePicture;
use opsis_types::actions::Viewport;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::cdp::{Arg, Result, Tab};
use crate::clock::{as_ms, wall_ms};
use crate::encoder::FrameFeed;
use crate::live::FrameSink;
use crate::page_scripts::{
    PageNow, STEP_FRAME, STEP_OVERLAYS, SteppedPage, call, page_time_for_new_pages,
};
use crate::take_clock::{Stepper, TickFuture};

pub struct FramePictures {
    pub encoder: Option<FrameFeed>,
    pub view: Option<Arc<dyn FrameSink>>,
    pub picture: FramePicture,
    pub quality: u32,
    pub viewport: Viewport,
    pub dpr: f64,
}

impl fmt::Debug for FramePictures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramePictures")
            .field("encoder", &self.encoder)
            .field("picture", &self.picture)
            .field("quality", &self.quality)
            .field("viewport", &self.viewport)
            .field("dpr", &self.dpr)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Smooth {
    tab: Weak<Tab>,
    fps: u32,
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

pub fn page_ms(frame: u64, fps: u32) -> u64 {
    let fps = u128::from(fps.max(1));
    let at = |n: u64| (u128::from(n) * 2000 + fps) / (2 * fps);
    u64::try_from(at(frame) - at(frame.saturating_sub(1))).unwrap_or(u64::MAX)
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
            fps: fps.max(1),
            pictures,
            new_page_time: Mutex::new(None),
            scrolled: std::sync::Mutex::new(scrolled),
        }))
    }

    pub async fn picture(&self, at: Duration) {
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
                as_ms(at),
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

    async fn advance_page(&self, tab: &Tab, ms: u64) {
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
        if let Err(error) = tab
            .injected_value::<Value>(STEP_OVERLAYS, &[Arg::Value(json!(ms))])
            .await
        {
            log::debug!("moving the overlays on a frame: {error}");
        }
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
    fn fps(&self) -> u32 {
        self.fps
    }

    fn tick(&self, frame: u64, at: Duration) -> TickFuture<'_> {
        Box::pin(async move {
            let Some(tab) = self.tab.upgrade() else {
                return;
            };
            self.advance_page(&tab, page_ms(frame, self.fps)).await;
            self.picture(at).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn moves_the_page_by_whole_milliseconds_that_add_up_to_the_frame_times(
            fps in prop::sample::select(vec![12u32, 24, 25, 30, 48, 50, 60, 120]),
            frames in 1u64..10_000,
        ) {
            let shortest = 1000 / u64::from(fps);
            let longest = 1000_u64.div_ceil(u64::from(fps));
            let steps: Vec<u64> = (1..=frames).map(|f| page_ms(f, fps)).collect();
            prop_assert!(steps.iter().all(|s| (shortest..=longest).contains(s)));
            let total = u128::from(steps.iter().sum::<u64>());
            let exact = u128::from(frames) * 1000;
            let off = (total * u128::from(fps)).abs_diff(exact);
            prop_assert!(2 * off <= u128::from(fps), "{total} ms is not the nearest to {frames} frames");
        }
    }
}
