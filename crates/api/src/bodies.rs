use indexmap::IndexMap;
use opsis_types::workflow::{Position, Workflow};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct OpenBody {
    #[schemars(description = "The project folder")]
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct AddNodeBody {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Leave out to get a new id")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct UpdateNodeBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AddEdgeBody {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

pub type VariablesBody = IndexMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum EditorRequest {
    Open {
        dir: String,
    },
    Read,
    Replace {
        workflow: Box<Workflow>,
    },
    AddNode(AddNodeBody),
    UpdateNode {
        id: String,
        #[serde(flatten)]
        patch: UpdateNodeBody,
    },
    RemoveNodes {
        ids: Vec<String>,
    },
    AddEdge(AddEdgeBody),
    RemoveEdges {
        ids: Vec<String>,
    },
    SetSettings {
        #[specta(type = crate::bodies::SettingsPatch)]
        settings: Map<String, Value>,
    },
    SetVariables {
        variables: VariablesBody,
    },
    Problems,
    Undo,
    Redo,
    Save,
    Run(RunBody),
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<opsis_types::workflow::Theme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<opsis_types::fonts::ProjectFonts>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_node_id: Option<String>,
}

fn friendly(message: &str) -> String {
    if message.starts_with("missing field") {
        return "is required.".to_owned();
    }
    if message.contains("expected f64") || message.contains("expected a number") {
        return "must be a number.".to_owned();
    }
    if message.contains("expected a boolean") {
        return "must be true or false.".to_owned();
    }
    if message.contains("expected a string") {
        return "must be text.".to_owned();
    }
    if message.contains("expected a sequence") {
        return "must be a list.".to_owned();
    }
    if message.contains("expected a map") || message.contains("expected struct") {
        return "must be an object.".to_owned();
    }
    message.to_owned()
}

fn missing_field(message: &str) -> Option<&str> {
    message
        .strip_prefix("missing field `")
        .and_then(|rest| rest.split('`').next())
}

pub fn parse_body<T: DeserializeOwned>(body: &Value) -> Result<T, String> {
    if !body.is_object() {
        return Err("The request body must be a JSON object.".to_owned());
    }
    serde_path_to_error::deserialize::<_, T>(body).map_err(|error| {
        let message = error.inner().to_string();
        let parent = error.path().to_string();
        let path = match (missing_field(&message), parent.as_str()) {
            (Some(field), ".") => field.to_owned(),
            (Some(field), parent) => format!("{parent}.{field}"),
            (None, ".") => String::new(),
            (None, parent) => parent.to_owned(),
        };
        let text = friendly(&message);
        if path.is_empty() {
            text
        } else {
            format!("{path}: {text}")
        }
    })
}
