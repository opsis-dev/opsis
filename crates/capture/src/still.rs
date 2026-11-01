use std::path::Path;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_compile::nodes::capture::{Region, Still};
use opsis_types::actions::Bounds;
use opsis_types::events::LogLevel;
use opsis_types::values::BrowserSessionValue;
use serde::Deserialize;
use serde_json::json;

use crate::cdp::{CaptureError, Handle, Tab};
use crate::still_layout::{Area, Size, capture_canvas, capture_scale, padded_region, tallest_shot};
use crate::take::{Played, StepError, TakeEnv, TakeRun, failed, play_take};
use crate::trace::CallInfo;

const SIZE_TRIES: usize = 20;
const SIZE_POLL: Duration = Duration::from_millis(50);
const VIEW: &str = "({ x: window.scrollX, y: window.scrollY, width: window.innerWidth, height: window.innerHeight, pageWidth: document.documentElement.scrollWidth, pageHeight: document.documentElement.scrollHeight, dpr: window.devicePixelRatio })";
const SCROLL: &str = "({ x: window.scrollX, y: window.scrollY })";

#[derive(Debug, Clone, PartialEq)]
pub struct Shot {
    pub width: f64,
    pub height: f64,
    pub scale: f64,
    pub body: Option<Bounds>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct View {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    page_width: f64,
    page_height: f64,
    dpr: f64,
}

#[derive(Deserialize)]
struct Scroll {
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
struct Captured {
    data: String,
}

#[derive(Debug)]
pub struct StillRequest<'a> {
    pub params: &'a Still,
    pub project: Size,
    pub file: &'a Path,
}

async fn sized_box(
    tab: &Tab,
    handle: &Handle,
) -> Result<Option<opsis_types::actions::Rect>, CaptureError> {
    let mut found = tab.bounding_box(handle).await?;
    for _ in 0..SIZE_TRIES {
        match found {
            Some(rect) if rect.w < 1.0 || rect.h < 1.0 => {
                tab.clock().wait(SIZE_POLL).await;
                found = tab.bounding_box(handle).await?;
            }
            _ => break,
        }
    }
    Ok(found)
}

async fn element_region(take: &TakeRun<'_>, p: &Still) -> Result<(Area, Option<Area>), StepError> {
    let resolved = take.resolve(&p.target, None).await?;
    let tab = take.tab();
    let handle = take.element(&resolved.target, &p.target).await?;
    if let Err(error) = tab.scroll_into_view_if_needed(&handle).await {
        log::debug!("scrolling to the screenshot: {error}");
    }
    let measured = sized_box(tab, &handle).await;
    tab.release(&handle).await;
    let found = measured?.ok_or_else(|| {
        failed(format!(
            "Couldn't find “{}” on the page for the screenshot.",
            p.target
        ))
    })?;
    if found.w < 1.0 || found.h < 1.0 {
        return Err(failed(format!(
            "“{}” has no size on the page, so there's nothing to capture. Add space around it, or pick another element.",
            p.target
        )));
    }
    let scroll: Scroll = tab.evaluate_main(SCROLL).await?;
    let (region, body) = padded_region(
        Area {
            x: found.x,
            y: found.y,
            width: found.w,
            height: found.h,
        },
        scroll.x,
        scroll.y,
        f64::from(p.padding),
    );
    Ok((region, Some(body)))
}

async fn region_for(
    take: &TakeRun<'_>,
    p: &Still,
    view: &View,
    canvas: Size,
) -> Result<(Area, Option<Area>), StepError> {
    match p.region {
        Region::Element => element_region(take, p).await,
        Region::FullPage => {
            let tallest = tallest_shot(p.look, view.page_width, canvas);
            if view.page_height > tallest {
                take.log(LogLevel::Info, &format!("The page is {} px tall; keeping the top {tallest} px, as much as the video can show.", view.page_height));
            }
            Ok((
                Area {
                    x: 0.0,
                    y: 0.0,
                    width: view.page_width,
                    height: view.page_height.min(tallest),
                },
                None,
            ))
        }
        Region::Viewport => Ok((
            Area {
                x: view.x,
                y: view.y,
                width: view.width,
                height: view.height,
            },
            None,
        )),
    }
}

async fn capture_shot(take: &TakeRun<'_>, request: &StillRequest<'_>) -> Result<Shot, StepError> {
    let p = request.params;
    let tab = take.tab();
    let view: View = tab.evaluate_main(VIEW).await?;
    let canvas = capture_canvas(request.project);
    let (region, body) = region_for(take, p, &view, canvas).await?;
    let scale = capture_scale(
        p.look,
        Size {
            width: region.width,
            height: region.height,
        },
        canvas,
        body,
        p.fit(),
        view.dpr,
    );
    let beyond =
        p.region == Region::FullPage || region.width > view.width || region.height > view.height;
    let clip = json!({ "x": region.x, "y": region.y, "width": region.width, "height": region.height, "scale": scale / take.dpr() });
    let shot: Captured = tab
        .traced(
            || CallInfo::new("screenshot", "Screenshot"),
            tab.send_as(
                "Page.captureScreenshot",
                json!({ "format": "png", "clip": clip, "captureBeyondViewport": beyond }),
            ),
        )
        .await?;
    let png = STANDARD
        .decode(shot.data)
        .map_err(|e| failed(format!("The screenshot couldn't be read: {e}")))?;
    tokio::fs::write(request.file, png)
        .await
        .map_err(CaptureError::from)?;
    Ok(Shot {
        width: region.width,
        height: region.height,
        scale,
        body: body.map(|b| Bounds {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        }),
    })
}

pub async fn still_take(
    env: &TakeEnv,
    session: &BrowserSessionValue,
    request: &StillRequest<'_>,
) -> Played<Shot> {
    play_take(env, session, None, |take| {
        Box::pin(capture_shot(take, request))
    })
    .await
}
