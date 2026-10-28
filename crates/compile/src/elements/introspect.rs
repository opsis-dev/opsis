use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::params::ParamSchema;
use crate::text::round_to;

pub const ANY_VALUE: &str = "*";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum FieldWidget {
    Action,
    Asset,
    Backdrop,
    Boxes,
    Camera,
    Code,
    Color,
    Cut,
    Datetime,
    Envelope,
    Font,
    Gradient,
    Keys,
    Loudness,
    Marks,
    Multiselect,
    Music,
    Offsets,
    Port,
    Quality,
    Sound,
    Spot,
    Target,
    Textarea,
    Theme,
    Timezone,
    Transition,
    Variable,
}

impl FieldWidget {
    pub fn named(name: &str) -> Option<Self> {
        serde_json::from_value(Value::from(name)).ok()
    }
}

fn branch(schema: &Value) -> &Value {
    match schema.get("anyOf").and_then(Value::as_array) {
        Some(options) => options
            .iter()
            .find(|o| o.get("type") != Some(&Value::from("null")))
            .unwrap_or(schema),
        None => schema,
    }
}

pub fn widget_of(field: &Value) -> Option<&str> {
    field.get("x-widget").and_then(Value::as_str)
}

pub fn has_check(field: &Value, rule: &str) -> bool {
    match field.get("x-check") {
        Some(Value::Array(list)) => list
            .iter()
            .any(|c| c.get("rule").and_then(Value::as_str) == Some(rule)),
        Some(one) => one.get("rule").and_then(Value::as_str) == Some(rule),
        None => false,
    }
}

pub fn is_color_schema(field: &Value) -> bool {
    let field = branch(field);
    widget_of(field) == Some("color")
        || has_check(field, "colour")
        || has_check(field, "colour-or-empty")
}

pub fn widget_keys(schema: &ParamSchema, widget: &str) -> Vec<String> {
    schema
        .keys()
        .iter()
        .filter(|k| schema.field(k).and_then(widget_of) == Some(widget))
        .cloned()
        .collect()
}

pub fn map_leaves(
    schema: &Value,
    value: &Value,
    leaf: &dyn Fn(&Value, &Value) -> Option<Value>,
) -> Option<Value> {
    let inner = branch(schema);
    match (
        inner.get("properties").and_then(Value::as_object),
        inner.get("items"),
        value,
    ) {
        (Some(properties), _, Value::Object(map)) => {
            let mut out = map.clone();
            for (key, field) in properties {
                let Some(v) = map.get(key) else { continue };
                match map_leaves(field, v, leaf) {
                    Some(mapped) => out.insert(key.clone(), mapped),
                    None => out.remove(key),
                };
            }
            Some(Value::Object(out))
        }
        (_, Some(item), Value::Array(items)) => Some(Value::Array(
            items
                .iter()
                .map(|v| map_leaves(item, v, leaf).unwrap_or(Value::Null))
                .collect(),
        )),
        _ => leaf(schema, value),
    }
}

fn to_frames(seconds: f64, rate: f64) -> f64 {
    round_to(seconds * rate, 2)
}

fn number_text(n: f64) -> String {
    n.to_string()
}

pub fn frames_from_seconds(field: &Value, value: &Value) -> Option<Value> {
    let Some(rate) = field
        .get("x-frames-at")
        .and_then(Value::as_f64)
        .filter(|r| *r != 0.0)
    else {
        return Some(value.clone());
    };
    match value {
        Value::Number(n) => Some(Value::from(to_frames(n.as_f64().unwrap_or(0.0), rate))),
        Value::String(text) => {
            let parts: Vec<String> = text
                .split(',')
                .map(|part| {
                    let trimmed = part.trim();
                    match trimmed.parse::<f64>() {
                        Ok(seconds) if !trimmed.is_empty() && seconds.is_finite() => {
                            number_text(to_frames(seconds, rate))
                        }
                        _ => trimmed.to_owned(),
                    }
                })
                .collect();
            Some(Value::from(parts.join(", ")))
        }
        other => Some(other.clone()),
    }
}

pub fn settings_in_frames(
    schema: &ParamSchema,
    settings: &Map<String, Value>,
) -> Map<String, Value> {
    match map_leaves(
        schema.json(),
        &Value::Object(settings.clone()),
        &frames_from_seconds,
    ) {
        Some(Value::Object(map)) => map,
        _ => settings.clone(),
    }
}
