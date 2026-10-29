use garde::Validate;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use specta::Type;

use crate::engine::{CheckedTool, EngineTool};

pub const PROJECTS_FOLDER_NAME: &str = "OpsisProjects";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WholeRange {
    pub min: u32,
    pub max: u32,
}

pub const KEEP_LAST_TRACES: WholeRange = WholeRange { min: 1, max: 500 };

pub const LIVE_FPS: WholeRange = WholeRange { min: 1, max: 30 };

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum StartOption {
    #[default]
    StartScreen,
    LastProject,
    NewProject,
}

impl StartOption {
    pub const ALL: [Self; 3] = [Self::StartScreen, Self::LastProject, Self::NewProject];

    pub fn label(self) -> &'static str {
        match self {
            Self::StartScreen => "Show the start screen",
            Self::LastProject => "Open the last project",
            Self::NewProject => "Start a new project",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum TraceRetention {
    #[default]
    Always,
    Failures,
    LastN,
    Off,
}

impl TraceRetention {
    pub const ALL: [Self; 4] = [Self::Always, Self::Failures, Self::LastN, Self::Off];

    pub fn label(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::Failures => "Failures only",
            Self::LastN => "Last N",
            Self::Off => "Off",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum CanvasMode {
    #[default]
    Mouse,
    Touchpad,
}

impl CanvasMode {
    pub const ALL: [Self; 2] = [Self::Mouse, Self::Touchpad];
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    SmartDefault,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSection {
    #[garde(skip)]
    pub on_start: StartOption,
    #[default = true]
    #[garde(skip)]
    pub confirm_exit: bool,
    #[garde(skip)]
    pub setup_skipped: bool,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    SmartDefault,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct TracesSection {
    #[garde(skip)]
    pub retention: TraceRetention,
    #[default = 10]
    #[garde(range(min = KEEP_LAST_TRACES.min, max = KEEP_LAST_TRACES.max))]
    pub keep_last: u32,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    SmartDefault,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LiveSection {
    #[default = 12]
    #[garde(range(min = LIVE_FPS.min, max = LIVE_FPS.max))]
    pub fps: u32,
    #[default = true]
    #[garde(skip)]
    pub auto_open: bool,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderSection {
    #[garde(length(max = 200))]
    pub license_key: String,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectsSection {
    #[garde(skip)]
    pub default_dir: String,
    #[garde(skip)]
    pub last_dir: String,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct EditorSection {
    #[garde(skip)]
    pub canvas_mode: CanvasMode,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSection {
    #[garde(skip)]
    pub browser: Option<CheckedTool>,
    #[garde(skip)]
    pub ffmpeg: Option<CheckedTool>,
}

impl EngineSection {
    pub fn tool(&self, tool: EngineTool) -> Option<&CheckedTool> {
        match tool {
            EngineTool::Browser => self.browser.as_ref(),
            EngineTool::Ffmpeg => self.ffmpeg.as_ref(),
        }
    }

    pub fn set(&mut self, tool: EngineTool, checked: Option<CheckedTool>) {
        match tool {
            EngineTool::Browser => self.browser = checked,
            EngineTool::Ffmpeg => self.ffmpeg = checked,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiSection {
    #[garde(skip)]
    pub enabled: bool,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, Validate, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    #[garde(dive)]
    pub app: AppSection,
    #[garde(dive)]
    pub traces: TracesSection,
    #[garde(dive)]
    pub live: LiveSection,
    #[garde(dive)]
    pub render: RenderSection,
    #[garde(dive)]
    pub projects: ProjectsSection,
    #[garde(dive)]
    pub editor: EditorSection,
    #[garde(dive)]
    pub api: ApiSection,
    #[garde(dive)]
    pub engine: EngineSection,
}

pub const SETTINGS_SECTIONS: [&str; 8] = [
    "app", "traces", "live", "render", "projects", "editor", "api", "engine",
];

pub const RUST_ONLY_SECTIONS: [&str; 1] = ["engine"];

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBook {
    pub defaults: AppSettings,
    pub start_options: Vec<(StartOption, String)>,
    pub trace_retention: Vec<(TraceRetention, String)>,
    pub canvas_modes: Vec<CanvasMode>,
    pub keep_last_traces: WholeRange,
    pub live_fps: WholeRange,
}

pub fn settings_book() -> SettingsBook {
    SettingsBook {
        defaults: AppSettings::default(),
        start_options: StartOption::ALL.map(|o| (o, o.label().to_owned())).to_vec(),
        trace_retention: TraceRetention::ALL
            .map(|r| (r, r.label().to_owned()))
            .to_vec(),
        canvas_modes: CanvasMode::ALL.to_vec(),
        keep_last_traces: KEEP_LAST_TRACES,
        live_fps: LIVE_FPS,
    }
}
