use opsis_compile::catalog::EditorNode;
use opsis_qt::{Choice, FieldKind, FieldView};
use opsis_types::workflow::WorkflowNode;
use serde_json::{Map, Value};

const MILLIS_PER_SECOND: f64 = 1000.0;

pub fn fields(def: &EditorNode, node: &WorkflowNode) -> Vec<FieldView> {
    let Some(properties) = def.params.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let values = current_values(def, node, properties);
    properties
        .iter()
        .filter(|(_, schema)| !flag(schema, "x-hidden"))
        .filter(|(_, schema)| shown(schema, &values))
        .map(|(key, schema)| field(key, schema, values.get(key)))
        .collect()
}

fn current_values(
    def: &EditorNode,
    node: &WorkflowNode,
    properties: &Map<String, Value>,
) -> Map<String, Value> {
    properties
        .iter()
        .filter_map(|(key, schema)| {
            node.params
                .get(key)
                .or_else(|| def.defaults.get(key))
                .or_else(|| schema.get("default"))
                .map(|value| (key.clone(), value.clone()))
        })
        .collect()
}

fn flag(schema: &Value, key: &str) -> bool {
    schema.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn shown(schema: &Value, values: &Map<String, Value>) -> bool {
    let Some(rules) = schema.get("x-show-if").and_then(Value::as_object) else {
        return true;
    };
    rules.iter().all(|(key, allowed)| {
        let current = values.get(key).unwrap_or(&Value::Null);
        allowed
            .as_array()
            .is_some_and(|list| list.iter().any(|v| v == current))
    })
}

fn types(schema: &Value) -> Vec<&str> {
    match schema.get("type") {
        Some(Value::String(kind)) => vec![kind.as_str()],
        Some(Value::Array(kinds)) => kinds.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn text(schema: &Value, key: &str) -> String {
    schema
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn field(key: &str, schema: &Value, value: Option<&Value>) -> FieldView {
    let label = match text(schema, "description") {
        label if label.is_empty() => key.to_owned(),
        label => label,
    };
    let mut view = FieldView {
        key: key.to_owned(),
        label,
        help: text(schema, "x-help"),
        kind: FieldKind::Other,
        text: String::new(),
        number: 0.0,
        minimum: f64::MIN,
        maximum: f64::MAX,
        step: 1.0,
        decimals: 0,
        unit: text(schema, "x-unit"),
        zero_label: text(schema, "x-zero-label"),
        scale: 1.0,
        checked: false,
        choices: Vec::new(),
        advanced: flag(schema, "x-advanced"),
        pauses: flag(schema, "x-pauses"),
    };
    let widget = text(schema, "x-widget");
    let kinds = types(schema);
    if let Some(choices) = schema.get("enum").and_then(Value::as_array) {
        view.kind = FieldKind::Choice;
        view.choices = choice_list(schema, choices);
        view.text = value.map_or_default(plain);
    } else if widget == "code" {
        view.kind = FieldKind::Code;
        view.text = value.map_or_default(plain);
    } else if widget == "textarea" {
        view.kind = FieldKind::Textarea;
        view.text = value.map_or_default(plain);
    } else if !widget.is_empty() {
        view.kind = FieldKind::Other;
        view.text = summary(&widget, value);
    } else if kinds.contains(&"boolean") {
        view.kind = FieldKind::Switch;
        view.checked = value.and_then(Value::as_bool).unwrap_or(false);
    } else if kinds.contains(&"integer") || kinds.contains(&"number") {
        number_field(&mut view, schema, value, kinds.contains(&"integer"));
    } else if kinds.contains(&"string") {
        view.kind = FieldKind::Text;
        view.text = value.map_or_default(plain);
    } else {
        view.text = summary("value", value);
    }
    view
}

fn number_field(view: &mut FieldView, schema: &Value, value: Option<&Value>, integer: bool) {
    view.kind = FieldKind::Number;
    let seconds = view.unit == "ms";
    view.scale = if seconds { MILLIS_PER_SECOND } else { 1.0 };
    if seconds {
        "s".clone_into(&mut view.unit);
    }
    let read = |key: &str| schema.get(key).and_then(Value::as_f64);
    view.number = value.and_then(Value::as_f64).unwrap_or(0.0) / view.scale;
    view.minimum = read("minimum").map_or(-1e9, |v| v / view.scale);
    view.maximum = read("maximum").map_or(1e9, |v| v / view.scale);
    view.step = read("x-step").unwrap_or(1.0) / view.scale;
    view.decimals = decimals(view.step, integer && !seconds);
}

fn decimals(step: f64, integer: bool) -> u32 {
    if integer {
        return 0;
    }
    let mut places = 0;
    let mut scaled = step;
    while places < 4 && (scaled - scaled.round()).abs() > 1e-9 {
        scaled *= 10.0;
        places += 1;
    }
    places.max(1)
}

fn choice_list(schema: &Value, choices: &[Value]) -> Vec<Choice> {
    let labels = schema.get("x-labels").and_then(Value::as_object);
    choices
        .iter()
        .map(|choice| {
            let value = plain(choice);
            let label = labels
                .and_then(|l| l.get(&value))
                .and_then(Value::as_str)
                .map_or_else(|| value.clone(), str::to_owned);
            Choice { value, label }
        })
        .collect()
}

fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn summary(widget: &str, value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => format!("{widget}: not set"),
        Some(Value::String(text)) if text.is_empty() => format!("{widget}: not set"),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => format!("{widget}: {} items", items.len()),
        Some(Value::Object(_)) => format!("{widget}: set"),
        Some(other) => other.to_string(),
    }
}

pub fn parsed_value(json: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|_| Value::String(json.to_owned()))
}
