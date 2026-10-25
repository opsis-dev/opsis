use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn number_text(n: &serde_json::Number) -> String {
    if n.is_f64() {
        let f = n.as_f64().unwrap_or(0.0);
        if f.is_finite() {
            ryu_js::Buffer::new().format_finite(f).to_owned()
        } else {
            "null".to_owned()
        }
    } else {
        n.to_string()
    }
}

fn write_stable(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&number_text(n)),
        Value::String(s) => out.push_str(&serde_json::to_string(s).expect("strings serialize")),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_stable(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).expect("keys serialize"));
                out.push(':');
                write_stable(&map[key], out);
            }
            out.push('}');
        }
    }
}

pub fn stable_json(value: &impl Serialize) -> String {
    let mut out = String::new();
    write_stable(
        &serde_json::to_value(value).expect("value serializes"),
        &mut out,
    );
    out
}

pub fn sha256_hex(data: &[u8], length: usize) -> String {
    let digest = hex::encode(Sha256::digest(data));
    digest[..length.min(digest.len())].to_owned()
}

pub fn versioned_hash(version: &str, value: &impl Serialize, length: usize) -> String {
    sha256_hex(
        format!("{version}\0{}", stable_json(value)).as_bytes(),
        length,
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn ignores_key_order_and_changes_with_the_version() {
        assert_eq!(
            versioned_hash("v1", &json!({ "a": 1, "b": 2 }), 24),
            versioned_hash("v1", &json!({ "b": 2, "a": 1 }), 24)
        );
        assert_ne!(
            versioned_hash("v1", &json!({ "a": 1 }), 24),
            versioned_hash("v2", &json!({ "a": 1 }), 24)
        );
        assert_eq!(versioned_hash("v1", &json!({ "a": 1 }), 24).len(), 24);
    }

    #[test]
    fn writes_numbers_as_javascript_does() {
        assert_eq!(
            stable_json(&json!({ "b": 1.0, "a": [0.1, 1e21, 2.5e-7, -0.0] })),
            r#"{"a":[0.1,1e+21,2.5e-7,0],"b":1}"#
        );
    }
}
