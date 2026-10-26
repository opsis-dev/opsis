use std::sync::{Arc, Mutex, MutexGuard};

use chromiumoxide::cdp::browser_protocol::network::{
    EventLoadingFailed, EventLoadingFinished, EventRequestWillBeSent, EventResponseReceived,
    Response,
};
use chromiumoxide::cdp::js_protocol::runtime::{
    ConsoleApiCalledType, EventConsoleApiCalled, EventExceptionThrown, RemoteObject,
};
use futures::StreamExt;
use indexmap::IndexMap;
use opsis_runner::context::now_ms;
use opsis_traces::{TakeFrame, TraceTake};
use opsis_types::actions::{Bounds, Point, Viewport};
use opsis_types::trace::{ConsoleLevel, TraceAction, TraceConsole, TraceError, TraceRequest};
use tokio::task::JoinHandle;

use crate::cdp::{Result, Tab};

const COLLAPSIBLE: [&str; 2] = ["mouseMove", "mouseWheel"];
const WAIT: &str = "waitForTimeout";
const PAUSE_MS: f64 = 100.0;
pub const FRAME_GAP_MS: f64 = 250.0;

pub struct CallInfo {
    pub method: &'static str,
    pub title: String,
    pub target: Option<String>,
    pub point: Option<Point>,
    pub bounds: Option<Bounds>,
}

impl CallInfo {
    pub fn new(method: &'static str, title: impl Into<String>) -> Self {
        Self {
            method,
            title: title.into(),
            target: None,
            point: None,
            bounds: None,
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_owned());
        self
    }

    pub fn at(mut self, point: Point) -> Self {
        self.point = Some(point);
        self
    }
}

struct Call {
    method: &'static str,
    action: TraceAction,
    count: usize,
}

struct Pending {
    request: TraceRequest,
    started: f64,
}

#[derive(Default)]
struct State {
    calls: Vec<Call>,
    group: Option<usize>,
    console: Vec<TraceConsole>,
    errors: Vec<TraceError>,
    pending: IndexMap<String, Pending>,
    network: Vec<TraceRequest>,
    frames: Vec<TakeFrame>,
    last_frame: Option<f64>,
}

impl State {
    fn finish(&mut self, id: &str, ended: f64, update: impl FnOnce(&mut TraceRequest)) {
        if let Some(mut pending) = self.pending.shift_remove(id) {
            pending.request.duration = ((ended - pending.started) * 1000.0).max(0.0);
            update(&mut pending.request);
            self.network.push(pending.request);
        }
    }

    fn group_id(&self) -> Option<String> {
        self.group.map(|g| self.calls[g].action.id.clone())
    }

    fn push(&mut self, method: &'static str, action: TraceAction) -> usize {
        self.calls.push(Call {
            method,
            action,
            count: 1,
        });
        self.calls.len() - 1
    }
}

pub struct Trace {
    state: Arc<Mutex<State>>,
    listeners: Vec<JoinHandle<()>>,
}

impl Drop for Trace {
    fn drop(&mut self) {
        for listener in &self.listeners {
            listener.abort();
        }
    }
}

fn level_of(kind: &ConsoleApiCalledType) -> ConsoleLevel {
    match kind {
        ConsoleApiCalledType::Debug => ConsoleLevel::Debug,
        ConsoleApiCalledType::Info => ConsoleLevel::Info,
        ConsoleApiCalledType::Error | ConsoleApiCalledType::Assert => ConsoleLevel::Error,
        ConsoleApiCalledType::Warning => ConsoleLevel::Warning,
        _ => ConsoleLevel::Log,
    }
}

fn text_of(arg: &RemoteObject) -> String {
    match &arg.value {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(value) => value.to_string(),
        None => arg
            .description
            .clone()
            .or_else(|| arg.unserializable_value.as_ref().map(|v| v.inner().clone()))
            .unwrap_or_else(|| arg.r#type.as_ref().to_owned()),
    }
}

fn status_of(response: &Response) -> Option<u16> {
    u16::try_from(response.status).ok()
}

fn absorbs(prev: &Call, next: &Call) -> bool {
    COLLAPSIBLE.contains(&prev.method)
        && prev.action.parent_id == next.action.parent_id
        && prev.action.error.is_none()
        && next.action.error.is_none()
        && (next.method == prev.method
            || (next.method == WAIT && next.action.end - next.action.start < PAUSE_MS))
}

fn collapsed(calls: Vec<Call>) -> Vec<TraceAction> {
    let mut kept: Vec<Call> = Vec::new();
    for call in calls {
        if let Some(prev) = kept.last_mut()
            && absorbs(prev, &call)
        {
            if call.method == prev.method {
                prev.count += 1;
            }
            prev.action.end = prev.action.end.max(call.action.end);
            prev.action.logs.extend(call.action.logs);
            if call.action.point.is_some() {
                prev.action.point = call.action.point;
            }
            continue;
        }
        kept.push(call);
    }
    kept.into_iter()
        .map(
            |Call {
                 mut action, count, ..
             }| {
                if count > 1 {
                    action.title = format!("{} ×{count}", action.title);
                }
                action
            },
        )
        .collect()
}

impl Trace {
    pub async fn start(tab: &Tab) -> Result<Self> {
        let state: Arc<Mutex<State>> = Arc::default();
        let mut console = tab.page.event_listener::<EventConsoleApiCalled>().await?;
        let mut exceptions = tab.page.event_listener::<EventExceptionThrown>().await?;
        let mut sent = tab.page.event_listener::<EventRequestWillBeSent>().await?;
        let mut received = tab.page.event_listener::<EventResponseReceived>().await?;
        let mut finished = tab.page.event_listener::<EventLoadingFinished>().await?;
        let mut failed = tab.page.event_listener::<EventLoadingFailed>().await?;
        let mut listeners = Vec::new();
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = console.next().await {
                let url = event
                    .stack_trace
                    .as_ref()
                    .and_then(|t| t.call_frames.first())
                    .map(|f| f.url.clone())
                    .filter(|u| !u.is_empty());
                let text = event.args.iter().map(text_of).collect::<Vec<_>>().join(" ");
                lock(&s).console.push(TraceConsole {
                    level: level_of(&event.r#type),
                    text,
                    time: *event.timestamp.inner(),
                    url,
                });
            }
        }));
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = exceptions.next().await {
                let details = &event.exception_details;
                let description = details
                    .exception
                    .as_ref()
                    .and_then(|e| e.description.clone())
                    .unwrap_or_else(|| details.text.clone());
                let message = description.lines().next().unwrap_or_default().to_owned();
                let stack = (description.lines().count() > 1).then_some(description);
                let mut state = lock(&s);
                let action_id = state.group_id();
                state.errors.push(TraceError {
                    message,
                    stack,
                    time: *event.timestamp.inner(),
                    action_id,
                });
            }
        }));
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = sent.next().await {
                let id = event.request_id.inner().clone();
                let started = *event.timestamp.inner();
                let mut state = lock(&s);
                if let Some(redirect) = &event.redirect_response {
                    let status = status_of(redirect);
                    state.finish(&id, started, |r| r.status = status);
                }
                let request = TraceRequest {
                    method: event.request.method.clone(),
                    url: format!(
                        "{}{}",
                        event.request.url,
                        event.request.url_fragment.clone().unwrap_or_default()
                    ),
                    status: None,
                    mime_type: String::new(),
                    resource_type: event
                        .r#type
                        .as_ref()
                        .map(|t| t.as_ref().to_owned())
                        .unwrap_or_default(),
                    size: None,
                    start: *event.wall_time.inner() * 1000.0,
                    duration: 0.0,
                    failure: None,
                };
                state.pending.insert(id, Pending { request, started });
            }
        }));
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = received.next().await {
                if let Some(pending) = lock(&s).pending.get_mut(event.request_id.inner()) {
                    pending.request.status = status_of(&event.response);
                    pending.request.mime_type = event.response.mime_type.clone();
                }
            }
        }));
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = finished.next().await {
                let size = event.encoded_data_length;
                lock(&s).finish(event.request_id.inner(), *event.timestamp.inner(), |r| {
                    r.size = Some(size)
                });
            }
        }));
        let s = Arc::clone(&state);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = failed.next().await {
                let failure = event.error_text.clone();
                lock(&s).finish(event.request_id.inner(), *event.timestamp.inner(), |r| {
                    r.failure = Some(failure)
                });
            }
        }));
        Ok(Self { state, listeners })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        lock(&self.state)
    }

    pub fn group(&self, node_id: &str, title: &str) {
        let mut state = self.state();
        let id = format!("group@{}", state.calls.len());
        let now = now_ms();
        let action = TraceAction {
            id,
            parent_id: None,
            group: true,
            node_id: Some(node_id.to_owned()),
            title: title.to_owned(),
            target: None,
            start: now,
            end: now,
            error: None,
            logs: Vec::new(),
            point: None,
            bounds: None,
        };
        let index = state.push("tracingGroup", action);
        state.group = Some(index);
    }

    pub fn end_group(&self, error: Option<&str>) {
        let mut state = self.state();
        if let Some(index) = state.group.take() {
            let action = &mut state.calls[index].action;
            action.end = now_ms();
            action.error = error.map(str::to_owned);
        }
    }

    pub fn begin(&self, info: CallInfo) -> usize {
        let mut state = self.state();
        let now = now_ms();
        let action = TraceAction {
            id: format!("call@{}", state.calls.len()),
            parent_id: state.group_id(),
            group: false,
            node_id: None,
            title: info.title,
            target: info.target,
            start: now,
            end: now,
            error: None,
            logs: Vec::new(),
            point: info.point,
            bounds: info.bounds,
        };
        state.push(info.method, action)
    }

    pub fn end(&self, call: usize, error: Option<String>, bounds: Option<Bounds>) {
        let mut state = self.state();
        if let Some(Call { action, .. }) = state.calls.get_mut(call) {
            action.end = now_ms();
            action.error = error;
            if bounds.is_some() {
                action.bounds = bounds;
            }
        }
    }

    pub fn log(&self, line: &str) {
        let mut state = self.state();
        if let Some(index) = state.group {
            state.calls[index].action.logs.push(line.to_owned());
        }
    }

    pub fn frame(&self, jpeg: &[u8], time: f64) {
        let mut state = self.state();
        if state
            .last_frame
            .is_some_and(|last| time - last < FRAME_GAP_MS)
        {
            return;
        }
        let Ok(size) = imagesize::blob_size(jpeg) else {
            return;
        };
        state.last_frame = Some(time);
        state.frames.push(TakeFrame {
            jpeg: jpeg.to_vec(),
            time,
            width: size.width as u32,
            height: size.height as u32,
        });
    }

    pub fn finish(&self, viewport: Viewport) -> TraceTake {
        let mut state = std::mem::take(&mut *self.state());
        let pending: Vec<TraceRequest> = state.pending.drain(..).map(|(_, p)| p.request).collect();
        state.network.extend(pending);
        TraceTake {
            viewport: Some(viewport),
            actions: collapsed(state.calls),
            frames: state.frames,
            console: state.console,
            errors: state.errors,
            network: state.network,
        }
    }
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().expect("trace")
}
