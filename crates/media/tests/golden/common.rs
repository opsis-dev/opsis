pub use opsis_golden::{cases, number, text, typed};
use serde::Serialize;
use serde_json::Value;

pub fn json<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("serializes")
}

pub fn with(case: &Value, outputs: &[(&str, Value)]) -> Value {
    let mut case = case.clone();
    for (key, value) in outputs {
        case[*key] = value.clone();
    }
    case
}
