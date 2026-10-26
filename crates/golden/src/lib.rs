use std::path::{Path, PathBuf};

use serde_json::Value;
use serde_json::ser::{PrettyFormatter, Serializer};

const WRITE: &str = "OPSIS_WRITE_GOLDEN";
const SHOWN: usize = 40;
const NUMBER_SLACK: f64 = 1e-9;

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub fn path(name: &str) -> PathBuf {
    root().join(name)
}

pub fn writing() -> bool {
    std::env::var_os(WRITE).is_some_and(|v| !v.is_empty() && v != "0")
}

pub fn read(name: &str) -> Value {
    let file = path(name);
    let text = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("{}: {e}. Record it with {WRITE}=1.", file.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

pub fn write(name: &str, value: &Value) {
    let file = path(name);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    }
    let mut out = Vec::new();
    let mut serializer = Serializer::with_formatter(&mut out, PrettyFormatter::with_indent(b" "));
    serde::Serialize::serialize(value, &mut serializer).expect("JSON serializes");
    out.push(b'\n');
    std::fs::write(&file, out).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
}

pub fn check(name: &str, made: &Value) {
    if writing() {
        write(name, made);
        return;
    }
    let found = differences(&read(name), made);
    assert!(
        found.is_empty(),
        "{name}: {} differences from the recorded file (record it again with {WRITE}=1 when the change is intended):\n{}",
        found.len(),
        found
            .iter()
            .take(SHOWN)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

pub fn differences(expected: &Value, actual: &Value) -> Vec<String> {
    let mut found = Vec::new();
    collect(expected, actual, String::new(), &mut found);
    found
}

fn same_number(a: f64, b: f64) -> bool {
    a == b || (a - b).abs() <= NUMBER_SLACK * a.abs().max(b.abs())
}

fn collect(expected: &Value, actual: &Value, at: String, found: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(x), Some(y)) = (a.as_f64(), b.as_f64())
                && !same_number(x, y)
            {
                found.push(format!("{at}: expected {a} got {b}"));
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                found.push(format!("{at}: expected {} items got {}", a.len(), b.len()));
            }
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                collect(x, y, format!("{at}[{i}]"), found);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            for (key, x) in a {
                match b.get(key) {
                    Some(y) => collect(x, y, format!("{at}.{key}"), found),
                    None => found.push(format!("{at}.{key}: missing, expected {x}")),
                }
            }
            for (key, y) in b.iter().filter(|(k, _)| !a.contains_key(*k)) {
                found.push(format!("{at}.{key}: unexpected {y}"));
            }
        }
        _ if expected != actual => found.push(format!("{at}: expected {expected} got {actual}")),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::differences;

    #[test]
    fn lists_every_difference_by_its_path() {
        let found = differences(
            &json!({ "a": [1, 2], "b": "x", "c": true }),
            &json!({ "a": [1, 3], "b": "y", "d": 1 }),
        );
        assert_eq!(
            found,
            [
                ".a[1]: expected 2 got 3",
                ".b: expected \"x\" got \"y\"",
                ".c: missing, expected true",
                ".d: unexpected 1",
            ]
        );
    }

    #[test]
    fn whole_and_fractional_numbers_compare_by_value() {
        assert!(differences(&json!(1920), &json!(1920.0)).is_empty());
        assert!(differences(&json!(0.1 + 0.2), &json!(0.3)).is_empty());
        assert_eq!(differences(&json!(1.0), &json!(1.001)).len(), 1);
    }
}
