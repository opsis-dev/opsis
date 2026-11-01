use opsis_macros::choice;
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
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
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
    pub fn distance(self, other: Self) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }

    #[must_use]
    pub fn lerp(self, to: Self, t: f64) -> Self {
        Self {
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
        Self {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
        }
    }
}

#[choice(export)]
pub enum ClickButton {
    #[default]
    #[label("Left")]
    Left,
    #[label("Right")]
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    None,
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

impl MouseButton {
    pub const fn mask(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Middle => 4,
            Self::Back => 8,
            Self::Forward => 16,
        }
    }
}

impl From<ClickButton> for MouseButton {
    fn from(button: ClickButton) -> Self {
        match button {
            ClickButton::Left => Self::Left,
            ClickButton::Right => Self::Right,
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
    pub t_second_down: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<ClickButton>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_times: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
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
            t_second_down: None,
            button: None,
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
            key_times: None,
            silent: None,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<PageAddress>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct PageAddress {
    pub t: f64,
    pub url: String,
}
