use garde::Validate;
use opsis_macros::choice;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use specta::Type;

use crate::engine::CheckedTool;

pub const PROJECTS_FOLDER_NAME: &str = "OpsisProjects";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WholeRange {
    pub min: u32,
    pub max: u32,
}

pub const KEEP_LAST_TRACES: WholeRange = WholeRange { min: 1, max: 500 };

pub const LIVE_FPS: WholeRange = WholeRange { min: 1, max: 30 };
pub const SCRIPT_FONT_SIZE: WholeRange = WholeRange { min: 10, max: 24 };
pub const DEFAULT_SCRIPT_THEME: &str = "default";

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

    pub const fn label(self) -> &'static str {
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

    pub const fn label(self) -> &'static str {
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
    Eq,
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
    Eq,
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
    Eq,
    SmartDefault,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptSection {
    #[default(_code = "DEFAULT_SCRIPT_THEME.to_owned()")]
    #[garde(length(min = 1))]
    pub theme: String,
    #[default = 12]
    #[garde(range(min = SCRIPT_FONT_SIZE.min, max = SCRIPT_FONT_SIZE.max))]
    pub font_size: u32,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
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
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectsSection {
    #[garde(skip)]
    pub default_dir: String,
    #[garde(skip)]
    pub last_dir: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct EditorSection {
    #[garde(skip)]
    pub canvas_mode: CanvasMode,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSection {
    #[garde(skip)]
    pub browser: Option<CheckedTool>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AssistantSection {
    #[garde(skip)]
    pub enabled: bool,
    #[garde(dive)]
    pub agents: Vec<OwnAgent>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct OwnAgent {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub command: String,
    #[garde(skip)]
    pub args: Vec<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdatesSection {
    #[garde(skip)]
    pub check_on_start: bool,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
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
    pub projects: ProjectsSection,
    #[garde(dive)]
    pub editor: EditorSection,
    #[garde(dive)]
    pub assistant: AssistantSection,
    #[garde(dive)]
    pub engine: EngineSection,
    #[garde(dive)]
    pub media: MediaSection,
    #[garde(dive)]
    pub updates: UpdatesSection,
    #[garde(dive)]
    pub script: ScriptSection,
}

#[choice(export)]
pub enum EncodePreset {
    #[label("Fastest, biggest")]
    Ultrafast,
    #[label("Fast")]
    Fast,
    #[default]
    #[label("Balanced")]
    Medium,
    #[label("Small")]
    Slow,
    #[label("Smallest, slowest")]
    Veryslow,
}

impl EncodePreset {
    pub const ALL: [Self; 5] = [
        Self::Ultrafast,
        Self::Fast,
        Self::Medium,
        Self::Slow,
        Self::Veryslow,
    ];
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Validate,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaSection {
    #[garde(skip)]
    pub encode_speed: EncodePreset,
    #[garde(range(min = 1))]
    pub export_threads: Option<u32>,
}

pub fn computer_threads() -> u32 {
    std::thread::available_parallelism().map_or(1, |n| crate::numbers::count(n.get()))
}

pub fn default_export_threads(threads: u32) -> u32 {
    threads.max(1).div_ceil(2)
}

pub fn export_threads(chosen: Option<u32>, threads: u32) -> u32 {
    let threads = threads.max(1);
    chosen.map_or_else(|| default_export_threads(threads), |n| n.clamp(1, threads))
}

pub const SETTINGS_SECTIONS: [&str; 10] = [
    "app",
    "traces",
    "live",
    "projects",
    "editor",
    "assistant",
    "engine",
    "media",
    "updates",
    "script",
];

pub const RUST_ONLY_SECTIONS: [&str; 1] = ["engine"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBook {
    pub defaults: AppSettings,
    pub start_options: Vec<(StartOption, String)>,
    pub trace_retention: Vec<(TraceRetention, String)>,
    pub canvas_modes: Vec<CanvasMode>,
    pub keep_last_traces: WholeRange,
    pub live_fps: WholeRange,
    pub script_font_size: WholeRange,
    pub encode_speeds: Vec<(EncodePreset, String)>,
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
        script_font_size: SCRIPT_FONT_SIZE,
        encode_speeds: EncodePreset::ALL
            .map(|p| (p, p.label().to_owned()))
            .to_vec(),
    }
}
