use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalLineKind {
    Command,
    Log,
    Success,
    Error,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulatorLine {
    #[setting("Text", default = "Done")]
    pub text: String,
    #[setting("Kind", default = TerminalLineKind::Log)]
    pub r#type: TerminalLineKind,
    #[setting(
        "Wait before",
        default = 0.2667,
        range(min = 0, max = 20),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub delay: f64,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulator {
    #[setting("Prompt", default = "$")]
    pub prompt: String,
    #[setting("Title", default = "~/projects/acme")]
    pub title: String,
    #[setting("Background", default = "#0a0a0a", check = colour)]
    pub background: String,
    #[setting("Title bar colour", default = "#1a1a1a", check = colour)]
    pub chrome_color: String,
    #[setting("Font size", default = 18, range(min = 10, max = 32), step = 1)]
    pub font_size: u32,
    #[setting(
        "Chars / frame",
        default = 1.0,
        range(min = 0.25, max = 6),
        step = 0.25
    )]
    pub chars_per_frame: f64,
    #[setting("Chunk size", default = 1, range(min = 1, max = 20), step = 1)]
    pub chunk_size: u32,
    #[setting(
        "Lines",
        default = vec![
            TerminalSimulatorLine {
                text: "deploy --production".into(),
                r#type: TerminalLineKind::Command,
                delay: 0.0,
            },
            TerminalSimulatorLine { text: "Running tests...".into(), delay: 0.2, ..Default::default() },
            TerminalSimulatorLine {
                text: "128 tests passed".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.3333,
            },
            TerminalSimulatorLine { text: "Building the image...".into(), ..Default::default() },
            TerminalSimulatorLine {
                text: "Image ready in 41s".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.3333,
            },
            TerminalSimulatorLine { text: "Rolling out to everyone...".into(), ..Default::default() },
            TerminalSimulatorLine {
                text: "Live in production".into(),
                r#type: TerminalLineKind::Success,
                delay: 0.4,
            },
        ],
        help = "Lines ending in ... pause before the next one."
    )]
    pub lines: Vec<TerminalSimulatorLine>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TerminalSimulator>(
        element(
            ElementKind::Component,
            "terminal-simulator",
            "Terminal simulator",
            "Terminal window that types out commands and rolls old lines off the top.",
            ElementCategory::Ui,
            240,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:square-terminal")),
    )
}
