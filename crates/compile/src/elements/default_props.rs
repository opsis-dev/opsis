use serde_json::{Map, Value};

use crate::elements::elements;
use crate::nodes::elements::element_props;

pub const DEFAULT_PROPS_FILE: &str = "packages/video/src/registry/default-props.generated.json";

pub fn default_props() -> String {
    let props: Map<String, Value> = elements()
        .iter()
        .map(|meta| {
            (
                meta.key.clone(),
                Value::Object(element_props(meta, &meta.default_props)),
            )
        })
        .collect();
    format!(
        "{}\n",
        serde_json::to_string_pretty(&props).expect("props serialise")
    )
}
