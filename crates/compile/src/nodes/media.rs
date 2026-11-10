use std::collections::BTreeSet;

use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::mix::LOUDNESS_TARGETS;
use opsis_types::numbers;
use opsis_types::numbers::exact_eq;
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit, VisualEnvelope};
use opsis_types::values::{
    ClipValue, CompileValue, Measured, MediaKind, MediaValue, NodeResult, PortValues,
};
use opsis_types::workflow::PortType;
use serde::Serialize;

use crate::asset_ref::{extension_of, kind_from_path, project_ref};
use crate::card_rows::{file_row, range_row, truncate_card};
use crate::clip::{ANIMATION, IMAGE, ItemExtra, VIDEO, item_clip, placeholder_clip};
use crate::elements::element_meta;
use crate::elements::overlay::lottie::AnimationFit;
use crate::elements::shared::Fit;
use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, FileNeed, GRADE_PORT, Inputs, NodeDef, Outputs, Phase, PortFamily,
    Ports, RowExt, missing, port, row,
};
use crate::nodes::grade::{grade_changes_nothing, grade_needs_video, grade_step_title};
use crate::params::{Params, Values, to_values};
use crate::ports::{IN_PREFIX, in_index};
use crate::text::{format_duration, grouped, percent_text, plural, signed_text};
use crate::time::ms_to_frames;

pub const THUMBNAIL_TYPE: &str = "media.thumbnail";

pub const LOUDNORM_TYPE: &str = "media.loudnorm";
pub const FIX_CLICKS_TYPE: &str = "media.fixClicks";
pub const FIX_CLIPPING_TYPE: &str = "media.fixClipping";
pub const HUM_TYPE: &str = "media.hum";
pub const REMOVE_NOISE_TYPE: &str = "media.removeNoise";
pub const SOFTEN_S_TYPE: &str = "media.softenS";
pub const PAUSES_TYPE: &str = "media.shortenPauses";
pub const PITCH_TYPE: &str = "media.pitch";
pub const VOICE_TYPE: &str = "media.voiceEffect";
pub const FIT_MUSIC_TYPE: &str = "media.fitMusic";
pub const SOUND_PICTURE_TYPE: &str = "media.soundPicture";

pub const EXTRACT_AUDIO_TYPE: &str = "media.extractAudio";

pub const GIF_TYPE: &str = "media.gif";

pub const APPLY_GRADE_TYPE: &str = "media.applyGrade";

pub const CONVERT_TYPE: &str = "media.convert";

pub const SCALE_PAD_TYPE: &str = "media.scalePad";

pub const REVERSE_TYPE: &str = "media.reverse";

pub const SPEED_TYPE: &str = "media.speed";

pub const CONCAT_TYPE: &str = "media.concat";

pub const TRIM_TYPE: &str = "media.trim";

pub const IMPORT_TYPE: &str = "media.import";
pub const USE_AS_CLIP_TYPE: &str = "media.toClip";

const MEDIA_PORT: &str = "media";
pub const CLIP_ELEMENTS: [&str; 3] = [IMAGE, VIDEO, ANIMATION];
const NEW_FILE: &str = " Makes a new file.";
const MEDIA_MS: f64 = 5000.0;
const MIN_VIDEO_MS: u32 = 100;
const MIN_SPEED: f64 = 0.01;

#[choice(export)]
pub enum Codec {
    #[label("MP4")]
    H264,
    #[label("MP4 (HEVC)")]
    H265,
    #[label("WebM (VP8)")]
    Vp8,
    #[label("WebM")]
    Vp9,
    #[label("ProRes")]
    Prores,
    #[label("GIF")]
    Gif,
}

pub const QUALITY_PRESETS: [(&str, u32); 4] =
    [("Low", 28), ("Medium", 23), ("High", 18), ("Max", 12)];

pub fn quality_name(crf: u32) -> &'static str {
    let mut best = QUALITY_PRESETS[0];
    for preset in &QUALITY_PRESETS[1..] {
        let distance = preset.1.abs_diff(crf);
        let best_distance = best.1.abs_diff(crf);
        if distance < best_distance || (distance == best_distance && preset.1 < best.1) {
            best = *preset;
        }
    }
    best.0
}

pub fn loudness_presets() -> Vec<(String, f64)> {
    LOUDNESS_TARGETS
        .iter()
        .filter_map(|t| Some((t.label().to_owned(), t.target()?.lufs)))
        .collect()
}

pub fn quality_presets() -> Vec<(String, f64)> {
    QUALITY_PRESETS
        .iter()
        .map(|(label, crf)| ((*label).to_owned(), f64::from(*crf)))
        .collect()
}

pub fn loudness_name(lufs: f64) -> String {
    loudness_presets()
        .into_iter()
        .find(|(_, v)| exact_eq(*v, lufs))
        .map_or_else(
            || format!("{} LUFS", signed_text(lufs, None, false)),
            |(name, _)| name,
        )
}

#[choice]
pub enum ConvertCodec {
    #[label("MP4")]
    H264,
    #[label("MP4 (HEVC)")]
    H265,
    #[label("WebM")]
    Vp9,
    #[label("ProRes")]
    Prores,
}

impl ConvertCodec {
    const fn codec(self) -> Codec {
        match self {
            Self::H264 => Codec::H264,
            Self::H265 => Codec::H265,
            Self::Vp9 => Codec::Vp9,
            Self::Prores => Codec::Prores,
        }
    }
}

#[choice]
pub enum ProresProfile {
    #[label("Proxy")]
    Proxy,
    #[label("Light")]
    Lt,
    #[label("Standard")]
    Standard,
    #[default]
    #[label("High quality")]
    Hq,
}

impl ProresProfile {
    pub const fn number(self) -> u32 {
        match self {
            Self::Proxy => 0,
            Self::Lt => 1,
            Self::Standard => 2,
            Self::Hq => 3,
        }
    }
}

#[choice]
pub enum SoundQuality {
    #[label("Low")]
    Low,
    #[label("Medium")]
    Medium,
    #[default]
    #[label("High")]
    High,
}

#[settings(eq)]
pub struct Encoding {
    #[setting(
        "Keep its format",
        help = "Only makes a video web-ready, so it starts playing before it fully loads, with nothing converted. A video converted to a web format is web-ready anyway."
    )]
    pub keep_format: bool,
    #[setting(
        "Format",
        default = ConvertCodec::H264,
        show_if = { "keepFormat": ["false"] },
        advanced
    )]
    pub codec: ConvertCodec,
    #[setting(
        "Quality",
        default = 18,
        range(min = 0, max = 51),
        widget = "quality",
        show_if = { "keepFormat": ["false"], "codec": ["h264", "h265", "vp9"] }
    )]
    pub crf: u32,
    #[setting(
        "ProRes kind",
        show_if = { "keepFormat": ["false"], "codec": ["prores"] },
        advanced
    )]
    pub prores_profile: ProresProfile,
    #[setting(
        "Sound quality",
        show_if = { "keepFormat": ["false"], "codec": ["h264", "h265", "vp9"] },
        advanced
    )]
    pub sound_quality: SoundQuality,
}

fn format_row(codec: ConvertCodec, crf: u32) -> CardRow {
    let parts: Vec<String> = [
        Some(codec.codec().label().to_owned()),
        matches!(
            codec,
            ConvertCodec::H264 | ConvertCodec::H265 | ConvertCodec::Vp9
        )
        .then(|| format!("{} quality", quality_name(crf))),
        (codec == ConvertCodec::Prores).then(|| "can't play in a film".to_owned()),
        (codec == ConvertCodec::H265).then(|| "may not play in a film".to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect();
    row(Icon::FileVideo, parts.join(" · "))
}

fn media_of(result: Option<&NodeResult>, port: &str) -> Option<MediaValue> {
    result
        .and_then(|r| r.output(port))
        .and_then(CompileValue::as_media)
        .cloned()
}

fn checked_media(result: Option<&NodeResult>, path: &str) -> Option<MediaValue> {
    media_of(result, MEDIA_PORT)
        .filter(|m| !path.is_empty() && m.src.as_deref() == Some(project_ref(path).as_str()))
}

fn file_measured(result: Option<&NodeResult>) -> Option<(u64, Option<f64>)> {
    result?.measured.iter().find_map(|m| match m {
        Measured::File { bytes, fps } => Some((*bytes, *fps)),
        Measured::NoiseLevel { .. }
        | Measured::Changes { .. }
        | Measured::Fitted { .. }
        | Measured::Tape { .. }
        | Measured::Cast { .. } => None,
    })
}

fn file_details_row(media: &MediaValue, path: &str, fps: Option<f64>) -> CardRow {
    let mut parts: Vec<String> = Some(extension_of(path).to_uppercase())
        .filter(|f| !f.is_empty())
        .into_iter()
        .collect();
    if let (Some(w), Some(h)) = (media.width, media.height) {
        parts.push(format!("{w} × {h}"));
    }
    if let Some(fps) = fps {
        parts.push(format!("{} fps", grouped(fps, 2)));
    }
    let icon = match media.kind {
        MediaKind::Video => Icon::Film,
        MediaKind::Image => Icon::Image,
        MediaKind::Audio => {
            parts.push("Sound".to_owned());
            Icon::AudioLines
        }
        MediaKind::TerminalTake => {
            parts.push("Terminal take".to_owned());
            Icon::SquareTerminal
        }
        MediaKind::Animation => {
            parts.push("Animation".to_owned());
            Icon::Shapes
        }
    };
    row(icon, parts.join(" · "))
}

fn file_size_row(media: &MediaValue, bytes: u64) -> CardRow {
    let size = opsis_types::numbers::byte_size(bytes);
    match (media.kind, media.has_audio) {
        (MediaKind::Video, Some(false)) => row(Icon::VolumeX, format!("{size} · No sound")),
        (MediaKind::Video, Some(true)) => row(Icon::HardDrive, format!("{size} · With sound")),
        _ => row(Icon::HardDrive, size),
    }
}

fn length_row(result: Option<&NodeResult>, icon: Icon) -> Option<CardRow> {
    let duration = media_of(result, MEDIA_PORT)?
        .duration_ms
        .filter(|d| *d != 0)?;
    Some(row(icon, format_duration(f64::from(duration))))
}

fn pass_media(result: Option<&NodeResult>, inputs: &Inputs, fallback_port: &str) -> Outputs {
    let media = media_of(result, MEDIA_PORT).or_else(|| {
        inputs
            .get(fallback_port)
            .and_then(CompileValue::as_media)
            .cloned()
    });
    media
        .map(|m| (MEDIA_PORT.to_owned(), CompileValue::Media(m)))
        .into_iter()
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageClipProps<'a> {
    src: &'a str,
    fit: Fit,
    background: &'a str,
    ken_burns: bool,
    ken_burns_zoom: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoClipProps<'a> {
    src: &'a str,
    fit: Fit,
    background: &'a str,
    #[serde(rename = "loop")]
    looped: bool,
    muted: bool,
    volume: f64,
    playback_rate: f64,
    trim_before_ms: u32,
    dim: f64,
    dim_color: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnimationClipProps<'a> {
    src: &'a str,
    fit: AnimationFit,
    background: &'a str,
    #[serde(rename = "loop")]
    looped: bool,
    speed: f64,
    backwards: bool,
}

fn file_props(component: &str, file: &impl Serialize) -> Values {
    let mut props = element_meta(component).map_or_default(|meta| meta.default_props.clone());
    props.extend(to_values(file));
    props
}

const fn transparent_if_empty(background: &str) -> &str {
    if background.is_empty() {
        "transparent"
    } else {
        background
    }
}

fn clip_ms(media: &MediaValue, p: &UseAsClip) -> f64 {
    match media.kind {
        MediaKind::Image | MediaKind::Animation => f64::from(p.length_ms),
        _ => {
            let whole = media.duration_ms.map_or(MEDIA_MS, f64::from);
            let left = (whole - f64::from(p.trim_ms)).max(f64::from(MIN_VIDEO_MS));
            left / p.speed.max(MIN_SPEED)
        }
    }
}

fn media_to_clip(node_id: &str, media: Option<&MediaValue>, p: &UseAsClip, fps: u32) -> ClipValue {
    let extra = ItemExtra {
        envelope: Some(p.envelope.clone()),
        ..ItemExtra::default()
    };
    let Some((media, src)) = media.and_then(|m| m.src.as_ref().map(|s| (m, s))) else {
        let frames = ms_to_frames(f64::from(p.length_ms), fps);
        return placeholder_clip(
            node_id,
            "No file connected",
            "Connect a file to use it as a clip",
            frames,
            extra,
        );
    };
    let frames = ms_to_frames(clip_ms(media, p), fps);
    let length = || EditHandles {
        trim_end: Some(ParamRef::new(node_id, "lengthMs", ParamUnit::Ms)),
        ..EditHandles::default()
    };
    match media.kind {
        MediaKind::Audio => placeholder_clip(
            node_id,
            "Sound file",
            "Sound files can't be shown as a clip",
            frames,
            extra,
        ),
        MediaKind::TerminalTake => placeholder_clip(
            node_id,
            "Terminal take",
            crate::node::TERMINAL_TAKE_FILE,
            frames,
            extra,
        ),
        MediaKind::Image => {
            let props = file_props(
                IMAGE,
                &ImageClipProps {
                    src,
                    fit: p.fit,
                    background: &p.background,
                    ken_burns: p.ken_burns,
                    ken_burns_zoom: p.ken_burns_zoom,
                },
            );
            let extra = ItemExtra {
                handles: Some(length()),
                ..extra
            };
            item_clip(node_id, IMAGE, props, frames, extra)
        }
        MediaKind::Animation => {
            let props = file_props(
                ANIMATION,
                &AnimationClipProps {
                    src,
                    fit: if p.fit == Fit::Contain {
                        AnimationFit::Contain
                    } else {
                        AnimationFit::Cover
                    },
                    background: transparent_if_empty(&p.background),
                    looped: p.looped,
                    speed: p.speed,
                    backwards: p.backwards,
                },
            );
            let extra = ItemExtra {
                handles: Some(length()),
                ..extra
            };
            item_clip(node_id, ANIMATION, props, frames, extra)
        }
        MediaKind::Video => {
            let props = file_props(
                VIDEO,
                &VideoClipProps {
                    src,
                    fit: p.fit,
                    background: &p.background,
                    looped: p.looped,
                    muted: p.muted || media.has_audio == Some(false),
                    volume: p.volume,
                    playback_rate: p.speed,
                    trim_before_ms: p.trim_ms,
                    dim: p.dim,
                    dim_color: &p.dim_color,
                },
            );
            item_clip(node_id, VIDEO, props, frames, extra)
        }
    }
}

fn one() -> (Ports, Ports) {
    (
        vec![(MEDIA_PORT, port(PortType::Media, "File"))],
        vec![(MEDIA_PORT, port(PortType::Media, "File"))],
    )
}

fn file_op<P: Params>(kind: &'static str, title: &str, description: impl Into<String>) -> NodeDef {
    NodeDef::of::<P>(kind, title, Category::Media, Phase::Media)
        .describe(description)
        .executes()
        .compile(|ctx: &Ctx<'_, P>| Ok(Some(pass_media(ctx.result, ctx.inputs, MEDIA_PORT))))
}

fn one_file_op<P: Params>(kind: &'static str, title: &str, description: &str) -> NodeDef {
    let (inputs, outputs) = one();
    file_op::<P>(kind, title, format!("{description}{NEW_FILE}"))
        .inputs(inputs)
        .outputs(outputs)
}

#[settings(eq)]
pub struct MediaFile {
    #[setting("File", widget = "asset", accept = "*/*")]
    pub path: String,
}

fn import() -> NodeDef {
    file_op::<MediaFile>(
        IMPORT_TYPE,
        "Media file",
        "A video, image or sound from your project. Drop files on the canvas to add one.",
    )
    .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .card(|ctx: &CardCtx<'_, MediaFile>| {
        let p = &ctx.params;
        if p.path.is_empty() {
            return vec![Some(missing(Icon::FileVideo, "Choose a file"))];
        }
        let name = Some(file_row(Icon::FileVideo, &p.path));
        if ctx.inputs.missing_files.contains(&p.path) {
            return vec![
                name,
                Some(missing(Icon::TriangleAlert, "Not in the project folder")),
            ];
        }
        let Some(media) = checked_media(ctx.result, &p.path) else {
            return vec![
                name,
                Some(row(Icon::Info, "Not read yet").tone(Tone::Muted)),
            ];
        };
        let measured = file_measured(ctx.result);
        let length = media.duration_ms.filter(|d| *d != 0).map(f64::from);
        vec![
            name,
            Some(file_details_row(
                &media,
                &p.path,
                measured.and_then(|m| m.1),
            )),
            length.map(|ms| row(Icon::Clock, format_duration(ms))),
            measured.map(|(bytes, _)| file_size_row(&media, bytes)),
        ]
    })
    .compiles(|ctx: &Ctx<'_, MediaFile>| {
        let p = &ctx.params;
        let media = checked_media(ctx.result, &p.path).unwrap_or_else(|| MediaValue {
            src: (!p.path.is_empty()).then(|| project_ref(&p.path)),
            kind: kind_from_path(&p.path),
            duration_ms: None,
            width: None,
            height: None,
            has_audio: None,
        });
        crate::node::outputs([(MEDIA_PORT, CompileValue::Media(media))])
    })
}

#[settings]
pub struct UseAsClip {
    #[setting("Fit", default = Fit::Contain)]
    pub fit: Fit,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty,
        help = "Fills the room Show all leaves around the picture.",
        advanced
    )]
    pub background: String,
    #[setting(
        "Length",
        default = 3000,
        range(min = 100, max = 3_600_000),
        step = 50,
        unit = "ms",
        help = "How long the picture or animation stays on screen.",
        show_if_file = { "media": ["image", "animation"] }
    )]
    pub length_ms: u32,
    #[setting("Slow push-in", show_if_file = { "media": ["image"] })]
    pub ken_burns: bool,
    #[setting(
        "Push-in zoom",
        default = 1.06,
        range(min = 1, max = 1.5),
        step = 0.01,
        show_if = { "kenBurns": ["true"] },
        show_if_file = { "media": ["image"] },
        advanced
    )]
    pub ken_burns_zoom: f64,
    #[serde(rename = "loop")]
    #[setting("Loop", show_if_file = { "media": ["video", "animation"] })]
    pub looped: bool,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.05,
        show_if_file = { "media": ["video", "animation"] }
    )]
    pub speed: f64,
    #[setting("Play backwards", show_if_file = { "media": ["animation"] })]
    pub backwards: bool,
    #[setting(
        "Start at",
        range(min = 0, max = 3_600_000),
        step = 100,
        unit = "ms",
        show_if_file = { "media": ["video"] }
    )]
    pub trim_ms: u32,
    #[setting("Muted", show_if_file = { "media": ["video"] })]
    pub muted: bool,
    #[setting(
        "Volume",
        default = 1.0,
        range(min = 0, max = 2),
        step = 0.05,
        show_if = { "muted": ["false"] },
        show_if_file = { "media": ["video"] }
    )]
    pub volume: f64,
    #[setting(
        "Dim",
        range(min = 0, max = 1),
        step = 0.05,
        help = "Darkens the video, so text laid over it reads.",
        show_if_file = { "media": ["video"] }
    )]
    pub dim: f64,
    #[setting(
        "Dim colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if_file = { "media": ["video"] },
        advanced
    )]
    pub dim_color: String,
    #[setting(
        "Keyframes",
        help = "Change opacity, position and size over time. Edit them here or on the timeline.",
        widget = "envelope",
        advanced
    )]
    pub envelope: VisualEnvelope,
}

fn use_as_clip() -> NodeDef {
    NodeDef::of::<UseAsClip>(
        USE_AS_CLIP_TYPE,
        "Use as clip",
        Category::Media,
        Phase::Compile,
    )
    .describe(
        "Turns a file into a clip you can place in the video: an image, a video or an animation.",
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .outputs(vec![(crate::node::CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, UseAsClip>| {
        let p = &ctx.params;
        vec![
            Some(row(Icon::Maximize, p.fit.label())),
            p.muted.then(|| row(Icon::VolumeX, "Muted")),
            p.looped.then(|| row(Icon::Repeat, "Loops")),
            (!exact_eq(p.speed, 1.0))
                .then(|| row(Icon::FastForward, format!("{}×", grouped(p.speed, 2)))),
        ]
    })
    .compiles(|ctx: &Ctx<'_, UseAsClip>| {
        let clip = media_to_clip(
            ctx.node_id,
            ctx.input(MEDIA_PORT).and_then(CompileValue::as_media),
            &ctx.params,
            ctx.settings.fps,
        );
        crate::node::outputs([(crate::node::CLIP_PORT, CompileValue::Clip(clip))])
    })
}

#[settings(check, eq)]
pub struct TrimFile {
    #[setting("Start", range(min = 0, max = 36_000_000), step = 50, unit = "ms")]
    pub start_ms: u32,
    #[setting(
        "End",
        range(min = 0, max = 36_000_000),
        step = 50,
        unit = "ms",
        zero_label = "To the end"
    )]
    pub end_ms: u32,
}

impl Params for TrimFile {
    fn check(&self) -> Result<(), SettingsError> {
        if self.end_ms != 0 && self.end_ms <= self.start_ms {
            return Err(SettingsError::rule("End must come after Start"));
        }
        Ok(())
    }
}

#[choice]
pub enum JoinSizeFrom {
    #[default]
    #[label("Of the first file")]
    First,
    #[label("Of the project")]
    Project,
}

#[settings(eq)]
pub struct JoinFiles {
    #[setting("Always convert", advanced)]
    pub force_reencode: bool,
    #[setting(
        "Picture length",
        default = 3000,
        range(min = 100, max = 3_600_000),
        step = 50,
        unit = "ms",
        help = "How long an image joined in stays on screen.",
        advanced
    )]
    pub still_ms: u32,
    #[setting(
        "Size and frame rate",
        help = "What files that don't match are converted to.",
        advanced
    )]
    pub match_to: JoinSizeFrom,
    #[setting(
        "Background",
        default = "#000000",
        widget = "color",
        check = colour,
        help = "Behind sound-only parts and around files of another shape.",
        advanced
    )]
    pub background: String,
}

fn file_label(i: usize, _last: Option<usize>) -> String {
    format!("File {}", i + 1)
}

pub fn joined_files(inputs: &PortValues) -> Vec<&CompileValue> {
    let mut found: Vec<(usize, &CompileValue)> = inputs
        .iter()
        .filter_map(|(k, v)| Some((in_index(k)?, v)))
        .collect();
    found.sort_by_key(|(i, _)| *i);
    found.into_iter().map(|(_, v)| v).collect()
}

fn join_files() -> NodeDef {
    NodeDef::of::<JoinFiles>(CONCAT_TYPE, "Join files", Category::Media, Phase::Media)
        .describe(format!(
            "Joins files end to end. Fast when they match, otherwise converts them.{NEW_FILE}"
        ))
        .executes()
        .uses_project_size()
        .family(PortFamily::new(IN_PREFIX, PortType::Media, 2, file_label))
        .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
        .card(|ctx: &CardCtx<'_, JoinFiles>| {
            let n = ctx
                .inputs
                .connected
                .iter()
                .filter(|c| in_index(c).is_some())
                .count();
            vec![
                Some(match n {
                    0 => missing(Icon::Combine, "Connect two files"),
                    1 => missing(Icon::Combine, "Connect one more file"),
                    n => row(Icon::Combine, plural(numbers::float(n), "file")),
                }),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .compile(|ctx: &Ctx<'_, JoinFiles>| Ok(Some(pass_media(ctx.result, ctx.inputs, "in1"))))
}

#[settings]
pub struct ChangeSpeed {
    #[setting(
        "Speed",
        default = 2.0,
        range(min = 0.25, max = 64),
        step = 0.05,
        unit = "×"
    )]
    pub rate: f64,
    #[setting("Keep the sound", default = true)]
    pub keep_audio: bool,
    #[setting(
        "Keep the pitch",
        default = true,
        help = "Off, the sound goes higher when faster and lower when slower.",
        show_if = { "keepAudio": ["true"] },
        advanced
    )]
    pub keep_pitch: bool,
}

#[settings(eq)]
pub struct ReverseFile {
    #[setting(
        "Keep the sound",
        default = true,
        help = "Plays the sound backwards with the picture."
    )]
    pub keep_audio: bool,
}

#[settings(eq)]
pub struct NoSettings {}

#[choice]
pub enum ScaleMode {
    #[label("Add bars")]
    Pad,
    #[label("Crop to fill")]
    Crop,
    #[label("Stretch")]
    Stretch,
}

impl ScaleMode {
    const fn row(self) -> &'static str {
        match self {
            Self::Pad => "Fit with bars",
            Self::Crop => "Crop to fill",
            Self::Stretch => "Stretch",
        }
    }
}

#[settings(eq)]
pub struct Resize {
    #[setting(
        "Width",
        default = 1920,
        range(min = 16, max = 7680),
        step = 2,
        unit = "px",
        help = "A video takes even sizes; an odd one loses a pixel."
    )]
    pub width: u32,
    #[setting(
        "Height",
        default = 1080,
        range(min = 16, max = 7680),
        step = 2,
        unit = "px",
        help = "A video takes even sizes; an odd one loses a pixel."
    )]
    pub height: u32,
    #[setting(
        "Fit",
        default = ScaleMode::Pad
    )]
    pub mode: ScaleMode,
    #[setting(
        "Bar colour",
        default = "black",
        widget = "color",
        zero_label = "Default",
        show_if = { "mode": ["pad"] },
        check = colour_or_empty
    )]
    pub background: String,
    #[setting(
        "Place",
        labels = { "start": "Top or left", "center": "Centre", "end": "Bottom or right" },
        help = "Where the picture sits in the bars, or which part a crop keeps.",
        show_if = { "mode": ["pad", "crop"] },
        advanced
    )]
    pub anchor: crate::nodes::image::PlaceAlign,
}

#[choice(rename_all = "snake_case")]
pub enum Dither {
    #[serde(rename = "sierra2_4a")]
    #[label("Smooth")]
    Sierra24a,
    #[label("Patterned")]
    Bayer,
    #[label("Fine")]
    FloydSteinberg,
    #[label("Off")]
    None,
}

#[settings(eq)]
pub struct MakeGif {
    #[setting("Frame rate", default = 15, range(min = 5, max = 50), unit = "fps")]
    pub fps: u32,
    #[setting("Width", default = 960, range(min = 64, max = 3840), unit = "px")]
    pub width: u32,
    #[setting("Loop forever", default = true)]
    pub r#loop: bool,
    #[setting(
        "Smooth colours",
        default = Dither::Sierra24a,
        advanced
    )]
    pub dither: Dither,
    #[setting("Colours", default = 256, range(min = 2, max = 256), advanced)]
    pub colours: u32,
    #[setting(
        "Colours from",
        help = "What the colour table is chosen from.",
        advanced
    )]
    pub palette_from: PaletteFrom,
}

#[choice]
pub enum PaletteFrom {
    #[default]
    #[label("What moves")]
    Moving,
    #[label("The whole picture")]
    Whole,
}

#[choice]
pub enum SoundFormat {
    #[label("M4A")]
    M4a,
    #[label("MP3")]
    Mp3,
    #[label("WAV")]
    Wav,
}

#[settings(eq)]
pub struct ExtractSound {
    #[setting(
        "Format",
        default = SoundFormat::M4a
    )]
    pub format: SoundFormat,
    #[setting(
        "Bit rate",
        default = 192,
        range(min = 64, max = 320),
        step = 32,
        unit = "kbps",
        show_if = { "format": ["m4a"] },
        advanced
    )]
    pub bitrate: u32,
}

#[settings]
pub struct EvenOutVolume {
    #[setting(
        "Loudness",
        default = -16.0,
        range(min = -40, max = -5),
        widget = "loudness",
        unit = "LUFS"
    )]
    pub integrated: f64,
    #[setting(
        "Peak limit",
        default = -1.5,
        range(min = -9, max = 0),
        step = 0.1,
        unit = "dB",
        advanced
    )]
    pub true_peak: f64,
    #[setting("Dynamic range", default = 11.0, range(min = 1, max = 20), advanced)]
    pub lra: f64,
    #[setting(
        "Lift quiet words",
        help = "Lifts quiet words and holds back loud ones before setting the loudness, for a voice that comes and goes."
    )]
    pub lift_quiet: bool,
    #[setting(
        "Most lift",
        default = 16.0,
        range(min = 0, max = 30),
        step = 1,
        unit = "dB",
        help = "How far a quiet word may be lifted.",
        show_if = { "liftQuiet": ["true"] },
        advanced
    )]
    pub most_lift: f64,
}

#[settings]
pub struct FixClicks {
    #[setting(
        "Strength",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        help = "Stronger finds more, but can soften sharp sounds that are meant to be there."
    )]
    pub strength: f64,
}

#[settings]
pub struct FixClipping {
    #[setting(
        "Strength",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        help = "Stronger rebuilds more peaks, but can change loud sounds that weren't cut off."
    )]
    pub strength: f64,
}

#[choice]
pub enum Mains {
    #[default]
    #[label("Find it")]
    Find,
    #[label("50 Hz")]
    Fifty,
    #[label("60 Hz")]
    Sixty,
}

impl Mains {
    pub const fn hertz(self) -> Option<f64> {
        match self {
            Self::Find => None,
            Self::Fifty => Some(50.0),
            Self::Sixty => Some(60.0),
        }
    }
}

#[settings]
pub struct RemoveHum {
    #[setting(
        "Hum",
        help = "Mains hum is 50 Hz in most of the world and 60 Hz in the Americas. Find it listens for both first."
    )]
    pub mains: Mains,
    #[setting(
        "Harmonics",
        default = 4,
        range(min = 0, max = 10),
        help = "Hum also rings at twice, three times and more of its pitch.",
        advanced
    )]
    pub harmonics: u32,
    #[setting(
        "Notch width",
        default = 2.0,
        range(min = 0.5, max = 20),
        step = 0.5,
        unit = "Hz",
        advanced
    )]
    pub width: f64,
    #[setting(
        "Depth",
        default = -40.0,
        range(min = -80, max = -6),
        step = 1,
        unit = "dB",
        help = "How far each notch cuts.",
        advanced
    )]
    pub depth: f64,
}

#[settings]
pub struct ChangePitch {
    #[setting(
        "Semitones",
        default = 3.0,
        range(min = -24, max = 24),
        step = 0.5,
        help = "Up or down, 12 to an octave. The length stays the same."
    )]
    pub semitones: f64,
    #[setting(
        "Keep the voice's character",
        default = true,
        help = "Keeps a voice sounding like the same person. Turn it off for a cartoon or a deep voice."
    )]
    pub keep_character: bool,
}

#[choice]
pub enum VoiceEffect {
    #[default]
    #[label("Telephone")]
    Telephone,
    #[label("Radio")]
    Radio,
    #[label("Small room")]
    SmallRoom,
    #[label("Hall")]
    Hall,
    #[label("Echo")]
    Echo,
}

#[settings]
pub struct VoiceEffects {
    #[setting("Effect")]
    pub effect: VoiceEffect,
    #[setting(
        "Mix",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        help = "How much of the effect against the original. Rooms and echoes sound natural around 0.3."
    )]
    pub mix: f64,
    #[setting(
        "Echo after",
        default = 300,
        range(min = 20, max = 2000),
        step = 10,
        unit = "ms",
        show_if = { "effect": ["echo"] }
    )]
    pub delay_ms: u32,
    #[setting(
        "Echo level",
        default = 0.4,
        range(min = 0.05, max = 0.9),
        step = 0.05,
        help = "How loud the echo is against the sound.",
        show_if = { "effect": ["echo"] }
    )]
    pub decay: f64,
    #[setting(
        "Repeats",
        default = 1,
        range(min = 1, max = 8),
        help = "Each repeat comes one delay later at the echo level of the one before.",
        show_if = { "effect": ["echo"] },
        advanced
    )]
    pub repeats: u32,
    #[setting(
        "Grit",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.1,
        help = "How rough the line sounds.",
        show_if = { "effect": ["telephone", "radio"] },
        advanced
    )]
    pub grit: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Room length",
        range(min = 50, max = 10000),
        step = 50,
        unit = "ms",
        zero_label = "The room's own",
        help = "How long the room rings on.",
        show_if = { "effect": ["small-room", "hall"] },
        "type" = "integer",
        advanced
    )]
    pub reverb_ms: Option<u32>,
}

#[choice]
pub enum FitWay {
    #[default]
    #[label("Stretch, or loop when that's too far")]
    Both,
    #[label("Stretch only")]
    Stretch,
    #[label("Loop only")]
    Loop,
}

#[settings]
pub struct FitMusic {
    #[setting(
        "Song",
        widget = "music",
        accept = "audio/*",
        help = "A connected sound file takes its place."
    )]
    pub src: String,
    #[setting(
        "Way",
        help = "Stretching changes the tempo a little and keeps the song whole. Looping repeats a part of it with a crossfade."
    )]
    pub way: FitWay,
    #[setting(
        "Largest stretch",
        default = 8.0,
        range(min = 1, max = 25),
        step = 1,
        unit = "%",
        show_if = { "way": ["both"] },
        advanced
    )]
    pub max_stretch: f64,
    #[setting(
        "Loop from",
        range(min = 0, max = 3_600_000),
        step = 100,
        unit = "ms",
        show_if = { "way": ["both", "loop"] },
        advanced
    )]
    pub loop_from_ms: u32,
    #[setting(
        "Loop to",
        range(min = 0, max = 3_600_000),
        step = 100,
        unit = "ms",
        zero_label = "The end of the song",
        show_if = { "way": ["both", "loop"] },
        advanced
    )]
    pub loop_to_ms: u32,
    #[setting(
        "Crossfade",
        default = 1000,
        range(min = 10, max = 10000),
        step = 50,
        unit = "ms",
        show_if = { "way": ["both", "loop"] },
        advanced
    )]
    pub crossfade_ms: u32,
    #[setting(
        "Fade at the end",
        default = 2000,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms"
    )]
    pub fade_out_ms: u32,
    #[setting("Fade curve", advanced)]
    pub fade_curve: FadeCurve,
}

pub use opsis_types::timeline::FadeCurve;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitPlan {
    Stretch {
        tempo: f64,
    },
    Loop {
        from_ms: f64,
        to_ms: f64,
        repeats: u32,
    },
    Cut,
}

const SHORTEST_LOOP_MS: f64 = 500.0;

pub fn fit_plan(song_ms: f64, target_ms: f64, p: &FitMusic) -> FitPlan {
    let tempo = song_ms / target_ms.max(1.0);
    let change = (target_ms / song_ms.max(1.0) - 1.0).abs() * 100.0;
    let stretch = match p.way {
        FitWay::Stretch => true,
        FitWay::Both => change <= p.max_stretch,
        FitWay::Loop => false,
    };
    if stretch {
        return FitPlan::Stretch { tempo };
    }
    if target_ms <= song_ms {
        return FitPlan::Cut;
    }
    let crossfade = f64::from(p.crossfade_ms);
    let to_ms = match p.loop_to_ms {
        0 => song_ms,
        to => f64::from(to).min(song_ms),
    };
    let mut from_ms = f64::from(p.loop_from_ms).min(to_ms);
    if to_ms - from_ms < crossfade + SHORTEST_LOOP_MS {
        from_ms = 0.0;
    }
    let step = (to_ms - from_ms - crossfade).max(SHORTEST_LOOP_MS);
    let repeats = numbers::truncate::<u32>(((target_ms - to_ms) / step).ceil().max(1.0));
    FitPlan::Loop {
        from_ms,
        to_ms,
        repeats,
    }
}

fn fitted_row(result: Option<&NodeResult>) -> Option<CardRow> {
    result?.measured.iter().find_map(|m| match m {
        Measured::Fitted { tempo: Some(t), .. } => {
            let change = 1.0 / t - 1.0;
            let shown = percent_text(change.abs(), f64::INFINITY);
            Some(if shown == "0%" {
                row(Icon::Music, "Already fits")
            } else if change > 0.0 {
                row(Icon::Music, format!("Stretched {shown}"))
            } else {
                row(Icon::FastForward, format!("Sped up {shown}"))
            })
        }
        Measured::Fitted { loops, .. } if *loops > 0 => Some(row(
            Icon::Repeat,
            match loops {
                1 => "Looped once".to_owned(),
                2 => "Looped twice".to_owned(),
                n => format!("Looped {n} times"),
            },
        )),
        Measured::Fitted { .. } => Some(row(Icon::Scissors, "Cut and faded")),
        Measured::NoiseLevel { .. }
        | Measured::File { .. }
        | Measured::Changes { .. }
        | Measured::Tape { .. }
        | Measured::Cast { .. } => None,
    })
}

pub fn pitch_ratio(semitones: f64) -> f64 {
    (semitones / 12.0).exp2()
}

#[choice]
pub enum NoiseMethod {
    #[default]
    #[label("Learned, for speech")]
    Learned,
    #[label("Spectral, for steady noise")]
    Spectral,
}

#[settings]
pub struct RemoveNoise {
    #[setting(
        "Strength",
        default = 0.8,
        range(min = 0, max = 1),
        step = 0.05,
        help = "How much of the background noise to take out. At 0 only the rumble goes."
    )]
    pub noise: f64,
    #[setting(
        "Way",
        help = "The learned way keeps only the voice, so music under it goes too. The spectral way takes out a steady hiss or hum and leaves the rest."
    )]
    pub method: NoiseMethod,
    #[setting(
        "Cut rumble below",
        default = 80,
        range(min = 0, max = 300),
        step = 10,
        unit = "Hz",
        help = "0 keeps the lowest sounds.",
        advanced
    )]
    pub rumble_hz: u32,
}

#[settings]
pub struct SoftenS {
    #[setting("Strength", default = 0.5, range(min = 0, max = 1), step = 0.05)]
    pub strength: f64,
}

fn noise_level_row(result: Option<&NodeResult>) -> Option<CardRow> {
    result?.measured.iter().find_map(|m| match m {
        Measured::NoiseLevel {
            before_db,
            after_db,
        } => Some(row(
            Icon::Volume2,
            format!("Noise {before_db:.0} dB → {after_db:.0} dB"),
        )),
        Measured::File { .. }
        | Measured::Changes { .. }
        | Measured::Fitted { .. }
        | Measured::Tape { .. }
        | Measured::Cast { .. } => None,
    })
}

#[settings]
pub struct ShortenPauses {
    #[setting(
        "Silence below",
        default = -40.0,
        range(min = -70, max = -20),
        step = 1,
        unit = "dB",
        help = "Quieter than this counts as a pause. Raise it for a noisy room.",
        advanced
    )]
    pub threshold: f64,
    #[setting(
        "Shorten pauses longer than",
        default = 700,
        range(min = 100, max = 10000),
        step = 50,
        unit = "ms"
    )]
    pub longer_than_ms: u32,
    #[setting(
        "Down to",
        default = 400,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms"
    )]
    pub keep_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pause {
    pub start_ms: f64,
    pub end_ms: f64,
}

pub fn kept_parts(duration_ms: f64, pauses: &[Pause], keep_ms: f64) -> Vec<(f64, f64)> {
    let mut parts = Vec::new();
    let mut from = 0.0;
    for pause in pauses {
        let length = pause.end_ms - pause.start_ms;
        if length <= keep_ms {
            continue;
        }
        let cut_from = pause.start_ms + keep_ms / 2.0;
        let cut_to = pause.end_ms - keep_ms / 2.0;
        if cut_from > from {
            parts.push((from, cut_from));
        }
        from = cut_to;
    }
    if duration_ms > from {
        parts.push((from, duration_ms));
    }
    parts
}

#[choice]
pub enum SoundPictureKind {
    #[default]
    #[label("Waveform")]
    Waveform,
    #[label("Spectrum")]
    Spectrum,
}

#[choice]
pub enum SpectrumColours {
    #[default]
    #[label("Magma")]
    Magma,
    #[label("Fire")]
    Fire,
    #[label("Viridis")]
    Viridis,
    #[label("Rainbow")]
    Rainbow,
    #[label("Cool")]
    Cool,
}

impl SpectrumColours {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Magma => "magma",
            Self::Fire => "fire",
            Self::Viridis => "viridis",
            Self::Rainbow => "rainbow",
            Self::Cool => "cool",
        }
    }
}

#[settings(eq)]
pub struct SoundPicture {
    #[setting(
        "Picture",
        help = "A waveform shows loudness over time; a spectrum also shows pitch, low at the bottom."
    )]
    pub kind: SoundPictureKind,
    #[setting("Width", default = 1920, range(min = 64, max = 8192), unit = "px")]
    pub width: u32,
    #[setting("Height", default = 400, range(min = 32, max = 4096), unit = "px")]
    pub height: u32,
    #[setting(
        "Colour",
        default = "#6366f1",
        widget = "color",
        check = colour,
        show_if = { "kind": ["waveform"] }
    )]
    pub color: String,
    #[setting(
        "Colours",
        show_if = { "kind": ["spectrum"] }
    )]
    pub colours: SpectrumColours,
    #[setting("Background", default = "#00000000", widget = "color", check = colour)]
    pub background: String,
    #[setting(
        "Each channel apart",
        help = "Draws left and right one above the other.",
        show_if = { "kind": ["waveform"] },
        advanced
    )]
    pub split_channels: bool,
    #[setting(
        "Loudness scale",
        help = "Log and the roots show quiet parts larger.",
        show_if = { "kind": ["waveform"] },
        advanced
    )]
    pub wave_scale: WaveScale,
    #[setting(
        "Pitch scale",
        help = "Log gives low pitches more room.",
        show_if = { "kind": ["spectrum"] },
        advanced
    )]
    pub pitch_scale: PitchScale,
}

#[choice]
pub enum WaveScale {
    #[default]
    #[label("Even")]
    Lin,
    #[label("Log")]
    Log,
    #[label("Square root")]
    Sqrt,
    #[label("Cube root")]
    Cbrt,
}

#[choice]
pub enum PitchScale {
    #[default]
    #[label("Even")]
    Lin,
    #[label("Log")]
    Log,
}

pub const SOUND_PICTURE_PORT: &str = "image";

fn sound_picture() -> NodeDef {
    NodeDef::of::<SoundPicture>(
        SOUND_PICTURE_TYPE,
        "Sound picture",
        Category::Media,
        Phase::Media,
    )
    .describe("Draws a sound's waveform or spectrum as an image.")
    .executes()
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "Sound"))])
    .outputs(vec![(SOUND_PICTURE_PORT, port(PortType::Image, "Image"))])
    .card(|ctx: &CardCtx<'_, SoundPicture>| {
        let p = &ctx.params;
        let kind = p.kind.label();
        vec![Some(row(
            Icon::AudioLines,
            format!("{kind} · {} × {} px", p.width, p.height),
        ))]
    })
    .compile(|ctx: &Ctx<'_, SoundPicture>| {
        let image = ctx
            .result
            .and_then(|r| r.output(SOUND_PICTURE_PORT))
            .filter(|v| matches!(v, CompileValue::Image(_)));
        Ok(Some(
            image
                .map(|v| (SOUND_PICTURE_PORT.to_owned(), v.clone()))
                .into_iter()
                .collect(),
        ))
    })
}

#[settings(eq)]
pub struct GrabFrame {
    #[setting(
        "At",
        default = 1000,
        range(min = 0, max = 36_000_000),
        step = 50,
        unit = "ms"
    )]
    pub at_ms: u32,
    #[setting(
        "Width",
        default = 1280,
        range(min = 0, max = 7680),
        unit = "px",
        help = "0 keeps the frame's own size."
    )]
    pub width: u32,
}

fn apply_grade() -> NodeDef {
    file_op::<NoSettings>(
        APPLY_GRADE_TYPE,
        "Apply grade",
        format!(
            "Applies a colour grade to a whole video or image file: everything in the picture, window and backdrop included.{NEW_FILE}"
        ),
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File")), (GRADE_PORT, port(PortType::Grade, "Grade"))])
    .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .card(|ctx: &CardCtx<'_, NoSettings>| {
        let entries = ctx.inputs.grade.get(GRADE_PORT);
        let image = ctx.inputs.media.get(MEDIA_PORT).is_some_and(|m| m.kind == MediaKind::Image);
        let mut seen = BTreeSet::new();
        let titles: Vec<String> =
            entries.into_iter().flatten().map(|e| grade_step_title(&e.kind)).filter(|t| seen.insert(t.clone())).collect();
        let has = entries.is_some_and(|e| !e.is_empty());
        vec![
            Some(if has {
                row(Icon::Palette, truncate_card(&format!("{}: {}", plural(numbers::float(entries.map_or(0, Vec::len)), "step"), titles.join(", "))))
            } else {
                missing(Icon::Palette, "Connect a grade")
            }),
            (has && entries.is_some_and(|e| grade_changes_nothing(e))).then(|| missing(Icon::TriangleAlert, "The grade changes nothing")),
            Some(if image { row(Icon::Image, "Image file") } else { row(Icon::FileVideo, "Video file") }),
            if image && entries.is_some_and(|e| grade_needs_video(e)) {
                Some(missing(Icon::TriangleAlert, "Trails and film grain need a video"))
            } else {
                length_row(ctx.result, Icon::Clock)
            },
        ]
    })
}

fn thumbnail() -> NodeDef {
    file_op::<GrabFrame>(
        THUMBNAIL_TYPE,
        "Grab frame",
        format!("Saves one frame of a video as an image.{NEW_FILE}"),
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .outputs(vec![("image", port(PortType::Image, "Image"))])
    .card(|ctx: &CardCtx<'_, GrabFrame>| {
        let p = &ctx.params;
        vec![Some(row(
            Icon::Image,
            if p.width == 0 {
                format!("At {} · Its own size", format_duration(f64::from(p.at_ms)))
            } else {
                format!(
                    "At {} · {} px",
                    format_duration(f64::from(p.at_ms)),
                    p.width
                )
            },
        ))]
    })
    .compile(|ctx: &Ctx<'_, GrabFrame>| {
        let image = ctx
            .result
            .and_then(|r| r.output("image"))
            .filter(|v| matches!(v, CompileValue::Image(_)));
        Ok(Some(
            image
                .map(|v| ("image".to_owned(), v.clone()))
                .into_iter()
                .collect(),
        ))
    })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        import().look(NodeLook::of("lucide:file-play")),
        use_as_clip().look(NodeLook::of("lucide:film").morphs("lucide:clapperboard")),
        one_file_op::<TrimFile>(
            TRIM_TYPE,
            "Trim file",
            "Cuts a file down to a start and end.",
        )
        .card(|ctx: &CardCtx<'_, TrimFile>| {
            let p = &ctx.params;
            vec![
                Some(range_row(
                    f64::from(p.start_ms),
                    (p.end_ms != 0).then_some(f64::from(p.end_ms)),
                )),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:scissors-line-dashed"))
        .needs(FileNeed::Moving),
        join_files()
            .look(NodeLook::of("lucide:combine"))
            .needs(FileNeed::Moving),
        one_file_op::<ChangeSpeed>(
            SPEED_TYPE,
            "Change speed",
            "Speeds a file up or slows it down.",
        )
        .uses_project_size()
        .card(|ctx: &CardCtx<'_, ChangeSpeed>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::Gauge, format!("{}×", p.rate))),
                Some(if p.keep_audio {
                    row(Icon::Volume2, "Keeps sound")
                } else {
                    row(Icon::VolumeX, "No sound")
                }),
            ]
        })
        .look(NodeLook::of("lucide:fast-forward"))
        .needs(FileNeed::Moving),
        one_file_op::<ReverseFile>(REVERSE_TYPE, "Reverse", "Plays a file backwards.")
            .uses_project_size()
            .card(|ctx: &CardCtx<'_, ReverseFile>| {
                vec![
                    Some(row(Icon::RefreshCw, "Plays backwards")),
                    (!ctx.params.keep_audio).then(|| row(Icon::VolumeX, "No sound")),
                    length_row(ctx.result, Icon::Clock),
                ]
            })
            .look(NodeLook::of("lucide:rewind"))
            .needs(FileNeed::Moving),
        one_file_op::<Resize>(
            SCALE_PAD_TYPE,
            "Resize",
            "Resizes a file, adding bars or cropping to fit.",
        )
        .card(|ctx: &CardCtx<'_, Resize>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::Ratio, format!("{} × {}", p.width, p.height))),
                Some(row(Icon::Maximize, p.mode.row())),
            ]
        })
        .look(NodeLook::of("lucide:scaling"))
        .needs(FileNeed::Picture),
        one_file_op::<Encoding>(
            CONVERT_TYPE,
            "Convert",
            "Saves a file in another format or quality.",
        )
        .card(|ctx: &CardCtx<'_, Encoding>| {
            let p = &ctx.params;
            vec![Some(if p.keep_format {
                row(Icon::Globe, "Web-ready, nothing converted")
            } else {
                format_row(p.codec, p.crf)
            })]
        })
        .look(NodeLook::of("lucide:file-cog")),
        apply_grade()
            .look(NodeLook::of("lucide:swatch-book"))
            .needs(FileNeed::Picture),
        one_file_op::<MakeGif>(
            GIF_TYPE,
            "Make GIF",
            "Turns a video into a GIF with clean colours.",
        )
        .card(|ctx: &CardCtx<'_, MakeGif>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::ImagePlay,
                    format!("{} px · {} fps", p.width, p.fps),
                )),
                p.r#loop.then(|| row(Icon::Repeat, "Loops")),
            ]
        })
        .look(NodeLook::of("lucide:image-play"))
        .needs(FileNeed::Picture),
        one_file_op::<ExtractSound>(
            EXTRACT_AUDIO_TYPE,
            "Extract sound",
            "Saves the sound of a video as its own file.",
        )
        .card(|ctx: &CardCtx<'_, ExtractSound>| {
            let format = serde_json::to_value(ctx.params.format)
                .ok()
                .and_then(|v| v.as_str().map(str::to_uppercase))
                .unwrap_or_default();
            vec![Some(row(Icon::AudioLines, format))]
        })
        .look(NodeLook::of("lucide:file-music"))
        .needs(FileNeed::Sound),
        one_file_op::<EvenOutVolume>(
            LOUDNORM_TYPE,
            "Even out volume",
            "Makes the sound as loud as most online videos.",
        )
        .card(|ctx: &CardCtx<'_, EvenOutVolume>| {
            vec![Some(row(
                Icon::Volume2,
                loudness_name(ctx.params.integrated),
            ))]
        })
        .look(NodeLook::of("lucide:volume-2"))
        .needs(FileNeed::Sound),
        one_file_op::<FixClicks>(
            FIX_CLICKS_TYPE,
            "Fix clicks",
            "Removes clicks and pops from a recording.",
        )
        .card(|ctx: &CardCtx<'_, FixClicks>| {
            vec![
                Some(row(Icon::AudioLines, format!("Strength {}", percent_text(ctx.params.strength, 100.0)))),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:bandage"))
        .needs(FileNeed::Sound),
        one_file_op::<FixClipping>(
            FIX_CLIPPING_TYPE,
            "Fix clipping",
            "Rebuilds peaks that were cut off when the sound was recorded too loud.",
        )
        .card(|ctx: &CardCtx<'_, FixClipping>| {
            vec![
                Some(row(Icon::AudioLines, format!("Strength {}", percent_text(ctx.params.strength, 100.0)))),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("tabler:wave-square"))
        .needs(FileNeed::Sound),
        one_file_op::<RemoveHum>(
            HUM_TYPE,
            "Remove hum",
            "Removes mains hum and its harmonics from a recording.",
        )
        .card(|ctx: &CardCtx<'_, RemoveHum>| {
            let p = &ctx.params;
            let tones = plural(f64::from(p.harmonics), "harmonic");
            let text = match p.mains.hertz() {
                Some(hz) if p.harmonics == 0 => format!("{hz} Hz"),
                Some(hz) => format!("{hz} Hz and {tones}"),
                None if p.harmonics == 0 => "50 or 60 Hz, whichever is there".to_owned(),
                None => format!("50 or 60 Hz and {tones}"),
            };
            vec![
                Some(row(Icon::AudioLines, text)),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("iconoir:sine-wave"))
        .needs(FileNeed::Sound),
        sound_picture()
            .look(NodeLook::of("lucide:chart-no-axes-column"))
            .needs(FileNeed::Sound),
        one_file_op::<RemoveNoise>(
            REMOVE_NOISE_TYPE,
            "Remove noise",
            "Takes background noise and low rumble out of a voice recording.",
        )
        .card(|ctx: &CardCtx<'_, RemoveNoise>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::AudioLines, format!("{} · {}", p.method.label(), percent_text(p.noise, 100.0)))),
                noise_level_row(ctx.result),
                (p.method == NoiseMethod::Learned && p.noise > 0.0)
                    .then(|| row(Icon::Info, "For speech: music under the voice goes too").muted_if(true)),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:mic-vocal"))
        .needs(FileNeed::Sound),
        one_file_op::<SoftenS>(
            SOFTEN_S_TYPE,
            "Soften “s” sounds",
            "Softens harsh “s” sounds in a voice recording.",
        )
        .card(|ctx: &CardCtx<'_, SoftenS>| {
            vec![
                Some(row(Icon::AudioLines, format!("Strength {}", percent_text(ctx.params.strength, 100.0)))),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("tabler:letter-s"))
        .needs(FileNeed::Sound),
        one_file_op::<ChangePitch>(
            PITCH_TYPE,
            "Change pitch",
            "Raises or lowers a sound's pitch in semitones while its length stays, to fit a song's key or make a playful voice.",
        )
        .card(|ctx: &CardCtx<'_, ChangePitch>| {
            let p = &ctx.params;
            let steps = if p.semitones == 0.0 {
                "No change".to_owned()
            } else {
                format!(
                    "{} {}",
                    signed_text(p.semitones, None, true),
                    if exact_eq(p.semitones.abs(), 1.0) { "semitone" } else { "semitones" }
                )
            };
            vec![
                Some(row(Icon::Music, steps)),
                (!p.keep_character).then(|| row(Icon::AudioLines, "The voice's character changes too")),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("tabler:music-up").morphs("tabler:music-down"))
        .needs(FileNeed::Sound),
        one_file_op::<VoiceEffects>(
            VOICE_TYPE,
            "Voice effects",
            "Makes a voice or a sound sound like a telephone, a radio, a small room, a hall or an echo.",
        )
        .card(|ctx: &CardCtx<'_, VoiceEffects>| {
            let p = &ctx.params;
            let what = match p.effect {
                VoiceEffect::Echo => format!(
                    "Echo after {}",
                    format_duration(f64::from(p.delay_ms))
                ),
                effect => effect.label().to_owned(),
            };
            vec![
                Some(row(
                    Icon::AudioLines,
                    format!("{what} · {} mixed", percent_text(p.mix, f64::INFINITY)),
                )),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:speech"))
        .needs(FileNeed::Sound),
        file_op::<FitMusic>(
            FIT_MUSIC_TYPE,
            "Fit music to the film",
            format!(
                "Makes a song end with the film: stretched a little, or looped with a crossfade, then faded.{NEW_FILE}"
            ),
        )
        .uses_project_size()
        .inputs(vec![
            (crate::node::CLIP_PORT, port(PortType::Clip, "Film")),
            (MEDIA_PORT, port(PortType::Media, "Song file").optional()),
        ])
        .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
        .card(|ctx: &CardCtx<'_, FitMusic>| {
            let p = &ctx.params;
            let song = if ctx.connected(MEDIA_PORT) {
                Some(row(Icon::Music, "From the connected file"))
            } else if p.src.is_empty() {
                Some(missing(Icon::Music, "Choose a song"))
            } else {
                Some(crate::sounds::builtin_label(&p.src).map_or_else(
                    || file_row(Icon::Music, &p.src),
                    |label| row(Icon::Music, label),
                ))
            };
            vec![
                song,
                (!ctx.connected(crate::node::CLIP_PORT))
                    .then(|| missing(Icon::Film, "Connect the film for its length")),
                fitted_row(ctx.result),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("tabler:arrow-autofit-width").moves(NodeMotion::Stretch))
        .needs(FileNeed::SoundFile),
        one_file_op::<ShortenPauses>(
            PAUSES_TYPE,
            "Shorten pauses",
            "Shortens long silences in a voice recording without cutting words. Use it on a voice before placing it against a take.",
        )
        .card(|ctx: &CardCtx<'_, ShortenPauses>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::Scissors,
                    format!(
                        "Pauses over {} cut to {}",
                        format_duration(f64::from(p.longer_than_ms)),
                        format_duration(f64::from(p.keep_ms))
                    ),
                )),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:fold-horizontal"))
        .needs(FileNeed::SoundFile),
        thumbnail()
            .look(NodeLook::of("tabler:photo-scan"))
            .needs(FileNeed::Picture),
    ]
}
