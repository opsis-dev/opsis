use std::sync::LazyLock;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::playwright::tables;

const ALIASES: [(&str, &[&str]); 5] = [
    ("ShiftLeft", &["Shift"]),
    ("ControlLeft", &["Control"]),
    ("AltLeft", &["Alt"]),
    ("MetaLeft", &["Meta"]),
    ("Enter", &["\n", "\r"]),
];
pub const MODIFIERS: [&str; 4] = ["Alt", "Control", "Meta", "Shift"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyDescription {
    pub key: String,
    pub key_code: i64,
    pub key_code_without_location: i64,
    pub code: String,
    pub text: String,
    pub location: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shifted: Option<Box<KeyDescription>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyDefinition {
    key: String,
    key_code: Option<i64>,
    key_code_without_location: Option<i64>,
    shift_key: Option<String>,
    shift_key_code: Option<i64>,
    text: Option<String>,
    location: Option<i64>,
}

fn build_layout(layout: &IndexMap<String, KeyDefinition>) -> IndexMap<String, KeyDescription> {
    let mut result = IndexMap::new();
    for (code, definition) in layout {
        let key_code = definition.key_code.unwrap_or(0);
        let mut description = KeyDescription {
            key: definition.key.clone(),
            key_code,
            key_code_without_location: definition.key_code_without_location.unwrap_or(key_code),
            code: code.clone(),
            text: definition.text.clone().unwrap_or_default(),
            location: definition.location.unwrap_or(0),
            shifted: None,
        };
        if definition.key.encode_utf16().count() == 1 {
            description.text = description.key.clone();
        }
        let shifted = definition.shift_key.as_ref().map(|shift_key| {
            let mut shifted = description.clone();
            shifted.key.clone_from(shift_key);
            shifted.text.clone_from(shift_key);
            if let Some(code) = definition.shift_key_code.filter(|c| *c != 0) {
                shifted.key_code = code;
                shifted.key_code_without_location = code;
            }
            shifted
        });
        result.insert(
            code.clone(),
            KeyDescription {
                shifted: shifted.clone().map(Box::new),
                ..description.clone()
            },
        );
        if let Some((_, aliases)) = ALIASES.iter().find(|(c, _)| c == code) {
            for alias in *aliases {
                result.insert((*alias).to_owned(), description.clone());
            }
        }
        if definition.location.unwrap_or(0) != 0 {
            continue;
        }
        if description.key.encode_utf16().count() == 1 {
            result.insert(description.key.clone(), description.clone());
        }
        if let Some(shifted) = shifted {
            result.insert(shifted.key.clone(), shifted);
        }
    }
    result
}

static LAYOUT: LazyLock<IndexMap<String, KeyDescription>> = LazyLock::new(|| {
    let layout: IndexMap<String, KeyDefinition> =
        serde_json::from_value(tables()["keyboard"].clone()).expect("keyboard layout parses");
    build_layout(&layout)
});

pub fn key_layout() -> &'static IndexMap<String, KeyDescription> {
    &LAYOUT
}

pub fn keypad_location() -> i64 {
    tables()["keypadLocation"].as_i64().unwrap_or(3)
}

pub fn mac_editing_commands(shortcut: &str) -> Vec<String> {
    let commands = match &tables()["macEditingCommands"][shortcut] {
        Value::String(one) => vec![one.clone()],
        Value::Array(many) => many
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    };
    commands
        .into_iter()
        .filter(|c| !c.starts_with("insert"))
        .map(|c| c[..c.len() - 1].to_owned())
        .collect()
}

pub fn split_chord(chord: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut building = String::new();
    for c in chord.chars() {
        if c == '+' && !building.is_empty() {
            keys.push(std::mem::take(&mut building));
        } else {
            building.push(c);
        }
    }
    keys.push(building);
    keys
}

pub fn pressed_chord(chord: &str) -> String {
    opsis_compile::nodes::browser::chord_keys(chord).join("+")
}
