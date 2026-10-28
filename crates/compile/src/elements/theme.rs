use indexmap::IndexMap;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Theme {
    pub mode: ThemeMode,
    #[schemars(extend("propertyNames" = { "type": "string" }))]
    pub colors: IndexMap<String, String>,
}

impl Theme {
    pub fn of(mode: ThemeMode) -> Self {
        Self {
            mode,
            colors: IndexMap::new(),
        }
    }
}

pub fn with_themes(schema: &mut Schema, themes: Value) {
    if let Some(object) = schema.as_object_mut() {
        object.insert("x-themes".to_owned(), themes);
    }
}
