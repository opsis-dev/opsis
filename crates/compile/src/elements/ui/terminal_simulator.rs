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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulatorLine {
    #[default = "Done"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default(TerminalLineKind::Log)]
    #[schemars(description = "Kind")]
    pub r#type: TerminalLineKind,
    #[default = 0.2667]
    #[schemars(
        description = "Wait before",
        range(min = 0, max = 20),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub delay: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulator {
    #[default = "$"]
    #[schemars(description = "Prompt")]
    pub prompt: String,
    #[default = "~/projects/acme"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "#0a0a0a"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
    #[default = "#1a1a1a"]
    #[schemars(
        description = "Title bar colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub chrome_color: String,
    #[default = 18]
    #[schemars(description = "Font size", range(min = 10, max = 32), extend("x-step" = 1))]
    pub font_size: u32,
    #[default = 1.0]
    #[schemars(description = "Chars / frame", range(min = 0.25, max = 6), extend("x-step" = 0.25))]
    pub chars_per_frame: f64,
    #[default = 1]
    #[schemars(description = "Chunk size", range(min = 1, max = 20), extend("x-step" = 1))]
    pub chunk_size: u32,
    #[default(vec![
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
    ])]
    #[schemars(
        description = "Lines",
        extend("x-help" = "Lines ending in ... pause before the next one.")
    )]
    pub lines: Vec<TerminalSimulatorLine>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
