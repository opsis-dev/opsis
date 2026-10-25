use opsis_types::cards::{Icon, Tone};
use opsis_types::timeline::ReframeMode;
use opsis_types::values::{Artifact, CompileValue};
use opsis_types::workflow::{PortType, WorkflowNode, WorkflowSettings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::node::{CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Outputs, Phase, RowExt, port, row};
use crate::nodes::media::Codec;
use crate::params::Params;
use crate::text::{format_bytes, format_duration, slug_name};
use crate::time::{even_px, frames_to_ms};

pub const EXPORT_FOLDER_FALLBACK: &str = "output";
pub const RENDER_TYPE: &str = "output.render";
pub const STILL_TYPE: &str = "output.still";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Gl {
    Angle,
    Swangle,
    Egl,
    Vulkan,
    Swiftshader,
}

impl Codec {
    pub fn extension(self) -> &'static str {
        match self {
            Self::H264 | Self::H265 => "mp4",
            Self::Vp8 | Self::Vp9 => "webm",
            Self::Prores => "mov",
            Self::Gif => "gif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum Variant {
    #[serde(rename = "16:9")]
    Wide,
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "4:5")]
    Portrait,
    #[serde(rename = "9:16")]
    Tall,
}

impl Variant {
    pub fn id(self) -> &'static str {
        match self {
            Self::Wide => "16:9",
            Self::Square => "1:1",
            Self::Portrait => "4:5",
            Self::Tall => "9:16",
        }
    }

    fn ratio(self) -> (f64, f64) {
        match self {
            Self::Wide => (16.0, 9.0),
            Self::Square => (1.0, 1.0),
            Self::Portrait => (4.0, 5.0),
            Self::Tall => (9.0, 16.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Long,
    Short,
}

fn size_for(variant: Variant, edge: f64, anchor: Edge) -> Size {
    let (w, h) = variant.ratio();
    let landscape = w >= h;
    let edge_is_width = if anchor == Edge::Long {
        landscape
    } else {
        !landscape
    };
    if edge_is_width {
        Size {
            width: even_px(edge),
            height: even_px(edge * h / w),
        }
    } else {
        Size {
            width: even_px(edge * w / h),
            height: even_px(edge),
        }
    }
}

pub fn variant_size(variant: Variant, settings: &WorkflowSettings) -> Size {
    size_for(variant, settings.width.max(settings.height), Edge::Long)
}

pub fn variant_canvas(variant: Variant, short: f64) -> Size {
    size_for(variant, short, Edge::Short)
}

fn exported_short_side(variant: Variant, settings: &WorkflowSettings, scale: f64) -> f64 {
    let size = variant_size(variant, settings);
    (size.width.min(size.height) * scale).round()
}

pub fn video_file_names(base: &str, codec: Codec, variants: &[Variant]) -> Vec<String> {
    variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let suffix = if i == 0 {
                String::new()
            } else {
                format!("-{}", v.id().replacen(':', "x", 1))
            };
            format!("{base}{suffix}.{}", codec.extension())
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum StillPreset {
    Wide,
    Og,
    Square,
    Portrait,
    Story,
}

impl StillPreset {
    pub fn id(self) -> &'static str {
        match self {
            Self::Wide => "wide",
            Self::Og => "og",
            Self::Square => "square",
            Self::Portrait => "portrait",
            Self::Story => "story",
        }
    }

    pub fn size(self) -> Size {
        let (width, height) = match self {
            Self::Wide => (3840.0, 2160.0),
            Self::Og => (2400.0, 1260.0),
            Self::Square => (2160.0, 2160.0),
            Self::Portrait => (1728.0, 2160.0),
            Self::Story => (1080.0, 1920.0),
        };
        Size { width, height }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Wide => "Wide 4K",
            Self::Og => "Link preview",
            Self::Square => "Square",
            Self::Portrait => "Portrait",
            Self::Story => "Story",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum StillFormat {
    Png,
    Jpeg,
}

pub fn still_file_names(base: &str, presets: &[StillPreset], format: StillFormat) -> Vec<String> {
    let ext = if format == StillFormat::Jpeg {
        "jpg"
    } else {
        "png"
    };
    presets
        .iter()
        .map(|p| format!("{base}-{}.{ext}", p.id()))
        .collect()
}

pub fn export_base_name(file_name: &str) -> String {
    slug_name(file_name, EXPORT_FOLDER_FALLBACK)
}

fn file_bytes(artifacts: &[Artifact]) -> f64 {
    artifacts.iter().map(|a| a.bytes.unwrap_or(0.0)).sum()
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExportVideo {
    #[default = "{{Project name}}"]
    #[schemars(description = "File name")]
    pub file_name: String,
    #[default(Codec::H264)]
    #[schemars(
        description = "Format",
        extend("x-labels" = { "h264": "MP4", "h265": "MP4 (HEVC)", "vp8": "WebM (VP8)", "vp9": "WebM", "prores": "ProRes", "gif": "GIF" })
    )]
    pub codec: Codec,
    #[default = 18]
    #[schemars(description = "Quality", range(min = 0, max = 51), extend("x-widget" = "quality"))]
    pub crf: i64,
    #[default(vec![Variant::Wide])]
    #[schemars(
        description = "Shapes",
        length(min = 1),
        extend("x-widget" = "multiselect", "x-check" = { "rule": "unique", "message": "Pick each shape once" })
    )]
    pub variants: Vec<Variant>,
    #[default(ReframeMode::Follow)]
    #[schemars(description = "Other shapes", extend("x-labels" = { "follow": "Follow the action", "center": "Crop the centre", "fit": "Add bars" }))]
    pub reframe: ReframeMode,
    #[default = 1.0]
    #[schemars(
        description = "Size",
        range(min = 0.25, max = 2),
        extend("x-step" = 0.25, "x-unit" = "×", "x-labels" = { "0.5": "Half", "1": "Full", "2": "Double" })
    )]
    pub scale: f64,
    #[default = true]
    #[schemars(
        description = "Chapters in the file",
        extend(
            "x-help" = "Players list the film's chapters, taken from its Chapter actions.",
            "x-show-if" = { "codec": ["h264", "h265", "vp8", "vp9", "prores"] }
        )
    )]
    pub chapters: bool,
    #[schemars(
        description = "YouTube chapter list",
        extend(
            "x-help" = "Also saves the chapter times as text, to paste into a video's description. It needs at least three chapters of ten seconds or more.",
            "x-show-if" = { "codec": ["h264", "h265", "vp8", "vp9", "prores"] }
        )
    )]
    pub chapter_list: bool,
    #[default(Gl::Angle)]
    #[schemars(
        description = "Draw with",
        extend(
            "x-labels" = {
                "angle": "Graphics card",
                "swangle": "Software",
                "egl": "Graphics card, direct",
                "vulkan": "Graphics card, newer drivers",
                "swiftshader": "Software (legacy)"
            },
            "x-help" = "The graphics card is about ten times faster. Use Software if exports fail.",
            "x-advanced" = true
        )
    )]
    pub gl: Gl,
}

impl Params for ExportVideo {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExportImages {
    #[default = "{{Project name}}"]
    #[schemars(description = "File name")]
    pub file_name: String,
    #[default = 1000]
    #[schemars(description = "Frame at", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub at_ms: i64,
    #[default(vec![StillPreset::Og])]
    #[schemars(
        description = "Sizes",
        length(min = 1),
        extend(
            "x-widget" = "multiselect",
            "x-labels" = { "wide": "Wide 4K", "og": "Link preview", "square": "Square", "portrait": "Portrait", "story": "Story" },
            "x-check" = { "rule": "unique", "message": "Pick each size once" }
        )
    )]
    pub presets: Vec<StillPreset>,
    #[default(StillFormat::Png)]
    #[schemars(description = "Format", extend("x-labels" = { "png": "PNG", "jpeg": "JPEG" }))]
    pub format: StillFormat,
    #[default(Gl::Angle)]
    #[schemars(
        description = "Draw with",
        extend(
            "x-labels" = {
                "angle": "Graphics card",
                "swangle": "Software",
                "egl": "Graphics card, direct",
                "vulkan": "Graphics card, newer drivers",
                "swiftshader": "Software (legacy)"
            },
            "x-help" = "The graphics card is about ten times faster. Use Software if exports fail.",
            "x-advanced" = true
        )
    )]
    pub gl: Gl,
}

impl Params for ExportImages {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExportCode {
    #[default = "video-code"]
    #[schemars(description = "Folder name")]
    pub folder_name: String,
}

impl Params for ExportCode {}

fn output_node<P: Params>(
    kind: &'static str,
    title: &str,
    phase: Phase,
    description: &str,
    port_id: &'static str,
    port_type: PortType,
    label: &'static str,
) -> NodeDef {
    NodeDef::of::<P>(kind, title, Category::Output, phase)
        .describe(description)
        .executes()
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .outputs(vec![(port_id, port(port_type, label))])
        .compile(move |ctx: &Ctx<'_, P>| {
            let value = ctx
                .result
                .and_then(|r| r.output(port_id))
                .filter(|v| v.port_type() == port_type);
            Ok(Some(
                value
                    .map(|v| (port_id.to_owned(), v.clone()))
                    .into_iter()
                    .collect::<Outputs>(),
            ))
        })
}

fn export_video() -> NodeDef {
    output_node::<ExportVideo>(
        RENDER_TYPE,
        "Export video",
        Phase::Render,
        "Renders the final video, one file per shape (16:9, 9:16…).",
        "media",
        PortType::Media,
        "File",
    )
    .card(|ctx: &CardCtx<'_, ExportVideo>| {
        let p = &ctx.params;
        let media = ctx
            .result
            .and_then(|r| r.output("media"))
            .and_then(CompileValue::as_media);
        let bytes = ctx.result.map_or(0.0, |r| file_bytes(&r.artifacts));
        let mut sides: Vec<String> = Vec::new();
        for v in &p.variants {
            let side = format!("{}p", (exported_short_side(*v, ctx.settings, p.scale)));
            if !sides.contains(&side) {
                sides.push(side);
            }
        }
        let shapes: Vec<&str> = p.variants.iter().map(|v| v.id()).collect();
        vec![
            Some(row(Icon::Ratio, shapes.join(" + "))),
            Some(row(
                Icon::Film,
                format!(
                    "{} · {} · {} fps",
                    p.codec.name(),
                    sides.join(" / "),
                    ctx.fps()
                ),
            )),
            media
                .and_then(|m| m.duration_ms)
                .filter(|d| *d != 0.0)
                .map(|d| row(Icon::Clock, format_duration(d))),
            (bytes > 0.0).then(|| row(Icon::HardDrive, format_bytes(bytes))),
        ]
    })
}

fn export_images() -> NodeDef {
    output_node::<ExportImages>(
        STILL_TYPE,
        "Export images",
        Phase::Render,
        "Saves a frame as images at social sizes: wide, link preview, square, portrait, story.",
        "image",
        PortType::Image,
        "Image",
    )
    .card(|ctx: &CardCtx<'_, ExportImages>| {
        let p = &ctx.params;
        let film_ms = ctx
            .inputs
            .frames
            .get(CLIP_PORT)
            .filter(|f| **f != 0.0)
            .map(|f| frames_to_ms(*f, ctx.fps()));
        let names: Vec<&str> = p.presets.iter().map(|k| k.name()).collect();
        let format = if p.format == StillFormat::Jpeg {
            "JPEG"
        } else {
            "PNG"
        };
        vec![
            Some(row(Icon::Image, names.join(" · "))),
            Some(row(
                Icon::Clock,
                format!("At {} · {format}", format_duration(p.at_ms as f64)),
            )),
            film_ms
                .filter(|ms| *ms != 0.0 && p.at_ms as f64 >= *ms)
                .map(|ms| {
                    row(
                        Icon::TriangleAlert,
                        format!(
                            "The film ends at {}, so it uses the last frame",
                            format_duration(ms)
                        ),
                    )
                    .tone(Tone::Warn)
                }),
        ]
    })
}

fn export_code() -> NodeDef {
    output_node::<ExportCode>(
        "output.exportProject",
        "Export as code",
        Phase::Post,
        "Saves this video as an editable code project for developers.",
        "folder",
        PortType::Folder,
        "Folder",
    )
    .card(|ctx: &CardCtx<'_, ExportCode>| {
        vec![Some(row(
            Icon::FolderOutput,
            slug_name(&ctx.params.folder_name, EXPORT_FOLDER_FALLBACK),
        ))]
    })
}

pub fn export_reframe(nodes: &[WorkflowNode]) -> ReframeMode {
    nodes
        .iter()
        .find(|n| n.kind == RENDER_TYPE && !n.is_disabled())
        .and_then(|n| n.params.get("reframe"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(ReframeMode::Follow)
}

pub fn nodes() -> Vec<NodeDef> {
    vec![export_video(), export_images(), export_code()]
}
