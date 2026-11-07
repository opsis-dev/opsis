use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use opsis_qt::{FrameRef, LiveCaptureView, LiveClickView, LiveView};
use opsis_types::live::{LiveEvent, LiveStepStatus};

use crate::streams::{FrameHeader, Shown, next_target};

pub const MAX_CLICKS: usize = 9;
pub const RIPPLE: Duration = Duration::from_millis(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Live,
    Done,
    Error,
}

#[derive(Debug, Clone)]
struct Step {
    node_id: String,
    label: String,
    status: Option<LiveStepStatus>,
}

#[derive(Debug, Clone)]
struct Target {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: String,
}

#[derive(Debug, Clone)]
pub struct Capture {
    node_id: String,
    title: String,
    viewport: Option<(u32, u32)>,
    steps: Vec<Step>,
    shown: Shown,
    cursor: Option<(f64, f64)>,
    target: Option<Target>,
    clicks: Vec<(f64, f64, Instant)>,
    pub status: Status,
    message: String,
}

impl Capture {
    fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_owned(),
            title: String::new(),
            viewport: None,
            steps: Vec::new(),
            shown: Shown::default(),
            cursor: None,
            target: None,
            clicks: Vec::new(),
            status: Status::Live,
            message: String::new(),
        }
    }
}

pub fn recent_clicks(clicks: &[(f64, f64, Instant)], now: Instant) -> Vec<(f64, f64, Instant)> {
    clicks
        .iter()
        .copied()
        .filter(|&(_, _, at)| now.saturating_duration_since(at) < RIPPLE)
        .collect()
}

pub fn with_click(
    clicks: &[(f64, f64, Instant)],
    click: (f64, f64, Instant),
) -> Vec<(f64, f64, Instant)> {
    let mut kept = recent_clicks(clicks, click.2);
    kept.push(click);
    let extra = kept.len().saturating_sub(MAX_CLICKS);
    kept.drain(..extra);
    kept
}

pub fn apply(capture: &mut Capture, event: &LiveEvent, now: Instant) {
    match event {
        LiveEvent::Start {
            title,
            viewport,
            steps,
        } => {
            capture.title.clone_from(title);
            capture.viewport = Some((viewport.width, viewport.height));
            capture.steps = steps
                .iter()
                .map(|s| Step {
                    node_id: s.node_id.clone(),
                    label: s.label.clone(),
                    status: None,
                })
                .collect();
            capture.status = Status::Live;
        }
        LiveEvent::Step {
            node_id, status, ..
        } => {
            for step in capture.steps.iter_mut().filter(|s| &s.node_id == node_id) {
                step.status = Some(*status);
            }
        }
        LiveEvent::Cursor { x, y } => capture.cursor = Some((*x, *y)),
        LiveEvent::Target { rect, label, .. } => {
            capture.target = Some(Target {
                x: rect.x,
                y: rect.y,
                width: rect.w,
                height: rect.h,
                label: label.clone(),
            });
        }
        LiveEvent::Click { x, y } => capture.clicks = with_click(&capture.clicks, (*x, *y, now)),
        LiveEvent::End { ok, message } => {
            capture.status = if *ok { Status::Done } else { Status::Error };
            capture.message = message.clone().unwrap_or_default();
        }
    }
}

pub fn focus_after_start(
    focus: Option<&str>,
    captures: &IndexMap<String, Capture>,
    started: &str,
) -> String {
    focus
        .filter(|id| captures.get(*id).is_some_and(|c| c.status == Status::Live))
        .map_or_else(|| started.to_owned(), str::to_owned)
}

#[derive(Debug, Default)]
struct State {
    captures: IndexMap<String, Capture>,
    focus: Option<String>,
    pip_open: bool,
}

#[derive(Debug, Clone)]
pub struct Live {
    state: Arc<Mutex<State>>,
    target: u64,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            state: Arc::default(),
            target: next_target(),
        }
    }
}

const fn status_word(status: Status) -> &'static str {
    match status {
        Status::Live => "live",
        Status::Done => "done",
        Status::Error => "error",
    }
}

impl Live {
    pub const fn target(&self) -> u64 {
        self.target
    }

    fn capture<'a>(
        state: &'a mut State,
        capture_id: &str,
        node_id: &str,
        auto_open: &dyn Fn() -> bool,
    ) -> &'a mut Capture {
        if !state.captures.contains_key(capture_id) {
            let focus = focus_after_start(state.focus.as_deref(), &state.captures, capture_id);
            state.focus = Some(focus);
            state.pip_open = state.pip_open || auto_open();
            state
                .captures
                .insert(capture_id.to_owned(), Capture::new(node_id));
        }
        state
            .captures
            .get_mut(capture_id)
            .expect("the capture was just added")
    }

    pub fn noted(
        &self,
        capture_id: &str,
        node_id: &str,
        event: &LiveEvent,
        auto_open: &dyn Fn() -> bool,
    ) {
        {
            let mut state = self.state.lock().expect("live");
            let capture = Self::capture(&mut state, capture_id, node_id, auto_open);
            apply(capture, event, Instant::now());
        }
        opsis_qt::wake(self.target);
    }

    pub fn framed(&self, header: &FrameHeader, jpeg: &[u8], auto_open: &dyn Fn() -> bool) {
        let capture_id = header.capture_id.clone().unwrap_or_default();
        let node_id = header.node_id.clone().unwrap_or_default();
        {
            let mut state = self.state.lock().expect("live");
            let capture = Self::capture(&mut state, &capture_id, &node_id, auto_open);
            capture.shown = capture.shown.next(header, jpeg);
            if capture.viewport.is_none() {
                capture.viewport = Some((capture.shown.width, capture.shown.height));
            }
            capture.clicks = recent_clicks(&capture.clicks, Instant::now());
        }
        opsis_qt::wake(self.target);
    }

    pub fn focus(&self, capture_id: &str) {
        self.state.lock().expect("live").focus = Some(capture_id.to_owned());
        opsis_qt::wake(self.target);
    }

    pub fn set_pip(&self, open: bool) {
        self.state.lock().expect("live").pip_open = open;
        opsis_qt::wake(self.target);
    }

    pub fn reset(&self) {
        let mut state = self.state.lock().expect("live");
        state.captures.clear();
        state.focus = None;
        drop(state);
        opsis_qt::wake(self.target);
    }

    pub fn frame(&self, capture_id: &str) -> FrameRef {
        self.state
            .lock()
            .expect("live")
            .captures
            .get(capture_id)
            .map_or_else(FrameRef::empty, |c| c.shown.frame())
    }

    pub fn view(&self, name_of: &dyn Fn(&str) -> String) -> LiveView {
        let now = Instant::now();
        let state = self.state.lock().expect("live");
        let focus = state
            .focus
            .as_deref()
            .and_then(|id| state.captures.get_index_of(id))
            .and_then(|i| i32::try_from(i).ok())
            .unwrap_or(-1);
        LiveView {
            captures: state
                .captures
                .iter()
                .map(|(id, c)| {
                    let (width, height) = c.viewport.unwrap_or((0, 0));
                    let current = c
                        .steps
                        .iter()
                        .find(|s| s.status == Some(LiveStepStatus::Running))
                        .map_or_default(|s| s.label.clone());
                    let title = if c.title.is_empty() {
                        name_of(&c.node_id)
                    } else {
                        c.title.clone()
                    };
                    LiveCaptureView {
                        capture_id: id.clone(),
                        title,
                        status: status_word(c.status).to_owned(),
                        message: c.message.clone(),
                        width,
                        height,
                        frame: c.shown.serial,
                        has_cursor: c.cursor.is_some(),
                        cursor_x: c.cursor.map_or(0.0, |p| p.0),
                        cursor_y: c.cursor.map_or(0.0, |p| p.1),
                        has_target: c.target.is_some(),
                        target_x: c.target.as_ref().map_or(0.0, |t| t.x),
                        target_y: c.target.as_ref().map_or(0.0, |t| t.y),
                        target_width: c.target.as_ref().map_or(0.0, |t| t.width),
                        target_height: c.target.as_ref().map_or(0.0, |t| t.height),
                        target_label: c.target.as_ref().map_or_default(|t| t.label.clone()),
                        step: current,
                        clicks: recent_clicks(&c.clicks, now)
                            .iter()
                            .map(|&(x, y, at)| LiveClickView {
                                x,
                                y,
                                age: now.saturating_duration_since(at).as_secs_f64()
                                    / RIPPLE.as_secs_f64(),
                            })
                            .collect(),
                    }
                })
                .collect(),
            focus,
            pip_open: state.pip_open,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use indexmap::IndexMap;
    use proptest::prelude::*;

    use super::{Capture, MAX_CLICKS, RIPPLE, Status, focus_after_start, with_click};

    proptest! {
        #[test]
        fn clicks_keep_the_recent_few(gaps in proptest::collection::vec(0u64..400, 1..40)) {
            let start = Instant::now();
            let mut clicks = Vec::new();
            let mut at = start;
            for gap in gaps {
                at += Duration::from_millis(gap);
                clicks = with_click(&clicks, (0.0, 0.0, at));
                prop_assert!(clicks.len() <= MAX_CLICKS);
                prop_assert!(clicks.iter().all(|c| at.saturating_duration_since(c.2) < RIPPLE));
                prop_assert_eq!(clicks.last().map(|c| c.2), Some(at));
            }
        }

        #[test]
        fn a_new_capture_takes_focus_unless_a_live_one_has_it(live in any::<bool>(), focused in any::<bool>()) {
            let mut captures = IndexMap::new();
            let mut first = Capture::new("a");
            first.status = if live { Status::Live } else { Status::Done };
            captures.insert("one".to_owned(), first);
            let focus = focused.then_some("one");
            let next = focus_after_start(focus, &captures, "two");
            prop_assert_eq!(next == "one", live && focused);
        }
    }
}
