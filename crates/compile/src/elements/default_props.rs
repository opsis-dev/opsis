use serde_json::{Map, Value};

use crate::elements::elements;

pub const DEFAULT_PROPS_FILE: &str = "packages/video/src/registry/default-props.generated.json";

pub fn default_props() -> String {
    let props: Map<String, Value> = elements()
        .iter()
        .map(|meta| {
            let mut props = meta.default_props.clone();
            props.extend(meta.fixed_props.clone());
            (meta.key.clone(), Value::Object(props))
        })
        .collect();
    format!(
        "{}\n",
        serde_json::to_string_pretty(&props).expect("props serialise")
    )
}
