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
    use proptest::prelude::*;
    use serde_json::{Map, Value};

    use super::*;

    fn json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(Value::from),
            "[a-z ]{0,8}".prop_map(Value::from),
        ];
        leaf.prop_recursive(4, 48, 6, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
                prop::collection::btree_map("[a-z]{1,4}", inner, 0..6)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    fn reordered(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut out = Map::new();
                for (key, item) in map.iter().rev() {
                    out.insert(key.clone(), reordered(item));
                }
                Value::Object(out)
            }
            Value::Array(items) => Value::Array(items.iter().map(reordered).collect()),
            other => other.clone(),
        }
    }

    proptest! {
        #[test]
        fn ignores_key_order_at_every_depth(value in json(), length in 1usize..=64) {
            prop_assert_eq!(
                versioned_hash("v1", &value, length),
                versioned_hash("v1", &reordered(&value), length)
            );
            prop_assert_eq!(versioned_hash("v1", &value, length).len(), length);
        }

        #[test]
        fn changes_with_the_version_and_the_value(a in json(), b in json(), version in "[a-z0-9.]{1,6}") {
            prop_assert_ne!(versioned_hash("v1", &a, 64), versioned_hash(&format!("{version}+"), &a, 64));
            if stable_json(&a) != stable_json(&b) {
                prop_assert_ne!(versioned_hash("v1", &a, 64), versioned_hash("v1", &b, 64));
            }
        }
    }
}
