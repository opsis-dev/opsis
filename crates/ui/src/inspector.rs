use opsis_compile::catalog::{EditorCatalog, EditorNode};
use opsis_compile::inspector::InspectorContext;
use opsis_compile::text::format_duration;
use opsis_qt::{Choice, FieldKind, FieldView, PresetView};
use opsis_types::timeline::EnvelopeKey;
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value};

const MILLIS_PER_SECOND: f64 = 1000.0;
const ANY_VALUE: &str = "*";
const COLOUR_RULE: &str = "colour";

#[derive(Debug)]
pub struct FormCtx<'a> {
    pub inspector: Option<&'a InspectorContext>,
    pub catalog: &'a EditorCatalog,
    pub workflow: &'a Workflow,
    pub playhead: Option<f64>,
}

const NESTING: usize = 2;
const SPOT_CENTRE: f64 = 0.5;
const SUMMARY_SKIPS: [&str; 5] = ["asset", "color", "code", "sound", "music"];
const SUMMARY_CHARS: usize = 60;

pub fn fields(def: &EditorNode, node: &WorkflowNode, ctx: &FormCtx<'_>) -> Vec<FieldView> {
    let Some(properties) = def.params.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let values = current_values(def, node, properties);
    let mut out = Vec::new();
    form(properties, &values, "", "", ctx, 0, &mut out);
    out
}

fn form(
    properties: &Map<String, Value>,
    values: &Map<String, Value>,
    prefix: &str,
    group: &str,
    ctx: &FormCtx<'_>,
    depth: usize,
    out: &mut Vec<FieldView>,
) {
    for (key, schema) in properties {
        if flag(schema, "x-hidden") || !shown(schema, values, ctx) {
            continue;
        }
        let path = format!("{prefix}{key}");
        let mut view = field(&path, schema, values.get(key), ctx);
        group.clone_into(&mut view.group);
        if view.kind == FieldKind::Spot {
            view.extra_json = spot_extra(key, schema, prefix, values, ctx).to_string();
        } else if view.kind == FieldKind::Boxes {
            view.extra_json = Value::Object(frame_extra(schema, prefix, values, ctx)).to_string();
        } else if view.kind == FieldKind::Envelope {
            view.extra_json = envelope_extra(schema, ctx).to_string();
        } else if view.kind == FieldKind::Camera {
            view.extra_json = camera_extra(ctx).to_string();
        } else if view.kind == FieldKind::Marks {
            view.extra_json = marks_extra(key, schema, prefix, values, ctx).to_string();
            view.new_item_json = schema
                .get("items")
                .map_or_else(|| "{}".to_owned(), |item| new_item(item).to_string());
        }
        let nested =
            matches!(view.kind, FieldKind::ObjectList | FieldKind::Marks) && depth < NESTING;
        out.push(view);
        if !nested {
            continue;
        }
        let Some(item) = schema.get("items") else {
            continue;
        };
        let item_props = item
            .get("properties")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for (index, entry) in values
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let item_group = format!("{path}/{index}");
            let item_values = with_defaults(&item_props, entry);
            form(
                &item_props,
                &item_values,
                &format!("{item_group}/"),
                &item_group,
                ctx,
                depth + 1,
                out,
            );
        }
    }
}

fn with_defaults(properties: &Map<String, Value>, entry: &Value) -> Map<String, Value> {
    properties
        .iter()
        .filter_map(|(key, schema)| {
            entry
                .get(key)
                .or_else(|| schema.get("default"))
                .map(|v| (key.clone(), v.clone()))
        })
        .collect()
}

fn new_item(item: &Value) -> Value {
    let props = item
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Value::Object(with_defaults(&props, &Value::Object(Map::new())))
}

pub fn row_summary(item: &Value, entry: &Value, index: usize, ctx: &FormCtx<'_>) -> String {
    let props = item
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let ports = ctx.inspector.map_or_default(|i| i.ports.as_slice());
    let text_of = |key: &str, schema: &Value| -> Option<String> {
        let value = entry.get(key)?.as_str()?;
        let widget = text(schema, "x-widget");
        if widget == "cut" || widget == "port" {
            return Some(
                ports
                    .iter()
                    .find(|p| p.port == value)
                    .map_or_else(|| "Not connected".to_owned(), |p| p.label.clone()),
            );
        }
        value.lines().next().map(|line| line.trim().to_owned())
    };
    let string_field = |schema: &Value| {
        types(schema).contains(&"string")
            && schema.get("enum").is_none()
            && !SUMMARY_SKIPS.contains(&text(schema, "x-widget").as_str())
    };
    let shown = props
        .iter()
        .filter(|(_, s)| string_field(s))
        .find_map(|(k, s)| text_of(k, s).filter(|t| !t.is_empty()))
        .or_else(|| {
            props
                .iter()
                .filter(|(_, s)| text(s, "x-widget") == "asset")
                .find_map(|(k, s)| text_of(k, s).filter(|t| !t.is_empty()))
                .map(|file| file.rsplit('/').next().unwrap_or(&file).to_owned())
        })
        .or_else(|| {
            props
                .keys()
                .find_map(|k| entry.get(k).filter(|v| v.is_number()).map(Value::to_string))
        });
    shown.map_or_else(
        || format!("Item {}", index + 1),
        |s| s.chars().take(SUMMARY_CHARS).collect(),
    )
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
        group: String::new(),
        min_items: schema
            .get("minItems")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        max_items: schema
            .get("maxItems")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(u32::MAX),
        new_item_json: String::new(),
        extra_json: String::new(),
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
        "gradient" => FieldKind::Gradient,
        "asset" => FieldKind::Asset,
        "target" => FieldKind::Target,
        _ => return None,
    })
}

fn named_widget(
    view: &mut FieldView,
    widget: &str,
    value: Option<&Value>,
    ctx: &FormCtx<'_>,
) -> bool {
    if widget == "font" {
        view.kind = FieldKind::Font;
        view.text = value.map_or_default(plain);
        "Project font".clone_into(&mut view.none_label);
        return true;
    }
    if let Some((choices, none)) = select_choices(widget, ctx) {
        select(view, value, choices, none);
        return true;
    }
    if let Some(kind) = string_widget(widget) {
        view.kind = kind;
        view.text = value.map_or_default(plain);
        return true;
    }
    if widget == "theme" {
        theme_field(view, value);
        return true;
    }
    if widget == "offsets" {
        view.kind = FieldKind::Offsets;
        view.text = value.map_or_default(plain);
        view.number = ctx.inspector.map_or(1.0, |i| {
            f64::from(u32::try_from(i.layers.max(1)).unwrap_or(1))
        });
        return true;
    }
    if widget == "envelope" {
        view.kind = FieldKind::Envelope;
        return true;
    }
    if widget == "camera" {
        view.kind = FieldKind::Camera;
        return true;
    }
    if widget == "marks" {
        view.kind = FieldKind::Marks;
        return true;
    }
    if widget == "boxes" {
        view.kind = FieldKind::Boxes;
        return true;
    }
    if widget == "spot" {
        view.kind = FieldKind::Spot;
        view.number = value.and_then(Value::as_f64).unwrap_or(SPOT_CENTRE);
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

pub fn spot_y_key(x_key: &str) -> String {
    if x_key == "x" {
        "y".to_owned()
    } else if let Some(stem) = x_key.strip_suffix('X') {
        format!("{stem}Y")
    } else {
        format!("{x_key}Y")
    }
}

fn frame_extra(
    schema: &Value,
    prefix: &str,
    values: &Map<String, Value>,
    ctx: &FormCtx<'_>,
) -> Map<String, Value> {
    let background = ctx.inspector.and_then(|i| i.spot_background.as_ref());
    let settings = &ctx.workflow.settings;
    let project_aspect = f64::from(settings.width) / f64::from(settings.height.max(1));
    let time_key = text(schema, "x-time-key");
    let mut extra = Map::new();
    extra.insert(
        "aspect".to_owned(),
        Value::from(background.and_then(|b| b.aspect).unwrap_or(project_aspect)),
    );
    extra.insert(
        "picture".to_owned(),
        Value::from(
            background
                .filter(|b| b.still)
                .map_or_default(|b| b.src.clone()),
        ),
    );
    extra.insert(
        "durationMs".to_owned(),
        Value::from(background.map_or(0.0, |b| b.duration_ms)),
    );
    if !time_key.is_empty() {
        extra.insert(
            "time".to_owned(),
            Value::from(values.get(&time_key).and_then(Value::as_f64).unwrap_or(0.0)),
        );
        extra.insert(
            "timeKey".to_owned(),
            Value::from(format!("{prefix}{time_key}")),
        );
    }
    extra
}

const fn track_format(key: EnvelopeKey) -> (&'static str, f64) {
    match key {
        EnvelopeKey::Scale => ("×", 1.0),
        EnvelopeKey::Pan => ("", 100.0),
        _ => ("%", 100.0),
    }
}

fn keyframe_book(ctx: &FormCtx<'_>) -> Map<String, Value> {
    let mut extra = Map::new();
    extra.insert(
        "eases".to_owned(),
        serde_json::to_value(&ctx.catalog.timeline.eases).unwrap_or(Value::Null),
    );
    extra.insert("fps".to_owned(), Value::from(ctx.workflow.settings.fps));
    extra
}

fn envelope_extra(schema: &Value, ctx: &FormCtx<'_>) -> Value {
    let mut extra = keyframe_book(ctx);
    let tracks: Vec<Value> = opsis_compile::envelope::envelope_tracks()
        .into_iter()
        .filter_map(|track| {
            let key = serde_json::to_value(track.key).ok()?;
            let name = key.as_str()?.to_owned();
            schema.pointer(&format!("/properties/{name}"))?;
            let (unit, factor) = track_format(track.key);
            Some(serde_json::json!({
                "key": name,
                "label": track.label,
                "unit": unit,
                "factor": factor,
            }))
        })
        .collect();
    extra.insert("tracks".to_owned(), Value::from(tracks));
    Value::Object(extra)
}

fn camera_extra(ctx: &FormCtx<'_>) -> Value {
    let mut extra = keyframe_book(ctx);
    extra.insert(
        "dials".to_owned(),
        serde_json::to_value(&ctx.catalog.timeline.camera_dials).unwrap_or(Value::Null),
    );
    extra.insert(
        "playhead".to_owned(),
        ctx.playhead.map_or(Value::Null, Value::from),
    );
    Value::Object(extra)
}

fn marks_extra(
    key: &str,
    schema: &Value,
    prefix: &str,
    values: &Map<String, Value>,
    ctx: &FormCtx<'_>,
) -> Value {
    let mut extra = frame_extra(schema, prefix, values, ctx);
    let first = values.get("firstStep").and_then(Value::as_i64).unwrap_or(1);
    let marks = values
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    extra.insert("firstStep".to_owned(), Value::from(first));
    extra.insert(
        "steps".to_owned(),
        Value::from(
            opsis_compile::marks::step_numbers(&marks, first)
                .into_iter()
                .map(|n| n.map_or(Value::Null, Value::from))
                .collect::<Vec<_>>(),
        ),
    );
    Value::Object(extra)
}

fn spot_extra(
    key: &str,
    schema: &Value,
    prefix: &str,
    values: &Map<String, Value>,
    ctx: &FormCtx<'_>,
) -> Value {
    let y_key = spot_y_key(key);
    let mut extra = frame_extra(schema, prefix, values, ctx);
    extra.insert("yKey".to_owned(), Value::from(format!("{prefix}{y_key}")));
    extra.insert(
        "y".to_owned(),
        Value::from(
            values
                .get(&y_key)
                .and_then(Value::as_f64)
                .unwrap_or(SPOT_CENTRE),
        ),
    );
    Value::Object(extra)
}

fn theme_field(view: &mut FieldView, value: Option<&Value>) {
    view.kind = FieldKind::Theme;
    let mode = value
        .and_then(|v| v.get("mode"))
        .and_then(Value::as_str)
        .unwrap_or("dark")
        .to_owned();
    let presets: Value = serde_json::from_str(&view.extra_json).unwrap_or(Value::Null);
    view.choices = presets
        .get(&mode)
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(token, _)| Choice {
            value: token.clone(),
            label: opsis_compile::text::humanize(token),
        })
        .collect();
    view.text = mode;
}

fn field(key: &str, schema: &Value, value: Option<&Value>, ctx: &FormCtx<'_>) -> FieldView {
    let mut view = blank(key, schema, value, ctx);
    let widget = text(schema, "x-widget");
    if let Some(themes) = schema.get("x-themes") {
        view.extra_json = themes.to_string();
    }
    let kinds = types(schema);
    if !widget.is_empty() && named_widget(&mut view, &widget, value, ctx) {
        return view;
    }
    if is_color(schema) && kinds.contains(&"string") {
        view.kind = FieldKind::Color;
        view.text = value.map_or_default(plain);
    } else if !widget.is_empty() && !matches!(widget.as_str(), "color" | "multiselect") {
        view.kind = FieldKind::Other;
        view.text = summary(&widget, value);
    } else if let Some(choices) = schema.get("enum").and_then(Value::as_array) {
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
    } else if kinds.contains(&"array")
        && let Some(item) = schema.get("items")
    {
        list_field(&mut view, item, value, ctx);
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

pub fn is_color(schema: &Value) -> bool {
    text(schema, "x-widget") == "color"
        || schema.pointer("/x-check/rule").and_then(Value::as_str) == Some(COLOUR_RULE)
}

fn list_field(view: &mut FieldView, item: &Value, value: Option<&Value>, ctx: &FormCtx<'_>) {
    let item_types = types(item);
    let color = is_color(item);
    view.kind = if item_types.contains(&"object") {
        FieldKind::ObjectList
    } else if color {
        FieldKind::ColorList
    } else if item_types.contains(&"number") || item_types.contains(&"integer") {
        FieldKind::NumberList
    } else {
        FieldKind::StringList
    };
    view.new_item_json = match view.kind {
        FieldKind::ObjectList => new_item(item).to_string(),
        FieldKind::NumberList => item
            .get("default")
            .map_or_else(|| "0".to_owned(), Value::to_string),
        FieldKind::ColorList => item
            .get("default")
            .map_or_else(|| "\"#ffffff\"".to_owned(), Value::to_string),
        _ => item
            .get("default")
            .map_or_else(|| "\"\"".to_owned(), Value::to_string),
    };
    if view.kind == FieldKind::ObjectList {
        view.choices = value
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, entry)| Choice {
                value: index.to_string(),
                label: row_summary(item, entry, index, ctx),
            })
            .collect();
    }
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

pub fn set_at(mut root: Value, path: &str, value: Value) -> Value {
    let mut slot = &mut root;
    for part in path.split('/') {
        slot = match slot {
            Value::Array(items) => match part.parse::<usize>() {
                Ok(index) if index < items.len() => &mut items[index],
                _ => return root,
            },
            Value::Object(map) => map.entry(part.to_owned()).or_insert(Value::Null),
            other => {
                *other = Value::Object(Map::new());
                match other {
                    Value::Object(map) => map.entry(part.to_owned()).or_insert(Value::Null),
                    _ => return root,
                }
            }
        };
    }
    *slot = value;
    root
}

pub fn parsed_value(json: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|_| Value::String(json.to_owned()))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    proptest! {
        #[test]
        fn a_spot_y_key_swaps_the_trailing_x_for_y(stem in "[a-z][a-zA-Z]{0,8}") {
            prop_assert_eq!(spot_y_key(&format!("{stem}X")), format!("{stem}Y"));
            prop_assert_ne!(spot_y_key(&stem), stem);
        }

        #[test]
        fn setting_an_item_field_changes_only_that_field(
            names in proptest::collection::vec("[a-z]{1,6}", 1..6),
            pick in any::<proptest::sample::Index>(),
            fresh in "[A-Z]{1,6}",
        ) {
            let items: Vec<Value> = names.iter().map(|n| json!({ "name": n, "keep": n })).collect();
            let index = pick.index(items.len());
            let changed = set_at(Value::Array(items.clone()), &format!("{index}/name"), json!(fresh));
            prop_assert_eq!(changed.pointer(&format!("/{index}/name")), Some(&json!(fresh)));
            for (i, item) in items.iter().enumerate() {
                prop_assert_eq!(changed.pointer(&format!("/{i}/keep")), item.get("keep"));
                if i != index {
                    prop_assert_eq!(changed.pointer(&format!("/{i}/name")), item.get("name"));
                }
            }
        }

        #[test]
        fn a_path_past_the_end_changes_nothing(
            names in proptest::collection::vec("[a-z]{1,6}", 0..4),
            extra in 0usize..4,
        ) {
            let items = Value::Array(names.iter().map(|n| json!({ "name": n })).collect());
            let past = names.len() + extra;
            prop_assert_eq!(set_at(items.clone(), &format!("{past}/name"), json!("x")), items);
        }
    }
}
