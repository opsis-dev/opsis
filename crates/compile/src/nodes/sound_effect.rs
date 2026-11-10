use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon};
use opsis_types::numbers::{signed, whole_signed};
use opsis_types::timeline::{
    Anchor, AudioEnvelope, AudioItem, AudioRole, EditHandles, ParamRef, ParamUnit,
};
use opsis_types::values::{AudioValue, CompileValue, MediaValue};
use opsis_types::workflow::PortType;

use crate::asset_ref::project_ref;
use crate::node::{
    CardCtx, Category, Ctx, NodeDef, Outputs, Phase, missing, opt, outputs, port, row,
};
use crate::nodes::audio::{
    AUDIO_PORT, END_HIT_FILE, MEDIA_PORT, SOUND_EFFECT_TYPE, SoundPan, SoundPreset, SoundRecipe,
    SoundShape, SoundVariation, Timing, ToneKind, ToneWave, frames_if_set, not_made_yet,
    offset_label, picked_name, silent_audio, sound_item, sound_shape, source_row, with_source,
};
use crate::sounds::builtin_sound;
use crate::text::{format_duration, plural};
use crate::time::{ms_to_frames, ms_to_frames_from, ms_to_signed_frames};

#[choice]
pub enum SoundSource {
    #[default]
    #[label("A file")]
    File,
    #[label("Made from settings")]
    Made,
}

#[choice]
pub enum MadeSound {
    #[default]
    #[label("Whoosh")]
    Whoosh,
    #[label("Riser")]
    Riser,
    #[label("Swell")]
    Swell,
    #[label("Pop")]
    Pop,
    #[label("Ding")]
    Ding,
    #[label("Tap")]
    Tap,
    #[label("Click")]
    Click,
    #[label("Tone")]
    Tone,
    #[label("Beep")]
    Beep,
    #[label("Countdown")]
    Countdown,
}

impl MadeSound {
    pub const fn preset(self) -> Option<SoundPreset> {
        match self {
            Self::Whoosh => Some(SoundPreset::Whoosh),
            Self::Riser => Some(SoundPreset::Riser),
            Self::Swell => Some(SoundPreset::Swell),
            Self::Pop => Some(SoundPreset::Pop),
            Self::Ding => Some(SoundPreset::Ding),
            Self::Tap => Some(SoundPreset::Tap),
            Self::Click => Some(SoundPreset::Click),
            Self::Tone | Self::Beep | Self::Countdown => None,
        }
    }

    pub const fn tone(self) -> Option<ToneKind> {
        match self {
            Self::Tone => Some(ToneKind::Tone),
            Self::Beep => Some(ToneKind::Beep),
            Self::Countdown => Some(ToneKind::Countdown),
            _ => None,
        }
    }
}

#[choice]
pub enum SoundTiming {
    #[default]
    #[label("At a time")]
    AtTime,
    #[label("On an action")]
    OnAction,
    #[label("Ending on an action")]
    EndsOnAction,
    #[label("Ending on a marker")]
    EndsOnMarker,
    #[label("Ending with the clip")]
    EndsWithClip,
}

impl SoundTiming {
    const fn ends(self) -> bool {
        matches!(
            self,
            Self::EndsOnAction | Self::EndsOnMarker | Self::EndsWithClip
        )
    }
}

#[choice]
pub enum BeforeClipStart {
    #[default]
    #[label("Start it with the clip")]
    Later,
    #[label("Cut its start")]
    Cut,
}

#[choice]
pub enum EndHit {
    #[default]
    #[label("None")]
    Off,
    #[label("Pop")]
    Pop,
    #[label("Ding")]
    Ding,
    #[label("Tap")]
    Tap,
    #[label("Click")]
    Click,
}

#[settings]
pub struct SoundEffect {
    #[setting("Source")]
    pub source: SoundSource,
    #[setting(
        "Sound",
        widget = "sound",
        accept = "audio/*",
        length_key = "srcMs",
        show_if = { "source": ["file"] }
    )]
    pub src: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Sound length", "exclusiveMinimum" = 0, hidden)]
    pub src_ms: Option<f64>,
    #[setting("Sound", show_if = { "source": ["made"] })]
    pub sound: MadeSound,
    #[setting(
        "Variation",
        help = "Custom sets the length, pitch, brightness and take yourself. On a click, shorter and longer change the gap between its press and release.",
        show_if = { "source": ["made"], "sound": ["whoosh", "riser", "swell", "pop", "ding", "tap", "click"] }
    )]
    pub variation: SoundVariation,
    #[setting(
        "Length",
        range(min = 0, max = 20000),
        step = 10,
        unit = "ms",
        zero_label = "The sound's usual length",
        show_if = { "source": ["made"], "variation": ["custom"] }
    )]
    pub custom_ms: u32,
    #[setting(
        "Pitch",
        range(min = -24, max = 24),
        step = 0.5,
        help = "Semitones up or down.",
        show_if = { "source": ["made"], "variation": ["custom"] }
    )]
    pub pitch: f64,
    #[setting(
        "Brightness",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "source": ["made"], "variation": ["custom"] }
    )]
    pub brightness: f64,
    #[setting(
        "Take",
        default = 1,
        range(min = 0, max = 1000),
        help = "Changes the noisy parts, and a click's gap. The same take always sounds the same.",
        show_if = { "source": ["made"], "variation": ["custom"] }
    )]
    pub take: u32,
    #[setting(
        "Pitch",
        default = 880.0,
        range(min = 100, max = 4000),
        step = 10,
        unit = "Hz",
        show_if = { "source": ["made"], "sound": ["tone", "beep", "countdown"] }
    )]
    pub pitch_hz: f64,
    #[setting(
        "Length",
        default = 150,
        range(min = 20, max = 10000),
        step = 10,
        unit = "ms",
        show_if = { "source": ["made"], "sound": ["tone", "beep", "countdown"] }
    )]
    pub tone_ms: u32,
    #[setting(
        "Short beeps",
        default = 3,
        range(min = 1, max = 10),
        help = "A countdown is short beeps a set time apart, then a long one.",
        show_if = { "source": ["made"], "sound": ["countdown"] }
    )]
    pub count: u32,
    #[setting(
        "Apart",
        default = 1000,
        range(min = 100, max = 5000),
        step = 50,
        unit = "ms",
        show_if = { "source": ["made"], "sound": ["countdown"] }
    )]
    pub spacing_ms: u32,
    #[setting("Timing")]
    pub timing: SoundTiming,
    #[setting(
        "At",
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "timing": ["at-time"] }
    )]
    pub at_ms: u32,
    #[setting(
        "Action",
        widget = "action",
        help = "The browser action, in the clip the sound is added to. It follows the action after you record again or edit the clip.",
        show_if = { "timing": ["on-action", "ends-on-action"] }
    )]
    pub at_action: String,
    #[setting(
        "From the action",
        range(min = -10000, max = 10000),
        unit = "ms",
        step = 50,
        help = "Negative plays it before the action, for sounds that build up to it.",
        show_if = { "timing": ["on-action"] }
    )]
    pub offset_ms: i32,
    #[setting(
        "Marker",
        widget = "marker",
        help = "A Timeline marker. Its time counts from the start of the clip the sound is added to.",
        show_if = { "timing": ["ends-on-marker"] }
    )]
    pub marker: String,
    #[setting(
        "A hit at the end",
        show_if = { "timing": ["ends-on-action", "ends-on-marker", "ends-with-clip"] }
    )]
    pub end_hit: EndHit,
    #[setting("Volume", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Hit volume",
        range(min = 0, max = 2),
        step = 0.05,
        zero_label = "Same as the sound",
        show_if = { "timing": ["ends-on-action", "ends-on-marker", "ends-with-clip"] },
        advanced
    )]
    pub hit_volume: Option<f64>,
    #[setting(
        "Loudest at",
        default = 75,
        range(min = 10, max = 100),
        unit = "%",
        help = "How far through the swell it is loudest before it falls away.",
        show_if = { "source": ["made"], "sound": ["swell"] },
        advanced
    )]
    pub peak_at: u32,
    #[setting(
        "Moves",
        show_if = { "source": ["made"], "sound": ["whoosh"] },
        advanced
    )]
    pub pan: SoundPan,
    #[setting("Wave", show_if = { "source": ["made"], "sound": ["tone", "beep", "countdown"] }, advanced)]
    pub wave: ToneWave,
    #[setting(
        "Last beep length",
        default = 3.0,
        range(min = 1, max = 10),
        step = 0.5,
        unit = "×",
        help = "How much longer the last beep of a countdown is.",
        show_if = { "source": ["made"], "sound": ["countdown"] },
        advanced
    )]
    pub final_length: f64,
    #[setting(
        "Last beep pitch",
        range(min = -24, max = 24),
        step = 1,
        help = "Semitones up or down from the others.",
        show_if = { "source": ["made"], "sound": ["countdown"] },
        advanced
    )]
    pub final_pitch: f64,
    #[setting(
        "Keep music up",
        help = "Music set to lower under other sounds stays at full volume for this one.",
        advanced
    )]
    pub keep_music: bool,
    #[setting(
        "Keyframes",
        help = "Change volume and left–right balance over time. Edit them here or on the timeline.",
        widget = "envelope",
        advanced
    )]
    pub envelope: AudioEnvelope,
    #[setting(
        "Play for",
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        zero_label = "The whole sound",
        advanced
    )]
    pub play_ms: u32,
    #[setting(
        "Before the clip starts",
        help = "When the action comes so early the sound would start before the clip.",
        show_if = { "timing": ["on-action"] },
        advanced
    )]
    pub before_clip_start: BeforeClipStart,
}

impl SoundEffect {
    pub fn recipe(&self) -> Option<SoundRecipe> {
        let preset = self.sound.preset()?;
        Some(SoundRecipe {
            variation: self.variation,
            length_ms: self.custom_ms,
            pitch: self.pitch,
            brightness: self.brightness,
            take: self.take,
            pan: self.pan,
            ..SoundRecipe::of(preset)
        })
    }

    pub fn shape(&self) -> Option<SoundShape> {
        let recipe = self.recipe()?;
        Some(SoundShape {
            peak_at: f64::from(self.peak_at) / 100.0,
            ..sound_shape(&recipe)
        })
    }

    pub fn hit_shape(&self) -> Option<SoundShape> {
        if !self.timing.ends() {
            return None;
        }
        let preset = match self.end_hit {
            EndHit::Off => return None,
            EndHit::Pop => SoundPreset::Pop,
            EndHit::Ding => SoundPreset::Ding,
            EndHit::Tap => SoundPreset::Tap,
            EndHit::Click => SoundPreset::Click,
        };
        Some(sound_shape(&SoundRecipe::of(preset)))
    }

    pub const fn makes_sound(&self) -> bool {
        matches!(self.source, SoundSource::Made) || !matches!(self.end_hit, EndHit::Off)
    }
}

fn made_file<P>(ctx: &Ctx<'_, P>, file: &str) -> Option<String> {
    ctx.result?
        .artifacts
        .iter()
        .find(|a| a.path.ends_with(&format!("/{file}")))
        .map(|a| project_ref(&a.path))
}

fn made_sound(ctx: &Ctx<'_, SoundEffect>) -> Option<(String, Option<f64>)> {
    let media = ctx.result?.output(AUDIO_PORT)?.as_media()?;
    Some((media.src.clone()?, media.duration_ms.map(f64::from)))
}

fn marker_frame(ctx: &Ctx<'_, SoundEffect>) -> Option<i32> {
    let marker = ctx
        .workflow
        .markers
        .iter()
        .find(|m| m.id == ctx.params.marker)?;
    Some(whole_signed(
        marker.at_ms * f64::from(ctx.settings.fps) / 1000.0,
    ))
}

fn what_row(p: &SoundEffect, inputs: &crate::node::CardInputs) -> CardRow {
    match p.source {
        SoundSource::File => source_row(Icon::AudioLines, &p.src, inputs, "Choose a sound"),
        SoundSource::Made => {
            let detail = match (p.sound, p.shape()) {
                (_, Some(shape)) => format!(
                    "{} · {}",
                    p.variation.label().to_lowercase(),
                    format_duration(shape.seconds * 1000.0)
                ),
                (MadeSound::Countdown, None) => plural(f64::from(p.count), "beep"),
                (_, None) => format!("{} Hz", p.pitch_hz),
            };
            row(Icon::Sparkles, format!("{} · {detail}", p.sound.label()))
        }
    }
}

fn timing_row(p: &SoundEffect, inputs: &crate::node::CardInputs) -> CardRow {
    match p.timing {
        SoundTiming::AtTime => row(
            Icon::Clock,
            format!("At {}", format_duration(f64::from(p.at_ms))),
        ),
        SoundTiming::OnAction => row(
            Icon::MousePointerClick,
            format!(
                "On {}{}",
                picked_name(inputs, &p.at_action, "an action"),
                offset_label(p.offset_ms)
            ),
        ),
        SoundTiming::EndsOnAction if p.at_action.is_empty() => {
            missing(Icon::TriangleAlert, "Pick the action it ends on")
        }
        SoundTiming::EndsOnMarker if p.marker.is_empty() => {
            missing(Icon::TriangleAlert, "Pick the marker it ends on")
        }
        SoundTiming::EndsOnAction => row(
            Icon::MousePointerClick,
            format!("Ends on {}", picked_name(inputs, &p.at_action, "an action")),
        ),
        SoundTiming::EndsOnMarker => row(
            Icon::Bookmark,
            format!("Ends on {}", picked_name(inputs, &p.marker, "a marker")),
        ),
        SoundTiming::EndsWithClip => row(Icon::Clock, "Ends with the clip"),
    }
}

fn card(ctx: &CardCtx<'_, SoundEffect>) -> Vec<Option<CardRow>> {
    let p = &ctx.params;
    vec![
        Some(what_row(p, ctx.inputs)),
        Some(timing_row(p, ctx.inputs)),
        p.keep_music.then(|| row(Icon::Music, "Music stays up")),
        (p.makes_sound() && ctx.result.is_none()).then(not_made_yet),
    ]
}

fn end_anchor(p: &SoundEffect) -> Option<Anchor> {
    match p.timing {
        SoundTiming::EndsOnAction => Some(Anchor {
            node_id: p.at_action.clone(),
            keep_end: true,
            ..Anchor::default()
        }),
        SoundTiming::EndsWithClip => Some(Anchor {
            clip_end: true,
            keep_end: true,
            ..Anchor::default()
        }),
        _ => None,
    }
}

fn ending_item(
    ctx: &Ctx<'_, SoundEffect>,
    src: String,
    end_frame: Option<i32>,
    lead_frames: u32,
    length_frames: u32,
) -> AudioItem {
    let p = &ctx.params;
    let anchor = end_anchor(p);
    let start = end_frame.unwrap_or(0) - signed(lead_frames);
    let timing = Timing {
        from: if anchor.is_some() {
            -signed(lead_frames)
        } else {
            start.max(0)
        },
        duration_in_frames: None,
        looped: None,
        trim_before: (anchor.is_none() && start < 0).then(|| start.unsigned_abs()),
        fade_in_frames: None,
        fade_out_frames: None,
        anchor,
        length_frames: Some(length_frames),
        keep_music: p.keep_music.then_some(true),
    };
    sound_item(
        ctx.node_id,
        src,
        AudioRole::Effect,
        p.volume,
        &p.envelope,
        timing,
        EditHandles::default(),
    )
}

fn audio(items: Vec<AudioItem>) -> Outputs {
    outputs([(
        AUDIO_PORT,
        CompileValue::Audio(AudioValue {
            items,
            duration_in_frames: None,
            hits: None,
        }),
    )])
}

fn ending(ctx: &Ctx<'_, SoundEffect>, src: String, length_ms: Option<f64>) -> Outputs {
    let p = &ctx.params;
    let fps = ctx.settings.fps;
    let end_frame = match p.timing {
        SoundTiming::EndsOnMarker => match marker_frame(ctx) {
            Some(frame) => Some(frame),
            None => return silent_audio(),
        },
        SoundTiming::EndsOnAction if p.at_action.is_empty() => return silent_audio(),
        _ => None,
    };
    let length = length_ms.map_or(1, |ms| ms_to_frames(ms, fps).max(1));
    let mut items = vec![ending_item(ctx, src, end_frame, length, length)];
    if let (Some(hit), Some(shape)) = (made_file(ctx, END_HIT_FILE), p.hit_shape()) {
        let hit_frames = ms_to_frames(shape.seconds * 1000.0, fps).max(1);
        let mut item = ending_item(ctx, hit, end_frame, 0, hit_frames);
        item.id = format!("{}:hit", ctx.node_id);
        item.trim_before = None;
        if let Some(volume) = p.hit_volume {
            item.volume = Some(volume);
        }
        items.push(item);
    }
    audio(items)
}

fn at_time(ctx: &Ctx<'_, SoundEffect>, src: String, length_ms: Option<f64>) -> Outputs {
    let p = &ctx.params;
    let fps = ctx.settings.fps;
    let on_action = p.timing == SoundTiming::OnAction;
    let timing = Timing {
        from: if on_action {
            ms_to_signed_frames(f64::from(p.offset_ms), fps)
        } else {
            signed(ms_to_frames_from(f64::from(p.at_ms), fps, 0))
        },
        duration_in_frames: frames_if_set(f64::from(p.play_ms), fps),
        looped: None,
        trim_before: None,
        fade_in_frames: None,
        fade_out_frames: None,
        anchor: on_action.then(|| Anchor {
            node_id: p.at_action.clone(),
            keep_end: p.before_clip_start == BeforeClipStart::Cut,
            ..Anchor::default()
        }),
        length_frames: length_ms.and_then(|ms| frames_if_set(ms, fps)),
        keep_music: p.keep_music.then_some(true),
    };
    let handles = EditHandles {
        move_to: Some(ParamRef::new(
            ctx.node_id,
            if on_action { "offsetMs" } else { "atMs" },
            ParamUnit::Ms,
        )),
        ..EditHandles::default()
    };
    audio(vec![sound_item(
        ctx.node_id,
        src,
        AudioRole::Effect,
        p.volume,
        &p.envelope,
        timing,
        handles,
    )])
}

fn placed(ctx: &Ctx<'_, SoundEffect>, src: String, length_ms: Option<f64>) -> Outputs {
    if ctx.params.timing.ends() {
        ending(ctx, src, length_ms)
    } else {
        at_time(ctx, src, length_ms)
    }
}

fn file_length(p: &SoundEffect, src: &str, media: Option<&MediaValue>) -> Option<f64> {
    let builtin_ms = builtin_sound(src).map(|s| f64::from(s.duration_ms));
    match media {
        Some(m) => m.duration_ms.map(f64::from).or(builtin_ms),
        None => builtin_ms.or(p.src_ms),
    }
}

fn compile(ctx: &Ctx<'_, SoundEffect>) -> Outputs {
    let p = &ctx.params;
    match p.source {
        SoundSource::File => with_source(ctx, &p.src, |src, media| {
            let length = file_length(p, &src, media);
            placed(ctx, src, length)
        }),
        SoundSource::Made => match made_sound(ctx) {
            Some((src, length)) => placed(ctx, src, length),
            None => silent_audio(),
        },
    }
}

pub fn sound_effect() -> NodeDef {
    NodeDef::of::<SoundEffect>(SOUND_EFFECT_TYPE, "Sound effect", Category::Audio, Phase::Media)
        .describe("A sound from a file or made from settings, at a set time, on a browser action, or building up to end exactly on an action, a marker or the end of a clip.")
        .executes()
        .inputs(vec![(MEDIA_PORT, opt(PortType::Media, "File"))])
        .outputs(vec![(AUDIO_PORT, port(PortType::Audio, "Sound"))])
        .card(card)
        .compiles(compile)
}
