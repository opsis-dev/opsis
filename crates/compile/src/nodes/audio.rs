use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::numbers::{signed, whole, whole_signed};
use opsis_types::timeline::{
    Anchor, AudioEnvelope, AudioItem, AudioRole, EditHandles, FadeCurve, LinkedParam, ParamRef,
    ParamUnit, Sign,
};
use opsis_types::values::{AudioValue, ClipValue, CompileValue, Hits, MediaKind, MediaValue};
use opsis_types::workflow::{PortType, WorkflowSettings};

use crate::asset_ref::{base_name, project_ref};
use crate::card_rows::{LABELLED_TEXT_MAX, named_file_row, quote};
use crate::clip::clip_to_spec;
use crate::envelope::has_audio_keys;
use crate::node::{
    CLIP_PORT, CardCtx, CardInputs, Category, Ctx, NodeDef, Outputs, Phase, RowExt, many, missing,
    opt, outputs, port, row, with_clip,
};
use crate::sounds::{builtin_label, builtin_song, builtin_sound};
use crate::text::{format_duration, percent_text, plural, sign_of};
use crate::time::{ms_to_frames, ms_to_frames_from, ms_to_signed_frames};
use crate::timeline::flatten::{Hit, HitKind, flatten_spec};

pub(crate) const MEDIA_PORT: &str = "media";
pub(crate) const AUDIO_PORT: &str = "audio";
const SOUNDS_PORT: &str = "sounds";

pub(crate) fn silent_audio() -> Outputs {
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

pub(crate) fn with_source<P>(
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

pub(crate) struct Timing {
    pub from: i32,
    pub duration_in_frames: Option<u32>,
    pub looped: Option<bool>,
    pub trim_before: Option<u32>,
    pub fade_in_frames: Option<u32>,
    pub fade_out_frames: Option<u32>,
    pub anchor: Option<Anchor>,
    pub length_frames: Option<u32>,
    pub keep_music: Option<bool>,
}

pub(crate) fn sound_item(
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
        volume: Some(ParamRef::new(node_id, "volume", ParamUnit::Number)),
        ..handles
    });
    item
}

pub(crate) fn frames_if_set(duration_ms: f64, fps: u32) -> Option<u32> {
    (duration_ms != 0.0).then(|| ms_to_frames(duration_ms, fps))
}

pub(crate) fn offset_label(offset_ms: i32) -> String {
    if offset_ms == 0 {
        return String::new();
    }
    format!(
        " {}{}",
        sign_of(f64::from(offset_ms), true),
        format_duration(f64::from(offset_ms.abs()))
    )
}

pub(crate) fn source_row(icon: Icon, src: &str, inputs: &CardInputs, empty: &str) -> CardRow {
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

#[settings]
pub struct Music {
    #[setting("Song", widget = "music", accept = "audio/*")]
    pub src: String,
    #[setting("Volume", default = 0.5, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[setting("Loop", default = true)]
    pub r#loop: bool,
    #[setting("Starts at", range(min = 0, max = 600_000), step = 50, unit = "ms")]
    pub start_ms: u32,
    #[setting(
        "Start the song from",
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "Skips the start of the song."
    )]
    pub trim_ms: u32,
    #[setting(
        "Play for",
        range(min = 0, max = 3_600_000),
        step = 50,
        unit = "ms",
        zero_label = "To the end",
        advanced
    )]
    pub length_ms: u32,
    #[setting(
        "Fade in",
        default = 800,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms"
    )]
    pub fade_in_ms: u32,
    #[setting(
        "Fade out",
        default = 1500,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms"
    )]
    pub fade_out_ms: u32,
    #[setting(
        "Keyframes",
        help = "Change volume and left–right balance over time. Edit them here or on the timeline.",
        widget = "envelope",
        advanced
    )]
    pub envelope: AudioEnvelope,
    #[setting("Fade curve", advanced)]
    pub fade_curve: FadeCurve,
}

fn music_handles(node_id: &str) -> EditHandles {
    let mut trim_start = ParamRef::new(node_id, "trimMs", ParamUnit::Ms);
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
        move_to: Some(ParamRef::new(node_id, "startMs", ParamUnit::Ms)),
        trim_start: Some(trim_start),
        trim_end: Some(ParamRef::new(node_id, "lengthMs", ParamUnit::Ms)),
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
                let mut item = sound_item(
                    ctx.node_id,
                    src,
                    AudioRole::Music,
                    p.volume,
                    &p.envelope,
                    timing,
                    music_handles(ctx.node_id),
                );
                item.fade_curve = Some(p.fade_curve).filter(|c| *c != FadeCurve::Tri);
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

pub const ROOM_TONE_TYPE: &str = "audio.roomTone";
pub const TRANSITION_SOUNDS_TYPE: &str = "audio.transitionSounds";
pub const SOUND_EFFECT_TYPE: &str = "audio.sfx";
pub const MADE_SOUND_FILE: &str = "made.wav";
pub const END_HIT_FILE: &str = "end-hit.wav";

#[choice]
pub enum NoiseColour {
    #[default]
    #[label("Pink (soft)")]
    Pink,
    #[label("Brown (deep)")]
    Brown,
    #[label("White (bright)")]
    White,
}

impl NoiseColour {
    const fn word(self) -> &'static str {
        match self {
            Self::Pink => "Pink",
            Self::Brown => "Brown",
            Self::White => "White",
        }
    }
}

#[settings]
pub struct RoomTone {
    #[setting(
        "Noise",
        help = "Pink sounds like a quiet room, brown like a distant rumble, white like air."
    )]
    pub colour: NoiseColour,
    #[setting("Level", default = -45.0, range(min = -70, max = -20), step = 1, unit = "dB")]
    pub level: f64,
    #[setting(
        "Fade in",
        default = 1000,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub fade_in_ms: u32,
    #[setting(
        "Fade out",
        default = 1000,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub fade_out_ms: u32,
}

fn made_sound<P>(ctx: &Ctx<'_, P>) -> Option<(String, Option<u32>)> {
    let media = ctx.result?.output(AUDIO_PORT)?.as_media()?;
    Some((media.src.clone()?, media.duration_ms))
}

pub(crate) fn not_made_yet() -> CardRow {
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

#[choice]
pub enum ToneKind {
    #[label("Tone")]
    Tone,
    #[default]
    #[label("Beep")]
    Beep,
    #[label("Countdown")]
    Countdown,
}

#[choice]
pub enum ToneWave {
    #[default]
    #[label("Pure")]
    Sine,
    #[label("Soft")]
    Triangle,
    #[label("Hollow")]
    Square,
    #[label("Buzzy")]
    Saw,
}

pub(crate) fn picked_name(inputs: &CardInputs, id: &str, unnamed: &str) -> String {
    inputs
        .names
        .get(id)
        .map_or_else(|| unnamed.to_owned(), |name| quote(name, LABELLED_TEXT_MAX))
}

#[choice]
pub enum SoundPreset {
    #[default]
    #[label("Whoosh")]
    Whoosh,
    #[label("Riser")]
    Riser,
    #[label("Pop")]
    Pop,
    #[label("Ding")]
    Ding,
    #[label("Tap")]
    Tap,
    #[label("Swell")]
    Swell,
    #[label("Click")]
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

    pub const fn usual_seconds(self) -> f64 {
        match self {
            Self::Whoosh => 0.8,
            Self::Riser | Self::Swell => 3.0,
            Self::Pop => 0.15,
            Self::Ding => 1.5,
            Self::Tap => 0.05,
            Self::Click => 0.1,
        }
    }

    const fn picked(self) -> (f64, f64) {
        match self {
            Self::Whoosh => (0.0, 0.15),
            Self::Riser => (-5.0, 0.5),
            Self::Pop | Self::Ding | Self::Tap => (5.0, 0.5),
            Self::Swell => (0.0, 0.5),
            Self::Click => (0.0, 0.9),
        }
    }

    const fn lengths(self) -> (f64, f64) {
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

#[choice]
pub enum SoundVariation {
    #[default]
    #[label("Default")]
    Picked,
    #[label("As first made")]
    AsMade,
    #[label("Another take")]
    AnotherTake,
    #[label("Darker")]
    Darker,
    #[label("Brighter")]
    Brighter,
    #[label("5 semitones lower")]
    Lower,
    #[label("5 semitones higher")]
    Higher,
    #[label("Shorter")]
    Shorter,
    #[label("Longer")]
    Longer,
    #[label("Custom")]
    Custom,
}

impl SoundVariation {
    const fn word(self) -> &'static str {
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

#[choice]
pub enum SoundPan {
    #[default]
    #[label("Left to right")]
    LeftToRight,
    #[label("Right to left")]
    RightToLeft,
    #[label("Stays in the middle")]
    Centre,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundShape {
    pub preset: SoundPreset,
    pub seconds: f64,
    pub pitch: f64,
    pub brightness: f64,
    pub seed: u64,
    pub pan: SoundPan,
    pub peak_at: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundRecipe {
    pub preset: SoundPreset,
    pub variation: SoundVariation,
    pub length_ms: u32,
    pub pitch: f64,
    pub brightness: f64,
    pub take: u32,
    pub pan: SoundPan,
}

impl SoundRecipe {
    pub const fn of(preset: SoundPreset) -> Self {
        Self {
            preset,
            variation: SoundVariation::Picked,
            length_ms: 0,
            pitch: 0.0,
            brightness: 0.5,
            take: 1,
            pan: SoundPan::LeftToRight,
        }
    }
}

pub const SWELL_PEAK_AT: f64 = 0.75;

const DARKER: f64 = 0.15;
const BRIGHTER: f64 = 0.9;
const STEP_SEMITONES: f64 = 5.0;
const CLICK_SHORT_GAP_TAKE: u64 = 0;
const CLICK_LONG_GAP_TAKE: u64 = 4;

pub fn sound_shape(p: &SoundRecipe) -> SoundShape {
    let preset = p.preset;
    let base = SoundShape {
        preset,
        seconds: preset.usual_seconds(),
        pitch: 0.0,
        brightness: 0.5,
        seed: 1,
        pan: p.pan,
        peak_at: SWELL_PEAK_AT,
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
            let at = if anchor.clip_end {
                f64::from(clip.duration_in_frames)
            } else {
                marks
                    .iter()
                    .find(|m| {
                        m.node_id.as_deref() == Some(anchor.node_id.as_str())
                            && m.kind != opsis_types::actions::ActionKind::Chapter
                    })?
                    .frame
            };
            let start = whole_signed(at) + a.from;
            Some(if anchor.keep_end && start < 0 {
                AudioItem {
                    from: 0,
                    trim_before: Some(a.trim_before.unwrap_or(0) + start.unsigned_abs()),
                    ..a
                }
            } else {
                AudioItem {
                    from: start.max(0),
                    ..a
                }
            })
        })
        .collect()
}

#[settings(eq)]
pub struct AddSound {
    #[setting(
        "Fade where the clip ends",
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        help = "Sounds still playing when the clip ends fade out over this long instead of stopping.",
        advanced
    )]
    pub end_fade_ms: u32,
}

fn add_sound() -> NodeDef {
    NodeDef::of::<AddSound>("audio.mix", "Add sound", Category::Audio, Phase::Compile)
        .describe("Puts sounds under a clip. Connect as many as you like. Sound stops where the clip ends.")
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip")), (SOUNDS_PORT, many(PortType::Audio, "Sounds"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, AddSound>| {
            let n = ctx.inputs.connected.iter().filter(|p| *p == SOUNDS_PORT).count();
            vec![Some(row(Icon::Volume2, plural(numbers::float(n), "sound")).warn_if(n == 0))]
        })
        .compiles(|ctx: &Ctx<'_, AddSound>| {
            with_clip(ctx, |clip| {
                let items = ctx.input(SOUNDS_PORT).and_then(CompileValue::as_audio).map_or_default(|a| a.items.clone());
                let placed = anchor_sounds(items, clip, ctx.settings).into_iter().filter_map(|a| {
                    let remaining = i64::from(clip.duration_in_frames) - i64::from(a.from);
                    let remaining = u32::try_from(remaining).ok().filter(|r| *r >= 1)?;
                    let natural = a.duration_in_frames.or_else(|| if a.looped == Some(true) { None } else { a.length_frames });
                    let length = a.duration_in_frames.map_or(remaining, |d| d.min(remaining)).max(1);
                    let cut = natural.is_none_or(|n| n > remaining);
                    let fade = frames_if_set(f64::from(ctx.params.end_fade_ms), ctx.settings.fps)
                        .filter(|_| cut)
                        .map(|f| f.min(length).max(a.fade_out_frames.unwrap_or(0)));
                    Some(AudioItem { duration_in_frames: Some(length), fade_out_frames: fade.or(a.fade_out_frames), ..a })
                });
                let mut clip = clip.clone();
                clip.audio.extend(placed);
                clip
            })
        })
}

pub const ACTION_SOUNDS_TYPE: &str = "audio.actionSounds";

#[settings]
pub struct ActionSounds {
    #[setting("Clicks", default = true)]
    pub clicks: bool,
    #[setting(
        "Click sound",
        default = "app:sfx/mouse-click.wav",
        widget = "sound",
        accept = "audio/*",
        length_key = "clickSoundMs",
        show_if = { "clicks": ["true"] }
    )]
    pub click_sound: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Click sound length", "exclusiveMinimum" = 0, hidden)]
    pub click_sound_ms: Option<f64>,
    #[setting("Keys", default = true)]
    pub keys: bool,
    #[setting(
        "Key sound",
        default = "app:sfx/key-press.wav",
        widget = "sound",
        accept = "audio/*",
        length_key = "keySoundMs",
        show_if = { "keys": ["true"] }
    )]
    pub key_sound: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Key sound length", "exclusiveMinimum" = 0, hidden)]
    pub key_sound_ms: Option<f64>,
    #[setting("Volume", default = 0.6, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[setting(
        "Vary each hit",
        default = true,
        help = "Changes the pitch and level of each click and key a little, so typing doesn't sound mechanical.",
        advanced
    )]
    pub vary: bool,
    #[setting(
        "Pitch change",
        default = 4,
        range(min = 0, max = 50),
        unit = "%",
        help = "How far each one's pitch may move up or down.",
        show_if = { "vary": ["true"] },
        advanced
    )]
    pub pitch_spread: u32,
    #[setting(
        "Level change",
        default = 12,
        range(min = 0, max = 100),
        unit = "%",
        help = "How far each one's level may move up or down.",
        show_if = { "vary": ["true"] },
        advanced
    )]
    pub level_spread: u32,
    #[setting(
        "Nudge",
        range(min = -500, max = 500),
        step = 5,
        unit = "ms",
        help = "Moves every hit earlier or later than the press.",
        advanced
    )]
    pub nudge_ms: i32,
    #[setting(
        "Drags",
        help = "A click where each drag presses and where it lets go.",
        show_if = { "clicks": ["true"] },
        advanced
    )]
    pub drags: bool,
}

impl ActionSounds {
    fn sound_of(&self, kind: HitKind) -> Option<(&str, Option<f64>)> {
        let (on, src, ms) = match kind {
            HitKind::Click => (self.clicks, &self.click_sound, self.click_sound_ms),
            HitKind::Drag => (
                self.clicks && self.drags,
                &self.click_sound,
                self.click_sound_ms,
            ),
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
    f64::mul_add(f64::from(high) / f64::from(u32::MAX), 2.0, -1.0)
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
    let (pitch, level) = varied(p.vary, seed, p.pitch_spread, p.level_spread);
    let timing = Timing {
        from: whole_signed(hit.frame) + ms_to_signed_frames(f64::from(p.nudge_ms), fps),
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

fn varied(vary: bool, seed: u64, pitch_spread: u32, level_spread: u32) -> (f64, f64) {
    if !vary {
        return (1.0, 1.0);
    }
    (
        f64::mul_add(f64::from(pitch_spread) / 100.0, spread(seed), 1.0),
        f64::mul_add(f64::from(level_spread) / 100.0, spread(seed + 1), 1.0),
    )
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
                    HitKind::Click | HitKind::Drag => hits.clicks += 1,
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

#[choice]
pub enum TransitionSoundFrom {
    #[default]
    #[label("A generated sound")]
    Generated,
    #[label("A sound file")]
    File,
}

#[settings]
pub struct TransitionSounds {
    #[setting("Sound")]
    pub sound_from: TransitionSoundFrom,
    #[setting(
        "Generated sound",
        show_if = { "soundFrom": ["generated"] }
    )]
    pub preset: SoundPreset,
    #[setting(
        "Variation",
        show_if = { "soundFrom": ["generated"] }
    )]
    pub variation: SoundVariation,
    #[setting(
        "Pitch",
        range(min = -24, max = 24),
        step = 0.5,
        help = "Semitones up or down.",
        show_if = { "soundFrom": ["generated"], "variation": ["custom"] }
    )]
    pub pitch: f64,
    #[setting(
        "Brightness",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "soundFrom": ["generated"], "variation": ["custom"] }
    )]
    pub brightness: f64,
    #[setting(
        "Take",
        default = 1,
        range(min = 0, max = 1000),
        show_if = { "soundFrom": ["generated"], "variation": ["custom"] }
    )]
    pub take: u32,
    #[setting(
        "Fit each sound to its transition",
        default = true,
        help = "Makes each sound as long as its transition. Off, every sound has the variation's own length.",
        show_if = { "soundFrom": ["generated"] },
        advanced
    )]
    pub fit: bool,
    #[setting(
        "Sound file",
        default = "app:sfx/whoosh.wav".to_owned(),
        widget = "sound",
        accept = "audio/*",
        show_if = { "soundFrom": ["file"] }
    )]
    pub sound: String,
    #[setting(
        "Also at plain cuts",
        help = "Plays the sound where one clip cuts straight to the next too, at the variation's own length."
    )]
    pub at_cuts: bool,
    #[setting(
        "Vary each one",
        default = true,
        help = "A small change of pitch and level on each, so they don't sound copied.",
        advanced
    )]
    pub vary: bool,
    #[setting("Volume", default = 0.7, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[setting(
        "Pitch change",
        default = 4,
        range(min = 0, max = 50),
        unit = "%",
        help = "How far each one's pitch may move up or down.",
        show_if = { "vary": ["true"] },
        advanced
    )]
    pub pitch_spread: u32,
    #[setting(
        "Level change",
        default = 12,
        range(min = 0, max = 100),
        unit = "%",
        help = "How far each one's level may move up or down.",
        show_if = { "vary": ["true"] },
        advanced
    )]
    pub level_spread: u32,
    #[setting("Place on the transition", advanced)]
    pub align: TransitionAlign,
    #[setting(
        "Length",
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        zero_label = "The variation's own",
        help = "Used when Fit is off and at plain cuts.",
        show_if = { "soundFrom": ["generated"] },
        advanced
    )]
    pub length_ms: u32,
}

#[choice]
pub enum TransitionAlign {
    #[label("From its start")]
    Start,
    #[default]
    #[label("Centred on it")]
    Middle,
    #[label("Ending with it")]
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionCue {
    pub middle: f64,
    pub frames: f64,
}

pub fn transition_cues(
    clip: &ClipValue,
    settings: &WorkflowSettings,
    with_cuts: bool,
) -> Vec<TransitionCue> {
    let flat = flatten_spec(&clip_to_spec(clip, settings));
    let mut cues: Vec<TransitionCue> = flat
        .transitions
        .iter()
        .map(|t| TransitionCue {
            middle: f64::midpoint(t.from, t.to),
            frames: t.to - t.from,
        })
        .collect();
    if with_cuts {
        cues.extend(flat.cuts.iter().map(|&at| TransitionCue {
            middle: at,
            frames: 0.0,
        }));
        cues.sort_by(|a, b| a.middle.total_cmp(&b.middle));
    }
    cues
}

impl TransitionSounds {
    pub fn shape(&self) -> SoundShape {
        sound_shape(&SoundRecipe {
            variation: self.variation,
            pitch: self.pitch,
            brightness: self.brightness,
            take: self.take,
            ..SoundRecipe::of(self.preset)
        })
    }

    pub fn length_ms(&self, cue: &TransitionCue, fps: u32) -> u32 {
        if self.fit && cue.frames > 0.0 {
            whole(cue.frames * 1000.0 / f64::from(fps.max(1))).max(MIN_TRANSITION_SOUND_MS)
        } else if self.length_ms > 0 {
            self.length_ms
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
    let own = f64::from(length_frames.unwrap_or(0));
    let seed = seed_of(ctx.node_id) ^ u64::try_from(index).unwrap_or(0).wrapping_mul(2);
    let (pitch, level) = varied(p.vary, seed, p.pitch_spread, p.level_spread);
    let start = match p.align {
        TransitionAlign::Start => cue.middle - cue.frames / 2.0,
        TransitionAlign::Middle => cue.middle - own / 2.0,
        TransitionAlign::End => cue.middle + cue.frames / 2.0 - own,
    };
    let timing = Timing {
        from: whole_signed(start),
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
                    format!("{}, {}", p.preset.label(), p.variation.word())
                }
                TransitionSoundFrom::File => builtin_label(&project_ref(&p.sound))
                    .map_or_else(|| base_name(&p.sound), str::to_owned),
            };
            let count = ctx.inputs.hits.map(|h| h.transitions);
            vec![
                Some(row(
                    Icon::Blend,
                    match (count, p.at_cuts) {
                        (Some(n), false) => {
                            format!("{} · {sound}", plural(f64::from(n), "transition"))
                        }
                        (Some(n), true) => format!("{n} at cuts and transitions · {sound}"),
                        (None, _) => sound,
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
            let cues = transition_cues(clip, ctx.settings, ctx.params.at_cuts);
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
        crate::nodes::sound_effect::sound_effect().look(NodeLook::of("lucide:audio-lines")),
        add_sound().look(NodeLook::of("lucide:audio-waveform")),
        action_sounds().look(NodeLook::of("lucide:keyboard-music")),
        room_tone().look(NodeLook::of("iconoir:sound-low")),
        transition_sounds().look(NodeLook::of("tabler:arrow-wave-right-up")),
    ]
}
