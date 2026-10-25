use std::collections::BTreeSet;

use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit, VisualEnvelope};
use opsis_types::values::{ClipValue, CompileValue, MediaKind, MediaValue, NodeResult};
use opsis_types::workflow::{DEFAULT_STILL_MS, PortType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smart_default::SmartDefault;

use crate::asset_ref::{kind_from_path, project_ref};
use crate::card::{file_row, range_row, truncate_card};
use crate::clip::{ItemExtra, item_clip, placeholder_clip};
use crate::node::{
    CardCtx, Category, Ctx, GRADE_PORT, Inputs, NodeDef, Outputs, Phase, Ports, RowExt, missing,
    opt, port, row,
};
use crate::nodes::grade::{grade_changes_nothing, grade_needs_video, grade_step_title};
use crate::params::Params;
use crate::text::{format_duration, number, plural, signed_text};
use crate::time::ms_to_frames;

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

    pub fn has_quality(self) -> bool {
        matches!(self, Self::H264 | Self::H265 | Self::Vp8 | Self::Vp9)
    }
}

pub const QUALITY_PRESETS: [(&str, i64); 4] =
    [("Low", 28), ("Medium", 23), ("High", 18), ("Max", 12)];

pub fn quality_name(crf: i64) -> &'static str {
    let mut best = QUALITY_PRESETS[0];
    for preset in &QUALITY_PRESETS[1..] {
        let distance = (preset.1 - crf).abs();
        let best_distance = (best.1 - crf).abs();
        if distance < best_distance || (distance == best_distance && preset.1 < best.1) {
            best = *preset;
        }
    }
    best.0
}

pub const LOUDNESS_PRESETS: [(&str, f64); 3] = [
    ("Online video", -14.0),
    ("Podcast", -16.0),
    ("Broadcast", -23.0),
];

pub fn loudness_name(lufs: f64) -> String {
    LOUDNESS_PRESETS
        .iter()
        .find(|(_, v)| *v == lufs)
        .map_or_else(
            || format!("{} LUFS", signed_text(lufs, None, false)),
            |(name, _)| (*name).to_owned(),
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
    pub crf: i64,
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

fn format_row(codec: ConvertCodec, crf: i64) -> CardRow {
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
        .filter(|d| *d != 0.0)?;
    Some(row(icon, format_duration(duration)))
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

    fn id(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
        }
    }
}

struct ClipLook<'a> {
    fit: Fit,
    muted: bool,
    envelope: &'a VisualEnvelope,
    fps: f64,
    unlinked: (&'a str, &'a str),
    sound_label: &'a str,
    still: Option<(f64, EditHandles)>,
}

fn clip_ms(media: Option<&MediaValue>, still: Option<f64>) -> f64 {
    match media {
        Some(m) if m.kind == MediaKind::Image => {
            still.or(m.duration_ms).unwrap_or(DEFAULT_STILL_MS)
        }
        _ => media.and_then(|m| m.duration_ms).unwrap_or(MEDIA_MS),
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
    let mut props = Map::new();
    match media.kind {
        MediaKind::Audio => {
            placeholder_clip(node_id, "Sound file", look.sound_label, frames, extra)
        }
        MediaKind::Image => {
            props.insert("src".into(), Value::from(src.clone()));
            props.insert("fit".into(), Value::from(look.fit.id()));
            props.insert("muted".into(), Value::from(look.muted));
            let extra = ItemExtra {
                handles: look.still.map(|s| s.1),
                ..extra
            };
            item_clip(node_id, "core/image", props, frames, extra)
        }
        MediaKind::Video => {
            props.insert("src".into(), Value::from(src.clone()));
            props.insert("fit".into(), Value::from(look.fit.id()));
            props.insert(
                "muted".into(),
                Value::from(look.muted || media.has_audio == Some(false)),
            );
            item_clip(node_id, "core/video", props, frames, extra)
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
    pub still_ms: i64,
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
        "media.import",
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
                Some(row(Icon::Clock, format_duration(p.still_ms as f64)))
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
            still: Some((p.still_ms as f64, handles)),
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
    pub start_ms: i64,
    #[schemars(description = "End", range(min = 0, max = 36000000), extend("x-step" = 50, "x-unit" = "ms", "x-zero-label" = "To the end"))]
    pub end_ms: i64,
}

impl Params for TrimFile {
    fn check(&self) -> Result<(), String> {
        if self.end_ms != 0 && self.end_ms <= self.start_ms {
            return Err("End must come after Start".to_owned());
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

pub const CONCAT_PORTS: [&str; 4] = ["in1", "in2", "in3", "in4"];

fn join_files() -> NodeDef {
    NodeDef::of::<JoinFiles>("media.concat", "Join files", Category::Media, Phase::Media)
        .describe(format!(
            "Joins files end to end. Fast when they match, otherwise converts them.{NEW_FILE}"
        ))
        .executes()
        .uses_project_size()
        .inputs(vec![
            ("in1", port(PortType::Media, "File 1")),
            ("in2", port(PortType::Media, "File 2")),
            ("in3", opt(PortType::Media, "File 3")),
            ("in4", opt(PortType::Media, "File 4")),
        ])
        .outputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
        .card(|ctx: &CardCtx<'_, JoinFiles>| {
            let n = CONCAT_PORTS.iter().filter(|p| ctx.connected(p)).count();
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
    pub width: i64,
    #[default = 1080]
    #[schemars(
        description = "Height",
        range(min = 16, max = 7680),
        extend("x-step" = 2, "x-unit" = "px", "x-help" = "A video takes even sizes; an odd one loses a pixel.")
    )]
    pub height: i64,
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
    pub fps: i64,
    #[default = 960]
    #[schemars(description = "Width", range(min = 64, max = 3840), extend("x-unit" = "px"))]
    pub width: i64,
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
pub struct GrabFrame {
    #[default = 1000]
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub at_ms: i64,
    #[default = 1280]
    #[schemars(description = "Width", range(min = 64, max = 3840), extend("x-unit" = "px"))]
    pub width: i64,
}

impl Params for GrabFrame {}

fn apply_grade() -> NodeDef {
    file_op::<Encoding>(
        "media.applyGrade",
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
        "media.probe",
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
        "media.thumbnail",
        "Grab frame",
        format!("Saves one frame of a video as an image.{NEW_FILE}"),
    )
    .inputs(vec![(MEDIA_PORT, port(PortType::Media, "File"))])
    .outputs(vec![("image", port(PortType::Image, "Image"))])
    .card(|ctx: &CardCtx<'_, GrabFrame>| {
        let p = &ctx.params;
        vec![Some(row(
            Icon::Image,
            format!("At {} · {} px", format_duration(p.at_ms as f64), p.width),
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
        import(),
        use_as_clip(),
        one_file_op::<TrimFile>(
            "media.trim",
            "Trim file",
            "Cuts a file down to a start and end.",
        )
        .card(|ctx: &CardCtx<'_, TrimFile>| {
            let p = &ctx.params;
            vec![
                Some(range_row(
                    p.start_ms as f64,
                    (p.end_ms != 0).then_some(p.end_ms as f64),
                )),
                length_row(ctx.result, Icon::Clock),
            ]
        }),
        join_files(),
        one_file_op::<ChangeSpeed>(
            "media.speed",
            "Change speed",
            "Speeds a file up or slows it down.",
        )
        .uses_project_size()
        .card(|ctx: &CardCtx<'_, ChangeSpeed>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::Gauge, format!("{}×", number(p.rate)))),
                Some(if p.keep_audio {
                    row(Icon::Volume2, "Keeps sound")
                } else {
                    row(Icon::VolumeX, "No sound")
                }),
            ]
        }),
        one_file_op::<NoSettings>("media.reverse", "Reverse", "Plays a file backwards.")
            .uses_project_size()
            .card(|ctx: &CardCtx<'_, NoSettings>| {
                vec![
                    Some(row(Icon::RefreshCw, "Plays backwards")),
                    length_row(ctx.result, Icon::Clock),
                ]
            }),
        one_file_op::<Resize>(
            "media.scalePad",
            "Resize",
            "Resizes a file, adding bars or cropping to fit.",
        )
        .card(|ctx: &CardCtx<'_, Resize>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::Ratio, format!("{} × {}", p.width, p.height))),
                Some(row(Icon::Maximize, p.mode.row())),
            ]
        }),
        one_file_op::<Encoding>(
            "media.convert",
            "Convert",
            "Saves a file in another format or quality.",
        )
        .card(|ctx: &CardCtx<'_, Encoding>| {
            vec![Some(format_row(ctx.params.codec, ctx.params.crf))]
        }),
        apply_grade(),
        one_file_op::<MakeGif>(
            "media.gif",
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
        }),
        one_file_op::<ExtractSound>(
            "media.extractAudio",
            "Extract sound",
            "Saves the sound of a video as its own file.",
        )
        .card(|ctx: &CardCtx<'_, ExtractSound>| {
            let format = serde_json::to_value(ctx.params.format)
                .ok()
                .and_then(|v| v.as_str().map(str::to_uppercase))
                .unwrap_or_default();
            vec![Some(row(Icon::AudioLines, format))]
        }),
        one_file_op::<EvenOutVolume>(
            "media.loudnorm",
            "Even out volume",
            "Makes the sound as loud as most online videos.",
        )
        .card(|ctx: &CardCtx<'_, EvenOutVolume>| {
            vec![Some(row(
                Icon::Volume2,
                loudness_name(ctx.params.integrated),
            ))]
        }),
        thumbnail(),
        one_file_op::<NoSettings>(
            "media.faststart",
            "Web-ready",
            "Prepares a video to start playing instantly on the web.",
        )
        .card(|ctx: &CardCtx<'_, NoSettings>| {
            vec![
                Some(row(Icon::Globe, "Starts playing before it fully loads")),
                length_row(ctx.result, Icon::Clock),
            ]
        }),
        probe(),
    ]
}
