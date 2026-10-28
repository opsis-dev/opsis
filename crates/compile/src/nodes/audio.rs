use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{signed, whole, whole_signed};
use opsis_types::timeline::{
    Anchor, AudioEnvelope, AudioItem, AudioRole, EditHandles, LinkedParam, ParamRef, ParamUnit,
    Sign,
};
use opsis_types::values::{AudioValue, ClipValue, CompileValue, Hits, MediaKind, MediaValue};
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
use crate::sounds::{builtin_label, builtin_song, builtin_sound};
use crate::text::{format_duration, percent_text, plural, sign_of};
use crate::time::{ms_to_frames, ms_to_frames_from, ms_to_signed_frames};
use crate::timeline::flatten::{Hit, HitKind, flatten_spec};

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
            hits: None,
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
    from: i32,
    duration_in_frames: Option<u32>,
    looped: Option<bool>,
    trim_before: Option<u32>,
    fade_in_frames: Option<u32>,
    fade_out_frames: Option<u32>,
    anchor: Option<Anchor>,
    length_frames: Option<u32>,
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

fn frames_if_set(duration_ms: f64, fps: u32) -> Option<u32> {
    (duration_ms != 0.0).then(|| ms_to_frames(duration_ms, fps))
}

fn offset_label(offset_ms: i32) -> String {
    if offset_ms == 0 {
        return String::new();
    }
    format!(
        " {}{}",
        sign_of(f64::from(offset_ms), true),
        format_duration(f64::from(offset_ms.abs()))
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
    let chosen =
        (!src.is_empty()).then(|| builtin_label(src).map_or_else(|| base_name(src), str::to_owned));
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
    #[schemars(description = "Song", extend("x-widget" = "music", "x-accept" = "audio/*"))]
    pub src: String,
    #[default = 0.5]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[default = true]
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[schemars(description = "Starts at", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub start_ms: u32,
    #[schemars(
        description = "Start the song from",
        range(min = 0, max = 600000),
        extend("x-step" = 50, "x-unit" = "ms", "x-help" = "Skips the start of the song.")
    )]
    pub trim_ms: u32,
    #[schemars(
        description = "Play for",
        range(min = 0, max = 3600000),
        extend("x-step" = 50, "x-unit" = "ms", "x-zero-label" = "To the end", "x-advanced" = true)
    )]
    pub length_ms: u32,
    #[default = 800]
    #[schemars(description = "Fade in", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_in_ms: u32,
    #[default = 1500]
    #[schemars(description = "Fade out", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_out_ms: u32,
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
            sign: Sign::Same,
            if_non_zero: None,
        },
        LinkedParam {
            param: "lengthMs".into(),
            sign: Sign::Opposite,
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
                        format!("Plays for {}", format_duration(f64::from(p.length_ms))),
                    )
                }),
            ]
        })
        .compiles(|ctx: &Ctx<'_, Music>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            with_source(ctx, &p.src, |src, media| {
                let song_ms = builtin_song(&src).map(|s| s.duration_ms);
                let timing = Timing {
                    from: signed(ms_to_frames_from(f64::from(p.start_ms), fps, 0)),
                    duration_in_frames: frames_if_set(f64::from(p.length_ms), fps),
                    looped: Some(p.r#loop),
                    trim_before: frames_if_set(f64::from(p.trim_ms), fps),
                    fade_in_frames: frames_if_set(f64::from(p.fade_in_ms), fps),
                    fade_out_frames: frames_if_set(f64::from(p.fade_out_ms), fps),
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
                let duration = media
                    .and_then(|m| m.duration_ms)
                    .or(song_ms)
                    .and_then(|ms| frames_if_set(f64::from(ms), fps));
                outputs([(
                    AUDIO_PORT,
                    CompileValue::Audio(AudioValue {
                        items: vec![item],
                        duration_in_frames: duration,
                        hits: None,
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
    pub at_ms: u32,
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
    pub offset_ms: i32,
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
                        format!("At {}", format_duration(f64::from(p.at_ms))),
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
                let builtin_ms = builtin_sound(&src).map(|s| f64::from(s.duration_ms));
                let length_ms = match media {
                    Some(m) => m.duration_ms.map(f64::from).or(builtin_ms),
                    None => builtin_ms.or(p.src_ms),
                };
                let on_action = !p.at_action.is_empty();
                let timing = Timing {
                    from: if on_action {
                        ms_to_signed_frames(f64::from(p.offset_ms), fps)
                    } else {
                        signed(ms_to_frames_from(f64::from(p.at_ms), fps, 0))
                    },
                    duration_in_frames: None,
                    looped: None,
                    trim_before: None,
                    fade_in_frames: None,
                    fade_out_frames: None,
                    anchor: on_action.then(|| Anchor {
                        node_id: p.at_action.clone(),
                    }),
                    length_frames: length_ms.and_then(|ms| frames_if_set(ms, fps)),
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
                        hits: None,
                    }),
                )])
            })
        })
}

pub const ROOM_TONE_TYPE: &str = "audio.roomTone";
pub const TONES_TYPE: &str = "audio.tones";
pub const GENERATE_TYPE: &str = "audio.generate";
pub const TRANSITION_SOUNDS_TYPE: &str = "audio.transitionSounds";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NoiseColour {
    #[default]
    Pink,
    Brown,
    White,
}

impl NoiseColour {
    fn word(self) -> &'static str {
        match self {
            Self::Pink => "Pink",
            Self::Brown => "Brown",
            Self::White => "White",
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RoomTone {
    #[schemars(
        description = "Noise",
        extend(
            "x-labels" = { "pink": "Pink (soft)", "brown": "Brown (deep)", "white": "White (bright)" },
            "x-help" = "Pink sounds like a quiet room, brown like a distant rumble, white like air."
        )
    )]
    pub colour: NoiseColour,
    #[default(-45.0)]
    #[schemars(description = "Level", range(min = -70, max = -20), extend("x-step" = 1, "x-unit" = "dB"))]
    pub level: f64,
    #[default = 1000]
    #[schemars(description = "Fade in", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_in_ms: u32,
    #[default = 1000]
    #[schemars(description = "Fade out", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub fade_out_ms: u32,
}

impl Params for RoomTone {}

fn made_sound<P>(ctx: &Ctx<'_, P>) -> Option<(String, Option<u32>)> {
    let media = ctx.result?.output(AUDIO_PORT)?.as_media()?;
    Some((media.src.clone()?, media.duration_ms))
}

fn not_made_yet() -> CardRow {
    row(Icon::Timer, "Made when the node runs").muted_if(true)
}

fn room_tone() -> NodeDef {
    NodeDef::of::<RoomTone>(ROOM_TONE_TYPE, "Room tone", Category::Audio, Phase::Media)
        .describe("A quiet bed of soft noise under the whole video, so cuts and silent stretches don't drop to dead silence.")
        .executes()
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, RoomTone>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::AudioLines,
                    format!("{} noise · {} dB", p.colour.word(), p.level),
                )),
                ctx.result.is_none().then(not_made_yet),
            ]
        })
        .compiles(|ctx: &Ctx<'_, RoomTone>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            let Some((src, _)) = made_sound(ctx) else {
                return silent_audio();
            };
            let mut item = AudioItem::new(ctx.node_id, src, 0);
            item.looped = Some(true);
            item.volume = Some(1.0);
            item.fade_in_frames = frames_if_set(f64::from(p.fade_in_ms), fps);
            item.fade_out_frames = frames_if_set(f64::from(p.fade_out_ms), fps);
            item.node_id = Some(ctx.node_id.to_owned());
            item.channel = Some(ctx.node_id.to_owned());
            item.role = Some(AudioRole::Music);
            item.keep_music = Some(true);
            outputs([(
                AUDIO_PORT,
                CompileValue::Audio(AudioValue {
                    items: vec![item],
                    duration_in_frames: None,
                    hits: None,
                }),
            )])
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToneKind {
    Tone,
    #[default]
    Beep,
    Countdown,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Tones {
    #[schemars(
        description = "Sound",
        extend(
            "x-labels" = { "tone": "Tone", "beep": "Beep", "countdown": "Countdown" },
            "x-help" = "A countdown is short beeps a set time apart, then a long one."
        )
    )]
    pub kind: ToneKind,
    #[default = 880.0]
    #[schemars(description = "Pitch", range(min = 100, max = 4000), extend("x-step" = 10, "x-unit" = "Hz"))]
    pub pitch_hz: f64,
    #[default = 150]
    #[schemars(description = "Length", range(min = 20, max = 10000), extend("x-step" = 10, "x-unit" = "ms"))]
    pub length_ms: u32,
    #[default = 3]
    #[schemars(description = "Short beeps", range(min = 1, max = 10), extend("x-show-if" = { "kind": ["countdown"] }))]
    pub count: u32,
    #[default = 1000]
    #[schemars(
        description = "Apart",
        range(min = 100, max = 5000),
        extend("x-step" = 50, "x-unit" = "ms", "x-show-if" = { "kind": ["countdown"] })
    )]
    pub spacing_ms: u32,
    #[schemars(
        description = "On an action",
        extend("x-widget" = "action", "x-help" = "Plays when this browser action happens in the clip the sound is added to.")
    )]
    pub at_action: String,
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms", "x-show-if" = { "atAction": [""] }))]
    pub at_ms: u32,
    #[schemars(
        description = "From the action",
        range(min = -10000, max = 10000),
        extend("x-unit" = "ms", "x-step" = 50, "x-show-if" = { "atAction": ["*"] })
    )]
    pub offset_ms: i32,
    #[default = 0.8]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
}

impl Params for Tones {}

fn tones() -> NodeDef {
    NodeDef::of::<Tones>(TONES_TYPE, "Tones and countdown beeps", Category::Audio, Phase::Media)
        .describe("A tone, a beep or a countdown at a set time or on a browser action, for timers and go moments.")
        .executes()
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, Tones>| {
            let p = &ctx.params;
            let what = match p.kind {
                ToneKind::Tone => format!("Tone · {} Hz", p.pitch_hz),
                ToneKind::Beep => format!("Beep · {} Hz", p.pitch_hz),
                ToneKind::Countdown => {
                    format!("Countdown · {}", plural(f64::from(p.count), "beep"))
                }
            };
            vec![
                Some(row(Icon::Timer, what)),
                Some(placing_row(&p.placing())),
                ctx.result.is_none().then(not_made_yet),
            ]
        })
        .compiles(|ctx: &Ctx<'_, Tones>| {
            placed_effect(ctx, &ctx.params.placing(), ctx.params.volume)
        })
}

struct Placing<'a> {
    at_action: &'a str,
    at_ms: u32,
    offset_ms: i32,
}

impl Tones {
    fn placing(&self) -> Placing<'_> {
        Placing {
            at_action: &self.at_action,
            at_ms: self.at_ms,
            offset_ms: self.offset_ms,
        }
    }
}

fn placing_row(placing: &Placing<'_>) -> CardRow {
    if placing.at_action.is_empty() {
        row(
            Icon::Clock,
            format!("At {}", format_duration(f64::from(placing.at_ms))),
        )
    } else {
        row(
            Icon::MousePointerClick,
            format!("On an action{}", offset_label(placing.offset_ms)),
        )
    }
}

fn placed_effect<P>(ctx: &Ctx<'_, P>, placing: &Placing<'_>, volume: f64) -> Outputs {
    let fps = ctx.settings.fps;
    let Some((src, length_ms)) = made_sound(ctx) else {
        return silent_audio();
    };
    let on_action = !placing.at_action.is_empty();
    let timing = Timing {
        from: if on_action {
            ms_to_signed_frames(f64::from(placing.offset_ms), fps)
        } else {
            signed(ms_to_frames_from(f64::from(placing.at_ms), fps, 0))
        },
        duration_in_frames: None,
        looped: None,
        trim_before: None,
        fade_in_frames: None,
        fade_out_frames: None,
        anchor: on_action.then(|| Anchor {
            node_id: placing.at_action.to_owned(),
        }),
        length_frames: length_ms.and_then(|ms| frames_if_set(f64::from(ms), fps)),
        keep_music: None,
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
        volume,
        &AudioEnvelope::default(),
        timing,
        handles,
    );
    outputs([(
        AUDIO_PORT,
        CompileValue::Audio(AudioValue {
            items: vec![item],
            duration_in_frames: None,
            hits: None,
        }),
    )])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SoundPreset {
    #[default]
    Whoosh,
    Riser,
    Pop,
    Ding,
    Tap,
    Swell,
    Click,
}

impl SoundPreset {
    pub const ALL: [Self; 7] = [
        Self::Whoosh,
        Self::Riser,
        Self::Pop,
        Self::Ding,
        Self::Tap,
        Self::Swell,
        Self::Click,
    ];

    pub fn word(self) -> &'static str {
        match self {
            Self::Whoosh => "Whoosh",
            Self::Riser => "Riser",
            Self::Pop => "Pop",
            Self::Ding => "Ding",
            Self::Tap => "Tap",
            Self::Swell => "Swell",
            Self::Click => "Click",
        }
    }

    pub fn usual_seconds(self) -> f64 {
        match self {
            Self::Whoosh => 0.8,
            Self::Riser | Self::Swell => 3.0,
            Self::Pop => 0.15,
            Self::Ding => 1.5,
            Self::Tap => 0.05,
            Self::Click => 0.1,
        }
    }

    fn picked(self) -> (f64, f64) {
        match self {
            Self::Whoosh => (0.0, 0.15),
            Self::Riser => (-5.0, 0.5),
            Self::Pop | Self::Ding | Self::Tap => (5.0, 0.5),
            Self::Swell => (0.0, 0.5),
            Self::Click => (0.0, 0.9),
        }
    }

    fn lengths(self) -> (f64, f64) {
        match self {
            Self::Whoosh => (0.4, 1.5),
            Self::Riser | Self::Swell => (1.5, 6.0),
            Self::Ding => (0.6, 2.5),
            Self::Pop => (0.08, 0.3),
            Self::Tap => (0.03, 0.1),
            Self::Click => (0.1, 0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SoundVariation {
    #[default]
    Picked,
    AsMade,
    AnotherTake,
    Darker,
    Brighter,
    Lower,
    Higher,
    Shorter,
    Longer,
    Custom,
}

impl SoundVariation {
    fn word(self) -> &'static str {
        match self {
            Self::Picked => "default",
            Self::AsMade => "as first made",
            Self::AnotherTake => "another take",
            Self::Darker => "darker",
            Self::Brighter => "brighter",
            Self::Lower => "5 semitones lower",
            Self::Higher => "5 semitones higher",
            Self::Shorter => "shorter",
            Self::Longer => "longer",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerateSound {
    #[schemars(
        description = "Sound",
        extend("x-labels" = { "whoosh": "Whoosh", "riser": "Riser", "pop": "Pop", "ding": "Ding", "tap": "Tap", "swell": "Swell", "click": "Click" })
    )]
    pub preset: SoundPreset,
    #[schemars(
        description = "Variation",
        extend(
            "x-labels" = {
                "picked": "Default", "as-made": "As first made", "another-take": "Another take", "darker": "Darker",
                "brighter": "Brighter", "lower": "5 semitones lower", "higher": "5 semitones higher",
                "shorter": "Shorter", "longer": "Longer", "custom": "Custom"
            },
            "x-help" = "Custom sets the length, pitch, brightness and take yourself. On a click, shorter and longer change the gap between its press and release."
        )
    )]
    pub variation: SoundVariation,
    #[schemars(
        description = "Length",
        range(min = 0, max = 20000),
        extend("x-step" = 10, "x-unit" = "ms", "x-zero-label" = "The sound's usual length", "x-show-if" = { "variation": ["custom"] })
    )]
    pub length_ms: u32,
    #[schemars(
        description = "Pitch",
        range(min = -24, max = 24),
        extend("x-step" = 0.5, "x-help" = "Semitones up or down.", "x-show-if" = { "variation": ["custom"] })
    )]
    pub pitch: f64,
    #[default = 0.5]
    #[schemars(description = "Brightness", range(min = 0, max = 1), extend("x-step" = 0.05, "x-show-if" = { "variation": ["custom"] }))]
    pub brightness: f64,
    #[default = 1]
    #[schemars(
        description = "Take",
        range(min = 0, max = 1000),
        extend("x-help" = "Changes the noisy parts, and a click's gap. The same take always sounds the same.", "x-show-if" = { "variation": ["custom"] })
    )]
    pub take: u32,
    #[schemars(
        description = "On an action",
        extend("x-widget" = "action", "x-help" = "Plays when this browser action happens in the clip the sound is added to.")
    )]
    pub at_action: String,
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms", "x-show-if" = { "atAction": [""] }))]
    pub at_ms: u32,
    #[schemars(
        description = "From the action",
        range(min = -10000, max = 10000),
        extend("x-unit" = "ms", "x-step" = 50, "x-show-if" = { "atAction": ["*"] })
    )]
    pub offset_ms: i32,
    #[default = 0.8]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
}

impl Params for GenerateSound {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundShape {
    pub preset: SoundPreset,
    pub seconds: f64,
    pub pitch: f64,
    pub brightness: f64,
    pub seed: u64,
}

const DARKER: f64 = 0.15;
const BRIGHTER: f64 = 0.9;
const STEP_SEMITONES: f64 = 5.0;
const CLICK_SHORT_GAP_TAKE: u64 = 0;
const CLICK_LONG_GAP_TAKE: u64 = 4;

pub fn sound_shape(p: &GenerateSound) -> SoundShape {
    let preset = p.preset;
    let base = SoundShape {
        preset,
        seconds: preset.usual_seconds(),
        pitch: 0.0,
        brightness: 0.5,
        seed: 1,
    };
    let (shorter, longer) = preset.lengths();
    let click = preset == SoundPreset::Click;
    match p.variation {
        SoundVariation::Picked => {
            let (pitch, brightness) = preset.picked();
            SoundShape {
                pitch,
                brightness,
                ..base
            }
        }
        SoundVariation::AsMade => base,
        SoundVariation::AnotherTake => SoundShape { seed: 2, ..base },
        SoundVariation::Darker => SoundShape {
            brightness: DARKER,
            ..base
        },
        SoundVariation::Brighter => SoundShape {
            brightness: BRIGHTER,
            ..base
        },
        SoundVariation::Lower => SoundShape {
            pitch: -STEP_SEMITONES,
            ..base
        },
        SoundVariation::Higher => SoundShape {
            pitch: STEP_SEMITONES,
            ..base
        },
        SoundVariation::Shorter if click => SoundShape {
            seed: CLICK_SHORT_GAP_TAKE,
            ..base
        },
        SoundVariation::Longer if click => SoundShape {
            seed: CLICK_LONG_GAP_TAKE,
            ..base
        },
        SoundVariation::Shorter => SoundShape {
            seconds: shorter,
            ..base
        },
        SoundVariation::Longer => SoundShape {
            seconds: longer,
            ..base
        },
        SoundVariation::Custom => SoundShape {
            seconds: if p.length_ms == 0 {
                base.seconds
            } else {
                f64::from(p.length_ms) / 1000.0
            },
            pitch: p.pitch,
            brightness: p.brightness.clamp(0.0, 1.0),
            seed: u64::from(p.take),
            ..base
        },
    }
}

impl GenerateSound {
    fn placing(&self) -> Placing<'_> {
        Placing {
            at_action: &self.at_action,
            at_ms: self.at_ms,
            offset_ms: self.offset_ms,
        }
    }
}

fn generate() -> NodeDef {
    NodeDef::of::<GenerateSound>(GENERATE_TYPE, "Generated sound", Category::Audio, Phase::Media)
        .describe("A whoosh, riser, pop, ding, tap, swell or click made from settings, at a set time or on a browser action. No file to find.")
        .executes()
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, GenerateSound>| {
            let p = &ctx.params;
            let shape = sound_shape(p);
            vec![
                Some(row(
                    Icon::Sparkles,
                    format!(
                        "{} · {} · {}",
                        p.preset.word(),
                        p.variation.word(),
                        format_duration(shape.seconds * 1000.0)
                    ),
                )),
                Some(placing_row(&p.placing())),
                ctx.result.is_none().then(not_made_yet),
            ]
        })
        .compiles(|ctx: &Ctx<'_, GenerateSound>| {
            placed_effect(ctx, &ctx.params.placing(), ctx.params.volume)
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
                from: (whole_signed(mark.frame) + a.from).max(0),
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
                    let remaining = i64::from(clip.duration_in_frames) - i64::from(a.from);
                    let remaining = u32::try_from(remaining).ok().filter(|r| *r >= 1)?;
                    let length = a.duration_in_frames.map_or(remaining, |d| d.min(remaining));
                    Some(AudioItem { duration_in_frames: Some(length.max(1)), ..a })
                });
                let mut clip = clip.clone();
                clip.audio.extend(placed);
                clip
            })
        })
}

pub const ACTION_SOUNDS_TYPE: &str = "audio.actionSounds";
const PITCH_SPREAD: f64 = 0.04;
const LEVEL_SPREAD: f64 = 0.12;

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ActionSounds {
    #[default = true]
    #[schemars(description = "Clicks")]
    pub clicks: bool,
    #[default = "app:sfx/mouse-click.wav"]
    #[schemars(
        description = "Click sound",
        extend("x-widget" = "sound", "x-accept" = "audio/*", "x-length-key" = "clickSoundMs", "x-show-if" = { "clicks": ["true"] })
    )]
    pub click_sound: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Click sound length", extend("exclusiveMinimum" = 0, "x-hidden" = true))]
    pub click_sound_ms: Option<f64>,
    #[default = true]
    #[schemars(description = "Keys")]
    pub keys: bool,
    #[default = "app:sfx/key-press.wav"]
    #[schemars(
        description = "Key sound",
        extend("x-widget" = "sound", "x-accept" = "audio/*", "x-length-key" = "keySoundMs", "x-show-if" = { "keys": ["true"] })
    )]
    pub key_sound: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Key sound length", extend("exclusiveMinimum" = 0, "x-hidden" = true))]
    pub key_sound_ms: Option<f64>,
    #[default = 0.6]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[default = true]
    #[schemars(
        description = "Vary each hit",
        extend("x-help" = "Changes the pitch and level of each click and key a little, so typing doesn't sound mechanical.")
    )]
    pub vary: bool,
}

impl Params for ActionSounds {}

impl ActionSounds {
    fn sound_of(&self, kind: HitKind) -> Option<(&str, Option<f64>)> {
        let (on, src, ms) = match kind {
            HitKind::Click => (self.clicks, &self.click_sound, self.click_sound_ms),
            HitKind::Key => (self.keys, &self.key_sound, self.key_sound_ms),
        };
        (on && !src.trim().is_empty()).then_some((src.as_str(), ms))
    }
}

fn spread(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    let high = u32::try_from(z >> 32).unwrap_or(u32::MAX);
    f64::from(high) / f64::from(u32::MAX) * 2.0 - 1.0
}

fn seed_of(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    })
}

fn hit_item(
    ctx: &Ctx<'_, ActionSounds>,
    index: usize,
    hit: Hit,
    src: &str,
    ms: Option<f64>,
) -> AudioItem {
    let p = &ctx.params;
    let fps = ctx.settings.fps;
    let src = project_ref(src);
    let length_ms = builtin_sound(&src).map(|s| f64::from(s.duration_ms)).or(ms);
    let seed = seed_of(ctx.node_id) ^ u64::try_from(index).unwrap_or(0).wrapping_mul(2);
    let (pitch, level) = if p.vary {
        (
            1.0 + PITCH_SPREAD * spread(seed),
            1.0 + LEVEL_SPREAD * spread(seed + 1),
        )
    } else {
        (1.0, 1.0)
    };
    let timing = Timing {
        from: whole_signed(hit.frame),
        duration_in_frames: None,
        looped: None,
        trim_before: None,
        fade_in_frames: None,
        fade_out_frames: None,
        anchor: None,
        length_frames: length_ms.and_then(|ms| frames_if_set(ms, fps)),
        keep_music: Some(true),
    };
    let mut item = sound_item(
        ctx.node_id,
        src,
        AudioRole::Effect,
        p.volume * level,
        &AudioEnvelope::default(),
        timing,
        EditHandles::default(),
    );
    item.id = format!("{}:{index}", ctx.node_id);
    item.pitch = p.vary.then_some(pitch);
    item.hit = Some(true);
    item
}

fn sound_name(src: &str) -> String {
    builtin_label(src).map_or_else(|| base_name(src), str::to_owned)
}

fn hit_row(icon: Icon, on: bool, count: Option<u32>, noun: &str, src: &str) -> CardRow {
    if !on {
        return row(icon, format!("No {noun}s")).tone(Tone::Muted);
    }
    let name = sound_name(src);
    match count {
        Some(n) => row(icon, format!("{} · {name}", plural(f64::from(n), noun))),
        None => row(icon, name),
    }
}

fn action_sounds() -> NodeDef {
    NodeDef::of::<ActionSounds>(ACTION_SOUNDS_TYPE, "Action sounds", Category::Audio, Phase::Compile)
        .describe("A click sound at every click and a key sound at every keystroke of the takes in a clip, on one Mixer channel.")
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, ActionSounds>| {
            let p = &ctx.params;
            if !ctx.inputs.is_connected(CLIP_PORT) {
                return vec![Some(missing(Icon::Film, "Connect a clip with a take"))];
            }
            let hits = ctx.inputs.hits;
            vec![
                Some(hit_row(Icon::MousePointerClick, p.clicks, hits.map(|h| h.clicks), "click", &p.click_sound)),
                Some(hit_row(Icon::Keyboard, p.keys, hits.map(|h| h.keys), "key", &p.key_sound)),
                Some(row(Icon::Volume2, percent_text(p.volume, f64::INFINITY))),
            ]
        })
        .compiles(|ctx: &Ctx<'_, ActionSounds>| {
            let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
                return silent_audio();
            };
            let flat = flatten_spec(&clip_to_spec(clip, ctx.settings));
            let mut hits = Hits {
                clicks: 0,
                keys: 0,
                transitions: 0,
            };
            let mut items = Vec::new();
            for (index, hit) in flat.hits.iter().enumerate() {
                let Some((src, ms)) = ctx.params.sound_of(hit.kind) else {
                    continue;
                };
                match hit.kind {
                    HitKind::Click => hits.clicks += 1,
                    HitKind::Key => hits.keys += 1,
                }
                items.push(hit_item(ctx, index, *hit, src, ms));
            }
            outputs([(
                AUDIO_PORT,
                CompileValue::Audio(AudioValue {
                    items,
                    duration_in_frames: None,
                    hits: Some(hits),
                }),
            )])
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionSoundFrom {
    #[default]
    Generated,
    File,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionSounds {
    #[schemars(description = "Sound", extend("x-labels" = { "generated": "A generated sound", "file": "A sound file" }))]
    pub sound_from: TransitionSoundFrom,
    #[schemars(
        description = "Generated sound",
        extend(
            "x-labels" = { "whoosh": "Whoosh", "riser": "Riser", "pop": "Pop", "ding": "Ding", "tap": "Tap", "swell": "Swell", "click": "Click" },
            "x-show-if" = { "soundFrom": ["generated"] }
        )
    )]
    pub preset: SoundPreset,
    #[schemars(
        description = "Variation",
        extend(
            "x-labels" = {
                "picked": "Default", "as-made": "As first made", "another-take": "Another take", "darker": "Darker",
                "brighter": "Brighter", "lower": "5 semitones lower", "higher": "5 semitones higher",
                "shorter": "Shorter", "longer": "Longer", "custom": "Custom"
            },
            "x-show-if" = { "soundFrom": ["generated"] }
        )
    )]
    pub variation: SoundVariation,
    #[schemars(
        description = "Pitch",
        range(min = -24, max = 24),
        extend("x-step" = 0.5, "x-help" = "Semitones up or down.", "x-show-if" = { "soundFrom": ["generated"], "variation": ["custom"] })
    )]
    pub pitch: f64,
    #[default = 0.5]
    #[schemars(
        description = "Brightness",
        range(min = 0, max = 1),
        extend("x-step" = 0.05, "x-show-if" = { "soundFrom": ["generated"], "variation": ["custom"] })
    )]
    pub brightness: f64,
    #[default = 1]
    #[schemars(description = "Take", range(min = 0, max = 1000), extend("x-show-if" = { "soundFrom": ["generated"], "variation": ["custom"] }))]
    pub take: u32,
    #[default = true]
    #[schemars(
        description = "Fit each sound to its transition",
        extend("x-help" = "Makes each sound as long as its transition. Off, every sound has the variation's own length.", "x-show-if" = { "soundFrom": ["generated"] })
    )]
    pub fit: bool,
    #[default("app:sfx/whoosh.wav".to_owned())]
    #[schemars(description = "Sound file", extend("x-widget" = "sound", "x-accept" = "audio/*", "x-show-if" = { "soundFrom": ["file"] }))]
    pub sound: String,
    #[default = true]
    #[schemars(description = "Vary each one", extend("x-help" = "A small change of pitch and level on each, so they don't sound copied."))]
    pub vary: bool,
    #[default = 0.7]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
}

impl Params for TransitionSounds {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionCue {
    pub middle: f64,
    pub frames: f64,
}

pub fn transition_cues(clip: &ClipValue, settings: &WorkflowSettings) -> Vec<TransitionCue> {
    flatten_spec(&clip_to_spec(clip, settings))
        .transitions
        .iter()
        .map(|t| TransitionCue {
            middle: (t.from + t.to) / 2.0,
            frames: t.to - t.from,
        })
        .collect()
}

impl TransitionSounds {
    pub fn shape(&self) -> SoundShape {
        sound_shape(&GenerateSound {
            preset: self.preset,
            variation: self.variation,
            pitch: self.pitch,
            brightness: self.brightness,
            take: self.take,
            ..GenerateSound::default()
        })
    }

    pub fn length_ms(&self, cue: &TransitionCue, fps: u32) -> u32 {
        if self.fit {
            whole(cue.frames * 1000.0 / f64::from(fps.max(1))).max(MIN_TRANSITION_SOUND_MS)
        } else {
            whole(self.shape().seconds * 1000.0)
        }
    }
}

const MIN_TRANSITION_SOUND_MS: u32 = 50;

pub fn transition_sound_file(length_ms: u32) -> String {
    format!("transition-{length_ms}.wav")
}

fn transition_source(
    ctx: &Ctx<'_, TransitionSounds>,
    cue: &TransitionCue,
) -> Option<(String, Option<f64>)> {
    let p = &ctx.params;
    match p.sound_from {
        TransitionSoundFrom::File => {
            let src = project_ref(&p.sound);
            let length = builtin_sound(&src).map(|s| f64::from(s.duration_ms));
            (!p.sound.is_empty()).then_some((src, length))
        }
        TransitionSoundFrom::Generated => {
            let ms = p.length_ms(cue, ctx.settings.fps);
            let file = transition_sound_file(ms);
            let made = ctx
                .result?
                .artifacts
                .iter()
                .find(|a| a.path.ends_with(&format!("/{file}")))?;
            Some((project_ref(&made.path), Some(f64::from(ms))))
        }
    }
}

fn transition_item(
    ctx: &Ctx<'_, TransitionSounds>,
    index: usize,
    cue: &TransitionCue,
    src: String,
    length_ms: Option<f64>,
) -> AudioItem {
    let p = &ctx.params;
    let fps = ctx.settings.fps;
    let length_frames = length_ms.and_then(|ms| frames_if_set(ms, fps));
    let half = f64::from(length_frames.unwrap_or(0)) / 2.0;
    let seed = seed_of(ctx.node_id) ^ u64::try_from(index).unwrap_or(0).wrapping_mul(2);
    let (pitch, level) = if p.vary {
        (
            1.0 + PITCH_SPREAD * spread(seed),
            1.0 + LEVEL_SPREAD * spread(seed + 1),
        )
    } else {
        (1.0, 1.0)
    };
    let timing = Timing {
        from: whole_signed(cue.middle - half),
        duration_in_frames: None,
        looped: None,
        trim_before: None,
        fade_in_frames: None,
        fade_out_frames: None,
        anchor: None,
        length_frames,
        keep_music: Some(true),
    };
    let mut item = sound_item(
        ctx.node_id,
        src,
        AudioRole::Effect,
        p.volume * level,
        &AudioEnvelope::default(),
        timing,
        EditHandles::default(),
    );
    item.id = format!("{}:{index}", ctx.node_id);
    item.pitch = p.vary.then_some(pitch);
    item.hit = Some(true);
    item
}

fn transition_sounds() -> NodeDef {
    NodeDef::of::<TransitionSounds>(TRANSITION_SOUNDS_TYPE, "Transition sounds", Category::Audio, Phase::Media)
        .describe("A whoosh at every transition in a clip, timed to its middle and fitted to its length, on one Mixer channel.")
        .executes()
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(|ctx: &CardCtx<'_, TransitionSounds>| {
            let p = &ctx.params;
            if !ctx.inputs.is_connected(CLIP_PORT) {
                return vec![Some(missing(Icon::Film, "Connect a clip with transitions"))];
            }
            let sound = match p.sound_from {
                TransitionSoundFrom::Generated => {
                    format!("{}, {}", p.preset.word(), p.variation.word())
                }
                TransitionSoundFrom::File => builtin_label(&project_ref(&p.sound))
                    .map_or_else(|| base_name(&p.sound), str::to_owned),
            };
            let count = ctx.inputs.hits.map(|h| h.transitions);
            vec![
                Some(row(
                    Icon::Blend,
                    match count {
                        Some(n) => format!("{} · {sound}", plural(f64::from(n), "transition")),
                        None => sound,
                    },
                )),
                Some(row(Icon::Volume2, percent_text(p.volume, f64::INFINITY))),
                (p.sound_from == TransitionSoundFrom::Generated && ctx.result.is_none())
                    .then(not_made_yet),
            ]
        })
        .compiles(|ctx: &Ctx<'_, TransitionSounds>| {
            let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
                return silent_audio();
            };
            let cues = transition_cues(clip, ctx.settings);
            let items = cues
                .iter()
                .enumerate()
                .filter_map(|(index, cue)| {
                    let (src, length) = transition_source(ctx, cue)?;
                    Some(transition_item(ctx, index, cue, src, length))
                })
                .collect();
            outputs([(
                AUDIO_PORT,
                CompileValue::Audio(AudioValue {
                    items,
                    duration_in_frames: None,
                    hits: Some(Hits {
                        clicks: 0,
                        keys: 0,
                        transitions: u32::try_from(cues.len()).unwrap_or(u32::MAX),
                    }),
                }),
            )])
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        music().look(NodeLook::of("lucide:eighth-notes")),
        sound_effect().look(NodeLook::of("lucide:audio-lines")),
        add_sound().look(NodeLook::of("lucide:audio-waveform")),
        action_sounds().look(NodeLook::of("lucide:keyboard-music")),
        room_tone().look(NodeLook::of("iconoir:sound-low")),
        tones().look(NodeLook::of("lucide:bell-ring")),
        generate().look(NodeLook::of("tabler:music-bolt")),
        transition_sounds().look(NodeLook::of("tabler:wind")),
    ]
}
