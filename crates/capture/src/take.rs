use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use opsis_compile::asset_ref::project_rel_path;
use opsis_compile::nodes::browser::{
    Assert, Chapter, Click, Drag, DragMethod as DragParam, DragMode, FreezeClock, HideElements,
    Hoist, Hover, MockRoute, Navigate, OPEN_BROWSER_TYPE, Open, OverlayHtml, PressKeys, Scroll,
    ScrollAlign, ScrollMode, Select, SetState, SignIn, Silence, Type, Wait, WaitMode, WaitUntil,
    read_path,
};
use opsis_compile::nodes::capture::{Cursor, Record, Timing};
use opsis_compile::registry::node_def;
use opsis_compile::text::{non_blank_lines, wall_time_in};
use opsis_compile::vars::PAUSE_MARK;
use opsis_project::paths::is_inside;
use opsis_traces::TraceTake;
use opsis_types::actions::{Action, ActionKind, ActionLog, CursorSample, Point};
use opsis_types::events::LogLevel;
use opsis_types::live::{LiveEvent, LiveStep, LiveStepStatus};
use opsis_types::secrets::{StorageState, secret_name};
use opsis_types::values::{BrowserAction, BrowserSessionValue, SessionStep};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

use crate::browser::Session;
use crate::cdp::{Arg, CaptureError, Handle};
use crate::clock::{Clock, as_ms, wall_ms, wall_time};
use crate::cursor::{
    ActionOpts, CursorDriver, DragMethod, DragOpts, DragTo, KeyEvent, Press, TakeClick, TakeSink,
};
use crate::ease::camera_ease;
use crate::encoder::{EncodeError, Encoded, FrameEncoder};
use crate::input::Input;
use crate::keyboard::pressed_chord;
use crate::live::{FrameSink, LiveSink};
use crate::locators::{first, text_string_selector};
use crate::overlays::Overlays;
use crate::page::Lifecycle;
use crate::page_scripts::{
    ADD_STYLES, CHAT_WIDGET_SELECTORS, COOKIE_BANNER_SELECTORS, INSTALL_CURSOR, INVALID_SELECTORS,
    OPTION_VALUE, PLAN_ELEMENT_SCROLL, RESTORE_LOCAL_STORAGE, STEP_ELEMENT_SCROLL, SavedStorage,
    before_first_load, call, cursor_look, hiding_css, run_user_script,
};
use crate::route::{MockResponse, Routes};
use crate::screencast::{DEFAULT_FRAME_QUALITY, FrameSinks, Screencast};
use crate::selectors::{
    CSS_PREFIX, ResolveError, ResolveOptions, Resolved, SelectorError, Target, dump_visible,
    resolve_target,
};
use crate::smooth::{FramePictures, Smooth};
use crate::take_clock::TakeClock;
use crate::trace::{CallInfo, Trace};

const LIVE_CURSOR_GAP_MS: f64 = 30.0;
const FIRST_PAGE_MIN_WAIT_MS: f64 = 500.0;
const SCROLL_TICK_MS: u32 = 16;
const SCROLL_SMOOTHLY: &str =
    "function(block) { this.scrollIntoView({ behavior: 'smooth', block }); }";
const SCROLL_INSTANTLY: &str =
    "function(block) { this.scrollIntoView({ behavior: 'instant', block }); }";
const ELEMENT_PLACE: &str =
    "function() { const r = this.getBoundingClientRect(); return [r.left, r.top]; }";
const SCROLL_POLL: Duration = Duration::from_millis(50);
const SCROLL_SETTLE_LIMIT: Duration = Duration::from_secs(3);

async fn scrolling_stopped(tab: &crate::cdp::Tab, handle: &Handle) -> Result<(), CaptureError> {
    let deadline = tab.clock().now() + SCROLL_SETTLE_LIMIT;
    let mut last: Option<(f64, f64)> = None;
    while tab.clock().now() < deadline {
        let place: (f64, f64) = tab.call_on(handle, ELEMENT_PLACE, &[]).await?;
        if last.is_some_and(|(x, y)| (x - place.0).abs() < 0.5 && (y - place.1).abs() < 0.5) {
            break;
        }
        last = Some(place);
        tab.clock().wait(SCROLL_POLL).await;
    }
    Ok(())
}

pub(crate) async fn smooth_scroll_into_view(
    tab: &crate::cdp::Tab,
    handle: &Handle,
    block: &str,
    duration_ms: f64,
) -> Result<(), CaptureError> {
    if !tab.clock().is_stepped() {
        tab.call_on::<Value>(handle, SCROLL_SMOOTHLY, &[Arg::Value(json!(block))])
            .await?;
        tab.clock()
            .wait(Duration::from_secs_f64(duration_ms.max(0.0) / 1000.0))
            .await;
        return scrolling_stopped(tab, handle).await;
    }
    tab.call_on::<Value>(handle, PLAN_ELEMENT_SCROLL, &[Arg::Value(json!(block))])
        .await?;
    let start = tab.clock().now();
    let span = duration_ms.max(1.0);
    loop {
        let p01 = (as_ms(tab.clock().now().saturating_sub(start)) / span).clamp(0.0, 1.0);
        tab.call_on::<Value>(
            handle,
            STEP_ELEMENT_SCROLL,
            &[Arg::Value(json!(camera_ease(p01)))],
        )
        .await?;
        if p01 >= 1.0 {
            return Ok(());
        }
        tab.clock().wait(Duration::from_millis(1)).await;
    }
}
const SELECT_OPTION: &str =
    "(injected, node, value) => injected.selectOptions(node, [{ valueOrLabel: value }])";
const PAGE_TIME_ZONE: &str = "Intl.DateTimeFormat().resolvedOptions().timeZone";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Start,
    Done,
    Error,
}

pub type SignInError = Box<dyn std::error::Error + Send + Sync>;

pub trait SignIns: Send + Sync {
    fn read(&self, name: &str) -> Result<Option<String>, SignInError>;
}

const SIGN_INS_UNREACHABLE: &str = "The saved sign-ins couldn't be reached.";

pub async fn off_runtime<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, SignInError> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| SIGN_INS_UNREACHABLE.to_owned())?
        .map_err(|e| e.to_string())
}

pub type LogFn = Arc<dyn Fn(LogLevel, &str) + Send + Sync>;
pub type StepFn = Arc<dyn Fn(&str, StepStatus, Option<&str>) + Send + Sync>;

pub struct TakeEnv {
    pub project_dir: PathBuf,
    pub executable: PathBuf,
    pub temp_dir: PathBuf,
    pub live: Option<Arc<dyn LiveSink>>,
    pub trace: bool,
    pub sign_ins: Arc<dyn SignIns>,
    pub cancel: CancellationToken,
    pub log: LogFn,
    pub step: StepFn,
}

impl fmt::Debug for TakeEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeEnv")
            .field("project_dir", &self.project_dir)
            .field("executable", &self.executable)
            .field("temp_dir", &self.temp_dir)
            .field("trace", &self.trace)
            .field("cancel", &self.cancel)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Recording {
    pub encoder: FrameEncoder,
    pub record: Record,
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error(transparent)]
    NotFound(#[from] SelectorError),
    #[error(transparent)]
    Capture(#[from] CaptureError),
}

impl From<ResolveError> for StepError {
    fn from(error: ResolveError) -> Self {
        match error {
            ResolveError::NotFound(error) => Self::NotFound(error),
            ResolveError::Other(error) => Self::Capture(error),
        }
    }
}

pub(crate) fn failed(message: impl Into<String>) -> StepError {
    StepError::Capture(CaptureError::failed(message))
}

type StepResult<T = ()> = Result<T, StepError>;

pub fn step_name(step: &SessionStep, fallback: Option<&str>) -> String {
    if let Some(label) = step
        .label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        return label.to_owned();
    }
    if let Some(fallback) = fallback {
        return fallback.to_owned();
    }
    node_def(step.kind.kind())
        .map(|d| d.title.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| step.kind.kind().to_owned())
}

fn custom_label(step: &SessionStep) -> Option<String> {
    step.label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
}

fn param_text<'a>(step: &'a SessionStep, key: &str) -> Option<&'a str> {
    step.params
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

pub fn trace_group_title(label: &str, step: &SessionStep) -> String {
    match ["target", "url", "text", "keys"]
        .iter()
        .find_map(|key| param_text(step, key))
    {
        Some(subject) => format!("{label} · {subject}"),
        None => label.to_owned(),
    }
}

fn plain_message(error: &StepError, step: &SessionStep) -> String {
    match error {
        StepError::Capture(CaptureError::NotFoundWithin(after)) => {
            let what = param_text(step, "target").map_or_else(
                || "the element".to_owned(),
                |t| format!("“{}”", t.strip_prefix(CSS_PREFIX).unwrap_or(t)),
            );
            format!(
                "Couldn't find {what} on the page within {}.",
                opsis_compile::text::format_duration(opsis_types::numbers::millis(*after))
            )
        }
        other => other.to_string(),
    }
}

fn same_page(a: &str, b: &str) -> bool {
    let norm = |raw: &str| {
        let url = url::Url::parse(raw).ok()?;
        let path = url.path();
        let path = path
            .strip_suffix("/index.html")
            .or_else(|| path.strip_suffix("/index.htm"))
            .or_else(|| path.strip_suffix('/'))
            .unwrap_or(path);
        Some(format!("{}{path}", url.origin().ascii_serialization()))
    };
    matches!((norm(a), norm(b)), (Some(a), Some(b)) if a == b)
}

pub fn landed_on_sign_in(requested: &str, current: &str, sign_in_url: &str) -> bool {
    !same_page(requested, sign_in_url) && same_page(current, sign_in_url)
}

pub fn at_take_start(mut action: Action) -> Action {
    action.t = 0.0;
    for time in [&mut action.t_end, &mut action.t_depart, &mut action.t_down] {
        if time.is_some() {
            *time = Some(0.0);
        }
    }
    action
}

pub fn video_start(steps: &[SessionStep]) -> usize {
    steps
        .iter()
        .position(|s| s.kind == BrowserAction::StartRecording)
        .unwrap_or(0)
}

fn silenced(step: &SessionStep) -> bool {
    serde_json::from_value::<Silence>(Value::Object(step.params.clone())).is_ok_and(|s| s.silent)
}

pub const fn click_kind_of(action: BrowserAction) -> Option<ActionKind> {
    match action {
        BrowserAction::Click => Some(ActionKind::Click),
        BrowserAction::Type => Some(ActionKind::Type),
        BrowserAction::Hover => Some(ActionKind::Hover),
        BrowserAction::Drag => Some(ActionKind::Drag),
        BrowserAction::Select => Some(ActionKind::Focus),
        _ => None,
    }
}

fn params<P: DeserializeOwned>(step: &SessionStep) -> StepResult<P> {
    serde_json::from_value(Value::Object(step.params.clone())).map_err(|e| {
        failed(format!(
            "“{}” has settings that can't be read: {e}",
            step_name(step, None)
        ))
    })
}

fn ms(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

#[derive(Debug, Default)]
struct Recorded {
    clicks: Vec<TakeClick>,
    keys: Vec<KeyEvent>,
    cursor: Vec<CursorSample>,
    last_live_cursor: f64,
}

struct Recorder {
    clock: TakeClock,
    t0: Mutex<Duration>,
    recorded: Mutex<Recorded>,
    live: Option<Arc<dyn LiveSink>>,
    overlays: Option<Arc<Overlays>>,
}

impl fmt::Debug for Recorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Recorder")
            .field("clock", &self.clock)
            .field("t0", &self.t0)
            .field("recorded", &self.recorded)
            .field("overlays", &self.overlays)
            .finish_non_exhaustive()
    }
}

impl Recorder {
    fn elapsed(&self) -> f64 {
        let t0 = *self.t0.lock().expect("take start");
        as_ms(self.clock.now().saturating_sub(t0))
    }

    fn recorded(&self) -> std::sync::MutexGuard<'_, Recorded> {
        self.recorded.lock().expect("take log")
    }
}

impl TakeSink for Recorder {
    fn aim(&self, label: &str, at: Point) {
        if let Some(overlays) = &self.overlays {
            overlays.annotate(label, Some(at));
        }
    }

    fn click(&self, event: TakeClick) -> usize {
        if let Some(live) = &self.live {
            live.event(LiveEvent::Click {
                x: event.x,
                y: event.y,
            });
            if let Some(rect) = event.rect {
                live.event(LiveEvent::Target {
                    rect,
                    label: event.label.clone().unwrap_or_default(),
                    selector: event.selector.clone(),
                });
            }
        }
        let mut recorded = self.recorded();
        recorded.clicks.push(event);
        recorded.clicks.len() - 1
    }

    fn type_end(&self, click: usize, t_ms: f64, key_times: Vec<f64>) {
        if let Some(event) = self.recorded().clicks.get_mut(click) {
            event.type_end_ms = Some(t_ms);
            event.key_times = Some(key_times);
        }
    }

    fn cursor(&self, sample: CursorSample) {
        let now = wall_ms();
        let (x, y) = (sample.x, sample.y);
        let send = {
            let mut recorded = self.recorded();
            recorded.cursor.push(sample);
            let send = now - recorded.last_live_cursor >= LIVE_CURSOR_GAP_MS;
            if send {
                recorded.last_live_cursor = now;
            }
            send
        };
        if send && let Some(live) = &self.live {
            live.event(LiveEvent::Cursor { x, y });
        }
    }

    fn key(&self, event: KeyEvent) {
        if let Some(overlays) = &self.overlays {
            overlays.annotate(event.label.as_deref().unwrap_or(&event.chord), None);
        }
        self.recorded().keys.push(event);
    }
}

#[derive(Debug)]
struct SignedIn {
    url: String,
    name: String,
}

#[derive(Debug)]
pub struct TakeRun<'e> {
    env: &'e TakeEnv,
    session: Session,
    lifecycle: Lifecycle,
    input: Arc<Input>,
    recorder: Arc<Recorder>,
    actions: Vec<Action>,
    hoisted: HashMap<String, String>,
    driver: Option<CursorDriver>,
    signed_in: Option<SignedIn>,
    recording: Option<Recording>,
    overlays: Option<Arc<Overlays>>,
    clock: Clock,
    routes: Routes,
    trace: Option<Arc<Trace>>,
    screencast: Option<Screencast>,
}

#[derive(Debug)]
pub struct Took<T> {
    pub log: ActionLog,
    pub from_page: T,
}

#[derive(Debug)]
pub struct Played<T> {
    pub outcome: Result<Took<T>, StepError>,
    pub encoded: Option<Encoded>,
    pub trace: Option<TraceTake>,
}

impl<'e> TakeRun<'e> {
    async fn open(
        env: &'e TakeEnv,
        session_value: &BrowserSessionValue,
        recording: Option<Recording>,
    ) -> Result<Self, CaptureError> {
        let browser_id = &session_value.browser.node_id;
        let params: Open =
            serde_json::from_value(Value::Object(session_value.browser.params.clone()))
                .map_err(|e| CaptureError::Failed(e.to_string()))?;
        (env.step)(browser_id, StepStatus::Start, None);
        let session = Session::open(&params, &env.executable, &env.temp_dir).await?;
        let (trace, overlays, lifecycle) =
            match Self::prepare(env, &session, recording.as_ref()).await {
                Ok(ready) => ready,
                Err(error) => {
                    session.close().await;
                    return Err(error);
                }
            };
        (env.step)(browser_id, StepStatus::Done, None);
        let recorder = Arc::new(Recorder {
            clock: session.tab.clock().clone(),
            t0: Mutex::new(wall_time()),
            recorded: Mutex::default(),
            live: env.live.clone(),
            overlays: overlays.clone(),
        });
        Ok(Self {
            env,
            session,
            lifecycle,
            input: Arc::default(),
            recorder,
            actions: Vec::new(),
            hoisted: HashMap::new(),
            driver: None,
            signed_in: None,
            recording,
            overlays,
            clock: Clock::default(),
            routes: Routes::default(),
            trace,
            screencast: None,
        })
    }

    async fn prepare(
        env: &TakeEnv,
        session: &Session,
        recording: Option<&Recording>,
    ) -> Result<(Option<Arc<Trace>>, Option<Arc<Overlays>>, Lifecycle), CaptureError> {
        let tab = &session.tab;
        let trace = if env.trace {
            Some(Arc::new(Trace::start(tab).await?))
        } else {
            None
        };
        if let Some(trace) = &trace {
            tab.set_trace(Arc::clone(trace));
        }
        if recording.is_some_and(|r| r.record.cursor == Cursor::Baked) {
            session
                .add_init_script(&call(
                    INSTALL_CURSOR,
                    &cursor_look(&recording.expect("a baked cursor records").record),
                ))
                .await?;
        }
        let overlays = match recording {
            Some(r) => Some(Arc::new(
                Overlays::start(tab, r.record.show_actions, r.record.action_label_ms).await?,
            )),
            None => None,
        };
        Ok((trace, overlays, Lifecycle::start(tab).await?))
    }

    pub(crate) const fn dpr(&self) -> f64 {
        self.session.dpr
    }

    pub(crate) const fn tab(&self) -> &Arc<crate::cdp::Tab> {
        &self.session.tab
    }

    pub(crate) fn log(&self, level: LogLevel, message: &str) {
        (self.env.log)(level, message);
    }

    async fn start_screencast(&mut self) -> Result<(), CaptureError> {
        let tracing_frames = self.recording.is_none() && self.trace.is_some();
        if self.recording.is_none() && self.env.live.is_none() && !tracing_frames {
            return Ok(());
        }
        let sinks = FrameSinks {
            encoder: self.recording.as_ref().map(|r| r.encoder.feed()),
            view: self.env.live.clone().map(|live| live as Arc<dyn FrameSink>),
            trace: if tracing_frames {
                self.trace.clone()
            } else {
                None
            },
        };
        let quality = self
            .recording
            .as_ref()
            .map_or(DEFAULT_FRAME_QUALITY, |r| r.record.quality);
        self.screencast =
            Some(Screencast::start(self.tab(), self.session.viewport, quality, sinks).await?);
        Ok(())
    }

    async fn sleep(&self, duration: Duration) {
        let wait = async {
            self.tab().clock().wait(duration).await;
            Ok(())
        };
        if let Err(error) = self
            .tab()
            .traced(
                || {
                    CallInfo::new(
                        "waitForTimeout",
                        format!(
                            "Wait {}",
                            opsis_compile::text::format_precise_duration(
                                opsis_types::numbers::millis(duration)
                            )
                        ),
                    )
                },
                wait,
            )
            .await
        {
            log::debug!("waiting: {error}");
        }
    }

    fn rolling(&self) -> StepResult<&CursorDriver> {
        if self.lifecycle.url() == "about:blank" {
            return Err(failed(
                "Mouse actions must come after the first Go to page action.",
            ));
        }
        self.driver
            .as_ref()
            .ok_or_else(|| failed("Mouse actions must come after Start recording."))
    }

    async fn pause_if_rolling(&self, after: u32) {
        if let Some(driver) = &self.driver {
            driver.pause(f64::from(after)).await;
        }
    }

    async fn settle_after_page(&self, after: u32) {
        match &self.driver {
            Some(driver) => driver.pause(f64::from(after)).await,
            None => {
                self.sleep(Duration::from_secs_f64(
                    FIRST_PAGE_MIN_WAIT_MS.max(f64::from(after)) / 1000.0,
                ))
                .await;
            }
        }
    }

    async fn press_key(&self, chord: &str, label: &str) -> StepResult {
        match &self.driver {
            Some(driver) => driver.press_key(chord, Some(label)).await?,
            None => self.input.press(self.tab(), chord, None).await?,
        }
        Ok(())
    }

    pub(crate) async fn resolve(
        &self,
        raw: &str,
        timeout: Option<Duration>,
    ) -> StepResult<Resolved> {
        let log = |message: &str| {
            self.log(LogLevel::Info, message);
            if let Some(trace) = &self.trace {
                trace.log(message);
            }
        };
        let options = ResolveOptions {
            log: &log,
            cancel: &self.env.cancel,
            hoisted: &self.hoisted,
            timeout,
        };
        Ok(resolve_target(self.tab(), raw, &options).await?)
    }

    pub(crate) async fn element(&self, target: &Target, raw: &str) -> StepResult<Handle> {
        let selector = target.element()?;
        self.tab()
            .query(selector)
            .await?
            .ok_or_else(|| failed(format!("Couldn't find “{raw}” on the page.")))
    }

    async fn motion(
        &self,
        selector: &str,
        beat: &opsis_compile::nodes::browser::Beat,
        frame: &str,
        pointer: Option<(f64, u32, u32)>,
    ) -> StepResult<ActionOpts> {
        let frame = if frame.trim().is_empty() {
            None
        } else {
            Some(self.resolve(frame, None).await?.target)
        };
        let (zoom_scale, travel_ms, hover_ms) =
            pointer.map_or((None, None, None), |(scale, travel, hover)| {
                (
                    (scale != 0.0).then_some(scale),
                    (travel != 0).then_some(f64::from(travel)),
                    Some(f64::from(hover)),
                )
            });
        Ok(ActionOpts {
            selector: Some(selector.to_owned()),
            cluster: Some(beat.cluster.clone()).filter(|c| !c.is_empty()),
            zoom: beat.zoom,
            frame,
            chunk_pause_ms: 0.0,
            zoom_scale,
            travel_ms,
            hover_ms,
            key_ms: 0.0,
            clear_first: true,
            smooth_scroll: false,
        })
    }

    fn push_action(&mut self, mut action: Action, step: &SessionStep) {
        action.node_id = Some(step.id.clone());
        self.actions.push(action);
    }

    async fn step_frames(&mut self, t0: Duration) -> StepResult {
        let Some(recording) = &self.recording else {
            return Ok(());
        };
        if recording.record.timing != Timing::Smooth {
            return Ok(());
        }
        let pictures = FramePictures {
            encoder: Some(recording.encoder.feed()),
            view: self.env.live.clone().map(|live| live as Arc<dyn FrameSink>),
            picture: recording.record.pictures,
            quality: recording.record.quality,
            viewport: self.session.viewport,
            dpr: self.session.dpr,
        };
        let fps = recording.record.fps;
        if let Some(screencast) = self.screencast.take() {
            screencast.stop(self.tab()).await?;
        }
        self.clock.hold_for_frames(&self.session).await?;
        let smooth = Smooth::start(self.tab(), fps, pictures).await?;
        smooth.picture(t0).await;
        self.tab().clock().step_from(t0, smooth);
        Ok(())
    }

    async fn begin_video(&mut self) -> StepResult {
        self.actions = std::mem::take(&mut self.actions)
            .into_iter()
            .map(at_take_start)
            .collect();
        let t0 = wall_time();
        *self.recorder.t0.lock().expect("take start") = t0;
        if let Some(recording) = &self.recording {
            recording.encoder.set_origin(t0);
        }
        self.step_frames(t0).await?;
        let recorder = Arc::clone(&self.recorder);
        let elapsed: Arc<dyn Fn() -> f64 + Send + Sync> = Arc::new(move || recorder.elapsed());
        let reads_shapes = self
            .recording
            .as_ref()
            .is_some_and(|r| r.record.cursor == Cursor::Synthetic);
        let sink: Arc<dyn TakeSink> = self.recorder.clone();
        self.driver = Some(
            CursorDriver::new(
                Arc::clone(self.tab()),
                Arc::clone(&self.input),
                elapsed,
                sink,
                reads_shapes,
                self.session.viewport,
            )
            .await,
        );
        Ok(())
    }

    fn live_step(&self, step: &SessionStep, status: LiveStepStatus, message: Option<&str>) {
        if let Some(live) = &self.env.live {
            live.event(LiveEvent::Step {
                node_id: step.id.clone(),
                status,
                message: message.map(str::to_owned),
            });
        }
    }

    async fn run(&mut self, step: &SessionStep) -> StepResult {
        if self.env.cancel.is_cancelled() {
            return Err(CaptureError::Cancelled.into());
        }
        (self.env.step)(&step.id, StepStatus::Start, None);
        self.live_step(step, LiveStepStatus::Running, None);
        let (clicks_before, keys_before) = {
            let recorded = self.recorder.recorded();
            (recorded.clicks.len(), recorded.keys.len())
        };
        if let Some(trace) = &self.trace {
            trace.group(&step.id, &trace_group_title(&step_name(step, None), step));
        }
        let played = self.play(step).await;
        let message = played.as_ref().err().map(|e| plain_message(e, step));
        if let Some(trace) = &self.trace {
            trace.end_group(message.as_deref());
        }
        if let Err(error) = played {
            let message = message.unwrap_or_default();
            (self.env.step)(&step.id, StepStatus::Error, Some(&message));
            self.live_step(step, LiveStepStatus::Error, Some(&message));
            return Err(match error {
                StepError::NotFound(error) => StepError::NotFound(error),
                StepError::Capture(CaptureError::Cancelled) => CaptureError::Cancelled.into(),
                StepError::Capture(_) => failed(message),
            });
        }
        self.collect_actions(step, clicks_before, keys_before);
        (self.env.step)(&step.id, StepStatus::Done, None);
        self.live_step(step, LiveStepStatus::Done, None);
        Ok(())
    }

    fn collect_actions(&mut self, step: &SessionStep, clicks_before: usize, keys_before: usize) {
        let (clicks, keys) = {
            let recorded = self.recorder.recorded();
            (
                recorded.clicks[clicks_before..].to_vec(),
                recorded.keys[keys_before..].to_vec(),
            )
        };
        let kind = click_kind_of(step.kind);
        let silent = silenced(step).then_some(true);
        for c in clicks {
            let Some(kind) = kind else {
                continue;
            };
            let mut action =
                Action::new(format!("{}:{}", step.id, self.actions.len()), kind, c.t_ms);
            action.t_end = c.type_end_ms;
            action.t_depart = c.t_depart_ms;
            action.t_down = c.t_down_ms;
            action.t_second_down = c.t_second_down_ms;
            action.button = c.button;
            action.point = Some(Point { x: c.x, y: c.y });
            action.rect = c.rect;
            action.cluster = c.cluster;
            action.label = c.label;
            action.selector_used = c.selector;
            action.zoom = c.zoom;
            action.zoom_scale = c.zoom_scale;
            action.text = if kind == ActionKind::Type {
                c.text
            } else {
                None
            };
            action.path = c.path;
            action.drop = c.drop;
            action.key_times = c.key_times.filter(|_| kind == ActionKind::Type);
            action.silent = silent;
            self.push_action(action, step);
        }
        for k in keys {
            let mut action = Action::new(
                format!("{}:k{}", step.id, self.actions.len()),
                ActionKind::Key,
                k.t_ms,
            );
            action.text = Some(k.chord);
            action.label = k.label;
            action.silent = silent;
            self.push_action(action, step);
        }
    }

    async fn play(&mut self, step: &SessionStep) -> StepResult {
        match step.kind {
            BrowserAction::Navigate => self.navigate(step, params(step)?).await,
            BrowserAction::Click => self.click(step, params(step)?).await,
            BrowserAction::Type => self.type_text(step, params(step)?).await,
            BrowserAction::Hover => self.hover(step, params(step)?).await,
            BrowserAction::Drag => self.drag(step, params(step)?).await,
            BrowserAction::Scroll => self.scroll(step, params(step)?).await,
            BrowserAction::Select => self.select(step, params(step)?).await,
            BrowserAction::Upload => self.upload(params(step)?).await,
            BrowserAction::Press => self.press(step, params(step)?).await,
            BrowserAction::Wait => self.wait(step, params(step)?).await,
            BrowserAction::Assert => self.assert(params(step)?).await,
            BrowserAction::Hoist => self.hoist(params(step)?).await,
            BrowserAction::SetState => self.set_state(step, params(step)?).await,
            BrowserAction::MockRoute => self.mock_route(params(step)?).await,
            BrowserAction::FreezeClock => self.freeze_clock(params(step)?).await,
            BrowserAction::HideElements => self.hide_elements(params(step)?).await,
            BrowserAction::Chapter => self.chapter(step, params(step)?).await,
            BrowserAction::OverlayHtml => self.overlay_html(params(step)?).await,
            BrowserAction::SignIn => self.sign_in(step, params(step)?).await,
            BrowserAction::StartRecording => Ok(()),
        }
    }

    async fn navigate(&mut self, step: &SessionStep, p: Navigate) -> StepResult {
        let t = self.recorder.elapsed();
        self.lifecycle
            .navigate(
                self.tab(),
                &p.url,
                p.wait_until,
                Duration::from_millis(u64::from(p.timeout_ms)),
            )
            .await?;
        let mut action = Action::new(format!("{}:nav", step.id), ActionKind::Navigate, t);
        action.t_end = Some(self.recorder.elapsed());
        action.label = Some(p.url.clone());
        self.push_action(action, step);
        self.settle_after_page(p.after).await;
        if let Some(signed) = &self.signed_in
            && landed_on_sign_in(&p.url, &self.lifecycle.url(), &signed.url)
        {
            return Err(failed(format!(
                "Signed out during the take. Run “{}” again.",
                signed.name
            )));
        }
        Ok(())
    }

    async fn click(&self, step: &SessionStep, p: Click) -> StepResult {
        let resolved = self
            .resolve(
                &p.target.target,
                Some(Duration::from_millis(u64::from(p.find_within_ms))),
            )
            .await?;
        let label = step_name(step, Some(&p.target.target));
        let mut opts = self
            .motion(
                &resolved.by,
                &p.beat,
                &p.pointer.frame.frame,
                Some((p.pointer.zoom_scale, p.pointer.travel_ms, p.hover_ms)),
            )
            .await?;
        opts.smooth_scroll = p.scroll_smoothly;
        let press = Press {
            button: p.button,
            double: p.double_clicks(),
        };
        let driver = self.rolling()?;
        driver
            .move_and_click(&resolved.target, Some(&label), &opts, press)
            .await?;
        driver.pause(f64::from(p.beat.pause.after)).await;
        Ok(())
    }

    async fn type_text(&self, step: &SessionStep, p: Type) -> StepResult {
        let resolved = self.resolve(&p.target.target, None).await?;
        let label = step_name(step, Some(&p.target.target));
        let mut opts = self
            .motion(
                &resolved.by,
                &p.beat,
                &p.pointer.frame.frame,
                Some((p.pointer.zoom_scale, p.pointer.travel_ms, p.hover_ms)),
            )
            .await?;
        opts.chunk_pause_ms = f64::from(p.chunk_pause_ms);
        opts.key_ms = f64::from(p.key_ms);
        opts.clear_first = p.clear_first;
        let chunks: Vec<String> = p.text.split(PAUSE_MARK).map(str::to_owned).collect();
        let driver = self.rolling()?;
        driver
            .type_into(&resolved.target, &chunks, Some(&label), &opts)
            .await?;
        if p.press_enter {
            driver.press_key("Enter", Some("Enter")).await?;
        }
        driver.pause(f64::from(p.beat.pause.after)).await;
        Ok(())
    }

    async fn hover(&self, step: &SessionStep, p: Hover) -> StepResult {
        let resolved = self.resolve(&p.target.target, None).await?;
        let label = step_name(step, Some(&p.target.target));
        let opts = self
            .motion(
                &resolved.by,
                &p.beat,
                &p.pointer.frame.frame,
                Some((p.pointer.zoom_scale, p.pointer.travel_ms, p.hover_ms)),
            )
            .await?;
        let driver = self.rolling()?;
        driver.focus(&resolved.target, Some(&label), &opts).await?;
        driver.pause(f64::from(p.beat.pause.after)).await;
        Ok(())
    }

    async fn drag(&self, step: &SessionStep, p: Drag) -> StepResult {
        let from = self.resolve(&p.target, None).await?;
        let label = step_name(step, Some(&p.target));
        self.rolling()?;
        let to = match p.mode {
            DragMode::By => DragTo::By(Point {
                x: f64::from(p.dx),
                y: f64::from(p.dy),
            }),
            DragMode::Path => {
                DragTo::Path(read_path(&p.path).map_err(|problem| failed(format!("{problem}.")))?)
            }
            DragMode::Element if p.to.trim().is_empty() => {
                return Err(failed("Pick where to drop it."));
            }
            DragMode::Element => DragTo::Target(self.resolve(&p.to, None).await?.target),
        };
        let action = self
            .motion(
                &from.by,
                &p.beat,
                &p.pointer.frame.frame,
                Some((p.pointer.zoom_scale, p.pointer.travel_ms, p.hover_ms)),
            )
            .await?;
        let opts = DragOpts {
            action,
            hold_ms: f64::from(p.hold_ms),
            drag_ms: f64::from(p.drag_ms),
            drop_hold_ms: f64::from(p.drop_hold_ms),
            method: if p.method == DragParam::Quick {
                DragMethod::Quick
            } else {
                DragMethod::Smooth
            },
        };
        let driver = self.rolling()?;
        driver.drag(&from.target, &to, Some(&label), &opts).await?;
        driver.pause(f64::from(p.beat.pause.after)).await;
        Ok(())
    }

    async fn scroll(&mut self, step: &SessionStep, p: Scroll) -> StepResult {
        let t = self.recorder.elapsed();
        match p.mode {
            ScrollMode::Element => {
                let resolved = self.resolve(&p.target, None).await?;
                let handle = self.element(&resolved.target, &p.target).await?;
                let block = p.align.block();
                let scrolled = if p.smooth {
                    smooth_scroll_into_view(self.tab(), &handle, block, f64::from(p.duration_ms))
                        .await
                } else if p.align == ScrollAlign::Center {
                    self.tab().scroll_into_view_if_needed(&handle).await
                } else {
                    self.tab()
                        .call_on::<Value>(&handle, SCROLL_INSTANTLY, &[Arg::Value(json!(block))])
                        .await
                        .map(drop)
                };
                self.tab().release(&handle).await;
                scrolled?;
            }
            ScrollMode::By if !p.smooth => {
                self.tab()
                    .evaluate_main::<Value>(&format!(
                        "window.scrollBy({}, {})",
                        p.delta_x, p.delta_y
                    ))
                    .await?;
            }
            ScrollMode::By => {
                let ticks =
                    (p.duration_ms.saturating_add(SCROLL_TICK_MS / 2) / SCROLL_TICK_MS).max(1);
                let start = self.tab().clock().now();
                for tick in 1..=ticks {
                    self.input
                        .wheel(
                            self.tab(),
                            f64::from(p.delta_x) / f64::from(ticks),
                            f64::from(p.delta_y) / f64::from(ticks),
                        )
                        .await?;
                    self.tab()
                        .clock()
                        .sleep_until(
                            start + Duration::from_millis(u64::from(SCROLL_TICK_MS)) * tick,
                        )
                        .await;
                }
            }
        }
        let mut action = Action::new(format!("{}:scroll", step.id), ActionKind::Scroll, t);
        action.t_end = Some(self.recorder.elapsed());
        action.label = custom_label(step);
        self.push_action(action, step);
        self.pause_if_rolling(p.pause.after).await;
        Ok(())
    }

    async fn select(&self, step: &SessionStep, p: Select) -> StepResult {
        let resolved = self.resolve(&p.target.target, None).await?;
        let label = step_name(step, Some(&p.target.target));
        let opts = self
            .motion(
                &resolved.by,
                &p.beat,
                &p.pointer.frame.frame,
                Some((p.pointer.zoom_scale, p.pointer.travel_ms, p.hover_ms)),
            )
            .await?;
        let driver = self.rolling()?;
        driver.focus(&resolved.target, Some(&label), &opts).await?;
        let handle = self.element(&resolved.target, &p.target.target).await?;
        let picked = async {
            let option: Option<String> = self
                .tab()
                .call_on(&handle, OPTION_VALUE, &[Arg::Value(json!(p.value))])
                .await?;
            let Some(option) = option else {
                return Err(failed(format!(
                    "“{}” isn't an option in “{label}”.",
                    p.value
                )));
            };
            let chosen: Value = self
                .tab()
                .injected_value(
                    SELECT_OPTION,
                    &[Arg::Handle(&handle), Arg::Value(json!(option))],
                )
                .await?;
            match chosen.as_str() {
                Some(problem) if problem.starts_with("error:") => Err(failed(format!(
                    "Couldn't choose “{}” in “{label}”.",
                    p.value
                ))),
                _ => Ok(()),
            }
        }
        .await;
        self.tab().release(&handle).await;
        picked?;
        self.rolling()?.pause(f64::from(p.beat.pause.after)).await;
        Ok(())
    }

    async fn upload(&self, p: opsis_compile::nodes::browser::Upload) -> StepResult {
        let resolved = self.resolve(&p.target, None).await?;
        let project = &self.env.project_dir;
        let file = project.join(project_rel_path(&p.file).unwrap_or_else(|| p.file.clone()));
        if !is_inside(project, &file) {
            return Err(failed("Choose a file inside the project folder."));
        }
        let handle = self.element(&resolved.target, &p.target).await?;
        let set = self
            .tab()
            .send(
                "DOM.setFileInputFiles",
                json!({ "files": [file.to_string_lossy()], "objectId": handle.0 }),
            )
            .await;
        self.tab().release(&handle).await;
        set?;
        self.pause_if_rolling(p.pause.after).await;
        Ok(())
    }

    async fn press(&self, step: &SessionStep, p: PressKeys) -> StepResult {
        let chord = pressed_chord(&p.keys);
        self.press_key(&chord, &step_name(step, Some(&p.keys)))
            .await?;
        self.pause_if_rolling(p.pause.after).await;
        Ok(())
    }

    async fn wait(&mut self, step: &SessionStep, p: Wait) -> StepResult {
        let t = self.recorder.elapsed();
        match p.mode {
            WaitMode::Element => {
                self.resolve(&p.target, Some(ms(p.timeout_ms))).await?;
            }
            WaitMode::NetworkIdle => {
                self.lifecycle
                    .wait_for_load_state(self.tab(), WaitUntil::Networkidle, ms(p.timeout_ms))
                    .await?;
            }
            WaitMode::Url => {
                self.lifecycle
                    .wait_for_url(self.tab(), &p.url, ms(p.timeout_ms))
                    .await?;
            }
            WaitMode::Ms => self.sleep(ms(p.ms)).await,
        }
        let mut action = Action::new(format!("{}:wait", step.id), ActionKind::Wait, t);
        action.t_end = Some(self.recorder.elapsed());
        self.push_action(action, step);
        Ok(())
    }

    async fn assert(&self, p: Assert) -> StepResult {
        let text = text_string_selector(&p.text, p.exact);
        let selector = if p.target.trim().is_empty() {
            first(&text)
        } else {
            let scope = self.resolve(&p.target, None).await?;
            first(&format!("{} >> {text}", scope.target.element()?))
        };
        match self.tab().wait_visible(&selector, ms(p.timeout_ms)).await {
            Ok(handle) => {
                self.tab().release(&handle).await;
                Ok(())
            }
            Err(CaptureError::NotFoundWithin(_)) => Err(failed(format!(
                "“{}” isn't on the page.\n{}",
                p.text,
                dump_visible(self.tab()).await
            ))),
            Err(error) => Err(error.into()),
        }
    }

    async fn hoist(&mut self, p: Hoist) -> StepResult {
        let resolved = self
            .resolve(
                &p.target.target,
                Some(Duration::from_millis(u64::from(p.timeout_ms))),
            )
            .await?;
        let selector = resolved.target.element()?.to_owned();
        self.hoisted.insert(p.target.target, selector);
        Ok(())
    }

    async fn set_state(&self, step: &SessionStep, p: SetState) -> StepResult {
        if p.every_page {
            self.session
                .add_init_script(&run_user_script(&p.script))
                .await?;
        }
        if self.lifecycle.url() == "about:blank" {
            if !p.every_page {
                self.session
                    .add_init_script(&before_first_load(&step.id, &p.script))
                    .await?;
            }
            return Ok(());
        }
        self.tab()
            .evaluate_main::<Value>(&run_user_script(&p.script))
            .await?;
        if p.reload {
            self.lifecycle.reload(self.tab()).await?;
        }
        Ok(())
    }

    async fn mock_route(&mut self, p: MockRoute) -> StepResult {
        let tab = Arc::clone(self.tab());
        self.routes
            .add(
                &tab,
                MockResponse {
                    url: p.url,
                    status: p.status,
                    content_type: p.content_type,
                    body: p.body,
                    delay_ms: p.delay_ms,
                },
            )
            .await?;
        Ok(())
    }

    async fn freeze_clock(&mut self, p: FreezeClock) -> StepResult {
        let zone: String = self.tab().evaluate_main(PAGE_TIME_ZONE).await?;
        let time = wall_time_in(&p.time, &zone).map_err(|e| failed(e.to_string()))?;
        self.clock
            .freeze(&self.session, p.mode, time.as_millisecond())
            .await?;
        Ok(())
    }

    async fn hide_elements(&self, p: HideElements) -> StepResult {
        let own = non_blank_lines(&p.selectors);
        let selectors: Vec<String> = (if p.cookie_banners {
            COOKIE_BANNER_SELECTORS.as_slice()
        } else {
            &[]
        })
        .iter()
        .chain(if p.chat_widgets {
            CHAT_WIDGET_SELECTORS.as_slice()
        } else {
            &[]
        })
        .map(|s| (*s).to_owned())
        .chain(own.iter().cloned())
        .collect();
        if selectors.is_empty() {
            return Ok(());
        }
        let invalid: Vec<String> = match self
            .tab()
            .evaluate_main(&call(INVALID_SELECTORS, &json!(own)))
            .await
        {
            Ok(invalid) => invalid,
            Err(error) => {
                self.log(
                    LogLevel::Warn,
                    &format!("Couldn't check which selectors are valid: {error}"),
                );
                Vec::new()
            }
        };
        if !invalid.is_empty() {
            let list = invalid
                .iter()
                .map(|s| format!("“{s}”"))
                .collect::<Vec<_>>()
                .join(", ");
            self.log(
                LogLevel::Warn,
                &format!("Can't hide {list}: not a CSS selector. The rest are hidden."),
            );
        }
        let styles = call(ADD_STYLES, &json!(hiding_css(&selectors)));
        self.session.add_init_script(&styles).await?;
        if let Err(error) = self.tab().evaluate_main::<Value>(&styles).await {
            self.log(
                LogLevel::Warn,
                &format!(
                    "Couldn't hide the elements on the current page, only on the next ones: {error}"
                ),
            );
        }
        Ok(())
    }

    async fn chapter(&mut self, step: &SessionStep, p: Chapter) -> StepResult {
        let description = Some(p.description.clone()).filter(|d| !d.is_empty());
        let mut action = Action::new(
            format!("{}:chapter", step.id),
            ActionKind::Chapter,
            self.recorder.elapsed(),
        );
        action.label = Some(p.title.clone());
        action.text = description;
        self.push_action(action, step);
        if p.show_in_video
            && let Some(overlays) = &self.overlays
        {
            overlays.chapter(&p).await;
            self.sleep(ms(p.after_ms)).await;
        }
        Ok(())
    }

    async fn overlay_html(&self, p: OverlayHtml) -> StepResult {
        let Some(overlays) = &self.overlays else {
            return Ok(());
        };
        let id = overlays.show_faded(&p.html, p.fade_ms).await;
        if p.duration_ms > 0 {
            overlays.remove_after(id, ms(p.duration_ms), p.fade_ms);
        }
        Ok(())
    }

    async fn sign_in(&mut self, step: &SessionStep, p: SignIn) -> StepResult {
        let name = secret_name(&p.secret_name);
        let node = step_name(step, None);
        let (sign_ins, wanted) = (Arc::clone(&self.env.sign_ins), name.clone());
        let saved = off_runtime(move || sign_ins.read(&wanted))
            .await
            .map_err(failed)?
            .ok_or_else(|| {
                failed(format!(
                    "The sign-in “{name}” isn't saved yet. Run “{node}” first."
                ))
            })?;
        let state: StorageState = serde_json::from_str(&saved).map_err(|_| {
            failed(format!(
                "The saved sign-in can't be read. Run “{node}” again."
            ))
        })?;
        let cookies: Vec<Value> = state
            .cookies
            .iter()
            .map(|c| {
                let mut cookie = json!({ "name": c.name, "value": c.value, "domain": c.domain, "path": c.path, "httpOnly": c.http_only, "secure": c.secure, "sameSite": c.same_site });
                if c.expires >= 0.0 {
                    cookie["expires"] = json!(c.expires);
                }
                cookie
            })
            .collect();
        self.tab()
            .send("Network.setCookies", json!({ "cookies": cookies }))
            .await?;
        let page_origin = url::Url::parse(&self.lifecycle.url())
            .ok()
            .map(|u| u.origin().ascii_serialization());
        for origin in &state.origins {
            let storage = SavedStorage {
                key: format!("opsis-sign-in:{}", step.id),
                origin: &origin.origin,
                items: &origin.local_storage,
            };
            let script = call(RESTORE_LOCAL_STORAGE, &json!(storage));
            self.session.add_init_script(&script).await?;
            if page_origin.as_deref() == Some(origin.origin.as_str()) {
                self.tab().evaluate_main::<Value>(&script).await?;
            }
        }
        self.signed_in = Some(SignedIn {
            url: p.url,
            name: node,
        });
        Ok(())
    }

    async fn play_chain(&mut self, steps: &[SessionStep]) -> StepResult {
        let start = video_start(steps);
        for step in &steps[..start] {
            self.run(step).await?;
        }
        self.begin_video().await?;
        if let Some(recording) = &self.recording {
            self.sleep(ms(recording.record.establish_ms)).await;
        }
        for step in &steps[start..] {
            self.run(step).await?;
        }
        Ok(())
    }

    async fn stop(mut self, failed: bool) -> Stopped {
        let end = self.tab().clock().now();
        self.tab().clock().stop();
        let t0 = *self.recorder.t0.lock().expect("take start");
        let mut duration_ms = as_ms(end.saturating_sub(t0));
        if let Some(screencast) = self.screencast.take()
            && let Err(error) = screencast.stop(self.tab()).await
        {
            self.log(
                LogLevel::Warn,
                &format!("The recording didn't stop cleanly: {error}"),
            );
        }
        let synthetic = self
            .recording
            .as_ref()
            .is_some_and(|r| r.record.cursor == Cursor::Synthetic);
        let mut encoded = Ok(None);
        if let Some(recording) = self.recording.take() {
            if self.env.cancel.is_cancelled() {
                recording.encoder.abort().await;
            } else {
                encoded = match recording.encoder.finish(end).await {
                    Ok(done) => {
                        if !failed {
                            duration_ms = done.duration_ms;
                        }
                        Ok(Some(done))
                    }
                    Err(_) if failed => Ok(None),
                    Err(error) => Err(error),
                };
            }
        }
        let trace = self.trace.take().map(|t| t.finish(self.session.viewport));
        let cursor_track = synthetic.then(|| self.recorder.recorded().cursor.clone());
        let log = ActionLog {
            viewport: self.session.viewport,
            dpr: self.session.dpr,
            t0: as_ms(t0),
            duration_ms,
            actions: std::mem::take(&mut self.actions),
            cursor_track,
            addresses: self.lifecycle.addresses_since(t0),
            caret_track: Vec::new(),
            titles: Vec::new(),
            cells: None,
        };
        self.session.close().await;
        Stopped {
            encoded,
            trace,
            log,
        }
    }
}

struct Stopped {
    encoded: Result<Option<Encoded>, EncodeError>,
    trace: Option<TraceTake>,
    log: ActionLog,
}

pub type FinaleFuture<'r, T> = Pin<Box<dyn Future<Output = StepResult<T>> + Send + 'r>>;

fn live_end(env: &TakeEnv, outcome: Result<(), &StepError>) {
    if let Some(live) = &env.live {
        live.event(match outcome {
            Ok(()) => LiveEvent::End {
                ok: true,
                message: None,
            },
            Err(error) => LiveEvent::End {
                ok: false,
                message: Some(error.to_string()),
            },
        });
    }
}

pub async fn play_take<'e, T>(
    env: &'e TakeEnv,
    session: &BrowserSessionValue,
    recording: Option<Recording>,
    finale: impl for<'r> FnOnce(&'r mut TakeRun<'e>) -> FinaleFuture<'r, T>,
) -> Played<T> {
    let mut take = match TakeRun::open(env, session, recording).await {
        Ok(take) => take,
        Err(error) => {
            return Played {
                outcome: Err(error.into()),
                encoded: None,
                trace: None,
            };
        }
    };
    if let Some(live) = &env.live {
        let title = node_def(OPEN_BROWSER_TYPE).map_or_default(|d| d.title.clone());
        let steps = session
            .steps
            .iter()
            .map(|s| LiveStep {
                node_id: s.id.clone(),
                label: step_name(s, None),
            })
            .collect();
        live.event(LiveEvent::Start {
            title,
            viewport: take.session.viewport,
            steps,
        });
    }
    let played = async {
        take.start_screencast().await?;
        take.play_chain(&session.steps).await?;
        finale(&mut take).await
    }
    .await;
    if let Err(error) = &played {
        if let StepError::NotFound(error) = error {
            take.log(LogLevel::Error, &error.to_string());
        }
        live_end(env, Err(error));
    }
    let stopped = take.stop(played.is_err()).await;
    let outcome = match (played, stopped.encoded.as_ref()) {
        (Ok(from_page), Ok(_)) => {
            live_end(env, Ok(()));
            Ok(Took {
                log: stopped.log,
                from_page,
            })
        }
        (Ok(_), Err(error)) => Err(failed(error.to_string())),
        (Err(error), _) => Err(error),
    };
    Played {
        outcome,
        encoded: stopped.encoded.ok().flatten(),
        trace: stopped.trace,
    }
}

pub async fn record_take(
    env: &TakeEnv,
    session: &BrowserSessionValue,
    recording: Recording,
) -> Played<()> {
    let tail = ms(recording.record.end_tail_ms);
    play_take(env, session, Some(recording), move |take| {
        Box::pin(async move {
            take.sleep(tail).await;
            Ok(())
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_what_it_couldnt_find() {
        let mut step = SessionStep {
            id: "n".into(),
            kind: BrowserAction::Click,
            label: None,
            params: serde_json::Map::new(),
        };
        let missing = StepError::Capture(CaptureError::NotFoundWithin(Duration::from_secs(15)));
        assert_eq!(
            plain_message(&missing, &step),
            "Couldn't find the element on the page within 15 s."
        );
        step.params
            .insert("target".into(), json!("css=#add-to-cart"));
        assert_eq!(
            plain_message(&missing, &step),
            "Couldn't find “#add-to-cart” on the page within 15 s."
        );
        assert_eq!(
            plain_message(&StepError::Capture(CaptureError::Closed), &step),
            "The browser closed before the action finished."
        );
    }
}
