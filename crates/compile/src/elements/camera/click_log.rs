use opsis_types::actions::{
    Action, ActionKind, ActionLog, ClickButton, CursorSample, Point, Rect, Viewport,
};

use crate::recording::{POINTER_KINDS, point_rect};

const DRAG_FIT_FRACTION: f64 = 0.5;
const DESIGN_WIDTH: f64 = 1920.0;

pub fn design_scale_for(width: f64) -> f64 {
    width / DESIGN_WIDTH
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClickEvent {
    pub label: Option<String>,
    pub t_ms: f64,
    pub t_depart_ms: Option<f64>,
    pub t_down_ms: Option<f64>,
    pub t_second_down_ms: Option<f64>,
    pub button: Option<ClickButton>,
    pub type_end_ms: Option<f64>,
    pub x: f64,
    pub y: f64,
    pub rect: Option<Rect>,
    pub cluster: Option<String>,
    pub zoom: Option<bool>,
    pub zoom_scale: Option<f64>,
    pub t_up_ms: Option<f64>,
    pub path: Option<Vec<Point>>,
    pub drop: Option<Rect>,
}

impl ClickEvent {
    pub const fn point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    pub t_ms: f64,
    pub chord: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClickLog {
    pub name: String,
    pub viewport: Viewport,
    pub cursor_track: Option<Vec<CursorSample>>,
    pub duration_ms: f64,
    pub clicks: Vec<ClickEvent>,
    pub keys: Vec<KeyEvent>,
}

fn carried(a: &Action) -> ClickEvent {
    ClickEvent {
        label: a.label.clone(),
        cluster: a.cluster.clone().filter(|c| !c.is_empty()),
        zoom: a.zoom,
        zoom_scale: a.zoom_scale,
        ..ClickEvent::default()
    }
}

pub fn rect_union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect {
        x,
        y,
        w: (a.x + a.w).max(b.x + b.w) - x,
        h: (a.y + a.h).max(b.y + b.h) - y,
    }
}

pub fn rect_center(r: Rect) -> Point {
    Point {
        x: r.x + r.w / 2.0,
        y: r.y + r.h / 2.0,
    }
}

fn drag_events(a: &Action, point: Point, vp: Viewport) -> Vec<ClickEvent> {
    let down = a.t_down.unwrap_or(a.t);
    let fallback = point_rect(point, design_scale_for(f64::from(vp.width)));
    let pick = a.rect.unwrap_or(fallback);
    let drop = a.drop.unwrap_or(fallback);
    let both = rect_union(pick, drop);
    let shared = ClickEvent {
        path: a.path.clone(),
        ..carried(a)
    };
    let press = ClickEvent {
        t_depart_ms: a.t_depart,
        t_down_ms: Some(down),
        t_up_ms: Some(a.t),
        ..shared.clone()
    };
    if both.w <= f64::from(vp.width) * DRAG_FIT_FRACTION
        && both.h <= f64::from(vp.height) * DRAG_FIT_FRACTION
    {
        let centre = rect_center(both);
        return vec![ClickEvent {
            t_ms: a.t,
            x: centre.x,
            y: centre.y,
            rect: Some(both),
            drop: Some(drop),
            ..press
        }];
    }
    let picked = rect_center(pick);
    vec![
        ClickEvent {
            t_ms: down,
            x: picked.x,
            y: picked.y,
            rect: Some(pick),
            ..press
        },
        ClickEvent {
            t_ms: a.t,
            t_depart_ms: Some(down),
            x: point.x,
            y: point.y,
            rect: Some(drop),
            drop: Some(drop),
            ..shared
        },
    ]
}

pub fn to_click_log(log: &ActionLog, name: &str, extra: &[Action], zoom_actions: bool) -> ClickLog {
    let mut actions: Vec<Action> = log
        .actions
        .iter()
        .map(|a| {
            if zoom_actions {
                a.clone()
            } else {
                Action {
                    zoom: Some(false),
                    ..a.clone()
                }
            }
        })
        .chain(extra.iter().cloned())
        .collect();
    actions.sort_by(|a, b| a.t.total_cmp(&b.t));
    let mut clicks = Vec::new();
    let mut keys = Vec::new();
    for a in &actions {
        if a.kind == ActionKind::Key
            && let Some(text) = a.text.as_ref().filter(|t| !t.is_empty())
        {
            keys.push(KeyEvent {
                t_ms: a.t,
                chord: text.clone(),
                label: a.label.clone(),
            });
            continue;
        }
        let Some(point) = a.point.filter(|_| POINTER_KINDS.contains(&a.kind)) else {
            continue;
        };
        if a.kind == ActionKind::Drag {
            clicks.extend(drag_events(a, point, log.viewport));
            continue;
        }
        clicks.push(ClickEvent {
            t_ms: a.t,
            x: point.x,
            y: point.y,
            t_depart_ms: a.t_depart,
            t_down_ms: a.t_down,
            t_second_down_ms: a.t_second_down,
            button: a.button,
            type_end_ms: (a.kind == ActionKind::Type).then_some(a.t_end).flatten(),
            rect: a.rect,
            ..carried(a)
        });
    }
    ClickLog {
        name: name.to_owned(),
        viewport: log.viewport,
        cursor_track: log.cursor_track.clone(),
        duration_ms: log.duration_ms,
        clicks,
        keys,
    }
}
