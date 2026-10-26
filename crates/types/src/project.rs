use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::edits::DocumentState;
use crate::values::LastRun;
use crate::workflow::Workflow;

pub const META_DIR: &str = ".opsis";
pub const CACHE_DIR: &str = ".opsis/cache";
pub const ANALYSIS_DIR: &str = ".opsis/cache/analysis";
pub const MIX_DIR: &str = ".opsis/cache/mix";
pub const PICTURES_DIR: &str = ".opsis/cache/pictures";
pub const TRACES_DIR: &str = ".opsis/traces";
pub const OUT_DIR: &str = ".opsis/out";
pub const LAST_RUN_FILE: &str = ".opsis/last-run.json";
pub const ASSETS_DIR: &str = "assets";
pub const MODELS_DIR: &str = "assets/models";
pub const MISSING_FILE: &str = "missing";

pub type FileVersions = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenProject {
    pub dir: String,
    pub document: DocumentState,
    pub last_run: Option<LastRun>,
    #[specta(type = crate::JsonObject)]
    pub view: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum SaveResult {
    Saved {
        version: String,
    },
    Changed {
        version: String,
        workflow: Option<Box<Workflow>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFileChange {
    pub dir: String,
    pub workflow: Option<Workflow>,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFilesChange {
    pub dir: String,
    pub files: FileVersions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLocations {
    pub default_dir: String,
    pub suggested_dir: String,
}
