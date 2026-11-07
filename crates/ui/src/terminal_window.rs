use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opsis_qt::{FrameRef, TerminalWindowView};
use opsis_types::terminal::TerminalInput;
use serde::Deserialize;

use crate::keys::DomKey;
use crate::streams::{FrameSlot, Shown, next_target, unpacked};

const MODIFIER_KEYS: [&str; 6] = ["Shift", "Control", "Alt", "Meta", "AltGraph", "CapsLock"];

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub skipped: u32,
    pub width: u32,
    pub height: u32,
    pub cols: u16,
    pub rows: u16,
    pub cell_width: f64,
    pub cell_height: f64,
    pub padding: u32,
}

pub fn cell_at(x: f64, y: f64, box_width: f64, box_height: f64, grid: &Grid) -> (u16, u16) {
    let width = f64::from(grid.width.max(1));
    let height = f64::from(grid.height.max(1));
    let scale = (box_width / width).min(box_height / height);
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let left = scale.mul_add(-width, box_width) / 2.0;
    let top = scale.mul_add(-height, box_height) / 2.0;
    let padding = f64::from(grid.padding);
    let inner_x = (x - left) / scale - padding;
    let inner_y = (y - top) / scale - padding;
    let at = |offset: f64, cell: f64, count: u16| -> u16 {
        let last = f64::from(count.saturating_sub(1));
        let cell = if cell > 0.0 { cell } else { 1.0 };
        let index = (offset / cell).floor();
        let index = if index.is_finite() {
            index.clamp(0.0, last)
        } else {
            0.0
        };
        opsis_types::numbers::truncate::<u16>(index)
    };
    (
        at(inner_x, grid.cell_width, grid.cols),
        at(inner_y, grid.cell_height, grid.rows),
    )
}

pub fn key_input(key: &DomKey, text: &str, modifiers: i64) -> Option<TerminalInput> {
    let (alt, ctrl, meta, shift) = (
        modifiers & 1 != 0,
        modifiers & 2 != 0,
        modifiers & 4 != 0,
        modifiers & 8 != 0,
    );
    if MODIFIER_KEYS.contains(&key.key.as_str()) || meta {
        return None;
    }
    let typed = text.chars().filter(|c| !c.is_control()).collect::<String>();
    if key.key.chars().count() == 1 && !ctrl && !alt {
        let text = if typed.is_empty() {
            key.key.clone()
        } else {
            typed
        };
        return Some(TerminalInput::Text { text });
    }
    Some(TerminalInput::Key {
        key: key.key.clone(),
        ctrl,
        alt,
        shift,
    })
}

#[derive(Debug, Clone)]
struct Window {
    target: u64,
    grid: Grid,
    shown: Shown,
}

#[derive(Debug, Default, Clone)]
pub struct TerminalWindows(Arc<Mutex<HashMap<String, Window>>>);

impl TerminalWindows {
    pub fn slot(&self, id: &str) -> (u64, FrameSlot) {
        let target = next_target();
        self.0.lock().expect("terminal windows").insert(
            id.to_owned(),
            Window {
                target,
                grid: Grid::default(),
                shown: Shown::default(),
            },
        );
        let windows = self.clone();
        let named = id.to_owned();
        let raw = FrameSlot::raw(Box::new(move |frame: &[u8]| {
            let Some((grid, jpeg)) = unpacked::<Grid>(frame) else {
                return;
            };
            let mut open = windows.0.lock().expect("terminal windows");
            let Some(window) = open.get_mut(&named) else {
                return;
            };
            window.grid = grid;
            window.shown = Shown {
                serial: window.shown.serial + 1,
                width: grid.width,
                height: grid.height,
                jpeg: Arc::new(jpeg.to_vec()),
            };
            let target = window.target;
            drop(open);
            opsis_qt::wake(target);
        }));
        (target, raw)
    }

    pub fn forget(&self, id: &str) {
        self.0.lock().expect("terminal windows").remove(id);
    }

    pub fn grid(&self, id: &str) -> Grid {
        self.0
            .lock()
            .expect("terminal windows")
            .get(id)
            .map_or_default(|w| w.grid)
    }

    pub fn view(&self, id: &str) -> TerminalWindowView {
        let open = self.0.lock().expect("terminal windows");
        let (serial, skipped) = open
            .get(id)
            .map_or((0, 0), |w| (w.shown.serial, w.grid.skipped));
        TerminalWindowView {
            frame: serial,
            status: status_text(skipped),
        }
    }

    pub fn frame(&self, id: &str) -> FrameRef {
        self.0
            .lock()
            .expect("terminal windows")
            .get(id)
            .map_or_else(FrameRef::empty, |w| w.shown.frame())
    }
}

pub fn status_text(skipped: u32) -> String {
    match skipped {
        0 => "Recording what you type. Pauses become waits in the tape.".to_owned(),
        1 => "Recording what you type. 1 key can't be saved in a tape file.".to_owned(),
        n => format!("Recording what you type. {n} keys can't be saved in a tape file."),
    }
}

#[cfg(test)]
mod tests {
    use opsis_types::terminal::TerminalInput;
    use proptest::prelude::*;

    use super::{Grid, cell_at, key_input};
    use crate::keys::dom_key;

    fn grid(cols: u16, rows: u16) -> Grid {
        Grid {
            skipped: 0,
            width: u32::from(cols) * 10 + 16,
            height: u32::from(rows) * 20 + 16,
            cols,
            rows,
            cell_width: 10.0,
            cell_height: 20.0,
            padding: 8,
        }
    }

    proptest! {
        #[test]
        fn a_point_always_lands_on_a_cell(
            cols in 1u16..300,
            rows in 1u16..120,
            x in -500f64..5000.0,
            y in -500f64..5000.0,
            box_width in 1f64..4000.0,
            box_height in 1f64..4000.0,
        ) {
            let (col, row) = cell_at(x, y, box_width, box_height, &grid(cols, rows));
            prop_assert!(col < cols);
            prop_assert!(row < rows);
        }

        #[test]
        fn a_cell_centre_maps_back_to_its_cell(cols in 1u16..200, rows in 1u16..80, col in 0u16..200, row in 0u16..80) {
            let col = col % cols;
            let row = row % rows;
            let g = grid(cols, rows);
            let x = f64::from(col).mul_add(10.0, 13.0);
            let y = f64::from(row).mul_add(20.0, 18.0);
            let (w, h) = (f64::from(g.width), f64::from(g.height));
            prop_assert_eq!(cell_at(x, y, w, h, &g), (col, row));
        }

        #[test]
        fn a_plain_letter_types_and_a_held_one_sends_a_key(letter in 0u32..26, ctrl in any::<bool>(), alt in any::<bool>()) {
            let upper = char::from_u32(u32::from(b'A') + letter).unwrap();
            let text = upper.to_ascii_lowercase().to_string();
            let key = dom_key(u32::from(upper), &text, false);
            let input = key_input(&key, &text, i64::from(ctrl) * 2 + i64::from(alt));
            if ctrl || alt {
                let is_key = matches!(input, Some(TerminalInput::Key { .. }));
                prop_assert!(is_key);
            } else {
                prop_assert_eq!(input, Some(TerminalInput::Text { text }));
            }
        }
    }
}
