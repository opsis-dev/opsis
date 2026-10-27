use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{signed, whole_signed};
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
            let mut hits = Hits { clicks: 0, keys: 0 };
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

pub fn nodes() -> Vec<NodeDef> {
    vec![
        music().look(NodeLook::of("lucide:eighth-notes")),
        sound_effect().look(NodeLook::of("lucide:audio-lines")),
        add_sound().look(NodeLook::of("lucide:audio-waveform")),
        action_sounds().look(NodeLook::of("lucide:keyboard-music")),
        room_tone().look(NodeLook::of("iconoir:sound-low")),
    ]
}
