use opsis_macros::{choice, settings};
use opsis_types::cards::{Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::whole;
use opsis_types::timeline::ReframeMode;
use opsis_types::values::{Artifact, CompileValue};
use opsis_types::workflow::{PortType, WorkflowSettings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::node::{
    CLIP_PORT, CardCtx, Category, Claim, Ctx, NodeDef, Outputs, Phase, RowExt, port, row,
};
use crate::nodes::media::{Codec, EncodePreset, ProresProfile};
use crate::params::Params;
use crate::text::{format_duration, slug_name};
use crate::time::{even_px, frames_to_ms};

const EXPORT_NAME_FALLBACK: &str = "output";
pub const RENDER_TYPE: &str = "output.render";
pub const STILL_TYPE: &str = "output.still";

#[choice(export)]
pub enum Gl {
    #[label("Graphics card")]
    Angle,
    #[label("Software")]
    Swangle,
    #[label("Graphics card, direct")]
    Egl,
    #[label("Graphics card, newer drivers")]
    Vulkan,
    #[label("Software (legacy)")]
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
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
    pub const ALL: [Self; 4] = [Self::Wide, Self::Square, Self::Portrait, Self::Tall];

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
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Long,
    Short,
}

fn size_for(variant: Variant, edge: u32, anchor: Edge) -> Size {
    let edge = f64::from(edge);
    let (w, h) = variant.ratio();
    let landscape = w >= h;
    let edge_is_width = if anchor == Edge::Long {
        landscape
    } else {
        !landscape
    };
    if edge_is_width {
        Size {
            width: whole(even_px(edge)),
            height: whole(even_px(edge * h / w)),
        }
    } else {
        Size {
            width: whole(even_px(edge * w / h)),
            height: whole(even_px(edge)),
        }
    }
}

pub fn variant_size(variant: Variant, settings: &WorkflowSettings) -> Size {
    size_for(variant, settings.width.max(settings.height), Edge::Long)
}

pub fn variant_canvas(variant: Variant, short: u32) -> Size {
    size_for(variant, short, Edge::Short)
}

pub const CANVAS_SHORT_SIDE: u32 = 1080;

fn exported_short_side(variant: Variant, settings: &WorkflowSettings, scale: f64) -> u32 {
    let size = variant_size(variant, settings);
    whole(f64::from(size.width.min(size.height)) * scale)
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
            Self::Wide => (3840, 2160),
            Self::Og => (2400, 1260),
            Self::Square => (2160, 2160),
            Self::Portrait => (1728, 2160),
            Self::Story => (1080, 1920),
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

#[choice(export)]
pub enum StillFormat {
    #[label("PNG")]
    Png,
    #[label("JPEG")]
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
    slug_name(file_name, EXPORT_NAME_FALLBACK)
}

fn file_bytes(artifacts: &[Artifact]) -> u64 {
    artifacts.iter().filter_map(|a| a.bytes).sum()
}

#[settings]
pub struct ExportVideo {
    #[setting("File name", default = "{{Project name}}")]
    pub file_name: String,
    #[setting(
        "Format",
        default = Codec::H264
    )]
    pub codec: Codec,
    #[setting("Quality", default = 18, range(min = 0, max = 51), widget = "quality")]
    pub crf: u32,
    #[setting(
        "Shapes",
        default = vec![Variant::Wide],
        length(min = 1),
        widget = "multiselect",
        check = { "rule": "unique", "message": "Pick each shape once" }
    )]
    pub variants: Vec<Variant>,
    #[setting(
        "Other shapes",
        default = ReframeMode::Follow
    )]
    pub reframe: ReframeMode,
    #[setting(
        "Size",
        default = 1.0,
        range(min = 0.25, max = 2),
        step = 0.25,
        unit = "×",
        labels = { "0.5": "Half", "1": "Full", "2": "Double" }
    )]
    pub scale: f64,
    #[setting(
        "Chapters in the file",
        default = true,
        help = "Players list the film's chapters, taken from its Chapter actions.",
        show_if = { "codec": ["h264", "h265", "vp8", "vp9", "prores"] },
        advanced
    )]
    pub chapters: bool,
    #[setting(
        "YouTube chapter list",
        help = "Also saves the chapter times as text, to paste into a video's description. It needs at least three chapters of ten seconds or more.",
        show_if = { "codec": ["h264", "h265", "vp8", "vp9", "prores"] }
    )]
    pub chapter_list: bool,
    #[setting(
        "Draw with",
        default = Gl::Angle,
        help = "The graphics card is about ten times faster. Use Software if exports fail.",
        advanced
    )]
    pub gl: Gl,
    #[setting(
        "First chapter name",
        default = "Intro",
        help = "Names the chapter added at the start when the first one comes later.",
        show_if = { "chapters": ["true"] },
        advanced
    )]
    pub first_chapter: String,
    #[setting(
        "Speed vs file size",
        default = EncodePreset::Medium,
        show_if = { "codec": ["h264"] },
        advanced
    )]
    pub preset: EncodePreset,
    #[setting(
        "Full colour detail",
        help = "Keeps every pixel's own colour instead of sharing it between neighbours. Sharper coloured text; fewer players open it.",
        show_if = { "codec": ["h264", "h265"] },
        advanced
    )]
    pub full_colour: bool,
    #[setting(
        "ProRes kind",
        show_if = { "codec": ["prores"] },
        advanced
    )]
    pub prores_profile: ProresProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Sound bit rate",
        range(min = 64, max = 512),
        step = 32,
        unit = "kbps",
        zero_label = "The format's own",
        show_if = { "codec": ["h264", "h265", "vp8", "vp9"] },
        "type" = "integer",
        advanced
    )]
    pub sound_kbps: Option<u32>,
    #[setting(
        "Plays",
        range(min = 0, max = 100),
        zero_label = "Forever",
        help = "How many times the GIF plays. 0 plays it forever.",
        show_if = { "codec": ["gif"] },
        advanced
    )]
    pub gif_plays: u32,
    #[setting(
        "Every nth frame",
        default = 1,
        range(min = 1, max = 10),
        help = "Keeps one frame in this many, for a smaller GIF.",
        show_if = { "codec": ["gif"] },
        advanced
    )]
    pub gif_every_nth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum PixelFormat {
    #[serde(rename = "yuv444p")]
    Yuv444p,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum RenderProres {
    Proxy,
    Light,
    Standard,
}

impl ExportVideo {
    pub fn x264_preset(&self) -> Option<EncodePreset> {
        (self.codec == Codec::H264 && self.preset != EncodePreset::Medium).then_some(self.preset)
    }

    pub fn pixel_format(&self) -> Option<PixelFormat> {
        (self.full_colour && matches!(self.codec, Codec::H264 | Codec::H265))
            .then_some(PixelFormat::Yuv444p)
    }

    pub fn prores(&self) -> Option<RenderProres> {
        if self.codec != Codec::Prores {
            return None;
        }
        match self.prores_profile {
            ProresProfile::Hq => None,
            ProresProfile::Proxy => Some(RenderProres::Proxy),
            ProresProfile::Lt => Some(RenderProres::Light),
            ProresProfile::Standard => Some(RenderProres::Standard),
        }
    }

    pub fn gif_loops(&self) -> Option<u32> {
        (self.codec == Codec::Gif && self.gif_plays > 0).then(|| self.gif_plays - 1)
    }

    pub fn every_nth_frame(&self) -> Option<u32> {
        (self.codec == Codec::Gif && self.gif_every_nth > 1).then_some(self.gif_every_nth)
    }
}

#[settings]
pub struct ExportImages {
    #[setting("File name", default = "{{Project name}}")]
    pub file_name: String,
    #[setting(
        "Frame at",
        default = 1000,
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms"
    )]
    pub at_ms: u32,
    #[setting(
        "Sizes",
        default = vec![StillPreset::Og],
        length(min = 1),
        widget = "multiselect",
        labels = { "wide": "Wide 4K", "og": "Link preview", "square": "Square", "portrait": "Portrait", "story": "Story" },
        check = { "rule": "unique", "message": "Pick each size once" }
    )]
    pub presets: Vec<StillPreset>,
    #[setting("Format", default = StillFormat::Png)]
    pub format: StillFormat,
    #[setting(
        "Draw with",
        default = Gl::Angle,
        help = "The graphics card is about ten times faster. Use Software if exports fail.",
        advanced
    )]
    pub gl: Gl,
    #[setting(
        "Other shapes",
        default = ReframeMode::Follow,
        advanced
    )]
    pub reframe: ReframeMode,
    #[setting(
        "JPEG quality",
        default = 80,
        range(min = 1, max = 100),
        show_if = { "format": ["jpeg"] },
        advanced
    )]
    pub jpeg_quality: u32,
    #[setting(
        "Size",
        default = 1.0,
        range(min = 0.25, max = 2),
        step = 0.25,
        unit = "×",
        labels = { "0.5": "Half", "1": "Full", "2": "Double" },
        advanced
    )]
    pub scale: f64,
}

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
    .claims(|p: &ExportVideo| {
        video_file_names(&export_base_name(&p.file_name), p.codec, &p.variants)
            .into_iter()
            .map(Claim::File)
            .collect()
    })
    .card(|ctx: &CardCtx<'_, ExportVideo>| {
        let p = &ctx.params;
        let media = ctx
            .result
            .and_then(|r| r.output("media"))
            .and_then(CompileValue::as_media);
        let bytes = ctx.result.map_or(0, |r| file_bytes(&r.artifacts));
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
                    p.codec.label(),
                    sides.join(" / "),
                    ctx.fps()
                ),
            )),
            media
                .and_then(|m| m.duration_ms)
                .filter(|d| *d != 0)
                .map(|d| row(Icon::Clock, format_duration(f64::from(d)))),
            (bytes > 0).then(|| row(Icon::HardDrive, opsis_types::numbers::byte_size(bytes))),
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
    .claims(|p: &ExportImages| {
        still_file_names(&export_base_name(&p.file_name), &p.presets, p.format)
            .into_iter()
            .map(Claim::File)
            .collect()
    })
    .card(|ctx: &CardCtx<'_, ExportImages>| {
        let p = &ctx.params;
        let film_ms = ctx
            .inputs
            .frames
            .get(CLIP_PORT)
            .filter(|f| **f != 0)
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
                format!("At {} · {format}", format_duration(f64::from(p.at_ms))),
            )),
            film_ms
                .filter(|ms| *ms != 0.0 && f64::from(p.at_ms) >= *ms)
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

pub fn nodes() -> Vec<NodeDef> {
    vec![
        export_video().look(NodeLook::of("lucide:clapperboard")),
        export_images().look(NodeLook::of("lucide:image-down")),
    ]
}
