use std::time::Duration;

use crate::cast::{Event, EventData};

pub const START: &str = "opsis:start";
pub const HIDE: &str = "opsis:hide";
pub const SHOW: &str = "opsis:show";
pub const END: &str = "opsis:end";
pub const SCREENSHOT: &str = "opsis:screenshot";
pub const STEP_PREFIX: &str = "opsis:step:";
pub const CHAPTER_PREFIX: &str = "opsis:chapter:";
pub const SCROLL_PREFIX: &str = "opsis:scroll:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub from: Duration,
    pub to: Duration,
}

impl Span {
    pub const fn length(self) -> Duration {
        self.to.saturating_sub(self.from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Timeline {
    spans: Vec<Span>,
}

fn marker(event: &Event) -> Option<&str> {
    match &event.data {
        EventData::Marker(label) => Some(label),
        _ => None,
    }
}

impl Timeline {
    pub fn from_events(events: &[Event]) -> Self {
        let last = events.iter().map(|e| e.time).max().unwrap_or_default();
        let start = events
            .iter()
            .find(|e| marker(e) == Some(START))
            .or_else(|| {
                events
                    .iter()
                    .find(|e| marker(e).is_some_and(|m| m.starts_with(STEP_PREFIX)))
            })
            .map_or(Duration::ZERO, |e| e.time);
        let end = events
            .iter()
            .find(|e| marker(e) == Some(END))
            .map_or(last, |e| e.time)
            .max(start);
        let mut spans = Vec::new();
        let mut open = Some(start);
        for event in events.iter().filter(|e| e.time > start && e.time < end) {
            match (marker(event), open) {
                (Some(HIDE), Some(from)) => {
                    if event.time > from {
                        spans.push(Span {
                            from,
                            to: event.time,
                        });
                    }
                    open = None;
                }
                (Some(SHOW), None) => open = Some(event.time),
                _ => {}
            }
        }
        if let Some(from) = open {
            spans.push(Span { from, to: end });
        }
        Self::from_spans(spans)
    }

    pub fn from_spans(spans: Vec<Span>) -> Self {
        let spans = spans.into_iter().filter(|s| s.to > s.from).collect();
        Self { spans }
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn length(&self) -> Duration {
        self.spans.iter().map(|s| s.length()).sum()
    }

    pub fn visible(&self, raw: Duration) -> Option<Duration> {
        let mut before = Duration::ZERO;
        for span in &self.spans {
            if raw >= span.from && raw <= span.to {
                return Some(before + raw.saturating_sub(span.from));
            }
            before += span.length();
        }
        None
    }

    pub fn raw(&self, visible: Duration) -> Duration {
        let mut before = Duration::ZERO;
        let mut found = None;
        for span in &self.spans {
            if visible >= before {
                found = Some((span, before));
            }
            before += span.length();
        }
        found.map_or(Duration::ZERO, |(span, before)| {
            span.from + visible.saturating_sub(before).min(span.length())
        })
    }

    pub fn start(&self) -> Duration {
        self.spans.first().map_or(Duration::ZERO, |s| s.from)
    }

    pub fn end(&self) -> Duration {
        self.spans.last().map_or(Duration::ZERO, |s| s.to)
    }
}
