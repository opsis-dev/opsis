use std::collections::BTreeSet;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::input::EventDragIntercepted;
use futures::StreamExt;
use opsis_types::actions::{MouseButton, Point};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::cdp::{CaptureError, Result, Tab};
use crate::keyboard::{
    KeyDescription, MODIFIERS, key_layout, keypad_location, mac_editing_commands, split_chord,
};
use crate::trace::CallInfo;

const SETUP_DRAG_LISTENERS: &str = r#"function setupDragListeners() {
  let didStartDrag = Promise.resolve(false);
  let dragEvent = null;
  const dragListener = (event) => dragEvent = event;
  const mouseListener = () => {
    didStartDrag = new Promise((callback) => {
      window.addEventListener('dragstart', dragListener, { once: true, capture: true });
      setTimeout(() => callback(dragEvent ? !dragEvent.defaultPrevented : false), 0);
    });
  };
  window.addEventListener('mousemove', mouseListener, { once: true, capture: true });
  window.__cleanupDrag = async () => {
    const val = await didStartDrag;
    window.removeEventListener('mousemove', mouseListener, { capture: true });
    window.removeEventListener('dragstart', dragListener, { capture: true });
    delete window.__cleanupDrag;
    return val;
  };
}"#;
const DRAG_INTERCEPT_WAIT: Duration = Duration::from_secs(5);

fn modifiers_mask(modifiers: &BTreeSet<String>) -> i64 {
    [("Alt", 1), ("Control", 2), ("Meta", 4), ("Shift", 8)]
        .iter()
        .filter(|(m, _)| modifiers.contains(*m))
        .map(|(_, bit)| bit)
        .sum()
}

fn resolve_smart_modifier(key: &str) -> &str {
    match key {
        "ControlOrMeta" if cfg!(target_os = "macos") => "Meta",
        "ControlOrMeta" => "Control",
        other => other,
    }
}

#[derive(Debug, Default)]
struct KeyboardState {
    modifiers: BTreeSet<String>,
    pressed: BTreeSet<String>,
}

#[derive(Debug, Default)]
struct MouseState {
    x: f64,
    y: f64,
    left_down: bool,
    drag: Option<Value>,
}

#[derive(Debug, Default)]
pub struct Input {
    keyboard: Mutex<KeyboardState>,
    mouse: Mutex<MouseState>,
}

impl Input {
    async fn description(&self, key: &str) -> Result<KeyDescription> {
        let key = resolve_smart_modifier(key);
        let base = key_layout()
            .get(key)
            .cloned()
            .ok_or_else(|| CaptureError::failed(format!("Unknown key: \"{key}\"")))?;
        let state = self.keyboard.lock().await;
        let description = match (&base.shifted, state.modifiers.contains("Shift")) {
            (Some(shifted), true) => (**shifted).clone(),
            _ => base,
        };
        let only_shift = state.modifiers.len() == 1 && state.modifiers.contains("Shift");
        if state.modifiers.len() > 1 || (!state.modifiers.is_empty() && !only_shift) {
            return Ok(KeyDescription {
                text: String::new(),
                ..description
            });
        }
        Ok(description)
    }

    fn commands(code: &str, modifiers: &BTreeSet<String>) -> Vec<String> {
        if !cfg!(target_os = "macos") {
            return Vec::new();
        }
        let mut parts: Vec<&str> = ["Shift", "Control", "Alt", "Meta"]
            .into_iter()
            .filter(|m| modifiers.contains(*m))
            .collect();
        parts.push(code);
        mac_editing_commands(&parts.join("+"))
    }

    pub async fn key_down(&self, tab: &Tab, key: &str) -> Result<()> {
        let description = self.description(key).await?;
        let (mask, commands, auto_repeat) = {
            let mut state = self.keyboard.lock().await;
            let auto_repeat = !state.pressed.insert(description.code.clone());
            if MODIFIERS.contains(&description.key.as_str()) {
                state.modifiers.insert(description.key.clone());
            }
            (
                modifiers_mask(&state.modifiers),
                Self::commands(&description.code, &state.modifiers),
                auto_repeat,
            )
        };
        if description.code == "Escape" && self.cancel_drag(tab).await? {
            return Ok(());
        }
        let has_text = !description.text.is_empty();
        let mut event = json!({
            "type": if has_text { "keyDown" } else { "rawKeyDown" },
            "modifiers": mask,
            "windowsVirtualKeyCode": description.key_code_without_location,
            "code": description.code,
            "commands": commands,
            "key": description.key,
            "autoRepeat": auto_repeat,
            "location": description.location,
            "isKeypad": description.location == keypad_location(),
        });
        if has_text {
            event["text"] = json!(description.text);
            event["unmodifiedText"] = json!(description.text);
        }
        tab.send("Input.dispatchKeyEvent", event).await?;
        Ok(())
    }

    pub async fn key_up(&self, tab: &Tab, key: &str) -> Result<()> {
        let description = self.description(key).await?;
        let mask = {
            let mut state = self.keyboard.lock().await;
            if MODIFIERS.contains(&description.key.as_str()) {
                state.modifiers.remove(&description.key);
            }
            state.pressed.remove(&description.code);
            modifiers_mask(&state.modifiers)
        };
        tab.send(
            "Input.dispatchKeyEvent",
            json!({
                "type": "keyUp",
                "modifiers": mask,
                "key": description.key,
                "windowsVirtualKeyCode": description.key_code_without_location,
                "code": description.code,
                "location": description.location,
                "isKeypad": description.location == keypad_location(),
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn press(&self, tab: &Tab, chord: &str, delay: Option<Duration>) -> Result<()> {
        tab.traced(
            || CallInfo::new("keyboardPress", format!("Press {chord}")),
            self.pressed(tab, chord, delay),
        )
        .await
    }

    async fn pressed(&self, tab: &Tab, chord: &str, delay: Option<Duration>) -> Result<()> {
        let tokens = split_chord(chord);
        let (key, held) = tokens
            .split_last()
            .ok_or_else(|| CaptureError::failed("Nothing to press."))?;
        for token in held {
            self.key_down(tab, token).await?;
        }
        self.key_down(tab, key).await?;
        if let Some(delay) = delay {
            tab.clock().wait(delay).await;
        }
        self.key_up(tab, key).await?;
        for token in held.iter().rev() {
            self.key_up(tab, token).await?;
        }
        Ok(())
    }

    pub async fn type_text(
        &self,
        tab: &Tab,
        text: &str,
        delay: Option<Duration>,
        keyed: &(dyn Fn() + Send + Sync),
    ) -> Result<()> {
        tab.traced(
            || CallInfo::new("keyboardType", format!("Type \"{text}\"")),
            self.typed(tab, text, delay, keyed),
        )
        .await
    }

    async fn typed(
        &self,
        tab: &Tab,
        text: &str,
        delay: Option<Duration>,
        keyed: &(dyn Fn() + Send + Sync),
    ) -> Result<()> {
        for c in text.chars() {
            keyed();
            let key = c.to_string();
            if key_layout().contains_key(&key) {
                self.pressed(tab, &key, delay).await?;
            } else {
                if let Some(delay) = delay {
                    tab.clock().wait(delay).await;
                }
                tab.send("Input.insertText", json!({ "text": key })).await?;
            }
        }
        Ok(())
    }

    async fn modifiers(&self) -> i64 {
        modifiers_mask(&self.keyboard.lock().await.modifiers)
    }

    pub async fn position(&self) -> Point {
        let mouse = self.mouse.lock().await;
        Point {
            x: mouse.x,
            y: mouse.y,
        }
    }

    pub async fn mouse_move(&self, tab: &Tab, x: f64, y: f64, steps: u32) -> Result<()> {
        tab.traced(
            || CallInfo::new("mouseMove", "Move mouse").at(Point { x, y }),
            self.moved(tab, x, y, steps),
        )
        .await
    }

    async fn moved(&self, tab: &Tab, x: f64, y: f64, steps: u32) -> Result<()> {
        let (from_x, from_y) = {
            let mut mouse = self.mouse.lock().await;
            let from = (mouse.x, mouse.y);
            (mouse.x, mouse.y) = (x, y);
            from
        };
        let steps = steps.max(1);
        for i in 1..=steps {
            let t = f64::from(i) / f64::from(steps);
            self.raw_move(tab, from_x + (x - from_x) * t, from_y + (y - from_y) * t)
                .await?;
        }
        Ok(())
    }

    async fn raw_move(&self, tab: &Tab, x: f64, y: f64) -> Result<()> {
        let modifiers = self.modifiers().await;
        let (left_down, drag) = {
            let mouse = self.mouse.lock().await;
            (mouse.left_down, mouse.drag.clone())
        };
        if let Some(data) = drag {
            tab.send(
                "Input.dispatchDragEvent",
                json!({ "type": "dragOver", "x": x, "y": y, "data": data, "modifiers": modifiers }),
            )
            .await?;
            return Ok(());
        }
        let moved = json!({ "type": "mouseMoved", "button": if left_down { "left" } else { "none" }, "buttons": if left_down { 1 } else { 0 }, "x": x, "y": y, "modifiers": modifiers, "force": if left_down { 0.5 } else { 0.0 } });
        if !left_down {
            tab.send("Input.dispatchMouseEvent", moved).await?;
            return Ok(());
        }
        self.move_watching_drags(tab, x, y, moved, modifiers).await
    }

    async fn move_watching_drags(
        &self,
        tab: &Tab,
        x: f64,
        y: f64,
        moved: Value,
        modifiers: i64,
    ) -> Result<()> {
        let mut intercepted = tab.page.event_listener::<EventDragIntercepted>().await?;
        tab.injected_value::<Value>(&format!("() => ({SETUP_DRAG_LISTENERS})()"), &[])
            .await?;
        tab.send("Input.setInterceptDrags", json!({ "enabled": true }))
            .await?;
        let moved_result = tab.send("Input.dispatchMouseEvent", moved).await;
        let expecting = match moved_result {
            Ok(_) => {
                tab.injected_value::<bool>(
                    "() => window.__cleanupDrag ? window.__cleanupDrag() : false",
                    &[],
                )
                .await
            }
            Err(error) => Err(error),
        };
        let disabled = tab
            .send("Input.setInterceptDrags", json!({ "enabled": false }))
            .await;
        let expecting = expecting?;
        disabled?;
        if !expecting {
            return Ok(());
        }
        let event = tokio::time::timeout(DRAG_INTERCEPT_WAIT, intercepted.next())
            .await
            .map_err(|_| {
                CaptureError::failed("The page started a drag but never said what it drags.")
            })?;
        let Some(event) = event else {
            return Ok(());
        };
        let data =
            serde_json::to_value(&event.data).map_err(|e| CaptureError::Cdp(e.to_string()))?;
        self.mouse.lock().await.drag = Some(data.clone());
        tab.send(
            "Input.dispatchDragEvent",
            json!({ "type": "dragEnter", "x": x, "y": y, "data": data, "modifiers": modifiers }),
        )
        .await?;
        Ok(())
    }

    async fn cancel_drag(&self, tab: &Tab) -> Result<bool> {
        let (x, y, dragging) = {
            let mut mouse = self.mouse.lock().await;
            let dragging = mouse.drag.take().is_some();
            (mouse.x, mouse.y, dragging)
        };
        if dragging {
            tab.send("Input.dispatchDragEvent", json!({ "type": "dragCancel", "x": x, "y": y, "data": { "items": [], "dragOperationsMask": 65535 } })).await?;
        }
        Ok(dragging)
    }

    pub async fn mouse_down(&self, tab: &Tab, button: MouseButton, count: u32) -> Result<()> {
        let at = self.position().await;
        tab.traced(
            || CallInfo::new("mouseDown", "Mouse down").at(at),
            self.down(tab, button, count),
        )
        .await
    }

    async fn down(&self, tab: &Tab, button: MouseButton, count: u32) -> Result<()> {
        let left = button == MouseButton::Left;
        let (x, y, dragging) = {
            let mut mouse = self.mouse.lock().await;
            mouse.left_down |= left;
            (mouse.x, mouse.y, left && mouse.drag.is_some())
        };
        if dragging {
            return Ok(());
        }
        let modifiers = self.modifiers().await;
        tab.send("Input.dispatchMouseEvent", json!({ "type": "mousePressed", "button": button, "buttons": button.mask(), "x": x, "y": y, "modifiers": modifiers, "clickCount": count, "force": 0.5 })).await?;
        Ok(())
    }

    pub async fn mouse_up(&self, tab: &Tab, button: MouseButton, count: u32) -> Result<()> {
        let at = self.position().await;
        tab.traced(
            || CallInfo::new("mouseUp", "Mouse up").at(at),
            self.up(tab, button, count),
        )
        .await
    }

    async fn up(&self, tab: &Tab, button: MouseButton, count: u32) -> Result<()> {
        let left = button == MouseButton::Left;
        let (x, y, drag) = {
            let mut mouse = self.mouse.lock().await;
            if left {
                mouse.left_down = false;
            }
            (
                mouse.x,
                mouse.y,
                if left { mouse.drag.take() } else { None },
            )
        };
        let modifiers = self.modifiers().await;
        if let Some(data) = drag {
            tab.send(
                "Input.dispatchDragEvent",
                json!({ "type": "drop", "x": x, "y": y, "data": data, "modifiers": modifiers }),
            )
            .await?;
            return Ok(());
        }
        tab.send("Input.dispatchMouseEvent", json!({ "type": "mouseReleased", "button": button, "buttons": 0, "x": x, "y": y, "modifiers": modifiers, "clickCount": count })).await?;
        Ok(())
    }

    pub async fn wheel(&self, tab: &Tab, delta_x: f64, delta_y: f64) -> Result<()> {
        let at = self.position().await;
        tab.traced(
            || CallInfo::new("mouseWheel", "Scroll wheel").at(at),
            self.wheeled(tab, at, delta_x, delta_y),
        )
        .await
    }

    async fn wheeled(
        &self,
        tab: &Tab,
        Point { x, y }: Point,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<()> {
        let modifiers = self.modifiers().await;
        tab.send("Input.dispatchMouseEvent", json!({ "type": "mouseWheel", "x": x, "y": y, "modifiers": modifiers, "deltaX": delta_x, "deltaY": delta_y })).await?;
        Ok(())
    }
}
