use std::sync::LazyLock;

use opsis_macros::settings;
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::timeline::{SequencerProps, TimelineTrack, TransitionEase, TransitionRef};
use opsis_types::values::{ClipValue, CompileValue};
use opsis_types::workflow::PortType;
use serde_json::{Map, Value, json};

use crate::card_rows::{frame_secs, sentence, squeezed_too_much, transition_length_text};
use crate::clip::{
    ItemExtra, SequenceSlots, item_clip, scene_sequence, sequence_clips, sequence_cuts,
};
use crate::elements::{ElementKind, ElementMeta, element_meta, elements};
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Outputs, Phase, RowExt, opt, outputs, port, row,
};
use crate::nodes::elements::{element_props, properties_of};
use crate::nodes::groups::{GroupKind, KIND, kind_properties, kind_values};
use crate::params::{ParamSchema, Values, schema_of, whole_number};

pub const TRANSITION_TYPE: &str = "timeline.transition";
pub const TRANSITION_PORT: &str = "transition";
pub const NO_TRANSITION: &str = "none";
pub const DEFAULT_TRANSITION: &str = "fade";
pub const LIGHT_LEAK_CUT: &str = "light-leak";
pub const SPRING_SETTLE: &str = "spring-settle";
const LENGTH: &str = "durationInFrames";
const KEEP_WHOLE: &str = "keepWhole";
const EASING: &str = "easing";
const LEAK_OFFSET: &str = "offset";
const NODE_ONLY_SETTINGS: [&str; 3] = [LENGTH, KEEP_WHOLE, EASING];

pub fn presentations() -> impl Iterator<Item = &'static ElementMeta> {
    elements()
        .iter()
        .filter(|e| e.kind == ElementKind::Presentation)
}

fn kind_label(key: &str) -> String {
    match key {
        NO_TRANSITION => "Cut".to_owned(),
        LIGHT_LEAK_CUT => "Light leak cut".to_owned(),
        _ => sentence(element_meta(key).map_or(key, |m| m.title.as_str())),
    }
}

pub fn transition_title(key: &str) -> String {
    kind_label(key)
}

pub fn transition_kinds() -> Vec<String> {
    std::iter::once(NO_TRANSITION.to_owned())
        .chain(KINDS.iter().map(|k| k.id.to_owned()))
        .collect()
}

static KINDS: LazyLock<Vec<GroupKind>> = LazyLock::new(|| {
    presentations()
        .map(|meta| GroupKind::new(meta.key.as_str(), kind_label(&meta.key), meta.key.as_str()))
        .chain([
            GroupKind::new(LIGHT_LEAK_CUT, "Light leak cut", LIGHT_LEAK_CUT),
            GroupKind::new(SPRING_SETTLE, "Spring settle", SPRING_SETTLE)
                .hide(&["loop", "bgFadeFrames"]),
        ])
        .collect()
});

fn usual_length(kind: &GroupKind) -> u32 {
    kind.meta().default_duration_in_frames
}

#[settings]
struct TransitionFields {
    #[setting(
        "Length",
        range(min = 0, max = 108_000),
        step = 1,
        unit = "frames",
        zero_label = "Its usual length",
        help = "For Spring settle, the pause between one clip leaving and the next landing."
    )]
    duration_in_frames: u32,
    #[setting(
        "Keep every clip whole",
        help = "The transition plays over a still of the last and first frames, so no clip loses screen time. The result gets longer by the transition.",
        advanced
    )]
    keep_whole: bool,
    #[setting(
        "Easing",
        default = TransitionEase::Linear,
        labels = { "linear": "Even", "in": "Slow start", "out": "Slow end", "in-out": "Slow start and end" },
        advanced
    )]
    easing: TransitionEase,
}

fn schema() -> ParamSchema {
    let kinds = &*KINDS;
    let mut properties = kind_properties("Kind", kinds, &[]);
    if let Some(kind) = properties.get_mut(KIND) {
        kind["default"] = Value::from(DEFAULT_TRANSITION);
    }
    let presentations: Vec<&str> = kinds
        .iter()
        .filter(|k| k.meta().kind == ElementKind::Presentation)
        .map(|k| k.id)
        .collect();
    let mut fields = properties_of(&schema_of::<TransitionFields>());
    if let Some(keep) = fields.get_mut(KEEP_WHOLE) {
        keep["x-show-if"] = json!({ KIND: presentations });
    }
    if let Some(easing) = fields.get_mut(EASING) {
        easing["x-show-if"] = json!({ KIND: presentations });
    }
    if let Some(Value::Object(leak)) = properties.get_mut(LIGHT_LEAK_CUT)
        && let Some(Value::Object(leak_props)) = leak.get_mut("properties")
    {
        leak_props.insert(
            LEAK_OFFSET.to_owned(),
            json!({
                "description": "Offset",
                "type": "integer",
                "minimum": -108_000,
                "maximum": 108_000,
                "default": 0,
                "x-unit": "frames",
                "x-help": "Moves the flare from the middle of the cut, later or earlier.",
                "x-advanced": true
            }),
        );
    }
    let mut ordered = Map::new();
    if let Some(kind) = properties.shift_remove(KIND) {
        ordered.insert(KIND.to_owned(), kind);
    }
    ordered.extend(fields);
    ordered.extend(properties);
    ParamSchema::new(json!({ "type": "object", "properties": ordered }))
}

fn picked(values: &Values) -> &'static GroupKind {
    let id = values
        .get(KIND)
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_TRANSITION);
    KINDS
        .iter()
        .find(|k| k.id == id)
        .unwrap_or_else(|| &KINDS[0])
}

fn length_of(kind: &GroupKind, values: &Values) -> u32 {
    match whole_number(values.get(LENGTH)).unwrap_or(0) {
        0 => usual_length(kind),
        length => length,
    }
}

pub fn transition_of(values: &Values) -> TransitionRef {
    let kind = picked(values);
    let meta = kind.meta();
    let mut own = kind_values(kind, values, &[]);
    for key in NODE_ONLY_SETTINGS {
        own.remove(key);
    }
    own.remove(crate::nodes::elements::CLIP_DURATION);
    own.remove(crate::nodes::elements::ENVELOPE);
    let presentation = meta.kind == ElementKind::Presentation;
    let params = if presentation {
        let mut params = meta.fixed_props.clone();
        params.extend(own);
        params
    } else if kind.id == LIGHT_LEAK_CUT {
        let offset = own.remove(LEAK_OFFSET);
        let mut props = element_props(meta, &own);
        if let Some(offset) = offset {
            props.insert(LEAK_OFFSET.to_owned(), offset);
        }
        props
    } else {
        element_props(meta, &own)
    };
    let keep_whole = presentation
        && values
            .get(KEEP_WHOLE)
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let easing: TransitionEase = values
        .get(EASING)
        .filter(|_| presentation)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    TransitionRef {
        kind: kind.id.to_owned(),
        duration_in_frames: length_of(kind, values),
        params: Some(params),
        hold: keep_whole.then_some(true),
        easing: (easing != TransitionEase::Linear).then_some(easing),
    }
}

pub fn default_transition(kind: &str) -> TransitionRef {
    let mut values = Values::new();
    values.insert(KIND.to_owned(), Value::from(kind));
    transition_of(&values)
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

fn card(ctx: &CardCtx<'_, Values>) -> Vec<Option<CardRow>> {
    let kind = picked(ctx.values);
    let transition = transition_of(ctx.values);
    let fps = ctx.fps();
    let keep_whole = transition.hold == Some(true);
    vec![
        Some(row(
            Icon::ArrowRightLeft,
            format!(
                "{} · {}",
                kind.label,
                transition_length_text(transition.duration_in_frames, fps, keep_whole)
            ),
        )),
        speed_row(transition.duration_in_frames, usual_length(kind), fps),
    ]
}

fn transition_node() -> NodeDef {
    NodeDef::new(
        TRANSITION_TYPE,
        "Transition",
        Category::Transitions,
        Phase::Compile,
        schema(),
    )
    .describe("Brings a clip in with a transition wherever it is joined after another clip in a Sequence. Its Transition output sets the joins of a whole Sequence or Repeat.")
    .look(NodeLook::of("lucide:blend").moves(NodeMotion::FadeThrough))
    .inputs(vec![(CLIP_PORT, opt(PortType::Clip, "Clip"))])
    .outputs(vec![
        (CLIP_PORT, port(PortType::Clip, "Clip")),
        (TRANSITION_PORT, port(PortType::Transition, "Transition")),
    ])
    .card(card)
    .compiles(|ctx: &Ctx<'_, Values>| -> Outputs {
        let transition = transition_of(ctx.values);
        let mut out = outputs([(TRANSITION_PORT, CompileValue::Transition(transition.clone()))]);
        if let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) {
            out.insert(
                CLIP_PORT.to_owned(),
                CompileValue::Clip(ClipValue {
                    enter: Some(transition),
                    ..clip.clone()
                }),
            );
        }
        out
    })
}

fn is_kind(join: Option<&TransitionRef>, kind: &str) -> bool {
    join.is_some_and(|t| t.kind == kind)
}

struct Run {
    clip: ClipValue,
    join: Option<TransitionRef>,
    port: String,
}

fn settled_run(
    node_id: &str,
    clips: &[(String, ClipValue)],
    join: &TransitionRef,
    index: usize,
) -> ClipValue {
    let props = join.params.clone().unwrap_or_default();
    let sequencer = SequencerProps {
        component: SPRING_SETTLE.to_owned(),
        props: {
            let mut props = props;
            props.insert("gapFrames".into(), Value::from(join.duration_in_frames));
            props
        },
        backgrounds: vec![None; clips.len()],
    };
    scene_sequence(
        &format!("{node_id}:settle{index}"),
        node_id,
        clips,
        join.duration_in_frames,
        sequencer,
    )
}

fn runs(
    node_id: &str,
    clips: &[ClipValue],
    joins: &[Option<TransitionRef>],
    ports: &[String],
) -> Vec<Run> {
    let join = |i: usize| joins.get(i).and_then(Option::as_ref);
    let port = |i: usize| ports.get(i).cloned().unwrap_or_default();
    let mut out = Vec::new();
    let mut i = 0;
    while i < clips.len() {
        let mut end = i + 1;
        while end < clips.len() && is_kind(join(end), SPRING_SETTLE) {
            end += 1;
        }
        if end - i > 1 {
            let scenes: Vec<(String, ClipValue)> =
                (i..end).map(|k| (port(k), clips[k].clone())).collect();
            let settle = join(i + 1)
                .cloned()
                .unwrap_or_else(|| default_transition(SPRING_SETTLE));
            out.push(Run {
                clip: settled_run(node_id, &scenes, &settle, i),
                join: join(i).cloned(),
                port: String::new(),
            });
        } else {
            out.push(Run {
                clip: clips[i].clone(),
                join: join(i).cloned(),
                port: port(i),
            });
        }
        i = end;
    }
    out
}

fn leak_track(
    node_id: &str,
    leaks: &[(usize, TransitionRef)],
    cuts: &[u32],
    total: u32,
) -> Option<TimelineTrack> {
    let items: Vec<_> = leaks
        .iter()
        .filter_map(|(i, leak)| {
            let cut = *cuts.get(*i)?;
            let mut props = leak.params.clone().unwrap_or_default();
            let offset = props
                .remove(LEAK_OFFSET)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let length = leak.duration_in_frames.min(total).max(1);
            let centred = i64::from(cut) - i64::from(length.div_ceil(2)) + offset;
            let latest = i64::from(total.saturating_sub(length));
            let start = u32::try_from(centred.clamp(0, latest)).unwrap_or(0);
            let mut clip = item_clip(
                &format!("{node_id}:leak{i}"),
                LIGHT_LEAK_CUT,
                props,
                length,
                ItemExtra::default(),
            );
            let mut item = clip.tracks.pop()?.items.pop()?;
            item.base_mut().from = start;
            Some(item)
        })
        .collect();
    (!items.is_empty()).then(|| TimelineTrack {
        id: format!("{node_id}:leaks"),
        items,
        layer: None,
    })
}

pub fn join_clips(
    node_id: &str,
    clips: &[ClipValue],
    joins: &[Option<TransitionRef>],
    ports: &[String],
) -> ClipValue {
    let runs = runs(node_id, clips, joins, ports);
    let leaks: Vec<(usize, TransitionRef)> = runs
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.join
                .clone()
                .filter(|j| j.kind == LIGHT_LEAK_CUT)
                .map(|j| (i, j))
        })
        .collect();
    let placed: Vec<Option<TransitionRef>> = runs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            r.join
                .clone()
                .filter(|j| i > 0 && j.kind != LIGHT_LEAK_CUT && j.kind != SPRING_SETTLE)
        })
        .collect();
    let joined_clips: Vec<ClipValue> = runs.iter().map(|r| r.clip.clone()).collect();
    let run_ports: Vec<String> = runs.iter().map(|r| r.port.clone()).collect();
    let mut joined = sequence_clips(
        node_id,
        &joined_clips,
        &placed,
        SequenceSlots {
            ports: Some(&run_ports),
            slot_node: None,
        },
    );
    let cuts = sequence_cuts(&joined_clips, &placed);
    if let Some(track) = leak_track(node_id, &leaks, &cuts, joined.duration_in_frames) {
        joined.tracks.push(track);
    }
    joined.enter = clips.first().and_then(|c| c.enter.clone());
    joined
}

pub fn joins_for(
    clips: &[ClipValue],
    shared: Option<&TransitionRef>,
) -> Vec<Option<TransitionRef>> {
    clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            if i == 0 {
                return None;
            }
            clip.enter.clone().or_else(|| shared.cloned())
        })
        .collect()
}

pub fn join_names(kinds: &[Option<String>]) -> String {
    let named: Vec<String> = kinds
        .iter()
        .map(|k| k.as_deref().map_or_else(|| "Cut".to_owned(), kind_label))
        .collect();
    let first = named.first().cloned().unwrap_or_default();
    if named.iter().all(|n| *n == first) {
        return format!(
            "{} {}",
            crate::text::plural(numbers::float(named.len()), "join"),
            if first == "Cut" {
                "cut".to_owned()
            } else {
                format!("with {}", first.to_lowercase())
            }
        );
    }
    named.join(", ")
}

pub fn nodes() -> Vec<NodeDef> {
    vec![transition_node()]
}
