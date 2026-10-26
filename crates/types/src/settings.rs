use garde::Validate;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use specta::Type;

use crate::engine::{CheckedTool, EngineTool};

pub const PROJECTS_FOLDER_NAME: &str = "OpsisProjects";

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum CanvasMode {
    #[default]
    Mouse,
    Touchpad,
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
    #[garde(range(min = 1, max = 500))]
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
    #[garde(range(min = 1, max = 30))]
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
    #[garde(skip)]
    pub ffprobe: Option<CheckedTool>,
}

impl EngineSection {
    pub fn tool(&self, tool: EngineTool) -> Option<&CheckedTool> {
        match tool {
            EngineTool::Browser => self.browser.as_ref(),
            EngineTool::Ffmpeg => self.ffmpeg.as_ref(),
            EngineTool::Ffprobe => self.ffprobe.as_ref(),
        }
    }

    pub fn set(&mut self, tool: EngineTool, checked: Option<CheckedTool>) {
        match tool {
            EngineTool::Browser => self.browser = checked,
            EngineTool::Ffmpeg => self.ffmpeg = checked,
            EngineTool::Ffprobe => self.ffprobe = checked,
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
