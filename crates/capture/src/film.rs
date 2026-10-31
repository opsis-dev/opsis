use std::path::Path;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use opsis_compile::nodes::output::Gl;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::browser::{Launched, effective_switches, launch_with};
use crate::cdp::{CaptureError, Result, Tab};

const FILM_SWITCHES: [&str; 46] = [
    "--allow-pre-commit-input",
    "--disable-background-networking",
    "--enable-features=NetworkService,NetworkServiceInProcess,CanvasDrawElement",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-breakpad",
    "--disable-client-side-phishing-detection",
    "--disable-component-extensions-with-background-pages",
    "--disable-default-apps",
    "--disable-dev-shm-usage",
    "--no-proxy-server",
    "--force-gpu-mem-available-mb=4096",
    "--disable-hang-monitor",
    "--disable-extensions",
    "--disable-ipc-flooding-protection",
    "--disable-popup-blocking",
    "--disable-prompt-on-repost",
    "--disable-renderer-backgrounding",
    "--disable-sync",
    "--force-color-profile=srgb",
    "--metrics-recording-only",
    "--mute-audio",
    "--no-first-run",
    "--password-store=basic",
    "--use-mock-keychain",
    "--intensive-wake-up-throttling-policy=0",
    "--headless",
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-background-media-suspend",
    "--allow-running-insecure-content",
    "--disable-component-update",
    "--disable-domain-reliability",
    "--disable-features=AudioServiceOutOfProcess,IsolateOrigins,site-per-process,Translate,BackForwardCache,AvoidUnnecessaryBeforeUnloadCheckSync,IntensiveWakeUpThrottling,LocalNetworkAccessChecks,BlockInsecurePrivateNetworkRequests,PrivateNetworkAccessSendPreflights,PrivateNetworkAccessRespectPreflightResults",
    "--disable-print-preview",
    "--disable-site-isolation-trials",
    "--disk-cache-size=268435456",
    "--hide-scrollbars",
    "--no-default-browser-check",
    "--no-pings",
    "--font-render-hinting=none",
    "--no-zygote",
    "--ignore-gpu-blocklist",
    "--enable-unsafe-webgpu",
    "--enable-unsafe-swiftshader",
    "--no-startup-window",
];
const BEGIN_FRAME_SWITCHES: [&str; 9] = [
    "--deterministic-mode",
    "--enable-begin-frame-control",
    "--disable-new-content-rendering-timeout",
    "--run-all-compositor-stages-before-draw",
    "--disable-threaded-animation",
    "--disable-threaded-scrolling",
    "--disable-checker-imaging",
    "--disable-image-animation-resync",
    "--enable-surface-synchronization",
];
const PNG_SIGNATURE: [u8; 4] = [0x89, b'P', b'N', b'G'];
const PROBE_PAGE: &str =
    "data:text/html,<style>html,body{margin:0;background:%23173}</style><div>probe</div>";
const PROBE_ATTEMPTS: usize = 2;
const PROBE_TIMEOUT: Duration = Duration::from_secs(8);
const PROBE_RETRY: Duration = Duration::from_millis(10);
const WARM_UP_MS: f64 = 33.0;
const CAPTURE_HEADROOM: f64 = 10.0;
const PAGE_TIMEOUT: Duration = Duration::from_secs(60);
const PAGE_POLL: Duration = Duration::from_millis(50);
const FRAME_TIMEOUT: Duration = Duration::from_secs(30);

pub fn gl_switches(gl: Gl) -> &'static [&'static str] {
    match gl {
        Gl::Angle => &["--use-gl=angle"],
        Gl::Swangle => &["--use-gl=angle", "--use-angle=swiftshader"],
        Gl::Egl => &["--use-gl=egl"],
        Gl::Vulkan => &[
            "--use-angle=vulkan",
            "--use-vulkan=native",
            "--disable-vulkan-fallback-to-gl-for-testing",
            "--disable-vulkan-surface",
            "--enable-gpu",
            "--enable-features=Vulkan,UseSkiaRenderer",
        ],
        Gl::Swiftshader => &["--use-gl=swiftshader"],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    BeginFrame,
    Screenshot,
}

impl Capture {
    pub fn for_this_computer() -> Self {
        if cfg!(target_os = "linux") {
            Self::BeginFrame
        } else {
            Self::Screenshot
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BeginFrame => "one frame at a time",
            Self::Screenshot => "a screenshot per frame",
        }
    }
}

pub fn film_switches(gl: Gl, capture: Capture) -> Vec<String> {
    let begin: &[&str] = if capture == Capture::BeginFrame {
        &BEGIN_FRAME_SWITCHES
    } else {
        &[]
    };
    effective_switches(
        FILM_SWITCHES
            .iter()
            .chain(gl_switches(gl))
            .chain(begin)
            .copied(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    Jpeg(u8),
    Png,
}

impl FrameFormat {
    fn screenshot(self) -> Value {
        match self {
            Self::Jpeg(quality) => {
                json!({ "format": "jpeg", "quality": quality, "optimizeForSpeed": true })
            }
            Self::Png => json!({ "format": "png", "optimizeForSpeed": true }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilmShape {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_in_frames: u32,
}

pub struct FilmPage<'a> {
    pub url: &'a str,
    pub spec: &'a Value,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

pub struct DriverOptions<'a> {
    pub executable: &'a Path,
    pub temp_dir: &'a Path,
    pub gl: Gl,
    pub capture: Capture,
}

pub struct FrameDriver {
    launched: Launched,
    tab: Tab,
    capture: Capture,
    film: FilmShape,
    interval: f64,
    ticks: f64,
    last: Option<Vec<u8>>,
    fell_back: Option<String>,
}

fn target(url: &str, capture: Capture) -> Result<CreateTargetParams> {
    let builder = CreateTargetParams::builder().url(url);
    let builder = if capture == Capture::BeginFrame {
        builder.enable_begin_frame_control(true)
    } else {
        builder
    };
    builder.build().map_err(CaptureError::Failed)
}

fn decoded(data: Option<&str>) -> Result<Option<Vec<u8>>> {
    data.filter(|d| !d.is_empty())
        .map(|d| {
            STANDARD
                .decode(d)
                .map_err(|e| CaptureError::failed(format!("A frame came back unreadable: {e}")))
        })
        .transpose()
}

async fn begin_frame(tab: &Tab, params: Value) -> Result<Option<Vec<u8>>> {
    let done = tab.send("HeadlessExperimental.beginFrame", params).await?;
    decoded(done["screenshotData"].as_str())
}

async fn probe(launched: &Launched) -> Result<()> {
    let page = launched
        .browser
        .new_page(target(PROBE_PAGE, Capture::BeginFrame)?)
        .await?;
    let tab = Tab::new(page);
    let checked = tokio::time::timeout(PROBE_TIMEOUT, async {
        tab.send("HeadlessExperimental.enable", json!({})).await?;
        begin_frame(
            &tab,
            json!({ "frameTimeTicks": 0, "interval": WARM_UP_MS, "noDisplayUpdates": true }),
        )
        .await?;
        let mut ticks = 1000.0;
        loop {
            let shot = begin_frame(
                &tab,
                json!({ "frameTimeTicks": ticks, "interval": WARM_UP_MS, "screenshot": { "format": "png" } }),
            )
            .await?;
            if shot.is_some_and(|bytes| bytes.starts_with(&PNG_SIGNATURE)) {
                return Ok(());
            }
            ticks += WARM_UP_MS;
            tokio::time::sleep(PROBE_RETRY).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        Err(CaptureError::failed(
            "The browser didn't draw a test frame in time.",
        ))
    });
    if let Err(error) = tab.page.clone().close().await {
        log::debug!("closing the test page: {error}");
    }
    checked
}

async fn launch_film(options: &DriverOptions<'_>) -> Result<(Launched, Capture, Option<String>)> {
    let start = |capture| {
        launch_with(
            options.executable,
            options.temp_dir,
            film_switches(options.gl, capture),
        )
    };
    if options.capture == Capture::Screenshot {
        return Ok((start(Capture::Screenshot).await?, Capture::Screenshot, None));
    }
    let mut reason = String::new();
    for _ in 0..PROBE_ATTEMPTS {
        let launched = start(Capture::BeginFrame).await?;
        match probe(&launched).await {
            Ok(()) => return Ok((launched, Capture::BeginFrame, None)),
            Err(error) => {
                reason = error.to_string();
                launched.close().await;
            }
        }
    }
    Ok((
        start(Capture::Screenshot).await?,
        Capture::Screenshot,
        Some(reason),
    ))
}

struct WarmUp {
    stop: CancellationToken,
    ticker: JoinHandle<f64>,
}

impl WarmUp {
    fn start(tab: Tab) -> Self {
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        let ticker = tokio::spawn(async move {
            let mut ticks = 0.0;
            while !stopped.is_cancelled() {
                let params = json!({ "frameTimeTicks": ticks, "interval": WARM_UP_MS, "noDisplayUpdates": true });
                if let Err(error) = tab.send("HeadlessExperimental.beginFrame", params).await {
                    log::debug!("warming the film page up: {error}");
                }
                ticks += WARM_UP_MS;
                tokio::select! {
                    () = stopped.cancelled() => {}
                    () = tokio::time::sleep(Duration::from_secs_f64(WARM_UP_MS / 1000.0)) => {}
                }
            }
            ticks
        });
        Self { stop, ticker }
    }

    async fn finish(self) -> f64 {
        self.stop.cancel();
        self.ticker.await.unwrap_or_default()
    }
}

async fn wait_for_page(tab: &Tab) -> Result<()> {
    tokio::time::timeout(PAGE_TIMEOUT, async {
        loop {
            let ready: bool = tab
                .evaluate_main("typeof window.opsis?.load === 'function'")
                .await
                .unwrap_or(false);
            if ready {
                return;
            }
            tokio::time::sleep(PAGE_POLL).await;
        }
    })
    .await
    .map_err(|_| CaptureError::failed("The film page didn't start."))
}

async fn open_holds(tab: &Tab) -> String {
    tab.evaluate_main::<Vec<String>>("window.opsis.holds()")
        .await
        .map(|holds| holds.join(", "))
        .unwrap_or_default()
}

async fn ready_within<T>(tab: &Tab, what: &str, run: impl Future<Output = Result<T>>) -> Result<T> {
    match tokio::time::timeout(FRAME_TIMEOUT, run).await {
        Ok(done) => done,
        Err(_) => {
            let holds = open_holds(tab).await;
            let waiting = if holds.is_empty() {
                String::new()
            } else {
                format!(" Still waiting for: {holds}.")
            };
            Err(CaptureError::failed(format!(
                "{what} wasn't ready after {} s.{waiting}",
                FRAME_TIMEOUT.as_secs()
            )))
        }
    }
}

impl FrameDriver {
    pub async fn open(options: &DriverOptions<'_>, film: &FilmPage<'_>) -> Result<Self> {
        let (launched, capture, fell_back) = launch_film(options).await?;
        match Self::load(&launched, capture, film).await {
            Ok((tab, shape, ticks)) => Ok(Self {
                launched,
                tab,
                capture,
                film: shape,
                interval: 1000.0 / f64::from(shape.fps.max(1)),
                ticks,
                last: None,
                fell_back,
            }),
            Err(error) => {
                launched.close().await;
                Err(error)
            }
        }
    }

    async fn load(
        launched: &Launched,
        capture: Capture,
        film: &FilmPage<'_>,
    ) -> Result<(Tab, FilmShape, f64)> {
        let page = launched
            .browser
            .new_page(target("about:blank", capture)?)
            .await?;
        let tab = Tab::new(page);
        tab.send("Emulation.setDeviceMetricsOverride", json!({ "width": film.width, "height": film.height, "deviceScaleFactor": film.scale, "mobile": false }))
            .await?;
        let warm_up = if capture == Capture::BeginFrame {
            tab.send("HeadlessExperimental.enable", json!({})).await?;
            Some(WarmUp::start(Tab::new(tab.page.clone())))
        } else {
            None
        };
        let loaded = async {
            tab.send("Page.navigate", json!({ "url": film.url }))
                .await?;
            wait_for_page(&tab).await?;
            let spec = serde_json::to_string(film.spec)
                .map_err(|e| CaptureError::failed(e.to_string()))?;
            ready_within(
                &tab,
                "The film",
                tab.evaluate_main::<FilmShape>(&format!("window.opsis.load({spec})")),
            )
            .await
        }
        .await;
        let warmed = match warm_up {
            Some(warm_up) => warm_up.finish().await,
            None => 0.0,
        };
        let shape = loaded?;
        let interval = 1000.0 / f64::from(shape.fps.max(1));
        let ticks = warmed + CAPTURE_HEADROOM * interval;
        if capture == Capture::BeginFrame {
            begin_frame(
                &tab,
                json!({ "frameTimeTicks": ticks - interval, "interval": interval, "noDisplayUpdates": false }),
            )
            .await?;
        }
        Ok((tab, shape, ticks))
    }

    pub fn film(&self) -> FilmShape {
        self.film
    }

    pub fn capture(&self) -> Capture {
        self.capture
    }

    pub fn fell_back(&self) -> Option<&str> {
        self.fell_back.as_deref()
    }

    pub async fn frame(&mut self, frame: u32, format: FrameFormat) -> Result<Vec<u8>> {
        ready_within(
            &self.tab,
            &format!("Frame {frame}"),
            self.tab
                .evaluate_main::<Value>(&format!("window.opsis.seek({frame})")),
        )
        .await?;
        let picture = match self.capture {
            Capture::BeginFrame => {
                self.ticks += self.interval;
                begin_frame(
                    &self.tab,
                    json!({ "frameTimeTicks": self.ticks, "interval": self.interval, "screenshot": format.screenshot() }),
                )
                .await?
            }
            Capture::Screenshot => {
                let shot = self
                    .tab
                    .send("Page.captureScreenshot", format.screenshot())
                    .await?;
                decoded(shot["data"].as_str())?
            }
        };
        let picture = match (picture, self.last.take()) {
            (Some(picture), _) | (None, Some(picture)) => picture,
            (None, None) => {
                return Err(CaptureError::failed(format!(
                    "Frame {frame} came back empty."
                )));
            }
        };
        self.last = Some(picture.clone());
        Ok(picture)
    }

    pub async fn close(self) {
        if let Err(error) = self.tab.page.clone().close().await {
            log::debug!("closing the film page: {error}");
        }
        self.launched.close().await;
    }
}
