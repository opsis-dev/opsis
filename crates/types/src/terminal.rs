use opsis_macros::choice;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalTheme {
    pub name: String,
    pub fg: String,
    pub bg: String,
    pub cursor: String,
    pub palette: [String; 16],
}

#[choice(export)]
pub enum ShellChoice {
    #[default]
    #[label("Clean")]
    Clean,
    #[label("My shell setup")]
    Own,
}

#[choice(export)]
pub enum WaitFor {
    #[default]
    #[label("The prompt")]
    Prompt,
    #[label("Quiet output")]
    Quiet,
}

#[choice(export)]
pub enum LookIn {
    #[default]
    #[label("The cursor's line")]
    Line,
    #[label("The whole screen")]
    Screen,
}

#[choice(export)]
pub enum PointerButton {
    #[default]
    #[label("Left")]
    Left,
    #[label("Right")]
    Right,
    #[label("Middle")]
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CellTarget {
    Cell { col: u16, row: u16 },
    Text { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TerminalAction {
    #[serde(rename_all = "camelCase")]
    Run {
        command: String,
        wait: WaitFor,
        limit_ms: u32,
        zoom_output: bool,
    },
    #[serde(rename_all = "camelCase")]
    Type {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        speed_ms: Option<u32>,
    },
    Keys {
        keys: String,
        repeat: u32,
    },
    Click {
        target: CellTarget,
        button: PointerButton,
        double: bool,
    },
    Drag {
        from: CellTarget,
        to: CellTarget,
    },
    Wheel {
        target: CellTarget,
        notches: i32,
    },
    #[serde(rename_all = "camelCase")]
    WaitText {
        pattern: String,
        scope: LookIn,
        limit_ms: u32,
    },
    #[serde(rename_all = "camelCase")]
    Sleep {
        ms: u32,
    },
    StartRecording,
    Hide,
    Show,
    Chapter {
        title: String,
    },
    Clear,
    Scroll {
        lines: i32,
    },
    Copy {
        text: String,
    },
    Paste,
    Env {
        name: String,
        value: String,
    },
    Require {
        program: String,
    },
    Screenshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalStep {
    pub id: String,
    pub node_id: String,
    pub label: String,
    pub action: TerminalAction,
    pub wait_after_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionValue {
    pub shell: ShellChoice,
    pub folder: String,
    pub cols: u16,
    pub rows: u16,
    pub env: Vec<EnvVar>,
    pub typing_ms: u32,
    pub pointer_ms: u32,
    pub steps: Vec<TerminalStep>,
}

#[choice(export)]
pub enum TerminalCursor {
    #[default]
    #[label("The program's")]
    Program,
    #[label("Block")]
    Block,
    #[label("Bar")]
    Bar,
    #[label("Underline")]
    Underline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLook {
    pub theme: TerminalTheme,
    pub font_size: u32,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub padding: u32,
    pub cursor: TerminalCursor,
    pub cursor_blink: bool,
}

impl TerminalLook {
    pub fn cell_width(&self) -> f64 {
        f64::from(self.font_size).mul_add(CELL_WIDTH_EM, self.letter_spacing)
    }

    pub fn cell_height(&self) -> f64 {
        (f64::from(self.font_size) * self.line_height).round()
    }
}

pub const CELL_WIDTH_EM: f64 = 0.6;
