use serde::{Deserialize, Serialize};
use specta::Type;

use crate::actions::{Rect, Viewport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum LiveStepStatus {
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LiveStep {
    pub node_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LiveEvent {
    Start {
        title: String,
        viewport: Viewport,
        steps: Vec<LiveStep>,
    },
    Step {
        node_id: String,
        status: LiveStepStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Cursor {
        x: f64,
        y: f64,
    },
    Target {
        rect: Rect,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
    },
    Click {
        x: f64,
        y: f64,
    },
    End {
        ok: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}
