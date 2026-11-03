pub mod actions;
pub mod analysis;
pub mod cards;
pub mod edits;
pub mod engine;
pub mod events;
pub mod fonts;
pub mod json;
pub mod live;
pub mod looks;
pub mod mix;
pub mod numbers;
pub mod project;
pub mod recording;
pub mod secrets;
pub mod settings;
pub mod shell;
pub mod terminal;
pub mod timeline;
pub mod trace;
pub mod ts_export;
pub mod values;
pub mod workflow;

use std::collections::HashMap;

use specta::datatype::DataType;
use specta::{Type, Types};
use specta_typescript::Unknown;

#[derive(Debug)]
pub struct JsonObject;

impl Type for JsonObject {
    fn definition(types: &mut Types) -> DataType {
        <HashMap<String, Unknown> as Type>::definition(types)
    }
}
