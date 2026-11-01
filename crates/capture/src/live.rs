use std::sync::{Arc, Mutex};

use opsis_types::live::LiveEvent;

use crate::clock::wall_ms;

pub trait FrameSink: Send + Sync {
    fn frame(&self, jpeg: &[u8], width: f64, height: f64, t: f64);
}

pub trait LiveSink: FrameSink {
    fn event(&self, event: LiveEvent);
}

pub struct LiveFrame {
    pub capture_id: String,
    pub node_id: String,
    pub jpeg: Vec<u8>,
    pub width: f64,
    pub height: f64,
    pub t: f64,
}

pub struct LiveNote {
    pub capture_id: String,
    pub node_id: String,
    pub t: f64,
    pub event: LiveEvent,
}

pub enum LiveMessage {
    Frame(LiveFrame),
    Event(LiveNote),
}

pub trait LiveFeed: Send + Sync {
    fn send(&self, message: LiveMessage);
}

pub struct NodeLive {
    feed: Arc<dyn LiveFeed>,
    capture_id: String,
    node_id: String,
    title: String,
    min_gap_ms: f64,
    last_frame: Mutex<f64>,
}

impl NodeLive {
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
            min_gap_ms: 1000.0 / f64::from(fps.max(1)),
            last_frame: Mutex::new(0.0),
        }
    }
}

impl FrameSink for NodeLive {
    fn frame(&self, jpeg: &[u8], width: f64, height: f64, t: f64) {
        {
            let mut last = self.last_frame.lock().expect("live frame");
            if t - *last < self.min_gap_ms {
                return;
            }
            *last = t;
        }
        self.feed.send(LiveMessage::Frame(LiveFrame {
            capture_id: self.capture_id.clone(),
            node_id: self.node_id.clone(),
            jpeg: jpeg.to_vec(),
            width,
            height,
            t,
        }));
    }
}

impl LiveSink for NodeLive {
    fn event(&self, event: LiveEvent) {
        let event = match event {
            LiveEvent::Start {
                viewport, steps, ..
            } => LiveEvent::Start {
                title: self.title.clone(),
                viewport,
                steps,
            },
            other => other,
        };
        self.feed.send(LiveMessage::Event(LiveNote {
            capture_id: self.capture_id.clone(),
            node_id: self.node_id.clone(),
            t: wall_ms(),
            event,
        }));
    }
}
