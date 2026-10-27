use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::TransitionRef;
use opsis_types::values::{ClipValue, CompileValue};
use opsis_types::workflow::PortType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smart_default::SmartDefault;

use crate::card::{
    cover, cover_row, frame_secs, sentence, squeezed_too_much, transition_length_text,
};
use crate::clip::{ItemExtra, SequenceSlots, item_clip, sequence_clips, stack_clips};
use crate::elements::{ElementKind, ElementMeta, element_meta, elements};
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, Inputs, NodeDef, Outputs, Phase, Ports, RowExt, outputs,
    port, row,
};
use crate::nodes::elements::extended_schema;
use crate::params::{Params, Values, schema_of, whole_number};

pub const NO_TRANSITION: &str = "none";
pub const DEFAULT_TRANSITION: &str = "core/fade";

pub fn transition_title(key: &str) -> String {
    if key == NO_TRANSITION {
        return "Cut".to_owned();
    }
    sentence(element_meta(key).map_or(key, |m| m.title.as_str()))
}

pub fn presentations() -> impl Iterator<Item = &'static ElementMeta> {
    elements()
        .iter()
        .filter(|e| e.kind == ElementKind::Presentation)
}

pub fn transition_kinds() -> Vec<String> {
    std::iter::once(NO_TRANSITION.to_owned())
        .chain(presentations().map(|p| p.key.clone()))
        .collect()
}

pub fn transition_ref(
    kind: &str,
    duration_in_frames: u32,
    keep_whole: bool,
    params: Option<Map<String, Value>>,
) -> TransitionRef {
    TransitionRef {
        kind: kind.to_owned(),
        duration_in_frames,
        params,
        hold: keep_whole.then_some(true),
    }
}

fn ab() -> (Ports, Ports) {
    (
        vec![
            ("a", port(PortType::Clip, "First")),
            ("b", port(PortType::Clip, "Then")),
        ],
        vec![(CLIP_PORT, port(PortType::Clip, "Clip"))],
    )
}

fn transition_node_type(key: &str) -> String {
    format!("transition.{}", key.replacen('/', ".", 1))
}

fn join_clips(inputs: &Inputs, join: impl FnOnce(&ClipValue, &ClipValue) -> ClipValue) -> Outputs {
    let a = inputs.get("a").and_then(CompileValue::as_clip);
    let b = inputs.get("b").and_then(CompileValue::as_clip);
    match (a, b) {
        (Some(a), Some(b)) => outputs([(CLIP_PORT, CompileValue::Clip(join(a, b)))]),
        (Some(one), None) | (None, Some(one)) => {
            outputs([(CLIP_PORT, CompileValue::Clip(one.clone()))])
        }
        (None, None) => Outputs::new(),
    }
}

fn speed_row(length: u32, designed: u32, fps: u32) -> Option<CardRow> {
    let (length, designed) = (f64::from(length), f64::from(designed));
    if length <= 0.0 || !squeezed_too_much(length, designed) {
        return None;
    }
    Some(
        row(
            Icon::FastForward,
            format!("{:.1}× faster than designed", designed / length),
        )
        .tone(Tone::Warn)
        .titled(format!(
            "This transition is made for about {}. Squeezed this much it can look like a cut.",
            frame_secs(designed, fps)
        )),
    )
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct KeepWholeField {
    #[schemars(
        description = "Keep both clips whole",
        extend(
            "x-help" = "The transition plays over a still of the last and first frames, so neither clip loses screen time. The result gets longer by the transition."
        )
    )]
    keep_whole: bool,
}

fn length_field(default: u32) -> Value {
    serde_json::json!({
        "description": "Length",
        "type": "integer",
        "minimum": 1,
        "maximum": crate::limits::MAX_FRAMES,
        "default": default,
        "x-step": 1,
        "x-unit": "frames"
    })
}

fn presentation_node(meta: &'static ElementMeta) -> NodeDef {
    let mut extra = Map::new();
    extra.insert(
        "durationInFrames".into(),
        length_field(meta.default_duration_in_frames),
    );
    if let Some(Value::Object(props)) = schema_of::<KeepWholeField>().get("properties") {
        extra.extend(props.clone());
    }
    let (inputs, outs) = ab();
    let lowered = sentence(&meta.title).to_lowercase();
    NodeDef::new(transition_node_type(&meta.key), sentence(&meta.title), Category::Transitions, Phase::Compile, extended_schema(&meta.schema, extra))
        .element(meta)
        .describe(format!(
            "Plays the first clip, then the second, joined with a {lowered} transition. Tip: cut where the picture is still."
        ))
        .inputs(inputs)
        .outputs(outs)
        .card(move |ctx: &CardCtx<'_, Values>| {
            let length = whole_number(ctx.values.get("durationInFrames")).unwrap_or(0);
            let keep_whole = ctx.values.get("keepWhole").and_then(Value::as_bool).unwrap_or(false);
            let fps = ctx.fps();
            let frames = |p: &str| ctx.inputs.frames.get(p).copied();
            vec![
                Some(row(Icon::ArrowRightLeft, transition_length_text(length, fps, keep_whole))),
                speed_row(length, meta.default_duration_in_frames, fps),
                if keep_whole { None } else { cover_row(cover(length, &[frames("a"), frames("b")]), fps) },
            ]
        })
        .compiles(move |ctx: &Ctx<'_, Values>| {
            let mut rest = ctx.values.clone();
            let length = whole_number(rest.remove("durationInFrames").as_ref()).unwrap_or(meta.default_duration_in_frames);
            let keep_whole = rest.remove("keepWhole").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut params = meta.fixed_props.clone();
            params.extend(rest);
            join_clips(ctx.inputs, |a, b| {
                let ports = ["a".to_owned(), "b".to_owned()];
                let enter = transition_ref(&meta.key, length, keep_whole, Some(params));
                sequence_clips(ctx.node_id, &[a.clone(), b.clone()], &[None, Some(enter)], SequenceSlots { ports: Some(&ports), slot_node: None })
            })
        })
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LightLeakCut {
    #[default = 30]
    #[schemars(description = "Length", range(min = 1, max = 108000), extend("x-step" = 1, "x-unit" = "frames"))]
    pub duration_in_frames: u32,
    #[default = 3]
    #[schemars(description = "Variation", range(min = 0, max = 100))]
    pub seed: u32,
    #[schemars(description = "Colour shift", range(min = 0, max = 360), extend("x-unit" = "°"))]
    pub hue_shift: f64,
}

impl Params for LightLeakCut {}

fn light_leak() -> NodeDef {
    let (inputs, outs) = ab();
    NodeDef::of::<LightLeakCut>(
        "transition.lightLeak",
        "Light leak cut",
        Category::Transitions,
        Phase::Compile,
    )
    .look(NodeLook::of("lucide:sun-dim").moves(NodeMotion::Flare))
    .describe("Cuts between two clips under a warm light flare.")
    .inputs(inputs)
    .outputs(outs)
    .card(|ctx: &CardCtx<'_, LightLeakCut>| {
        let p = &ctx.params;
        let a = ctx.inputs.frames.get("a").copied().unwrap_or(0);
        let b = ctx.inputs.frames.get("b").copied().unwrap_or(0);
        let film = a + b;
        let squeezed = a != 0 && b != 0 && p.duration_in_frames > film;
        vec![
            Some(row(Icon::Sun, frame_secs(p.duration_in_frames, ctx.fps()))),
            squeezed.then(|| {
                row(
                    Icon::TriangleAlert,
                    format!(
                        "Longer than both clips, so it plays for {}",
                        frame_secs(film, ctx.fps())
                    ),
                )
                .tone(Tone::Warn)
            }),
        ]
    })
    .compiles(|ctx: &Ctx<'_, LightLeakCut>| {
        let p = &ctx.params;
        join_clips(ctx.inputs, |a, b| {
            let ports = ["a".to_owned(), "b".to_owned()];
            let cut_id = format!("{}:cut", ctx.node_id);
            let cut = sequence_clips(
                &cut_id,
                &[a.clone(), b.clone()],
                &[],
                SequenceSlots {
                    ports: Some(&ports),
                    slot_node: Some(ctx.node_id),
                },
            );
            let length = p.duration_in_frames.min(cut.duration_in_frames);
            let mut props = Map::new();
            props.insert("seed".into(), Value::from(p.seed));
            props.insert("hueShift".into(), Value::from(p.hue_shift));
            let leak = item_clip(
                &format!("{}:leak", ctx.node_id),
                "core/light-leak",
                props,
                length,
                ItemExtra::default(),
            );
            let centred_on_cut = a.duration_in_frames.saturating_sub(length.div_ceil(2));
            let start = centred_on_cut.min(cut.duration_in_frames - length);
            stack_clips(ctx.node_id, &[cut, leak], &[0, start], None, None)
        })
    })
}

pub fn nodes() -> Vec<NodeDef> {
    presentations()
        .map(presentation_node)
        .chain(std::iter::once(light_leak()))
        .collect()
}
