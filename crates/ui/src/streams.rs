use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use opsis_qt::{FrameRef, StreamViewState};
use opsis_session::platform::FrameSender;
use serde::Deserialize;
use serde_json::Value;

static CHANNELS: AtomicU32 = AtomicU32::new(1);
static TARGETS: AtomicU64 = AtomicU64::new(1 << 40);

pub fn next_target() -> u64 {
    TARGETS.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameHeader {
    pub width: f64,
    pub height: f64,
}

pub fn unpacked(frame: &[u8]) -> Option<(FrameHeader, &[u8])> {
    let (length, rest) = frame.split_first_chunk::<4>()?;
    let length = usize::try_from(u32::from_le_bytes(*length)).ok()?;
    let header = rest.get(..length)?;
    let jpeg = rest.get(length..)?;
    Some((serde_json::from_slice(header).ok()?, jpeg))
}

#[derive(Debug, Default, Clone)]
pub struct Shown {
    pub serial: u64,
    pub width: u32,
    pub height: u32,
    pub jpeg: Arc<Vec<u8>>,
}

impl Shown {
    pub fn next(&self, header: &FrameHeader, jpeg: &[u8]) -> Self {
        Self {
            serial: self.serial + 1,
            width: opsis_types::numbers::whole(header.width),
            height: opsis_types::numbers::whole(header.height),
            jpeg: Arc::new(jpeg.to_vec()),
        }
    }

    pub fn frame(&self) -> FrameRef {
        FrameRef::jpeg(Arc::clone(&self.jpeg), self.width, self.height, self.serial)
    }
}

pub type OnFrame = Box<dyn Fn(&FrameHeader, &[u8]) + Send + Sync>;

pub struct FrameSlot {
    id: u32,
    on_frame: OnFrame,
}

impl std::fmt::Debug for FrameSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameSlot")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl FrameSlot {
    pub fn new(on_frame: OnFrame) -> Self {
        Self {
            id: CHANNELS.fetch_add(1, Ordering::Relaxed),
            on_frame,
        }
    }
}

impl FrameSender for FrameSlot {
    fn id(&self) -> u32 {
        self.id
    }

    fn send(&self, frame: Vec<u8>) -> bool {
        if let Some((header, jpeg)) = unpacked(&frame) {
            (self.on_frame)(&header, jpeg);
        } else {
            log::warn!("a frame came without its header");
        }
        true
    }
}

#[derive(Debug, Clone)]
struct ViewWindow {
    target: u64,
    shown: Shown,
    tabs: Vec<(String, String)>,
    active: usize,
    dialog: Option<(String, String, String)>,
    serial: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ViewWindows(Arc<Mutex<HashMap<String, ViewWindow>>>);

impl ViewWindows {
    pub fn slot(&self, id: &str) -> (u64, FrameSlot) {
        let target = next_target();
        self.0.lock().expect("view windows").insert(
            id.to_owned(),
            ViewWindow {
                target,
                shown: Shown::default(),
                tabs: Vec::new(),
                active: 0,
                dialog: None,
                serial: 0,
            },
        );
        let windows = self.clone();
        let id = id.to_owned();
        let slot = FrameSlot::new(Box::new(move |header, jpeg| {
            let mut open = windows.0.lock().expect("view windows");
            let Some(window) = open.get_mut(&id) else {
                return;
            };
            window.shown = window.shown.next(header, jpeg);
            let target = window.target;
            drop(open);
            opsis_qt::wake(target);
        }));
        (target, slot)
    }

    pub fn forget(&self, id: &str) {
        self.0.lock().expect("view windows").remove(id);
    }

    pub fn noted(&self, id: &str, event: &Value) -> bool {
        let mut open = self.0.lock().expect("view windows");
        let Some(window) = open.get_mut(id) else {
            return false;
        };
        match event.get("kind").and_then(Value::as_str) {
            Some("tabs") => {
                window.tabs = event
                    .get("tabs")
                    .and_then(Value::as_array)
                    .map_or_default(|tabs| {
                        tabs.iter()
                            .map(|tab| (text_of(tab, "id"), text_of(tab, "url")))
                            .collect()
                    });
                window.active = event
                    .get("active")
                    .and_then(Value::as_u64)
                    .and_then(|a| usize::try_from(a).ok())
                    .unwrap_or(0);
            }
            Some("dialog") => {
                window.dialog = Some((
                    text_of(event, "dialog"),
                    text_of(event, "message"),
                    text_of(event, "defaultPrompt"),
                ));
            }
            _ => return true,
        }
        window.serial += 1;
        let target = window.target;
        drop(open);
        opsis_qt::wake(target);
        false
    }

    pub fn answered(&self, id: &str) {
        if let Some(window) = self.0.lock().expect("view windows").get_mut(id) {
            window.dialog = None;
            window.serial += 1;
        }
    }

    pub fn state(&self, id: &str) -> StreamViewState {
        let window = self.0.lock().expect("view windows").get(id).cloned();
        let Some(window) = window else {
            return StreamViewState {
                serial: 0,
                frame: 0,
                url: String::new(),
                tab_ids: Vec::new(),
                tab_urls: Vec::new(),
                active: 0,
                dialog: String::new(),
                message: String::new(),
                prompt: String::new(),
            };
        };
        let (dialog, message, prompt) = window.dialog.unwrap_or_default();
        StreamViewState {
            serial: window.serial,
            frame: window.shown.serial,
            url: window
                .tabs
                .get(window.active)
                .map_or_default(|(_, url)| url.clone()),
            tab_ids: window.tabs.iter().map(|(id, _)| id.clone()).collect(),
            tab_urls: window.tabs.iter().map(|(_, url)| url.clone()).collect(),
            active: u32::try_from(window.active).unwrap_or(0),
            dialog,
            message,
            prompt,
        }
    }

    pub fn frame(&self, id: &str) -> FrameRef {
        self.0
            .lock()
            .expect("view windows")
            .get(id)
            .map_or_else(FrameRef::empty, |w| w.shown.frame())
    }
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use opsis_types::numbers::exact_eq;
    use proptest::prelude::*;
    use serde::Serialize;

    use super::unpacked;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Header {
        width: f64,
        height: f64,
        capture_id: String,
    }

    proptest! {
        #[test]
        fn a_packed_frame_unpacks_to_its_header_and_picture(
            width in 1u32..4000,
            height in 1u32..4000,
            capture in "[a-z0-9]{0,12}",
            jpeg in proptest::collection::vec(any::<u8>(), 0..256),
        ) {
            let header = Header { width: f64::from(width), height: f64::from(height), capture_id: capture };
            let frame = opsis_session::frames::packed(&header, &jpeg).unwrap();
            let (read, picture) = unpacked(&frame).unwrap();
            prop_assert!(exact_eq(read.width, f64::from(width)));
            prop_assert!(exact_eq(read.height, f64::from(height)));
            prop_assert_eq!(picture, jpeg.as_slice());
        }
    }
}
