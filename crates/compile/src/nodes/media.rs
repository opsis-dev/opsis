use std::collections::BTreeSet;

use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::mix::LOUDNESS_TARGETS;
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit, VisualEnvelope};
use opsis_types::values::{ClipValue, CompileValue, MediaKind, MediaValue, NodeResult, PortValues};
use opsis_types::workflow::{DEFAULT_STILL_MS, PortType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::asset_ref::{kind_from_path, project_ref};
use crate::card::{file_row, range_row, truncate_card};
use crate::clip::{IMAGE, ItemExtra, VIDEO, item_clip, placeholder_clip};
use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, GRADE_PORT, Inputs, NodeDef, Outputs, Phase, PortFamily, Ports, RowExt,
    missing, port, row,
};
use crate::nodes::grade::{grade_changes_nothing, grade_needs_video, grade_step_title};
use crate::params::{Params, to_values};
use crate::ports::{IN_PREFIX, in_index};
use crate::text::{format_duration, plural, signed_text};
use crate::time::ms_to_frames;

pub const PROBE_TYPE: &str = "media.probe";

pub const FASTSTART_TYPE: &str = "media.faststart";

pub const THUMBNAIL_TYPE: &str = "media.thumbnail";

pub const LOUDNORM_TYPE: &str = "media.loudnorm";
pub const REPAIR_TYPE: &str = "media.repair";
pub const HUM_TYPE: &str = "media.hum";

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

const MEDIA_PORT: &str = "media";
const NEW_FILE: &str = " Makes a new file.";
const MEDIA_MS: f64 = 5000.0;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    H265,
    Vp8,
    Vp9,
    Prores,
    Gif,
}

impl Codec {
    pub fn name(self) -> &'static str {
        match self {
            Self::H264 => "MP4",
            Self::H265 => "MP4 (HEVC)",
            Self::Vp8 => "WebM (VP8)",
            Self::Vp9 => "WebM",
            Self::Prores => "ProRes",
            Self::Gif => "GIF",
        }
    }
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
        .find(|(_, v)| *v == lufs)
        .map_or_else(
            || format!("{} LUFS", signed_text(lufs, None, false)),
            |(name, _)| name,
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConvertCodec {
    H264,
    H265,
    Vp9,
    Prores,
}

impl ConvertCodec {
    fn codec(self) -> Codec {
        match self {
            Self::H264 => Codec::H264,
            Self::H265 => Codec::H265,
            Self::Vp9 => Codec::Vp9,
            Self::Prores => Codec::Prores,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EncodePreset {
    Ultrafast,
    Fast,
    Medium,
    Slow,
    Veryslow,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Encoding {
    #[default(ConvertCodec::H264)]
    #[schemars(description = "Format", extend("x-labels" = { "h264": "MP4", "h265": "MP4 (HEVC)", "vp9": "WebM", "prores": "ProRes" }))]
    pub codec: ConvertCodec,
    #[default = 18]
    #[schemars(
        description = "Quality",
        range(min = 0, max = 51),
        extend("x-widget" = "quality", "x-show-if" = { "codec": ["h264", "h265", "vp9"] })
    )]
    pub crf: u32,
    #[default(EncodePreset::Medium)]
    #[schemars(
        description = "Speed vs file size",
        extend(
            "x-labels" = {
                "ultrafast": "Fastest, biggest",
                "fast": "Fast",
                "medium": "Balanced",
                "slow": "Small",
                "veryslow": "Smallest, slowest"
            },
            "x-advanced" = true,
            "x-show-if" = { "codec": ["h264", "h265"] }
        )
    )]
    pub preset: EncodePreset,
}

impl Params for Encoding {}

fn format_row(codec: ConvertCodec, crf: u32) -> CardRow {
    let parts: Vec<String> = [
        Some(codec.codec().name().to_owned()),
        matches!(
            codec,
            ConvertCodec::H264 | ConvertCodec::H265 | ConvertCodec::Vp9
        )
        .then(|| format!("{} quality", quality_name(crf))),
        (codec == ConvertCodec::Prores).then(|| "no Preview".to_owned()),
        (codec == ConvertCodec::H265).then(|| "may not preview".to_owned()),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Cover,
    Contain,
}

impl Fit {
    fn label(self) -> &'static str {
        match self {
            Self::Cover => "Fill the frame",
            Self::Contain => "Show all",
        }
    }
}

#[derive(Serialize)]
struct FileClipProps<'a> {
    src: &'a str,
    fit: Fit,
    muted: bool,
}

struct ClipLook<'a> {
    fit: Fit,
    muted: bool,
    envelope: &'a VisualEnvelope,
    fps: u32,
    unlinked: (&'a str, &'a str),
    sound_label: &'a str,
    still: Option<(f64, EditHandles)>,
}

fn clip_ms(media: Option<&MediaValue>, still: Option<f64>) -> f64 {
    match media {
        Some(m) if m.kind == MediaKind::Image => still
            .or(m.duration_ms.map(f64::from))
            .unwrap_or(DEFAULT_STILL_MS),
        _ => media
            .and_then(|m| m.duration_ms)
            .map_or(MEDIA_MS, f64::from),
    }
}

fn media_to_clip(node_id: &str, media: Option<&MediaValue>, look: ClipLook<'_>) -> ClipValue {
    let frames = ms_to_frames(clip_ms(media, look.still.as_ref().map(|s| s.0)), look.fps);
    let extra = ItemExtra {
        envelope: Some(look.envelope.clone()),
        ..ItemExtra::default()
    };
    let Some((media, src)) = media.and_then(|m| m.src.as_ref().map(|s| (m, s))) else {
        return placeholder_clip(node_id, look.unlinked.0, look.unlinked.1, frames, extra);
    };
    let props = |muted: bool| {
        to_values(&FileClipProps {
            src,
            fit: look.fit,
            muted,
        })
    };
    match media.kind {
        MediaKind::Audio => {
            placeholder_clip(node_id, "Sound file", look.sound_label, frames, extra)
        }
        MediaKind::Image => {
            let extra = ItemExtra {
                handles: look.still.map(|s| s.1),
                ..extra
            };
            item_clip(node_id, IMAGE, props(look.muted), frames, extra)
        }
        MediaKind::Video => {
            let muted = look.muted || media.has_audio == Some(false);
            item_clip(node_id, VIDEO, props(muted), frames, extra)
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaFile {
    #[schemars(description = "File", extend("x-widget" = "asset", "x-accept" = "*/*"))]
    pub path: String,
    #[default(Fit::Contain)]
    #[schemars(description = "Fit", extend("x-labels" = { "cover": "Fill the frame", "contain": "Show all" }))]
    pub fit: Fit,
    #[default = 3000]
    #[schemars(
        description = "Length",
        range(min = 100, max = 3600000),
        extend(
            "x-step" = 50,
            "x-unit" = "ms",
            "x-help" = "How long the image stays on screen.",
            "x-show-if-file" = { "path": ["image"] }
        )
    )]
    pub still_ms: u32,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change opacity, position and size over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    pub envelope: VisualEnvelope,
}

impl Params for MediaFile {}

fn import() -> NodeDef {
    file_op::<MediaFile>(
        IMPORT_TYPE,
        "Media file",
        "A video, image or sound from your project. Drop files on the canvas to add one.",
    )
    .outputs(vec![
        (MEDIA_PORT, port(PortType::Media, "File")),
        (crate::node::CLIP_PORT, port(PortType::Clip, "Clip")),
    ])
    .card(|ctx: &CardCtx<'_, MediaFile>| {
        let p = &ctx.params;
        vec![
            Some(if p.path.is_empty() {
                missing(Icon::FileVideo, "Choose a file")
            } else {
                file_row(Icon::FileVideo, &p.path)
            }),
            if !p.path.is_empty() && kind_from_path(&p.path) == MediaKind::Image {
                Some(row(Icon::Clock, format_duration(f64::from(p.still_ms))))
            } else {
                length_row(ctx.result, Icon::Clock)
            },
        ]
    })
    .compiles(|ctx: &Ctx<'_, MediaFile>| {
        let p = &ctx.params;
        let media = media_of(ctx.result, MEDIA_PORT).unwrap_or_else(|| MediaValue {
            src: (!p.path.is_empty()).then(|| project_ref(&p.path)),
            kind: kind_from_path(&p.path),
            duration_ms: None,
            width: None,
            height: None,
            has_audio: None,
        });
        let handles = EditHandles {
            trim_end: Some(ParamRef {
                node_id: ctx.node_id.to_owned(),
                param: "stillMs".into(),
                unit: ParamUnit::Ms,
                scale: None,
                index: None,
                linked: None,
            }),
            ..EditHandles::default()
        };
        let look = ClipLook {
            fit: p.fit,
            muted: false,
            envelope: &p.envelope,
            fps: ctx.settings.fps,
            unlinked: ("No file chosen", &p.path),
            sound_label: &p.path,
            still: Some((f64::from(p.still_ms), handles)),
        };
        let clip = media_to_clip(ctx.node_id, Some(&media), look);
        crate::node::outputs([
            (MEDIA_PORT, CompileValue::Media(media)),
            (crate::node::CLIP_PORT, CompileValue::Clip(clip)),
        ])
    })
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct UseAsClip {
    #[default(Fit::Contain)]
    #[schemars(description = "Fit", extend("x-labels" = { "cover": "Fill the frame", "contain": "Show all" }))]
    pub fit: Fit,
    #[schemars(description = "Muted")]
    pub muted: bool,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change opacity, position and size over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    pub envelope: VisualEnvelope,
}

impl Params for UseAsClip {}

fn use_as_clip() -> NodeDef {
    NodeDef::of::<UseAsClip>(
        "media.toClip",
        "Use as clip",
        Category::Media,
        Phase::Compile,
    )
    .describe("Turns a file into a clip you can place in the video.")
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .outputs(vec![(crate::node::CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, UseAsClip>| {
        vec![
            Some(row(Icon::Maximize, ctx.params.fit.label())),
            ctx.params.muted.then(|| row(Icon::VolumeX, "Muted")),
        ]
    })
    .compiles(|ctx: &Ctx<'_, UseAsClip>| {
        let p = &ctx.params;
        let look = ClipLook {
            fit: p.fit,
            muted: p.muted,
            envelope: &p.envelope,
            fps: ctx.settings.fps,
            unlinked: ("No file connected", "Connect a file to use it as a clip"),
            sound_label: "Sound files can't be shown as a clip",
            still: None,
        };
        let clip = media_to_clip(
            ctx.node_id,
            ctx.input(MEDIA_PORT).and_then(CompileValue::as_media),
            look,
        );
        crate::node::outputs([(crate::node::CLIP_PORT, CompileValue::Clip(clip))])
    })
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TrimFile {
    #[schemars(description = "Start", range(min = 0, max = 36000000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub start_ms: u32,
    #[schemars(description = "End", range(min = 0, max = 36000000), extend("x-step" = 50, "x-unit" = "ms", "x-zero-label" = "To the end"))]
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct JoinFiles {
    #[schemars(description = "Always convert", extend("x-advanced" = true))]
    pub force_reencode: bool,
}

impl Params for JoinFiles {}

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
                    n => row(Icon::Combine, plural(n as f64, "file")),
                }),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .compile(|ctx: &Ctx<'_, JoinFiles>| Ok(Some(pass_media(ctx.result, ctx.inputs, "in1"))))
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ChangeSpeed {
    #[default = 2.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 64), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub rate: f64,
    #[default = true]
    #[schemars(description = "Keep the sound")]
    pub keep_audio: bool,
}

impl Params for ChangeSpeed {}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct NoSettings {}

impl Params for NoSettings {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScaleMode {
    Pad,
    Crop,
    Stretch,
}

impl ScaleMode {
    fn row(self) -> &'static str {
        match self {
            Self::Pad => "Fit with bars",
            Self::Crop => "Crop to fill",
            Self::Stretch => "Stretch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Resize {
    #[default = 1920]
    #[schemars(
        description = "Width",
        range(min = 16, max = 7680),
        extend("x-step" = 2, "x-unit" = "px", "x-help" = "A video takes even sizes; an odd one loses a pixel.")
    )]
    pub width: u32,
    #[default = 1080]
    #[schemars(
        description = "Height",
        range(min = 16, max = 7680),
        extend("x-step" = 2, "x-unit" = "px", "x-help" = "A video takes even sizes; an odd one loses a pixel.")
    )]
    pub height: u32,
    #[default(ScaleMode::Pad)]
    #[schemars(description = "Fit", extend("x-labels" = { "pad": "Add bars", "crop": "Crop to fill", "stretch": "Stretch" }))]
    pub mode: ScaleMode,
    #[default = "black"]
    #[schemars(
        description = "Bar colour",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Default",
            "x-show-if" = { "mode": ["pad"] },
            "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" }
        )
    )]
    pub background: String,
}

impl Params for Resize {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Dither {
    #[serde(rename = "sierra2_4a")]
    Sierra24a,
    Bayer,
    FloydSteinberg,
    None,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MakeGif {
    #[default = 15]
    #[schemars(description = "Frame rate", range(min = 5, max = 50), extend("x-unit" = "fps"))]
    pub fps: u32,
    #[default = 960]
    #[schemars(description = "Width", range(min = 64, max = 3840), extend("x-unit" = "px"))]
    pub width: u32,
    #[default = true]
    #[schemars(description = "Loop forever")]
    pub r#loop: bool,
    #[default(Dither::Sierra24a)]
    #[schemars(
        description = "Smooth colours",
        extend("x-labels" = { "sierra2_4a": "Smooth", "bayer": "Patterned", "floyd_steinberg": "Fine", "none": "Off" })
    )]
    pub dither: Dither,
}

impl Params for MakeGif {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SoundFormat {
    M4a,
    Mp3,
    Wav,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtractSound {
    #[default(SoundFormat::M4a)]
    #[schemars(description = "Format", extend("x-labels" = { "m4a": "M4A", "mp3": "MP3", "wav": "WAV" }))]
    pub format: SoundFormat,
}

impl Params for ExtractSound {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct EvenOutVolume {
    #[default(-16.0)]
    #[schemars(description = "Loudness", range(min = -40, max = -5), extend("x-widget" = "loudness", "x-unit" = "LUFS"))]
    pub integrated: f64,
    #[default(-1.5)]
    #[schemars(description = "Peak limit", range(min = -9, max = 0), extend("x-step" = 0.1, "x-unit" = "dB", "x-advanced" = true))]
    pub true_peak: f64,
    #[default = 11.0]
    #[schemars(description = "Dynamic range", range(min = 1, max = 20), extend("x-advanced" = true))]
    pub lra: f64,
}

impl Params for EvenOutVolume {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FixClicks {
    #[default = true]
    #[schemars(description = "Remove clicks and pops")]
    pub clicks: bool,
    #[default = true]
    #[schemars(description = "Rebuild clipped peaks")]
    pub clipping: bool,
    #[default = 0.5]
    #[schemars(
        description = "Strength",
        range(min = 0, max = 1),
        extend("x-step" = 0.05, "x-help" = "Stronger finds more, but can soften sharp sounds that are meant to be there.")
    )]
    pub strength: f64,
}

impl Params for FixClicks {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Mains {
    #[default]
    Find,
    Fifty,
    Sixty,
}

impl Mains {
    pub fn hertz(self) -> Option<f64> {
        match self {
            Self::Find => None,
            Self::Fifty => Some(50.0),
            Self::Sixty => Some(60.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RemoveHum {
    #[schemars(
        description = "Hum",
        extend(
            "x-labels" = { "find": "Find it", "fifty": "50 Hz", "sixty": "60 Hz" },
            "x-help" = "Mains hum is 50 Hz in most of the world and 60 Hz in the Americas. Find it listens for both first."
        )
    )]
    pub mains: Mains,
    #[default = 4]
    #[schemars(description = "Harmonics", range(min = 0, max = 10), extend("x-help" = "Hum also rings at twice, three times and more of its pitch."))]
    pub harmonics: u32,
    #[default = 2.0]
    #[schemars(description = "Notch width", range(min = 0.5, max = 20), extend("x-step" = 0.5, "x-unit" = "Hz", "x-advanced" = true))]
    pub width: f64,
}

impl Params for RemoveHum {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GrabFrame {
    #[default = 1000]
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub at_ms: u32,
    #[default = 1280]
    #[schemars(description = "Width", range(min = 64, max = 3840), extend("x-unit" = "px"))]
    pub width: u32,
}

impl Params for GrabFrame {}

fn apply_grade() -> NodeDef {
    file_op::<Encoding>(
        APPLY_GRADE_TYPE,
        "Apply grade",
        format!(
            "Applies a colour grade to a whole video or image file: everything in the picture, window and backdrop included.{NEW_FILE}"
        ),
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File")), (GRADE_PORT, port(PortType::Grade, "Grade"))])
    .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .card(|ctx: &CardCtx<'_, Encoding>| {
        let p = &ctx.params;
        let entries = ctx.inputs.grade.get(GRADE_PORT);
        let image = ctx.inputs.media.get(MEDIA_PORT).is_some_and(|m| m.kind == MediaKind::Image);
        let mut seen = BTreeSet::new();
        let titles: Vec<String> =
            entries.into_iter().flatten().map(|e| grade_step_title(&e.kind)).filter(|t| seen.insert(t.clone())).collect();
        let has = entries.is_some_and(|e| !e.is_empty());
        vec![
            Some(if has {
                row(Icon::Palette, truncate_card(&format!("{}: {}", plural(entries.map_or(0, Vec::len) as f64, "step"), titles.join(", "))))
            } else {
                missing(Icon::Palette, "Connect a grade")
            }),
            (has && entries.is_some_and(|e| grade_changes_nothing(e))).then(|| missing(Icon::TriangleAlert, "The grade changes nothing")),
            Some(if image { row(Icon::Image, "PNG") } else { format_row(p.codec, p.crf) }),
            if image && entries.is_some_and(|e| grade_needs_video(e)) {
                Some(missing(Icon::TriangleAlert, "Trails and film grain need a video"))
            } else {
                length_row(ctx.result, Icon::Clock)
            },
        ]
    })
}

fn probe() -> NodeDef {
    file_op::<NoSettings>(
        PROBE_TYPE,
        "File info",
        "Shows a file's length, size, frame rate and format.",
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .outputs(vec![
        ("info", port(PortType::FileInfo, "Info")),
        (MEDIA_PORT, port(PortType::Media, "File")),
    ])
    .card(|ctx: &CardCtx<'_, NoSettings>| {
        vec![Some(length_row(ctx.result, Icon::Info).unwrap_or_else(
            || row(Icon::Info, "Run to read the file").tone(Tone::Muted),
        ))]
    })
    .compile(|ctx: &Ctx<'_, NoSettings>| {
        let mut out = Outputs::new();
        if let Some(info) = ctx
            .result
            .and_then(|r| r.output("info"))
            .filter(|v| matches!(v, CompileValue::FileInfo(_)))
        {
            out.insert("info".into(), info.clone());
        }
        out.extend(pass_media(ctx.result, ctx.inputs, MEDIA_PORT));
        Ok(Some(out))
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
            format!(
                "At {} · {} px",
                format_duration(f64::from(p.at_ms)),
                p.width
            ),
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
        .look(NodeLook::of("lucide:scissors-line-dashed")),
        join_files().look(NodeLook::of("lucide:combine")),
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
        .look(NodeLook::of("lucide:fast-forward")),
        one_file_op::<NoSettings>(REVERSE_TYPE, "Reverse", "Plays a file backwards.")
            .uses_project_size()
            .card(|ctx: &CardCtx<'_, NoSettings>| {
                vec![
                    Some(row(Icon::RefreshCw, "Plays backwards")),
                    length_row(ctx.result, Icon::Clock),
                ]
            })
            .look(NodeLook::of("lucide:rewind")),
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
        .look(NodeLook::of("lucide:scaling")),
        one_file_op::<Encoding>(
            CONVERT_TYPE,
            "Convert",
            "Saves a file in another format or quality.",
        )
        .card(|ctx: &CardCtx<'_, Encoding>| {
            vec![Some(format_row(ctx.params.codec, ctx.params.crf))]
        })
        .look(NodeLook::of("lucide:file-cog")),
        apply_grade().look(NodeLook::of("lucide:swatch-book")),
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
        .look(NodeLook::of("lucide:image-play")),
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
        .look(NodeLook::of("lucide:file-music")),
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
        .look(NodeLook::of("lucide:volume-2")),
        one_file_op::<FixClicks>(
            REPAIR_TYPE,
            "Fix clicks and clipping",
            "Removes clicks and pops and rebuilds peaks that were cut off when recording too loud.",
        )
        .card(|ctx: &CardCtx<'_, FixClicks>| {
            let p = &ctx.params;
            let what = match (p.clicks, p.clipping) {
                (true, true) => row(Icon::AudioLines, "Clicks and clipping"),
                (true, false) => row(Icon::AudioLines, "Clicks"),
                (false, true) => row(Icon::AudioLines, "Clipping"),
                (false, false) => missing(Icon::AudioLines, "Turn on clicks or clipping"),
            };
            vec![Some(what), length_row(ctx.result, Icon::Clock)]
        })
        .look(NodeLook::of("lucide:bandage")),
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
        .look(NodeLook::of("lucide:unplug")),
        thumbnail().look(NodeLook::of("lucide:scan")),
        one_file_op::<NoSettings>(
            FASTSTART_TYPE,
            "Web-ready",
            "Prepares a video to start playing instantly on the web.",
        )
        .card(|ctx: &CardCtx<'_, NoSettings>| {
            vec![
                Some(row(Icon::Globe, "Starts playing before it fully loads")),
                length_row(ctx.result, Icon::Clock),
            ]
        })
        .look(NodeLook::of("lucide:zap")),
        probe().look(NodeLook::of("lucide:info")),
    ]
}
