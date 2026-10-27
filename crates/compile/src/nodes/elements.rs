use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::timeline::{
    AudioItem, AudioRole, CameraPath, EditHandles, ElementRef, GroupProps, ParamRef, ParamUnit,
    SequencerProps, VisualEnvelope,
};
use opsis_types::values::{ClipValue, CompileValue};
use opsis_types::workflow::{PortType, WorkflowSettings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use smart_default::SmartDefault;

use crate::asset_ref::{kind_from_path, project_ref};
use crate::card::{file_row, frame_secs, quoted, sentence};
use crate::clip::{
    EMPTY_CLIP_FRAMES, ItemExtra, empty_clip, item_clip, scene_sequence, with_handles, wrap_clip,
};
use crate::elements::core::CAPTIONS_ELEMENT;
use crate::elements::introspect::{
    frames_from_seconds, is_color_schema, map_leaves, widget_keys, widget_of,
};
use crate::elements::lengths::{animation_length, fits_its_animation};
use crate::elements::prepared::prepare_props;
use crate::elements::{ElementCategory, ElementKind, ElementMeta, element_node_type, elements};
use crate::fonts::font_prop;
use crate::gradient::{GradientKind, parse_gradient};
use crate::limits::MAX_FRAMES;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Outputs, Phase, RowExt, outputs, port, row,
};
use crate::params::{ParamSchema, Values, schema_of, whole_number};
use crate::ports::{IN_PORTS, clip_ports, connected_clips, scene_label};
use crate::text::plural;

const TEXT_KEYS: [&str; 9] = [
    "title", "text", "headline", "heading", "label", "name", "words", "quote", "message",
];
const SHORT_ELEMENT_RATIO: f64 = 0.6;
const CLIP_DURATION: &str = "clipDuration";
const ENVELOPE: &str = "envelope";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct EnvelopeField {
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change opacity, position and size over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    envelope: VisualEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SceneSlot {
    In1,
    In2,
    In3,
    In4,
    In5,
    In6,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SceneBackground {
    #[default(SceneSlot::In1)]
    #[schemars(
        description = "Scene",
        extend("x-labels" = { "in1": "Scene 1", "in2": "Scene 2", "in3": "Scene 3", "in4": "Scene 4", "in5": "Scene 5", "in6": "Scene 6" })
    )]
    pub scene: SceneSlot,
    #[default = "#0b0b10"]
    #[schemars(description = "Colour", extend("x-widget" = "color"))]
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct SequencerFields {
    #[default = 1]
    #[schemars(
        description = "Pause between scenes",
        range(min = 0, max = 120),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "How long the stage stays empty between one scene leaving and the next landing."
        )
    )]
    gap_frames: u32,
    #[schemars(
        description = "Scene backgrounds",
        extend("x-help" = "A colour behind a scene. It fades into the next scene's colour.", "x-advanced" = true)
    )]
    backgrounds: Vec<SceneBackground>,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change opacity, position and size over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    envelope: VisualEnvelope,
}

fn node_category(category: ElementCategory) -> Category {
    match category {
        ElementCategory::Core
        | ElementCategory::Ui
        | ElementCategory::Data
        | ElementCategory::Scene
        | ElementCategory::Device => Category::Elements,
        ElementCategory::Text | ElementCategory::Overlay => Category::TextAndOverlays,
        ElementCategory::Background => Category::Backgrounds,
        ElementCategory::Effect => Category::Effects,
        ElementCategory::Transition => Category::Transitions,
    }
}

fn properties_of(schema: &Value) -> Map<String, Value> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

pub fn extended_schema(base: &ParamSchema, extra: Map<String, Value>) -> ParamSchema {
    let mut schema = base.json().clone();
    let mut properties = properties_of(&schema);
    properties.extend(extra);
    schema["properties"] = Value::Object(properties);
    schema["type"] = Value::from("object");
    ParamSchema::new(schema)
}

fn clip_duration_field(meta: &ElementMeta) -> Value {
    let wrapper = meta.kind == ElementKind::Wrapper;
    let fits = fits_its_animation(meta);
    let open = wrapper || fits;
    let mut field = json!({
        "description": "Length",
        "type": "integer",
        "minimum": if open { 0 } else { 1 },
        "maximum": MAX_FRAMES,
        "default": if open { 0 } else { meta.default_duration_in_frames },
        "x-unit": "frames"
    });
    if wrapper {
        field["x-zero-label"] = Value::from("Match the clip");
    }
    if meta.optional_clip {
        field["x-unconnected-zero-label"] = Value::from("Usual length");
    }
    if fits {
        field["x-zero-label"] = Value::from("Fit the animation");
    }
    field
}

fn element_schema(meta: &ElementMeta) -> ParamSchema {
    let mut extra = Map::new();
    extra.insert(CLIP_DURATION.to_owned(), clip_duration_field(meta));
    extra.extend(properties_of(&schema_of::<EnvelopeField>()));
    extended_schema(&meta.schema, extra)
}

fn setting_value(field: &Value, value: &Value) -> Option<Value> {
    if field.get("x-zero-label").is_some() && value.as_str() == Some("") {
        return None;
    }
    if widget_of(field) == Some("asset")
        && let Some(path) = value.as_str()
    {
        return Some(Value::from(project_ref(path)));
    }
    frames_from_seconds(field, value)
}

pub fn element_props(meta: &ElementMeta, params: &Values) -> Values {
    let mapped = match map_leaves(
        meta.schema.json(),
        &Value::Object(params.clone()),
        &setting_value,
    ) {
        Some(Value::Object(map)) => map,
        _ => Values::new(),
    };
    let mut props = meta.fixed_props.clone();
    props.extend(mapped);
    for (prop, setting) in &meta.kind_props {
        let path = props.get(setting).and_then(Value::as_str).map(str::trim);
        if let Some(path) = path.filter(|p| !p.is_empty()) {
            let kind = serde_json::to_value(kind_from_path(path)).unwrap_or(Value::Null);
            props.insert(prop.clone(), kind);
        }
    }
    for key in widget_keys(&meta.schema, "font") {
        match props.get(&key).and_then(Value::as_str).and_then(font_prop) {
            Some(stack) => props.insert(key, Value::from(stack)),
            None => props.remove(&key),
        };
    }
    props
}

pub fn sound_volume_key(sound_key: &str) -> String {
    format!("{sound_key}Volume")
}

fn element_sounds(
    meta: &ElementMeta,
    node_id: &str,
    props: &mut Values,
    frames: u32,
) -> Vec<AudioItem> {
    widget_keys(&meta.schema, "sound")
        .into_iter()
        .filter_map(|key| {
            let volume_key = sound_volume_key(&key);
            let src = props.remove(&key);
            let volume = props.remove(&volume_key).and_then(|v| v.as_f64());
            let src = src
                .as_ref()
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())?;
            let mut item = AudioItem::new(format!("{node_id}:{key}"), project_ref(src), 0);
            item.duration_in_frames = Some(frames);
            item.volume = Some(volume.unwrap_or(1.0));
            item.node_id = Some(node_id.to_owned());
            item.channel = Some(node_id.to_owned());
            item.role = Some(AudioRole::Music);
            item.handles = Some(EditHandles {
                volume: Some(ParamRef {
                    node_id: node_id.to_owned(),
                    param: volume_key,
                    unit: ParamUnit::Number,
                    scale: None,
                    index: None,
                    linked: None,
                }),
                ..EditHandles::default()
            });
            Some(item)
        })
        .collect()
}

fn active_camera(params: &Values) -> Option<CameraPath> {
    let path: CameraPath = serde_json::from_value(params.get("camera")?.clone()).ok()?;
    if path.keys.is_empty() {
        return None;
    }
    let preset = params.get("motion").or_else(|| params.get("move"));
    match preset {
        None => Some(path),
        Some(p) if p.as_str() == Some("keyframes") => Some(path),
        Some(_) => None,
    }
}

fn split_timing(values: &Values) -> (u32, VisualEnvelope, Values) {
    let mut params = values.clone();
    let clip_duration = whole_number(params.remove(CLIP_DURATION).as_ref()).unwrap_or(0);
    let envelope = params
        .remove(ENVELOPE)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    (clip_duration, envelope, params)
}

fn trim_end_handles(node_id: &str) -> EditHandles {
    EditHandles {
        trim_end: Some(ParamRef {
            node_id: node_id.to_owned(),
            param: CLIP_DURATION.to_owned(),
            unit: ParamUnit::Frames,
            scale: None,
            index: None,
            linked: None,
        }),
        ..EditHandles::default()
    }
}

fn compile_element(
    meta: &ElementMeta,
    node_id: &str,
    values: &Values,
    input: Option<&ClipValue>,
    settings: &WorkflowSettings,
) -> ClipValue {
    let fps = settings.fps;
    let (clip_duration, envelope, params) = split_timing(values);
    let mut props = element_props(meta, &params);
    prepare_props(&meta.key, &mut props, &settings.fonts);
    let handles = trim_end_handles(node_id);
    let camera = active_camera(&params);
    match input.filter(|_| meta.kind == ElementKind::Wrapper) {
        None => {
            let frames = if clip_duration != 0 {
                clip_duration
            } else {
                animation_length(meta, &params, fps)
            };
            let audio = element_sounds(meta, node_id, &mut props, frames);
            let extra = ItemExtra {
                handles: Some(handles),
                envelope: Some(envelope),
                audio,
                camera,
                design: meta.design.clone(),
                recording: None,
            };
            item_clip(node_id, &meta.key, props, frames, extra)
        }
        Some(input) => {
            let group = GroupProps {
                wrapper: Some(ElementRef {
                    component: meta.key.clone(),
                    props,
                }),
                design: meta
                    .design
                    .as_ref()
                    .map(|d| opsis_types::actions::Viewport {
                        width: d.width,
                        height: d.height,
                    }),
                ..GroupProps::default()
            };
            let duration = if clip_duration != 0 {
                clip_duration
            } else {
                input.duration_in_frames
            };
            let wrapped = wrap_clip(input, node_id, group, Some(duration));
            with_handles(wrapped, handles, Some(envelope), camera)
        }
    }
}

fn shown(field: &Value, params: &Values) -> bool {
    let Some(show_if) = field.get("x-show-if").and_then(Value::as_object) else {
        return true;
    };
    show_if.iter().all(|(key, allowed)| {
        let current = match params.get(key) {
            None | Some(Value::Null) => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.as_f64().map(|n| n.to_string()).unwrap_or_default(),
            Some(other) => other.to_string(),
        };
        allowed
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(current.as_str())))
    })
}

fn palette_row(colors: &Value) -> Option<CardRow> {
    let list = colors.as_array().filter(|l| !l.is_empty())?;
    let colors: Option<Vec<String>> = list.iter().map(|c| c.as_str().map(str::to_owned)).collect();
    let colors = colors?;
    Some(
        row(Icon::Palette, plural(colors.len() as f64, "colour"))
            .titled(colors.join(", "))
            .swatches(colors),
    )
}

fn gradient_row(value: &Value) -> Option<CardRow> {
    let text = value.as_str()?;
    let gradient = parse_gradient(text)?;
    let label = if gradient.kind == GradientKind::Linear {
        "Straight gradient"
    } else {
        "Round gradient"
    };
    Some(
        row(Icon::Blend, label)
            .titled(text)
            .swatches(gradient.stops.into_iter().map(|s| s.color).collect()),
    )
}

#[derive(Deserialize)]
struct ThemeValue {
    mode: String,
    colors: Map<String, Value>,
}

fn theme_row(value: &Value) -> Option<CardRow> {
    let theme: ThemeValue = serde_json::from_value(value.clone()).ok()?;
    if theme.mode != "light" && theme.mode != "dark" || !theme.colors.values().all(Value::is_string)
    {
        return None;
    }
    let changed: Vec<String> = theme
        .colors
        .values()
        .filter_map(Value::as_str)
        .filter(|c| !c.trim().is_empty())
        .map(str::to_owned)
        .collect();
    let light = theme.mode == "light";
    let mode = if light { "Light theme" } else { "Dark theme" };
    let icon = if light { Icon::Sun } else { Icon::Moon };
    Some(if changed.is_empty() {
        row(icon, mode)
    } else {
        row(
            icon,
            format!(
                "{mode} · {} changed",
                plural(changed.len() as f64, "colour")
            ),
        )
        .swatches(changed)
    })
}

fn look_row(meta: &ElementMeta, params: &Values) -> Option<CardRow> {
    meta.schema.keys().iter().find_map(|key| {
        let field = meta.schema.field(key)?;
        if !shown(field, params) {
            return None;
        }
        let value = params.get(key).unwrap_or(&Value::Null);
        let palette = field.get("type") == Some(&Value::from("array"))
            && field.get("items").is_some_and(is_color_schema);
        match widget_of(field) {
            _ if palette => palette_row(value),
            Some("gradient") => gradient_row(value),
            Some("theme") => theme_row(value),
            _ => None,
        }
    })
}

fn duration_row(meta: &ElementMeta, params: &Values, fps: u32, clip_connected: bool) -> CardRow {
    let duration = whole_number(params.get(CLIP_DURATION)).unwrap_or(0);
    if meta.kind == ElementKind::Wrapper && (clip_connected || !meta.optional_clip) {
        return if duration != 0 {
            row(Icon::Clock, frame_secs(duration, fps))
        } else {
            row(Icon::Clock, "Matches the clip").tone(Tone::Muted)
        };
    }
    let usual = animation_length(meta, params, fps);
    if duration > 0 && f64::from(duration) < SHORT_ELEMENT_RATIO * f64::from(usual) {
        return row(
            Icon::Clock,
            format!(
                "{} · usually {}",
                frame_secs(duration, fps),
                frame_secs(usual, fps)
            ),
        )
        .tone(Tone::Warn)
        .titled("Shorter than this element usually runs, so its animation may be cut off.");
    }
    row(
        Icon::Clock,
        frame_secs(if duration != 0 { duration } else { usual }, fps),
    )
}

fn element_card(
    meta: &ElementMeta,
    params: &Values,
    fps: u32,
    clip_connected: bool,
) -> Vec<CardRow> {
    let text_of = |k: &str| {
        params
            .get(k)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
    };
    let title = TEXT_KEYS
        .iter()
        .filter(|k| meta.schema.field(k).is_some())
        .find_map(|k| text_of(k));
    let file = widget_keys(&meta.schema, "asset")
        .iter()
        .find_map(|k| text_of(k));
    let mut rows = Vec::new();
    if let Some(title) = title {
        rows.push(row(Icon::Type, quoted(&title)));
    }
    if let Some(file) = file {
        rows.push(file_row(Icon::File, &file));
    }
    if let Some(look) = look_row(meta, params) {
        rows.push(look);
    }
    rows.push(duration_row(meta, params, fps, clip_connected));
    rows
}

fn element_node(meta: &'static ElementMeta) -> NodeDef {
    let wrapper = meta.kind == ElementKind::Wrapper;
    let inputs = if wrapper {
        let clip = port(PortType::Clip, "Clip");
        vec![(
            CLIP_PORT,
            if meta.optional_clip {
                clip.optional()
            } else {
                clip
            },
        )]
    } else {
        Vec::new()
    };
    let mut def = NodeDef::new(
        element_node_type(&meta.key),
        sentence(&meta.title),
        node_category(meta.category),
        Phase::Compile,
        element_schema(meta),
    )
    .element(meta)
    .inputs(inputs)
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(move |ctx: &CardCtx<'_, Values>| {
        element_card(meta, ctx.values, ctx.fps(), ctx.connected(CLIP_PORT))
            .into_iter()
            .map(Some)
            .collect()
    })
    .compiles(move |ctx: &Ctx<'_, Values>| {
        let input = if wrapper {
            ctx.input(CLIP_PORT).and_then(CompileValue::as_clip)
        } else {
            None
        };
        outputs([(
            CLIP_PORT,
            CompileValue::Clip(compile_element(
                meta,
                ctx.node_id,
                ctx.values,
                input,
                ctx.settings,
            )),
        )])
    });
    if let Some(description) = &meta.description {
        def = def.describe(description.clone());
    }
    def
}

fn sequencer_card(ctx: &CardCtx<'_, Values>) -> Vec<Option<CardRow>> {
    let ports: Vec<&str> = IN_PORTS
        .iter()
        .copied()
        .filter(|p| ctx.connected(p))
        .collect();
    let items: Vec<usize> = ports
        .iter()
        .map(|p| ctx.inputs.items.get(*p).copied().unwrap_or(1))
        .collect();
    let gap = ctx
        .values
        .get("gapFrames")
        .and_then(|gap| whole_number(Some(gap)))
        .unwrap_or(0);
    vec![
        Some(
            row(Icon::Clapperboard, plural(ports.len() as f64, "scene")).warn_if(ports.is_empty()),
        ),
        items.iter().any(|n| *n > 1).then(|| {
            let shown: Vec<String> = items.iter().map(|n| (*n).to_string()).collect();
            row(
                Icon::Layers,
                format!("Items per scene: {}", shown.join(" · ")),
            )
        }),
        (gap > 0).then(|| {
            row(
                Icon::Timer,
                format!("{} between scenes", frame_secs(gap, ctx.fps())),
            )
        }),
        ctx.inputs
            .clip_frames
            .filter(|f| *f != 0)
            .map(|f| row(Icon::Clock, frame_secs(f, ctx.fps()))),
    ]
}

fn compile_sequencer(
    meta: &ElementMeta,
    node_id: &str,
    values: &Values,
    ctx_inputs: &crate::node::Inputs,
) -> ClipValue {
    let mut params = values.clone();
    let gap = whole_number(params.remove("gapFrames").as_ref()).unwrap_or(0);
    let backgrounds: Vec<SceneBackground> = params
        .remove("backgrounds")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let envelope: VisualEnvelope = params
        .remove(ENVELOPE)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let scenes = connected_clips(ctx_inputs);
    if scenes.is_empty() {
        return empty_clip(EMPTY_CLIP_FRAMES);
    }
    let color_of = |handle: &str| {
        backgrounds
            .iter()
            .rev()
            .find(|b| {
                serde_json::to_value(b.scene)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .as_deref()
                    == Some(handle)
            })
            .map(|b| b.color.clone())
    };
    let mut props = element_props(meta, &params);
    props.insert("gapFrames".into(), Value::from(gap));
    let sequencer = SequencerProps {
        component: meta.key.clone(),
        props,
        backgrounds: scenes.iter().map(|(handle, _)| color_of(handle)).collect(),
    };
    let scenes: Vec<(String, ClipValue)> = scenes
        .into_iter()
        .map(|(p, c)| (p.to_owned(), c.clone()))
        .collect();
    let clip = scene_sequence(node_id, &scenes, gap, sequencer);
    with_handles(clip, EditHandles::default(), Some(envelope), None)
}

fn sequencer_node(meta: &'static ElementMeta) -> NodeDef {
    let schema = extended_schema(&meta.schema, properties_of(&schema_of::<SequencerFields>()));
    let mut def = NodeDef::new(
        element_node_type(&meta.key),
        sentence(&meta.title),
        Category::Timeline,
        Phase::Compile,
        schema,
    )
    .element(meta)
    .inputs(clip_ports(scene_label))
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(sequencer_card)
    .compiles(move |ctx: &Ctx<'_, Values>| -> Outputs {
        outputs([(
            CLIP_PORT,
            CompileValue::Clip(compile_sequencer(meta, ctx.node_id, ctx.values, ctx.inputs)),
        )])
    });
    if let Some(description) = &meta.description {
        def = def.describe(description.clone());
    }
    def
}

pub fn nodes() -> Vec<NodeDef> {
    elements()
        .iter()
        .filter(|e| e.kind != ElementKind::Presentation && e.key != CAPTIONS_ELEMENT)
        .map(|meta| {
            if meta.kind == ElementKind::Sequencer {
                sequencer_node(meta)
            } else {
                element_node(meta)
            }
        })
        .collect()
}
