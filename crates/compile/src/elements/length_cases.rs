use serde_json::{Map, Value, json};

use crate::elements::element_meta;
use crate::elements::introspect::settings_in_frames;
use crate::elements::lengths::animation_length;

pub const LENGTH_CASES_FILE: &str = "packages/video/src/registry/length-cases.generated.json";

const FPS: u32 = 30;

fn appended(defaults: &Map<String, Value>, key: &str, item: Value) -> Value {
    let mut list = defaults
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    list.push(item);
    Value::Array(list)
}

fn doubled_text(defaults: &Map<String, Value>, key: &str) -> Value {
    let text = defaults
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default();
    Value::from(format!("{text}\n\n{text}"))
}

fn changes(key: &str, defaults: &Map<String, Value>) -> Value {
    match key {
        "word-push" => json!({ "text": "one two three four five six seven eight nine ten" }),
        "search-reveal" => {
            json!({ "text": "a much longer search phrase to type", "framesPerCharacter": 0.2 })
        }
        "check-list" => json!({
            "items": [
                { "text": "Write the script", "checked": true },
                { "text": "Record every scene in one take", "checked": true },
                { "text": "Add music", "checked": false },
                { "text": "Check the captions twice before sending", "checked": true },
                { "text": "Publish", "checked": true }
            ],
            "itemGap": 0.8
        }),
        "code-morph" => json!({
            "steps": [{ "at": 0, "code": "a" }, { "at": 2, "code": "b" }, { "at": 4, "code": "c" }],
            "morphDuration": 1.5
        }),
        "keystroke" => {
            json!({ "steps": appended(defaults, "steps", json!({ "at": 9, "keys": "mod+s" })), "linger": 1.5 })
        }
        "agent-run" => json!({
            "runAnswer": doubled_text(defaults, "runAnswer"),
            "runSteps": appended(defaults, "runSteps", json!({ "name": "Write the summary", "duration": 2 }))
        }),
        "command-menu" => {
            json!({ "query": "settings", "letterSeconds": 0.15, "pauseSeconds": 1.2 })
        }
        "punch-lines" => {
            json!({ "script": "One. | Two. | Three. | Four. | Five.", "holds": "1.5, 2" })
        }
        _ => json!({}),
    }
}

const KEYS: [&str; 8] = [
    "word-push",
    "search-reveal",
    "check-list",
    "code-morph",
    "keystroke",
    "agent-run",
    "command-menu",
    "punch-lines",
];

fn framed(
    meta: &crate::elements::ElementMeta,
    settings: &Map<String, Value>,
) -> Map<String, Value> {
    let mut out = settings_in_frames(&meta.schema, settings);
    out.extend(meta.fixed_props.clone());
    out
}

pub fn length_cases() -> String {
    let cases: Vec<Value> = KEYS
        .iter()
        .map(|key| {
            let meta = element_meta(key).expect("a registry element");
            let usual = meta.default_props.clone();
            let mut settings = usual.clone();
            if let Value::Object(changed) = changes(key, &usual) {
                settings.extend(changed);
            }
            let growth = i64::from(animation_length(meta, &settings, FPS)) - i64::from(animation_length(meta, &usual, FPS));
            json!({ "key": key, "usual": framed(meta, &usual), "settings": framed(meta, &settings), "growth": growth })
        })
        .collect();
    format!(
        "{}\n",
        serde_json::to_string_pretty(&cases).expect("cases serialise")
    )
}
