use indexmap::IndexMap;
use opsis_types::edits::{EdgeEnds, NodePatch};
use opsis_types::workflow::Position;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenBody {
    #[schemars(description = "The project folder")]
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrowserBody {
    #[schemars(
        description = "A browser action's node id, or an Open browser node's: the browser plays its flow's actions up to and including it"
    )]
    pub up_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateProjectBody {
    #[schemars(description = "The project's name, which also names its folder")]
    pub name: String,
    #[schemars(description = "A template id from GET /templates")]
    pub template: String,
    #[schemars(description = "The absolute folder to create the project's folder in")]
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddEdgeBody {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

pub type VariablesBody = IndexMap<String, String>;

impl From<UpdateNodeBody> for NodePatch {
    fn from(body: UpdateNodeBody) -> Self {
        Self {
            label: body.label.map(Option::unwrap_or_default),
            disabled: body.disabled,
            position: body.position,
            size: None,
            params: body.params,
        }
    }
}

impl From<AddEdgeBody> for EdgeEnds {
    fn from(body: AddEdgeBody) -> Self {
        Self {
            source: body.source,
            source_handle: body.source_handle,
            target: body.target,
            target_handle: body.target_handle,
        }
    }
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

#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    #[error("The request body must be a JSON object.")]
    NotAnObject,
    #[error("{text}")]
    Whole { text: String },
    #[error("{path}: {text}")]
    Field { path: String, text: String },
}

pub fn parse_body<T: DeserializeOwned>(body: &Value) -> Result<T, BodyError> {
    if !body.is_object() {
        return Err(BodyError::NotAnObject);
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
            BodyError::Whole { text }
        } else {
            BodyError::Field { path, text }
        }
    })
}
