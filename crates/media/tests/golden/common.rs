use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn json<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("serializes")
}

pub fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("{e}: {value}"))
}

pub fn text(value: &Value) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("not a string: {value}"))
}

pub fn number(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("not a number: {value}"))
}

pub fn cases(value: &Value, record: impl Fn(&Value) -> Value) -> Value {
    Value::Array(
        value
            .as_array()
            .unwrap_or_else(|| panic!("not a list: {value}"))
            .iter()
            .map(record)
            .collect(),
    )
}

pub fn with(case: &Value, outputs: &[(&str, Value)]) -> Value {
    let mut case = case.clone();
    for (key, value) in outputs {
        case[*key] = value.clone();
    }
    case
}
