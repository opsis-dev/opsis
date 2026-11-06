use opsis_compile::catalog::{EditorCatalog, EditorNode};
use opsis_compile::inspector::InspectorContext;
use opsis_compile::text::format_duration;
use opsis_qt::{Choice, FieldKind, FieldView, PresetView};
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value};

const MILLIS_PER_SECOND: f64 = 1000.0;
const ANY_VALUE: &str = "*";

#[derive(Debug)]
pub struct FormCtx<'a> {
    pub inspector: Option<&'a InspectorContext>,
    pub catalog: &'a EditorCatalog,
    pub workflow: &'a Workflow,
    pub fonts: &'a [String],
}

pub fn fields(def: &EditorNode, node: &WorkflowNode, ctx: &FormCtx<'_>) -> Vec<FieldView> {
    let Some(properties) = def.params.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let values = current_values(def, node, properties);
    properties
        .iter()
        .filter(|(_, schema)| !flag(schema, "x-hidden"))
        .filter(|(_, schema)| shown(schema, &values, ctx))
        .map(|(key, schema)| field(key, schema, values.get(key), ctx))
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

fn name_of(value: &impl serde::Serialize) -> String {
    serde_json::to_value(value).map_or_default(|v| plain(&v))
}

fn lists(list: &Value, wanted: &str) -> bool {
    list.as_array()
        .is_some_and(|items| items.iter().any(|v| v.as_str() == Some(wanted)))
}

fn shown(schema: &Value, values: &Map<String, Value>, ctx: &FormCtx<'_>) -> bool {
    let rules_hold = schema
        .get("x-show-if")
        .and_then(Value::as_object)
        .is_none_or(|rules| {
            rules.iter().all(|(key, allowed)| {
                let current = values.get(key).map_or_default(plain);
                lists(allowed, &current) || (lists(allowed, ANY_VALUE) && !current.is_empty())
            })
        });
    let files_hold = schema
        .get("x-show-if-file")
        .and_then(Value::as_object)
        .is_none_or(|rules| {
            rules.iter().all(|(key, kinds)| {
                ctx.inspector
                    .and_then(|i| i.file_kinds.get(key))
                    .is_some_and(|kind| lists(kinds, &name_of(kind)))
            })
        });
    let take_holds = schema.get("x-show-if-take").is_none_or(|kinds| {
        ctx.inspector
            .and_then(|i| i.take_kind)
            .is_none_or(|take| lists(kinds, &name_of(&take)))
    });
    rules_hold && files_hold && take_holds
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

fn presets(schema: &Value) -> Vec<PresetView> {
    schema
        .get("x-presets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|preset| PresetView {
            label: text(preset, "label"),
            values_json: preset.get("values").map_or_default(Value::to_string),
        })
        .collect()
}

fn blank(key: &str, schema: &Value, value: Option<&Value>, ctx: &FormCtx<'_>) -> FieldView {
    let label = match text(schema, "description") {
        label if label.is_empty() => key.to_owned(),
        label => label,
    };
    let connected = ctx
        .inspector
        .is_some_and(|i| i.connected_inputs.iter().any(|input| input == key));
    let unconnected = text(schema, "x-unconnected-zero-label");
    let zero_label = if !connected && !unconnected.is_empty() {
        unconnected
    } else {
        text(schema, "x-zero-label")
    };
    FieldView {
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
        zero_label,
        scale: 1.0,
        checked: false,
        choices: Vec::new(),
        advanced: flag(schema, "x-advanced"),
        pauses: flag(schema, "x-pauses"),
        value_json: value.map_or_else(|| "null".to_owned(), Value::to_string),
        none_label: String::new(),
        clearable: types(schema).contains(&"null"),
        raw_choices: false,
        presets: presets(schema),
        presets_label: text(schema, "x-presets-label"),
    }
}

fn with_current(mut choices: Vec<Choice>, current: &str) -> Vec<Choice> {
    if !current.is_empty() && !choices.iter().any(|c| c.value == current) {
        choices.push(Choice {
            value: current.to_owned(),
            label: current.to_owned(),
        });
    }
    choices
}

fn pairs<'a>(items: impl IntoIterator<Item = (&'a str, String)>) -> Vec<Choice> {
    items
        .into_iter()
        .map(|(value, label)| Choice {
            value: value.to_owned(),
            label,
        })
        .collect()
}

fn select(view: &mut FieldView, value: Option<&Value>, choices: Vec<Choice>, none: &str) {
    view.kind = FieldKind::Select;
    view.text = value.map_or_default(plain);
    view.choices = with_current(choices, &view.text);
    none.clone_into(&mut view.none_label);
}

fn humanize(id: &str) -> String {
    let spaced = id.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn marker_label(label: &str, at_ms: f64) -> String {
    if label.is_empty() {
        format!("Marker at {}", format_duration(at_ms))
    } else {
        label.to_owned()
    }
}

fn select_choices(widget: &str, ctx: &FormCtx<'_>) -> Option<(Vec<Choice>, &'static str)> {
    let inspector = ctx.inspector;
    let catalog = ctx.catalog;
    Some(match widget {
        "action" => (
            pairs(
                inspector
                    .into_iter()
                    .flat_map(|i| &i.actions)
                    .map(|a| (a.id.as_str(), a.label.clone())),
            ),
            "None, use a time",
        ),
        "marker" => (
            pairs(
                ctx.workflow
                    .markers
                    .iter()
                    .map(|m| (m.id.as_str(), marker_label(&m.label, m.at_ms))),
            ),
            "None",
        ),
        "port" => (
            pairs(
                inspector
                    .into_iter()
                    .flat_map(|i| &i.ports)
                    .map(|p| (p.port.as_str(), p.label.clone())),
            ),
            "",
        ),
        "cut" => (
            pairs(
                inspector
                    .into_iter()
                    .flat_map(|i| i.ports.iter().skip(1))
                    .map(|p| (p.port.as_str(), p.label.clone())),
            ),
            "",
        ),
        "variable" => (
            pairs(
                inspector
                    .into_iter()
                    .flat_map(|i| &i.vars)
                    .map(|v| (v.name.as_str(), v.name.clone())),
            ),
            "",
        ),
        "backdrop" => (
            pairs(
                opsis_compile::backdrops::Backdrop::ALL
                    .iter()
                    .map(|b| (b.id(), humanize(b.id()))),
            ),
            "",
        ),
        "font" => (
            pairs(ctx.fonts.iter().map(|f| (f.as_str(), f.clone()))),
            "Project font",
        ),
        "sound" => (
            pairs(
                catalog
                    .sounds
                    .iter()
                    .map(|s| (s.reference.as_str(), s.label.clone())),
            ),
            "None",
        ),
        "music" => (
            pairs(
                catalog
                    .music
                    .iter()
                    .map(|s| (s.reference.as_str(), s.label.clone())),
            ),
            "None",
        ),
        "terminal-theme" => (
            pairs(
                catalog
                    .terminal_themes
                    .iter()
                    .map(|t| (t.name.as_str(), t.name.clone())),
            ),
            "",
        ),
        _ => return None,
    })
}

fn string_widget(widget: &str) -> Option<FieldKind> {
    Some(match widget {
        "code" => FieldKind::Code,
        "textarea" => FieldKind::Textarea,
        "timezone" => FieldKind::Timezone,
        "keys" => FieldKind::Keys,
        "datetime" => FieldKind::DateTime,
        _ => return None,
    })
}

fn named_widget(
    view: &mut FieldView,
    widget: &str,
    value: Option<&Value>,
    ctx: &FormCtx<'_>,
) -> bool {
    if let Some((choices, none)) = select_choices(widget, ctx) {
        select(view, value, choices, none);
        return true;
    }
    if let Some(kind) = string_widget(widget) {
        view.kind = kind;
        view.text = value.map_or_default(plain);
        return true;
    }
    let presets = match widget {
        "quality" => &ctx.catalog.quality_presets,
        "loudness" => &ctx.catalog.loudness_presets,
        _ => return false,
    };
    view.kind = FieldKind::PresetNumber;
    view.number = value.and_then(Value::as_f64).unwrap_or(0.0);
    view.choices = presets
        .iter()
        .map(|(label, v)| Choice {
            value: v.to_string(),
            label: label.clone(),
        })
        .collect();
    true
}

fn field(key: &str, schema: &Value, value: Option<&Value>, ctx: &FormCtx<'_>) -> FieldView {
    let mut view = blank(key, schema, value, ctx);
    let widget = text(schema, "x-widget");
    let kinds = types(schema);
    if !widget.is_empty() && named_widget(&mut view, &widget, value, ctx) {
        return view;
    }
    if let Some(choices) = schema.get("enum").and_then(Value::as_array) {
        view.kind = FieldKind::Choice;
        view.raw_choices = choices.iter().any(|c| !c.is_string());
        view.choices = choice_list(schema, choices);
        view.text = value.map_or_default(|v| {
            if v.is_string() {
                plain(v)
            } else {
                v.to_string()
            }
        });
    } else if kinds.contains(&"array")
        && let Some(choices) = schema.pointer("/items/enum").and_then(Value::as_array)
    {
        view.kind = FieldKind::EnumList;
        view.choices = choice_list(schema.get("items").unwrap_or(schema), choices);
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
        view.kind = FieldKind::Json;
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
            let key = plain(choice);
            let label = labels
                .and_then(|l| l.get(&key))
                .and_then(Value::as_str)
                .map_or_else(|| key.clone(), str::to_owned);
            Choice {
                value: if choice.is_string() {
                    key
                } else {
                    choice.to_string()
                },
                label,
            }
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
