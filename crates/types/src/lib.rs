pub mod actions;
pub mod analysis;
pub mod cards;
pub mod engine;
pub mod events;
pub mod fonts;
pub mod json;
pub mod mix;
pub mod project;
pub mod recording;
pub mod secrets;
pub mod settings;
pub mod timeline;
pub mod trace;
pub mod ts_export;
pub mod values;
pub mod workflow;

pub type JsonObject = std::collections::HashMap<String, specta_typescript::Unknown>;
