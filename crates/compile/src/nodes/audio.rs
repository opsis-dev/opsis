use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::timeline::{
    Anchor, AudioEnvelope, AudioItem, AudioRole, EditHandles, LinkedParam, ParamRef, ParamUnit,
};
use opsis_types::values::{AudioValue, ClipValue, CompileValue, MediaKind, MediaValue};
use opsis_types::workflow::{PortType, WorkflowSettings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::asset_ref::{base_name, project_ref};
use crate::card::{LABELLED_TEXT_MAX, named_file_row, quote};
use crate::clip::clip_to_spec;
use crate::envelope::has_audio_keys;
use crate::node::{
    CLIP_PORT, CardCtx, CardInputs, Category, Ctx, NodeDef, Outputs, Phase, RowExt, many, missing,
    opt, outputs, port, row, with_clip,
};
use crate::params::Params;
use crate::sounds::builtin_sound;
use crate::text::{format_duration, percent_text, plural, sign_of};
use crate::time::{ms_to_frames, ms_to_frames_from, ms_to_signed_frames};
use crate::timeline::flatten::flatten_spec;

const MEDIA_PORT: &str = "media";
const AUDIO_PORT: &str = "audio";
const SOUNDS_PORT: &str = "sounds";

fn param_ref(node_id: &str, param: &str, unit: ParamUnit) -> ParamRef {
    ParamRef {
        node_id: node_id.to_owned(),
        param: param.to_owned(),
        unit,
        scale: None,
        index: None,
        linked: None,
    }
}

fn silent_audio() -> Outputs {
    outputs([(
        AUDIO_PORT,
        CompileValue::Audio(AudioValue {
            items: Vec::new(),
            duration_in_frames: None,
        }),
    )])
}

fn is_silent(media: &MediaValue) -> bool {
    media.kind == MediaKind::Image || media.has_audio == Some(false)
}

fn with_source<P>(
    ctx: &Ctx<'_, P>,
    path: &str,
    build: impl FnOnce(String, Option<&MediaValue>) -> Outputs,
) -> Outputs {
    let media = ctx.input(MEDIA_PORT).and_then(CompileValue::as_media);
    let silent = media.is_some_and(is_silent);
    let src = media
        .and_then(|m| m.src.clone())
        .or_else(|| (!path.is_empty()).then(|| project_ref(path)));
    match src {
        Some(src) if !silent => build(src, media),
        _ => silent_audio(),
    }
}

struct Timing {
    from: f64,
    duration_in_frames: Option<f64>,
    looped: Option<bool>,
    trim_before: Option<f64>,
    fade_in_frames: Option<f64>,
    fade_out_frames: Option<f64>,
    anchor: Option<Anchor>,
    length_frames: Option<f64>,
    keep_music: Option<bool>,
}

fn sound_item(
    node_id: &str,
    src: String,
    role: AudioRole,
    volume: f64,
    envelope: &AudioEnvelope,
    timing: Timing,
    handles: EditHandles,
) -> AudioItem {
    let mut item = AudioItem::new(node_id, src, timing.from);
    item.duration_in_frames = timing.duration_in_frames;
    item.looped = timing.looped;
    item.trim_before = timing.trim_before;
    item.fade_in_frames = timing.fade_in_frames;
    item.fade_out_frames = timing.fade_out_frames;
    item.anchor = timing.anchor;
    item.length_frames = timing.length_frames;
    item.keep_music = timing.keep_music;
    item.volume = Some(volume);
    item.node_id = Some(node_id.to_owned());
    item.channel = Some(node_id.to_owned());
    item.role = Some(role);
    item.envelope = has_audio_keys(Some(envelope)).then(|| envelope.clone());
    item.handles = Some(EditHandles {
        volume: Some(param_ref(node_id, "volume", ParamUnit::Number)),
        ..handles
    });
    item
}

fn frames_if_set(duration_ms: f64, fps: f64) -> Option<f64> {
    (duration_ms != 0.0).then(|| ms_to_frames(duration_ms, fps))
}

fn offset_label(offset_ms: i64) -> String {
    if offset_ms == 0 {
        return String::new();
    }
    format!(
        " {}{}",
        sign_of(offset_ms as f64, true),
        format_duration(offset_ms.abs() as f64)
    )
}

fn source_row(icon: Icon, src: &str, inputs: &CardInputs, empty: &str) -> CardRow {
    if inputs
        .media
        .get(MEDIA_PORT)
        .is_some_and(|m| m.kind == MediaKind::Image || m.has_audio == Some(false))
    {
        return missing(icon, "The connected file has no sound");
    }
    let chosen = (!src.is_empty())
        .then(|| builtin_sound(src).map_or_else(|| base_name(src), |s| s.label.clone()));
    if inputs.is_connected(MEDIA_PORT) {
        return match chosen {
            Some(chosen) => row(icon, format!("Plays the connected file, not {}", quote(&chosen, LABELLED_TEXT_MAX)))
                .tone(Tone::Warn)
                .titled("A connected file takes the place of the chosen sound. Disconnect it to use the sound."),
            None => row(icon, "From the connected file"),
        };
    }
    match chosen {
        Some(chosen) => named_file_row(icon, src, &chosen),
        None => missing(icon, empty),
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Music {
    #[schemars(description = "Song", extend("x-widget" = "asset", "x-accept" = "audio/*"))]
    pub src: String,
    #[default = 0.5]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[default = true]
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[schemars(description = "Starts at", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub start_ms: i64,
    #[schemars(
        description = "Start the song from",
        range(min = 0, max = 600000),
        extend("x-step" = 50, "x-unit" = "ms", "x-help" = "Skips the start of the song.")
    )]
    pub trim_ms: i64,
    #[schemars(
        description = "Play for",
        range(min = 0, max = 3600000),
        extend("x-step" = 50, "x-unit" = "ms", "x-zero-label" = "To the end", "x-advanced" = true)
    )]
    pub length_ms: i64,
    #[default = 800]
    #[schemars(description = "Fade in", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_in_ms: i64,
    #[default = 1500]
    #[schemars(description = "Fade out", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_out_ms: i64,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change volume and left–right balance over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    pub envelope: AudioEnvelope,
}

impl Params for Music {}

fn music_handles(node_id: &str) -> EditHandles {
    let mut trim_start = param_ref(node_id, "trimMs", ParamUnit::Ms);
    trim_start.linked = Some(vec![
        LinkedParam {
            param: "startMs".into(),
            sign: 1.0,
            if_non_zero: None,
        },
        LinkedParam {
            param: "lengthMs".into(),
            sign: -1.0,
            if_non_zero: Some(true),
        },
    ]);
    EditHandles {
        move_to: Some(param_ref(node_id, "startMs", ParamUnit::Ms)),
        trim_start: Some(trim_start),
        trim_end: Some(param_ref(node_id, "lengthMs", ParamUnit::Ms)),
        ..EditHandles::default()
    }
}

fn music() -> NodeDef {
    NodeDef::of::<Music>("audio.music", "Music", Category::Audio, Phase::Compile)
        .describe("Background music under the video, with fades.")
        .inputs(vec![(MEDIA_PORT, opt(PortType::Media, "File"))])
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, Music>| {
            let p = &ctx.params;
            vec![
                Some(source_row(Icon::Music, &p.src, ctx.inputs, "Choose a song")),
                Some(row(Icon::Volume2, percent_text(p.volume, f64::INFINITY))),
                p.r#loop.then(|| row(Icon::Repeat, "Loops")),
                (p.length_ms > 0).then(|| {
                    row(
                        Icon::Clock,
                        format!("Plays for {}", format_duration(p.length_ms as f64)),
                    )
                }),
            ]
        })
        .compiles(|ctx: &Ctx<'_, Music>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            with_source(ctx, &p.src, |src, media| {
                let timing = Timing {
                    from: ms_to_frames_from(p.start_ms as f64, fps, 0.0),
                    duration_in_frames: frames_if_set(p.length_ms as f64, fps),
                    looped: Some(p.r#loop),
                    trim_before: frames_if_set(p.trim_ms as f64, fps),
                    fade_in_frames: frames_if_set(p.fade_in_ms as f64, fps),
                    fade_out_frames: frames_if_set(p.fade_out_ms as f64, fps),
                    anchor: None,
                    length_frames: None,
                    keep_music: None,
                };
                let item = sound_item(
                    ctx.node_id,
                    src,
                    AudioRole::Music,
                    p.volume,
                    &p.envelope,
                    timing,
                    music_handles(ctx.node_id),
                );
                let duration = frames_if_set(media.and_then(|m| m.duration_ms).unwrap_or(0.0), fps);
                outputs([(
                    AUDIO_PORT,
                    CompileValue::Audio(AudioValue {
                        items: vec![item],
                        duration_in_frames: duration,
                    }),
                )])
            })
        })
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SoundEffect {
    #[schemars(description = "Sound", extend("x-widget" = "sound", "x-accept" = "audio/*", "x-length-key" = "srcMs"))]
    pub src: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Sound length", extend("exclusiveMinimum" = 0, "x-hidden" = true))]
    pub src_ms: Option<f64>,
    #[schemars(
        description = "On an action",
        extend(
            "x-widget" = "action",
            "x-help" = "Plays when this browser action happens in the clip the sound is added to, even after you record again or edit the clip."
        )
    )]
    pub at_action: String,
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms", "x-show-if" = { "atAction": [""] }))]
    pub at_ms: i64,
    #[schemars(
        description = "From the action",
        range(min = -10000, max = 10000),
        extend(
            "x-unit" = "ms",
            "x-step" = 50,
            "x-help" = "Negative plays it before the action, for sounds that build up to it.",
            "x-show-if" = { "atAction": ["*"] }
        )
    )]
    pub offset_ms: i64,
    #[default = 1.0]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[schemars(
        description = "Keep music up",
        extend("x-help" = "Music set to lower under other sounds stays at full volume for this one.", "x-advanced" = true)
    )]
    pub keep_music: bool,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change volume and left–right balance over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    pub envelope: AudioEnvelope,
}

impl Params for SoundEffect {}

fn sound_effect() -> NodeDef {
    NodeDef::of::<SoundEffect>("audio.sfx", "Sound effect", Category::Audio, Phase::Compile)
        .describe("A sound at a set time.")
        .inputs(vec![(MEDIA_PORT, opt(PortType::Media, "File"))])
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, SoundEffect>| {
            let p = &ctx.params;
            vec![
                Some(source_row(
                    Icon::AudioLines,
                    &p.src,
                    ctx.inputs,
                    "Choose a sound",
                )),
                Some(if p.at_action.is_empty() {
                    row(
                        Icon::Clock,
                        format!("At {}", format_duration(p.at_ms as f64)),
                    )
                } else {
                    row(
                        Icon::MousePointerClick,
                        format!("On an action{}", offset_label(p.offset_ms)),
                    )
                }),
                p.keep_music.then(|| row(Icon::Music, "Music stays up")),
            ]
        })
        .compiles(|ctx: &Ctx<'_, SoundEffect>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            with_source(ctx, &p.src, |src, media| {
                let length_ms = match media {
                    Some(m) => m
                        .duration_ms
                        .or_else(|| builtin_sound(&src).map(|s| s.duration_ms))
                        .unwrap_or(0.0),
                    None => builtin_sound(&src)
                        .map(|s| s.duration_ms)
                        .or(p.src_ms)
                        .unwrap_or(0.0),
                };
                let on_action = !p.at_action.is_empty();
                let timing = Timing {
                    from: if on_action {
                        ms_to_signed_frames(p.offset_ms as f64, fps)
                    } else {
                        ms_to_frames_from(p.at_ms as f64, fps, 0.0)
                    },
                    duration_in_frames: None,
                    looped: None,
                    trim_before: None,
                    fade_in_frames: None,
                    fade_out_frames: None,
                    anchor: on_action.then(|| Anchor {
                        node_id: p.at_action.clone(),
                    }),
                    length_frames: frames_if_set(length_ms, fps),
                    keep_music: p.keep_music.then_some(true),
                };
                let handles = EditHandles {
                    move_to: Some(param_ref(
                        ctx.node_id,
                        if on_action { "offsetMs" } else { "atMs" },
                        ParamUnit::Ms,
                    )),
                    ..EditHandles::default()
                };
                let item = sound_item(
                    ctx.node_id,
                    src,
                    AudioRole::Effect,
                    p.volume,
                    &p.envelope,
                    timing,
                    handles,
                );
                outputs([(
                    AUDIO_PORT,
                    CompileValue::Audio(AudioValue {
                        items: vec![item],
                        duration_in_frames: None,
                    }),
                )])
            })
        })
}

fn anchor_sounds(
    items: Vec<AudioItem>,
    clip: &ClipValue,
    settings: &WorkflowSettings,
) -> Vec<AudioItem> {
    if !items.iter().any(|a| a.anchor.is_some()) {
        return items;
    }
    let marks = flatten_spec(&clip_to_spec(clip, settings)).marks;
    items
        .into_iter()
        .filter_map(|a| {
            let Some(anchor) = &a.anchor else {
                return Some(a);
            };
            let mark = marks.iter().find(|m| {
                m.node_id.as_deref() == Some(anchor.node_id.as_str())
                    && m.kind != opsis_types::actions::ActionKind::Chapter
            })?;
            Some(AudioItem {
                from: (mark.frame.round() + a.from).max(0.0),
                ..a
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct AddSound {}

impl Params for AddSound {}

fn add_sound() -> NodeDef {
    NodeDef::of::<AddSound>("audio.mix", "Add sound", Category::Audio, Phase::Compile)
        .describe("Puts sounds under a clip. Connect as many as you like. Sound stops where the clip ends.")
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip")), (SOUNDS_PORT, many(PortType::Audio, "Sounds"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, AddSound>| {
            let n = ctx.inputs.connected.iter().filter(|p| *p == SOUNDS_PORT).count();
            vec![Some(row(Icon::Volume2, plural(n as f64, "sound")).warn_if(n == 0))]
        })
        .compiles(|ctx: &Ctx<'_, AddSound>| {
            with_clip(ctx, |clip| {
                let items = ctx.input(SOUNDS_PORT).and_then(CompileValue::as_audio).map(|a| a.items.clone()).unwrap_or_default();
                let placed = anchor_sounds(items, clip, ctx.settings).into_iter().filter_map(|a| {
                    let remaining = clip.duration_in_frames - a.from;
                    if remaining < 1.0 {
                        return None;
                    }
                    let length = a.duration_in_frames.map_or(remaining, |d| d.min(remaining));
                    Some(AudioItem { duration_in_frames: Some(length.max(1.0)), ..a })
                });
                let mut clip = clip.clone();
                clip.audio.extend(placed);
                clip
            })
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![music(), sound_effect(), add_sound()]
}
