use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Point {
    pub fn distance(self, other: Point) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }

    pub fn lerp(self, to: Point, t: f64) -> Point {
        Point {
            x: self.x + (to.x - self.x) * t,
            y: self.y + (to.y - self.y) * t,
        }
    }
}

impl Rect {
    pub fn center(self) -> Point {
        Point {
            x: self.x + self.w / 2.0,
            y: self.y + self.h / 2.0,
        }
    }
}

impl From<Bounds> for Rect {
    fn from(b: Bounds) -> Self {
        Rect {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    Click,
    Type,
    Hover,
    Scroll,
    Navigate,
    Wait,
    Chapter,
    Key,
    Focus,
    Drag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub kind: ActionKind,
    pub t: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_end: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_depart: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_down: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point: Option<Point>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector_used: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zoom: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zoom_scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<Point>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop: Option<Rect>,
}

impl Action {
    pub fn new(id: impl Into<String>, kind: ActionKind, t: f64) -> Self {
        Self {
            id: id.into(),
            kind,
            t,
            t_end: None,
            t_depart: None,
            t_down: None,
            point: None,
            rect: None,
            cluster: None,
            label: None,
            selector_used: None,
            zoom: None,
            zoom_scale: None,
            text: None,
            node_id: None,
            path: None,
            drop: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct CursorSample {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionLog {
    pub viewport: Viewport,
    pub dpr: f64,
    pub t0: f64,
    pub duration_ms: f64,
    pub actions: Vec<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_track: Option<Vec<CursorSample>>,
}
