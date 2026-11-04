use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opsis_compile::terminal::{default_look, default_theme, grid, screen_size};
use opsis_types::live::{LiveEvent, LiveStep, LiveStepStatus};
use opsis_types::numbers::millis;
use opsis_types::terminal::{TerminalStep, TerminalTheme};
use opsis_vt::Terminal;

use crate::frames::write;
use crate::prepare::snapshot;

pub trait LiveFeed: Send + Sync {
    fn frame(&self, capture_id: &str, node_id: &str, frames: Vec<u8>, t: f64);
    fn event(&self, capture_id: &str, node_id: &str, event: LiveEvent, t: f64);
}

pub struct TakeLive {
    feed: Arc<dyn LiveFeed>,
    capture_id: String,
    node_id: String,
    title: String,
    gap: Duration,
    theme: TerminalTheme,
}

impl fmt::Debug for TakeLive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeLive")
            .field("capture_id", &self.capture_id)
            .field("node_id", &self.node_id)
            .field("title", &self.title)
            .field("gap", &self.gap)
            .finish_non_exhaustive()
    }
}

fn wall_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, millis)
}

impl TakeLive {
    pub fn new(
        feed: Arc<dyn LiveFeed>,
        run_id: &str,
        node_id: &str,
        title: &str,
        fps: u32,
    ) -> Self {
        Self {
            feed,
            capture_id: format!("{run_id}:{node_id}"),
            node_id: node_id.to_owned(),
            title: title.to_owned(),
            gap: Duration::from_millis(1000 / u64::from(fps.max(1))),
            theme: default_theme(),
        }
    }

    pub const fn gap(&self) -> Duration {
        self.gap
    }

    fn event(&self, event: LiveEvent) {
        self.feed
            .event(&self.capture_id, &self.node_id, event, wall_ms());
    }

    pub fn start(&self, cols: u16, rows: u16, steps: &[TerminalStep]) {
        self.event(LiveEvent::Start {
            title: self.title.clone(),
            viewport: screen_size(&grid(&default_look(Some(self.theme.clone())), cols, rows)),
            steps: steps
                .iter()
                .map(|s| LiveStep {
                    node_id: s.id.clone(),
                    label: s.label.clone(),
                })
                .collect(),
        });
    }

    pub fn step(&self, step: &TerminalStep, status: LiveStepStatus, message: Option<String>) {
        self.event(LiveEvent::Step {
            node_id: step.id.clone(),
            status,
            message,
        });
    }

    pub fn end(&self, ok: bool, message: Option<String>) {
        self.event(LiveEvent::End { ok, message });
    }

    pub fn frame(&self, terminal: &mut Terminal) {
        let frames = snapshot(terminal, &self.theme).map(|f| write(&f));
        match frames {
            Ok(Ok(bytes)) => {
                self.feed
                    .frame(&self.capture_id, &self.node_id, bytes, wall_ms());
            }
            Ok(Err(error)) => log::warn!("writing a live terminal frame: {error}"),
            Err(error) => log::warn!("reading a live terminal frame: {error}"),
        }
    }
}
