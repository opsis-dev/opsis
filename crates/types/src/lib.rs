pub mod actions;
pub mod cards;
pub mod events;
pub mod fonts;
pub mod json;
pub mod mix;
pub mod recording;
pub mod timeline;
pub mod ts_export;
pub mod values;
pub mod workflow;

pub type JsonObject = std::collections::HashMap<String, specta_typescript::Unknown>;
