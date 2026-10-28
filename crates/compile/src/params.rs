use std::sync::Arc;

use jsonschema::error::{TypeKind, ValidationErrorKind};
use jsonschema::paths::{Location, LocationSegment};
use jsonschema::{JsonType, Keyword, ValidationError, Validator};
use opsis_types::numbers;
use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use crate::asset_ref::stays_in_project;
use crate::color::is_color;
use crate::errors::SettingsError;
use crate::text::{
    format_duration, grouped, humanize, is_locale, is_time_zone, is_wall_time, sign_of,
};
use crate::vars::{PAUSE_MARK, Vars, has_variable, interpolate_map, names_for_unknown};

pub type Values = Map<String, Value>;

pub const CHECK_KEYWORD: &str = "x-check";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    NotBlank,
    Colour,
    ColourOrEmpty,
    LocaleOrEmpty,
    TimeZoneOrEmpty,
    WallTime,
    InProject,
    Unique,
}

impl Rule {
    pub fn id(self) -> &'static str {
        match self {
            Self::NotBlank => "not-blank",
            Self::Colour => "colour",
            Self::ColourOrEmpty => "colour-or-empty",
            Self::LocaleOrEmpty => "locale-or-empty",
            Self::TimeZoneOrEmpty => "time-zone-or-empty",
            Self::WallTime => "wall-time",
            Self::InProject => "in-project",
            Self::Unique => "unique",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        [
            Self::NotBlank,
            Self::Colour,
            Self::ColourOrEmpty,
            Self::LocaleOrEmpty,
            Self::TimeZoneOrEmpty,
            Self::WallTime,
            Self::InProject,
            Self::Unique,
        ]
        .into_iter()
        .find(|r| r.id() == id)
    }

    pub fn holds(self, value: &Value) -> bool {
        match (self, value) {
            (Self::Unique, Value::Array(items)) => items
                .iter()
                .enumerate()
                .all(|(i, a)| !items[..i].contains(a)),
            (Self::Unique, _) => true,
            (_, Value::String(text)) => self.holds_text(text),
            _ => true,
        }
    }

    fn holds_text(self, text: &str) -> bool {
        let t = text.trim();
        match self {
            Self::Unique => true,
            Self::NotBlank => !text.replace(PAUSE_MARK, "").trim().is_empty(),
            Self::Colour => is_color(t),
            Self::ColourOrEmpty => t.is_empty() || is_color(t),
            Self::LocaleOrEmpty => t.is_empty() || is_locale(t),
            Self::TimeZoneOrEmpty => t.is_empty() || is_time_zone(t),
            Self::WallTime => is_wall_time(t),
            Self::InProject => stays_in_project(text),
        }
    }
}

pub fn check(rule: Rule, message: &str) -> Value {
    json!({ "rule": rule.id(), "message": message })
}

struct Checks(Vec<(Rule, String)>);

impl<'i> Keyword<'i> for Checks {
    fn validate(&self, instance: &'i Value) -> Result<(), ValidationError<'i>> {
        match self.0.iter().find(|(rule, _)| !rule.holds(instance)) {
            Some((_, message)) => Err(ValidationError::custom(message.clone())),
            None => Ok(()),
        }
    }

    fn is_valid(&self, instance: &'i Value) -> bool {
        self.0.iter().all(|(rule, _)| rule.holds(instance))
    }
}

fn checks_of(value: &Value) -> Vec<(Rule, String)> {
    let list: Vec<&Value> = match value {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    list.into_iter()
        .filter_map(|c| {
            let rule = Rule::from_id(c.get("rule")?.as_str()?)?;
            Some((
                rule,
                c.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("isn't valid")
                    .to_owned(),
            ))
        })
        .collect()
}

fn validator(schema: &Value) -> Validator {
    jsonschema::options()
        .with_keyword(CHECK_KEYWORD, |_parent, value, _location| {
            Ok(Box::new(Checks(checks_of(value))))
        })
        .build(schema)
        .expect("parameter schemas are valid JSON Schema")
}

pub fn schema_of<T: JsonSchema>() -> Value {
    let generator = SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .into_generator();
    let mut schema =
        serde_json::to_value(generator.into_root_schema_for::<T>()).expect("schema serializes");
    if let Some(object) = schema.as_object_mut() {
        object.remove("title");
    }
    schema
}

fn object_defaults(schema: &Value) -> Option<Map<String, Value>> {
    let properties = schema.get("properties")?.as_object()?;
    let mut out = schema
        .get("default")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (key, field) in properties {
        if let Some(default) = field.get("default").filter(|d| !d.is_null()) {
            out.entry(key.clone()).or_insert_with(|| default.clone());
        }
    }
    Some(out)
}

pub fn with_row_defaults(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            for value in map.values_mut() {
                with_row_defaults(value);
            }
            if let Some(items) = map.get_mut("items").and_then(Value::as_object_mut)
                && let Some(row) = object_defaults(&Value::Object(items.clone()))
            {
                items.insert("default".to_owned(), Value::Object(row));
            }
        }
        Value::Array(values) => values.iter_mut().for_each(with_row_defaults),
        _ => {}
    }
}

#[derive(Clone)]
pub struct ParamSchema {
    schema: Arc<Value>,
    validator: Arc<Validator>,
    order: Arc<Vec<String>>,
}

impl std::fmt::Debug for ParamSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamSchema")
            .field("fields", &self.order)
            .finish()
    }
}

impl ParamSchema {
    pub fn new(mut schema: Value) -> Self {
        with_row_defaults(&mut schema);
        if let Some(defaults) = object_defaults(&schema)
            && let Some(object) = schema.as_object_mut()
        {
            object.insert("default".to_owned(), Value::Object(defaults));
        }
        let order = schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default();
        Self {
            validator: Arc::new(validator(&schema)),
            schema: Arc::new(schema),
            order: Arc::new(order),
        }
    }

    pub fn of<T: JsonSchema>() -> Self {
        Self::new(schema_of::<T>())
    }

    pub fn json(&self) -> &Value {
        &self.schema
    }

    pub fn keys(&self) -> &[String] {
        &self.order
    }

    pub fn field(&self, key: &str) -> Option<&Value> {
        self.schema.get("properties")?.get(key)
    }

    pub fn defaults(&self) -> Values {
        self.order
            .iter()
            .filter_map(|k| {
                self.field(k)?
                    .get("default")
                    .map(|d| (k.clone(), d.clone()))
            })
            .collect()
    }

    pub fn with_defaults(&self, values: &Values) -> Values {
        let mut out = self.defaults();
        for (k, v) in values {
            out.insert(k.clone(), v.clone());
        }
        out
    }

    fn known(&self, values: &Values) -> Values {
        self.order
            .iter()
            .filter_map(|k| values.get(k).map(|v| (k.clone(), v.clone())))
            .collect()
    }

    pub fn parse(
        &self,
        values: &Values,
        written: Option<&Values>,
    ) -> Result<Values, SettingsError> {
        let filled = match fill(
            &self.schema,
            Value::Object(self.known(&self.with_defaults(values))),
        ) {
            Value::Object(map) => map,
            _ => Values::new(),
        };
        let instance = Value::Object(filled.clone());
        let first = self
            .validator
            .iter_errors(&instance)
            .map(|e| (self.rank(&e), precedence(&e), self.wording(&e, written)))
            .min_by_key(|(rank, precedence, _)| (*rank, *precedence));
        match first {
            Some((_, _, problem)) => Err(problem),
            None => Ok(filled),
        }
    }

    fn rank(&self, error: &ValidationError) -> usize {
        let key = error_key(error);
        key.and_then(|k| self.order.iter().position(|o| *o == k))
            .unwrap_or(self.order.len())
    }

    fn wording(&self, error: &ValidationError, written: Option<&Values>) -> SettingsError {
        let key = error_key(error);
        let field = key.as_deref().and_then(|k| self.field(k));
        let nested = top_key(error.instance_path()).is_some();
        let unit = field.and_then(|f| f.get("x-unit")).and_then(Value::as_str);
        let said = match error.kind() {
            ValidationErrorKind::Custom { message, .. } => {
                return SettingsError::Rule(message.clone().into());
            }
            ValidationErrorKind::Required { .. } if nested => "isn't valid".to_owned(),
            ValidationErrorKind::Required { .. } => "is missing".to_owned(),
            ValidationErrorKind::Type { kind } => {
                let numeric = |t: &JsonType| matches!(t, JsonType::Number | JsonType::Integer);
                let wants_number = match kind {
                    TypeKind::Single(t) => numeric(t),
                    TypeKind::Multiple(set) => set.iter().any(|t| numeric(&t)),
                };
                let input = key
                    .as_deref()
                    .and_then(|k| written.and_then(|w| w.get(k)))
                    .unwrap_or(error.instance());
                if wants_number && input.as_str().is_some_and(has_variable) {
                    "takes a number. Variables work only in text settings".to_owned()
                } else {
                    "isn't valid".to_owned()
                }
            }
            ValidationErrorKind::Minimum { limit } => bound(limit, unit, true, true),
            ValidationErrorKind::Maximum { limit } => bound(limit, unit, false, true),
            ValidationErrorKind::ExclusiveMinimum { limit } => bound(limit, unit, true, false),
            ValidationErrorKind::ExclusiveMaximum { limit } => bound(limit, unit, false, false),
            ValidationErrorKind::MinLength { limit: 1 } => "is empty".to_owned(),
            ValidationErrorKind::MinLength { .. } => "is too short".to_owned(),
            ValidationErrorKind::MaxLength { .. } => "is too long".to_owned(),
            ValidationErrorKind::MinItems { limit } => format!("needs at least {limit}"),
            ValidationErrorKind::MaxItems { limit } => format!("takes at most {limit}"),
            ValidationErrorKind::Enum { options } => match options.as_array() {
                Some(list) if list.iter().all(Value::is_number) => {
                    format!("must be {}", choice_list(list, unit))
                }
                _ => "has a value that isn't allowed".to_owned(),
            },
            ValidationErrorKind::Constant { .. } => "has a value that isn't allowed".to_owned(),
            _ => "isn't valid".to_owned(),
        };
        let label = field
            .and_then(|f| f.get("description"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| key.as_deref().map(humanize))
            .unwrap_or_else(|| "A setting".to_owned());
        SettingsError::Field {
            label,
            problem: said,
        }
    }

    pub fn card_values(&self, params: &Values, vars: &Vars) -> Values {
        let mut values = match names_for_unknown(&Value::Object(interpolate_map(
            &self.with_defaults(params),
            vars,
        ))) {
            Value::Object(map) => map,
            _ => Values::new(),
        };
        let defaults = self.defaults();
        for (key, fallback) in &defaults {
            if !values.get(key).is_some_and(|v| same_kind(v, fallback)) {
                values.insert(key.clone(), fallback.clone());
            }
        }
        let instance = Value::Object(values.clone());
        let broken: Vec<String> = self
            .validator
            .iter_errors(&instance)
            .filter(|e| {
                matches!(
                    e.kind(),
                    ValidationErrorKind::Type { .. }
                        | ValidationErrorKind::Enum { .. }
                        | ValidationErrorKind::Constant { .. }
                )
            })
            .filter_map(|e| top_key(e.instance_path()))
            .collect();
        for key in broken {
            match defaults.get(&key) {
                Some(fallback) => values.insert(key, fallback.clone()),
                None => values.remove(&key),
            };
        }
        values
    }

    pub fn typed<T: DeserializeOwned>(&self, values: &Values) -> Result<T, SettingsError> {
        typed_values(values)
    }
}

fn whole_numbers(value: Value) -> Value {
    match value {
        Value::Number(n) => match n.as_f64() {
            Some(f) if n.is_f64() && f.fract() == 0.0 && f.abs() <= f64::from(u32::MAX) => {
                Value::from(numbers::truncate::<i64>(f))
            }
            _ => Value::Number(n),
        },
        Value::Array(items) => Value::Array(items.into_iter().map(whole_numbers).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, whole_numbers(v)))
                .collect(),
        ),
        other => other,
    }
}

pub fn typed_values<T: DeserializeOwned>(values: &Values) -> Result<T, SettingsError> {
    serde_json::from_value(whole_numbers(Value::Object(values.clone())))
        .map_err(|e| SettingsError::Unreadable(e.to_string()))
}

fn object_branch(schema: &Value) -> &Value {
    match schema.get("anyOf").and_then(Value::as_array) {
        Some(options) => options
            .iter()
            .find(|o| o.get("properties").is_some() || o.get("items").is_some())
            .unwrap_or(schema),
        None => schema,
    }
}

pub fn fill(schema: &Value, value: Value) -> Value {
    let schema = object_branch(schema);
    match value {
        Value::Object(map) => match schema.get("properties").and_then(Value::as_object) {
            Some(properties) => Value::Object(
                properties
                    .iter()
                    .filter_map(|(key, field)| match map.get(key) {
                        Some(v) => Some((key.clone(), fill(field, v.clone()))),
                        None => field.get("default").map(|d| (key.clone(), d.clone())),
                    })
                    .collect(),
            ),
            None => Value::Object(map),
        },
        Value::Array(items) => match schema.get("items") {
            Some(item) => Value::Array(items.into_iter().map(|v| fill(item, v)).collect()),
            None => Value::Array(items),
        },
        other => other,
    }
}

fn precedence(error: &ValidationError) -> u8 {
    match error.kind() {
        ValidationErrorKind::Enum { .. } | ValidationErrorKind::Constant { .. } => 0,
        _ => 1,
    }
}

fn same_kind(a: &Value, b: &Value) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

fn error_key(error: &ValidationError) -> Option<String> {
    match (top_key(error.instance_path()), error.kind()) {
        (Some(key), _) => Some(key),
        (None, ValidationErrorKind::Required { property }) => property.as_str().map(str::to_owned),
        (None, _) => None,
    }
}

fn top_key(location: &Location) -> Option<String> {
    location.iter().next().map(|segment| match segment {
        LocationSegment::Property(p) => p.to_string(),
        LocationSegment::Index(i) => i.to_string(),
    })
}

fn choice_list(values: &[Value], unit: Option<&str>) -> String {
    let words: Vec<String> = values
        .iter()
        .map(|v| v.as_f64().map(|n| n.to_string()).unwrap_or_default())
        .collect();
    let list = match words.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        None => String::new(),
    };
    match unit {
        Some(u) => format!("{list} {u}"),
        None => list,
    }
}

const ATTACHED_UNITS: [&str; 3] = ["×", "%", "°"];

fn limit_text(limit: f64, unit: Option<&str>) -> Option<String> {
    match unit {
        Some("frames") => None,
        Some("ms") => Some(format_duration(limit)),
        Some(u) => {
            let number = format!("{}{}", sign_of(limit, false), grouped(limit, 3));
            Some(if ATTACHED_UNITS.contains(&u) {
                format!("{number}{u}")
            } else {
                format!("{number} {u}")
            })
        }
        None => Some(format!("{}{}", sign_of(limit, false), grouped(limit, 3))),
    }
}

fn bound(limit: &Value, unit: Option<&str>, low: bool, inclusive: bool) -> String {
    let limit = limit.as_f64().unwrap_or(0.0);
    match limit_text(limit, unit) {
        None if unit == Some("frames") => {
            if low { "is too short" } else { "is too long" }.to_owned()
        }
        None => if low { "is too small" } else { "is too large" }.to_owned(),
        Some(text) if !inclusive => {
            format!("must be {} than {text}", if low { "more" } else { "less" })
        }
        Some(text) => format!("must be at {} {text}", if low { "least" } else { "most" }),
    }
}

pub trait Params:
    JsonSchema + Serialize + DeserializeOwned + Default + Send + Sync + 'static
{
    fn check(&self) -> Result<(), SettingsError> {
        Ok(())
    }
}

pub fn to_values<T: Serialize>(params: &T) -> Values {
    match serde_json::to_value(params) {
        Ok(Value::Object(map)) => map,
        _ => Values::new(),
    }
}

pub fn whole_number(value: Option<&Value>) -> Option<u32> {
    value
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
}

#[cfg(test)]
mod row_defaults {
    use serde_json::json;

    use super::with_row_defaults;

    #[test]
    fn gives_each_list_of_objects_its_default_row() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "cuts": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "default": { "before": "in3" },
                        "properties": {
                            "before": { "type": "string", "default": "in2" },
                            "length": { "type": "integer", "default": 15 },
                            "note": { "type": "string" },
                        },
                    },
                },
                "names": { "type": "array", "items": { "type": "string" } },
            },
        });
        with_row_defaults(&mut schema);
        assert_eq!(
            schema["properties"]["cuts"]["items"]["default"],
            json!({ "before": "in3", "length": 15 })
        );
        assert!(
            schema["properties"]["names"]["items"]
                .get("default")
                .is_none()
        );
    }
}
