use serde_json::{Map, Number, Value};

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

fn whole(n: f64) -> Option<i64> {
    (n.fract() == 0.0 && n.abs() <= MAX_SAFE_INTEGER).then_some(n as i64)
}

pub fn canonical(value: Value) -> Value {
    match value {
        Value::Number(n) => match n.as_f64().and_then(whole) {
            Some(i) if n.is_f64() => Value::Number(Number::from(i)),
            _ => Value::Number(n),
        },
        Value::Array(items) => Value::Array(items.into_iter().map(canonical).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, canonical(v)))
                .collect::<Map<_, _>>(),
        ),
        other => other,
    }
}

fn number_text(n: &Number) -> String {
    n.as_f64()
        .map_or_else(|| n.to_string(), |f| format!("{f:?}"))
}

pub fn first_difference(expected: &Value, actual: &Value) -> Option<String> {
    difference(expected, actual, String::new())
}

fn difference(expected: &Value, actual: &Value, at: String) -> Option<String> {
    match (expected, actual) {
        (Value::Number(a), Value::Number(b)) => {
            let (x, y) = (a.as_f64()?, b.as_f64()?);
            (x != y).then(|| format!("{at}: expected {} got {}", number_text(a), number_text(b)))
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{at}: expected {} items got {}", a.len(), b.len()));
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (x, y))| difference(x, y, format!("{at}[{i}]")))
        }
        (Value::Object(a), Value::Object(b)) => {
            if let Some(key) = a.keys().find(|k| !b.contains_key(*k)) {
                return Some(format!("{at}.{key}: missing, expected {}", a[key]));
            }
            if let Some(key) = b.keys().find(|k| !a.contains_key(*k)) {
                return Some(format!("{at}.{key}: unexpected {}", b[key]));
            }
            a.iter()
                .find_map(|(k, x)| difference(x, &b[k], format!("{at}.{k}")))
        }
        _ => (expected != actual).then(|| format!("{at}: expected {expected} got {actual}")),
    }
}
