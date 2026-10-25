use opsis_types::json::sorted_keys;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn stable_json(value: &impl Serialize) -> String {
    let value = sorted_keys(serde_json::to_value(value).expect("value serializes"));
    serde_json::to_string(&value).expect("values serialize")
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
}
