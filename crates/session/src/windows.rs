use opsis_store::windows::Bounds;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::Session;
use crate::state::AppState;

pub const MAIN_WINDOW_KEY: &str = "mainWindow";
const MIN_VISIBLE_PX: i32 = 40;
const ABOVE_WORK_AREA_PX: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedWindow {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

impl SavedWindow {
    pub const fn bounds(&self) -> Bounds {
        Bounds {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}

pub fn visible_bounds(bounds: Bounds, areas: &[WorkArea]) -> Option<Bounds> {
    let fits = areas.iter().any(|area| {
        let across = bounds.x < area.x + area.width - MIN_VISIBLE_PX
            && bounds.x + bounds.width > area.x + MIN_VISIBLE_PX;
        let reachable = bounds.y >= area.y - ABOVE_WORK_AREA_PX
            && bounds.y < area.y + area.height - MIN_VISIBLE_PX;
        across && reachable
    });
    fits.then_some(bounds)
}

pub fn saved_window(view: &Map<String, Value>) -> Option<SavedWindow> {
    view.get(MAIN_WINDOW_KEY)
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

pub async fn save_main_window(app: &Session, saved: SavedWindow) {
    let state = app.state::<AppState>();
    let Some(dir) = state.editor().dir() else {
        return;
    };
    let value = match serde_json::to_value(saved) {
        Ok(value) => value,
        Err(error) => {
            log::warn!("writing the window's size: {error}");
            return;
        }
    };
    if let Err(error) = state
        .db()
        .write_view_state(&dir.to_string_lossy(), MAIN_WINDOW_KEY, &value)
        .await
    {
        log::warn!("saving the window's size: {error}");
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn area() -> impl Strategy<Value = WorkArea> {
        (-4000..4000i32, -2000..2000i32, 640..4000i32, 480..2400i32).prop_map(
            |(x, y, width, height)| WorkArea {
                x,
                y,
                width,
                height,
            },
        )
    }

    proptest! {
        #[test]
        fn a_window_inside_a_work_area_stays(area in area(), inset in 0..200i32) {
            let bounds = Bounds {
                x: area.x + inset,
                y: area.y + inset,
                width: area.width - 2 * inset,
                height: area.height - 2 * inset,
            };
            prop_assert_eq!(visible_bounds(bounds, &[area]), Some(bounds));
        }

        #[test]
        fn a_window_right_of_every_area_is_dropped(
            areas in proptest::collection::vec(area(), 1..4),
            width in 100..2000i32,
            height in 100..2000i32,
        ) {
            let right = areas.iter().map(|a| a.x + a.width).max().unwrap_or_default();
            let top = areas.iter().map(|a| a.y).min().unwrap_or_default();
            let bounds = Bounds { x: right, y: top, width, height };
            prop_assert_eq!(visible_bounds(bounds, &areas), None);
        }
    }
}
