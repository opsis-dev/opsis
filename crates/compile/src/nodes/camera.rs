use opsis_types::actions::{Action, ActionKind, Point, Rect};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::recording::{CameraStyle, CameraTuning, FrameTheme, RecordingProps, WindowStyle};
use opsis_types::timeline::{ElementRef, GroupProps};
use opsis_types::values::{ClipValue, CompileValue, StyleEntry, TaggedRecording};
use opsis_types::workflow::{DEFAULT_TAKE_MS, PortType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smart_default::SmartDefault;

use crate::asset_ref::base_name;
use crate::backdrops::Backdrop;
use crate::card::toggle_row;
use crate::clip::{
    ItemExtra, clip_frame_of_recording, patch_recordings, placeholder_clip, recording_clip,
    wrap_clip,
};
use crate::color::is_color;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, InputTake, NodeDef, Phase, Platform, RECORDING_PORT, RowExt,
    STYLE_PORT, missing, opt, outputs, port, row, with_clip,
};
use crate::params::{Params, Values};
use crate::recording::{
    DEFAULT_ADDRESS_BAR, DEFAULT_FRAME_THEME, DEFAULT_WINDOW_STYLE, default_recording_props,
    follow_cursor_beats, host_window_style, idle_spans, point_rect, recording_speed,
    window_style_label,
};
use crate::remap::{RemapSegment, remap_duration, remap_from_segments};
use crate::style::{restyle_with, retime_with};
use crate::text::{format_duration, humanize, number, percent_text, plural};
use crate::time::{EPSILON, ms_to_frames, recording_ms_to_frames, whole_frames};

const FOCUS_LEAD_MS: f64 = 400.0;
pub const FOLLOW_CURSOR_TYPE: &str = "camera.followCursor";
const CURSOR_STYLE_TYPE: &str = crate::nodes::cursor::CURSOR_STYLE_TYPE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FrameChoice {
    None,
    Macos,
    Windows,
    Linux,
}

impl FrameChoice {
    fn style(self) -> Option<WindowStyle> {
        match self {
            Self::None => None,
            Self::Macos => Some(WindowStyle::Macos),
            Self::Windows => Some(WindowStyle::Windows),
            Self::Linux => Some(WindowStyle::Linux),
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoZoom {
    #[default = true]
    #[schemars(description = "Zoom to the clicks")]
    pub zoom: bool,
    #[default = 1.74]
    #[schemars(
        description = "Maximum zoom",
        range(min = 1.1, max = 2.5),
        extend("x-step" = 0.02, "x-unit" = "×", "x-help" = "Higher zooms closer but can look soft.")
    )]
    pub max_scale: f64,
    #[default = 1300]
    #[schemars(description = "Hold each zoom at least", range(min = 0, max = 10000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub hold: i64,
    #[default = 1.25]
    #[schemars(description = "Playback speed", range(min = 0.25, max = 4), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub speed: f64,
    #[default(CameraStyle::Classic)]
    #[schemars(description = "Camera style")]
    pub style: CameraStyle,
    #[default(Backdrop::Glaze)]
    #[schemars(description = "Backdrop", extend("x-show-if" = { "style": ["classic"] }))]
    pub backdrop: Backdrop,
    #[default(FrameChoice::None)]
    #[schemars(
        description = "Window frame",
        extend("x-labels" = { "none": "None", "macos": "macOS", "windows": "Windows", "linux": "Linux" })
    )]
    pub frame: FrameChoice,
    #[default(DEFAULT_FRAME_THEME)]
    #[schemars(description = "Frame colour", extend("x-show-if" = { "frame": ["macos", "windows", "linux"] }))]
    pub frame_theme: FrameTheme,
    #[default(DEFAULT_ADDRESS_BAR)]
    #[schemars(
        description = "Address bar",
        extend("x-help" = "Shows the page's address in the title bar.", "x-show-if" = { "frame": ["macos", "windows", "linux"] })
    )]
    pub address_bar: bool,
    #[default = true]
    #[schemars(description = "Motion blur")]
    pub motion_blur: bool,
}

impl Params for AutoZoom {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ManualZoom {
    #[default = 1000]
    #[schemars(description = "At", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub at_ms: i64,
    #[default = 1500]
    #[schemars(description = "Hold", range(min = 100, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub hold_ms: i64,
    #[default = 0.5]
    #[schemars(
        description = "Spot",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-widget" = "spot", "x-time-key" = "atMs", "x-help" = "Click the frame to choose where to zoom.")
    )]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Spot vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub y: f64,
    #[default = 1.5]
    #[schemars(description = "Zoom", range(min = 1.1, max = 2.5), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub scale: f64,
}

impl Params for ManualZoom {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Pan {
    #[default = 1000]
    #[schemars(description = "Start at", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub at_ms: i64,
    #[default = 1500]
    #[schemars(description = "Pan for", range(min = 100, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub duration_ms: i64,
    #[default = 0.3]
    #[schemars(
        description = "From",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-widget" = "spot", "x-time-key" = "atMs", "x-help" = "Click the frame to choose where to start.")
    )]
    pub from_x: f64,
    #[default = 0.4]
    #[schemars(description = "From vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub from_y: f64,
    #[default = 0.7]
    #[schemars(
        description = "To",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-widget" = "spot", "x-time-key" = "atMs", "x-help" = "Click the frame to choose where to end.")
    )]
    pub to_x: f64,
    #[default = 0.6]
    #[schemars(description = "To vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub to_y: f64,
    #[default = 1.4]
    #[schemars(description = "Zoom", range(min = 1.1, max = 2.5), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub scale: f64,
}

impl Params for Pan {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FollowCursor {
    #[default = 1.35]
    #[schemars(description = "Zoom", range(min = 1.1, max = 2.5), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub scale: f64,
    #[default = 700]
    #[schemars(description = "Re-aim every", range(min = 200, max = 5000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub every_ms: i64,
}

impl Params for FollowCursor {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct IdleSpeedup {
    #[default = 1500]
    #[schemars(description = "Quiet for longer than", range(min = 0, max = 20000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub threshold_ms: i64,
    #[default = 400]
    #[schemars(description = "Keep around each action", range(min = 0, max = 5000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub pad_ms: i64,
    #[default = 4.0]
    #[schemars(description = "Speed while quiet", range(min = 1.5, max = 64), extend("x-step" = 0.5, "x-unit" = "×"))]
    pub rate: f64,
}

impl Params for IdleSpeedup {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeedSegment {
    #[schemars(description = "From", range(min = 0, max = 9007199254740991_i64), extend("x-step" = 50, "x-unit" = "ms"))]
    pub from_ms: i64,
    #[default = 1000]
    #[schemars(description = "To", range(min = 0, max = 9007199254740991_i64), extend("x-step" = 50, "x-unit" = "ms"))]
    pub to_ms: i64,
    #[default = 2.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 64), extend("x-step" = 0.1, "x-unit" = "×"))]
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeedRamp {
    #[default(vec![SpeedSegment::default()])]
    #[schemars(description = "Parts")]
    pub segments: Vec<SpeedSegment>,
}

impl Params for SpeedRamp {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Crop {
    #[default = 0.1]
    #[schemars(description = "Left", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.1]
    #[schemars(description = "Top", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
    #[default = 0.8]
    #[schemars(description = "Width", range(min = 0.05, max = 1), extend("x-step" = 0.01))]
    pub w: f64,
    #[default = 0.8]
    #[schemars(description = "Height", range(min = 0.05, max = 1), extend("x-step" = 0.01))]
    pub h: f64,
}

impl Params for Crop {
    fn check(&self) -> Result<(), String> {
        if self.x + self.w > 1.0 + EPSILON {
            return Err("The area runs past the right edge".to_owned());
        }
        if self.y + self.h > 1.0 + EPSILON {
            return Err("The area runs past the bottom edge".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Laptop,
    Phone,
}

impl Device {
    fn label(self) -> &'static str {
        match self {
            Self::Laptop => "Laptop",
            Self::Phone => "Phone",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Laptop => Icon::Laptop,
            Self::Phone => Icon::Smartphone,
        }
    }

    fn component(self) -> &'static str {
        match self {
            Self::Laptop => "snapcn/laptop-frame",
            Self::Phone => "snapcn/phone-frame",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeviceEntrance {
    Rise,
    Open,
    None,
}

impl DeviceEntrance {
    fn label(self) -> &'static str {
        match self {
            Self::Rise => "Slide in",
            Self::Open => "Open up",
            Self::None => "None",
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceFrame {
    #[default(Device::Laptop)]
    #[schemars(description = "Device", extend("x-labels" = { "laptop": "Laptop", "phone": "Phone" }))]
    pub device: Device,
    #[default(DeviceEntrance::Open)]
    #[schemars(
        description = "Entrance",
        extend("x-labels" = { "rise": "Slide in", "open": "Open up", "none": "None" }, "x-show-if" = { "device": ["laptop"] })
    )]
    pub entrance: DeviceEntrance,
}

impl Params for DeviceFrame {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BackdropKind {
    Image,
    Solid,
    Gradient,
    Asset,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BackdropStyle {
    #[default(BackdropKind::Image)]
    #[schemars(
        description = "Kind",
        extend("x-labels" = { "image": "Studio image", "solid": "Colour", "gradient": "Gradient", "asset": "Your image" })
    )]
    pub kind: BackdropKind,
    #[default(Backdrop::Glaze)]
    #[schemars(description = "Studio image", extend("x-show-if" = { "kind": ["image"] }))]
    pub name: Backdrop,
    #[default = "#0b0b10"]
    #[schemars(description = "Colour", extend("x-widget" = "color", "x-show-if" = { "kind": ["solid"] }))]
    pub color: String,
    #[default = "linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)"]
    #[schemars(description = "Gradient", extend("x-widget" = "gradient", "x-show-if" = { "kind": ["gradient"] }))]
    pub gradient: String,
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*", "x-show-if" = { "kind": ["asset"] }))]
    pub asset: String,
}

impl Params for BackdropStyle {
    fn check(&self) -> Result<(), String> {
        match self.kind {
            BackdropKind::Solid if !is_color(&self.color) => Err("Pick a colour".to_owned()),
            BackdropKind::Gradient if self.gradient.trim().is_empty() => {
                Err("Pick a gradient".to_owned())
            }
            BackdropKind::Asset if self.asset.trim().is_empty() => {
                Err("Choose an image".to_owned())
            }
            _ => Ok(()),
        }
    }
}

impl BackdropStyle {
    fn card_row(&self) -> CardRow {
        match self.kind {
            BackdropKind::Image => row(Icon::Image, humanize(self.name.id())),
            BackdropKind::Solid => row(Icon::Palette, self.color.clone()),
            BackdropKind::Gradient => row(Icon::Blend, "Gradient").titled(self.gradient.clone()),
            BackdropKind::Asset if self.asset.is_empty() => missing(Icon::Image, "Choose an image"),
            BackdropKind::Asset => row(Icon::Image, base_name(&self.asset)),
        }
    }

    fn chosen(&self) -> Option<String> {
        match self.kind {
            BackdropKind::Image => Some(self.name.id().to_owned()),
            BackdropKind::Solid => Some(self.color.clone()),
            BackdropKind::Gradient => Some(self.gradient.clone()),
            BackdropKind::Asset => (!self.asset.is_empty()).then(|| self.asset.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MotionBlur {
    #[default = true]
    #[schemars(description = "Motion blur")]
    pub enabled: bool,
}

impl Params for MotionBlur {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowFrame {
    #[default(DEFAULT_WINDOW_STYLE)]
    #[schemars(description = "Window style", extend("x-labels" = { "macos": "macOS", "windows": "Windows", "linux": "Linux" }))]
    pub window_style: WindowStyle,
    #[default(DEFAULT_FRAME_THEME)]
    #[schemars(description = "Frame colour")]
    pub frame_theme: FrameTheme,
    #[default(DEFAULT_ADDRESS_BAR)]
    #[schemars(description = "Address bar", extend("x-help" = "Shows the page's address in the title bar."))]
    pub address_bar: bool,
}

impl Params for WindowFrame {}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ApplyStyle {}

impl Params for ApplyStyle {}

fn with_window_frame(
    props: RecordingProps,
    style: WindowStyle,
    theme: FrameTheme,
    address_bar: bool,
) -> RecordingProps {
    RecordingProps {
        chrome: true,
        window_style: Some(style),
        frame_theme: Some(theme),
        address_bar: Some(address_bar),
        ..props
    }
}

fn frame_theme_name(theme: FrameTheme) -> &'static str {
    match theme {
        FrameTheme::Light => "light",
        FrameTheme::Dark => "dark",
    }
}

fn take_row(take: Option<InputTake>, at_ms: i64) -> Option<CardRow> {
    let take = take?;
    (at_ms as f64 >= take.duration_ms).then(|| {
        row(
            Icon::TriangleAlert,
            format!(
                "After the take ends at {}",
                format_duration(take.duration_ms)
            ),
        )
        .tone(Tone::Warn)
    })
}

struct FocusSpot {
    id: &'static str,
    t: f64,
    x: f64,
    y: f64,
    t_depart: Option<f64>,
}

fn focus_beats(
    mut props: RecordingProps,
    node_id: &str,
    cluster: &str,
    scale: f64,
    spots: &[FocusSpot],
) -> RecordingProps {
    let vp = props.log.viewport;
    for spot in spots {
        let point = Point {
            x: spot.x * vp.width,
            y: spot.y * vp.height,
        };
        let mut beat = Action::new(format!("{node_id}:{}", spot.id), ActionKind::Focus, spot.t);
        beat.t_depart = spot.t_depart;
        beat.point = Some(point);
        beat.rect = Some(point_rect(point, 1.0));
        beat.cluster = Some(format!("{node_id}:{cluster}"));
        beat.zoom_scale = Some(scale);
        beat.node_id = Some(node_id.to_owned());
        props.extra_beats.push(beat);
    }
    props
}

fn remap_clip(clip: ClipValue, node_id: &str, fps: f64, segments: &[(f64, f64, f64)]) -> ClipValue {
    let to_frames = |recording_ms: f64| {
        clip_frame_of_recording(&clip.tracks, &|rec| {
            recording_ms_to_frames(recording_ms, recording_speed(rec), fps)
        })
        .unwrap_or_else(|| recording_ms_to_frames(recording_ms, 1.0, fps))
    };
    let parts: Vec<RemapSegment> = segments
        .iter()
        .map(|(from, to, rate)| RemapSegment {
            from: to_frames(*from),
            to: to_frames(*to),
            rate: *rate,
        })
        .collect();
    let remap = remap_from_segments(clip.duration_in_frames, &parts);
    let duration = whole_frames(remap_duration(&remap), 1.0);
    wrap_clip(
        &clip,
        node_id,
        GroupProps {
            remap: Some(remap),
            ..GroupProps::default()
        },
        Some(duration),
    )
}

fn entries_of(value: Option<&CompileValue>) -> Vec<StyleEntry> {
    value
        .and_then(CompileValue::as_style)
        .map(|s| s.entries.clone())
        .unwrap_or_default()
}

fn auto_zoom() -> NodeDef {
    NodeDef::of::<AutoZoom>("camera.autoZoom", "Auto zoom", Category::Camera, Phase::Compile)
        .describe("Turns a recording into a polished clip: the window floats on a backdrop and the camera zooms to every click.")
        .inputs(vec![(RECORDING_PORT, port(PortType::Recording, "Recording")), (STYLE_PORT, opt(PortType::RecordingStyle, "Style"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, AutoZoom>| {
            let p = &ctx.params;
            let no_track = ctx.inputs.takes.get(RECORDING_PORT).is_some_and(|t| t.is_some_and(|t| !t.cursor_track));
            let styled = |kind: &str| ctx.inputs.style.get(STYLE_PORT).is_some_and(|s| s.iter().any(|t| t == kind));
            vec![
                (no_track && styled(FOLLOW_CURSOR_TYPE)).then(|| {
                    row(Icon::TriangleAlert, "The take has no cursor to follow")
                        .tone(Tone::Warn)
                        .titled("Set Cursor to Smooth on the Record take node.")
                }),
                (no_track && styled(CURSOR_STYLE_TYPE)).then(|| {
                    row(Icon::TriangleAlert, "The take films its own cursor").tone(Tone::Warn).titled(
                        "The Cursor node's look, size and colours apply only to a drawn cursor. Set Cursor to Smooth on the Record take node.",
                    )
                }),
                Some(if p.zoom {
                    row(Icon::ZoomIn, format!("Up to {}×", number(p.max_scale)))
                } else {
                    row(Icon::ZoomOut, "No zoom").tone(Tone::Muted)
                }),
                p.zoom.then(|| row(Icon::Hourglass, format!("Hold {}", format_duration(p.hold as f64)))),
                (p.style == CameraStyle::Classic).then(|| row(Icon::Image, format!("{} backdrop", humanize(p.backdrop.id())))),
                (p.speed != 1.0).then(|| row(Icon::Gauge, format!("{}× speed", number(p.speed)))),
            ]
        })
        .compile(|ctx: &Ctx<'_, AutoZoom>| {
            let p = &ctx.params;
            let rec = ctx.input(RECORDING_PORT).and_then(CompileValue::as_recording);
            let mut entries = entries_of(ctx.input(STYLE_PORT));
            entries.extend(rec.map(|r| r.style.clone()).unwrap_or_default());
            let duration_ms = rec.map_or(DEFAULT_TAKE_MS, |r| r.duration_ms);
            let frames = ms_to_frames(duration_ms / p.speed, ctx.settings.fps);
            let (Some(rec), Some(src), Some(log)) = (rec, rec.and_then(|r| r.src.clone()), rec.and_then(|r| r.log.clone())) else {
                let placeholder = placeholder_clip(
                    ctx.node_id,
                    "Not recorded yet",
                    "Run to record the take. The camera moves appear here afterwards.",
                    frames,
                    ItemExtra::default(),
                );
                return Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(retime_with(placeholder, &entries, ctx.settings.fps)?))])));
            };
            let mut own = default_recording_props(src, log, rec.node_id.clone());
            own.proxy_src = rec.proxy_src.clone().filter(|s| !s.is_empty());
            own.speed = p.speed;
            own.zoom = p.zoom;
            own.tuning = CameraTuning { s_max: Some(p.max_scale), hold_min_s: Some(p.hold as f64 / 1000.0) };
            own.motion_blur = p.motion_blur;
            own.style = p.style;
            own = match p.frame.style() {
                None => RecordingProps { chrome: false, ..own },
                Some(style) => with_window_frame(own, style, p.frame_theme, p.address_bar),
            };
            own.backdrop = p.backdrop.id().to_owned();
            let props = restyle_with(own, &entries)?;
            let clip = recording_clip(ctx.node_id, props, frames, ItemExtra { recording: Some(rec.clone()), ..ItemExtra::default() });
            Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(retime_with(clip, &entries, ctx.settings.fps)?))])))
        })
}

fn apply_style() -> NodeDef {
    NodeDef::of::<ApplyStyle>(
        "camera.applyStyle",
        "Apply style",
        Category::Camera,
        Phase::Compile,
    )
    .describe(
        "Gives every recording in a clip the same style, for takes joined in a Sequence or Layers.",
    )
    .inputs(vec![
        (CLIP_PORT, port(PortType::Clip, "Clip")),
        (STYLE_PORT, port(PortType::RecordingStyle, "Style")),
    ])
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, ApplyStyle>| {
        let entries = ctx.inputs.style.get(STYLE_PORT).map_or(0, Vec::len);
        vec![
            Some(
                row(Icon::Palette, plural(entries as f64, "style setting")).muted_if(entries == 0),
            ),
            matches!(ctx.inputs.takes.get(CLIP_PORT), Some(None)).then(|| {
                row(Icon::TriangleAlert, "No recording in this clip")
                    .tone(Tone::Warn)
                    .titled("A style changes only recordings.")
            }),
        ]
    })
    .compile(|ctx: &Ctx<'_, ApplyStyle>| {
        let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
            return Ok(Some(Default::default()));
        };
        let entries = entries_of(ctx.input(STYLE_PORT));
        let failed = std::cell::RefCell::new(None);
        let styled = patch_recordings(clip, &|props| match restyle_with(props.clone(), &entries) {
            Ok(p) => p,
            Err(e) => {
                failed.borrow_mut().get_or_insert(e);
                props
            }
        });
        if let Some(e) = failed.into_inner() {
            return Err(e);
        }
        Ok(Some(outputs([(
            CLIP_PORT,
            CompileValue::Clip(retime_with(styled, &entries, ctx.settings.fps)?),
        )])))
    })
}

fn manual_zoom() -> NodeDef {
    NodeDef::recording_modifier::<ManualZoom>("camera.manualZoom", "Zoom here", Category::Camera)
        .describe("Zooms to a spot of your choice at a set time.")
        .card(|ctx: &CardCtx<'_, ManualZoom>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::ZoomIn,
                    format!(
                        "{}× at {}",
                        number(p.scale),
                        format_duration(p.at_ms as f64)
                    ),
                )),
                Some(row(
                    Icon::Hourglass,
                    format!("Hold {}", format_duration(p.hold_ms as f64)),
                )),
                take_row(
                    ctx.inputs.takes.get(RECORDING_PORT).copied().flatten(),
                    p.at_ms,
                ),
            ]
        })
        .restyle(|props: RecordingProps, p: &ManualZoom, node_id| {
            let (at, hold) = (p.at_ms as f64, p.hold_ms as f64);
            focus_beats(
                props,
                node_id,
                "zoom",
                p.scale,
                &[
                    FocusSpot {
                        id: "a",
                        t: at,
                        x: p.x,
                        y: p.y,
                        t_depart: Some((at - FOCUS_LEAD_MS).max(0.0)),
                    },
                    FocusSpot {
                        id: "b",
                        t: at + hold,
                        x: p.x,
                        y: p.y,
                        t_depart: None,
                    },
                ],
            )
        })
}

fn pan() -> NodeDef {
    NodeDef::recording_modifier::<Pan>("camera.pan", "Pan", Category::Camera)
        .describe("Moves the camera from one spot to another.")
        .card(|ctx: &CardCtx<'_, Pan>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::Move,
                    format!(
                        "{} → {}",
                        format_duration(p.at_ms as f64),
                        format_duration((p.at_ms + p.duration_ms) as f64)
                    ),
                )),
                Some(row(Icon::ZoomIn, format!("{}×", number(p.scale)))),
                take_row(
                    ctx.inputs.takes.get(RECORDING_PORT).copied().flatten(),
                    p.at_ms,
                ),
            ]
        })
        .restyle(|props: RecordingProps, p: &Pan, node_id| {
            let at = p.at_ms as f64;
            focus_beats(
                props,
                node_id,
                "pan",
                p.scale,
                &[
                    FocusSpot {
                        id: "a",
                        t: at,
                        x: p.from_x,
                        y: p.from_y,
                        t_depart: None,
                    },
                    FocusSpot {
                        id: "b",
                        t: at + p.duration_ms as f64,
                        x: p.to_x,
                        y: p.to_y,
                        t_depart: None,
                    },
                ],
            )
        })
}

fn follow_cursor() -> NodeDef {
    NodeDef::style_modifier::<FollowCursor>(FOLLOW_CURSOR_TYPE, "Follow cursor", Category::Camera)
        .describe("Keeps the camera on the cursor, good for tall formats.")
        .card(|ctx: &CardCtx<'_, FollowCursor>| {
            let p = &ctx.params;
            vec![Some(row(
                Icon::LocateFixed,
                format!(
                    "{}× · re-aim every {}",
                    number(p.scale),
                    format_duration(p.every_ms as f64)
                ),
            ))]
        })
        .restyle(|mut props: RecordingProps, p: &FollowCursor, node_id| {
            let beats = follow_cursor_beats(&props.log, p.scale, p.every_ms as f64, node_id);
            props.extra_beats.extend(beats);
            props
        })
}

fn recording_log(clip: &ClipValue) -> Option<&opsis_types::actions::ActionLog> {
    clip.recording
        .as_ref()
        .and_then(|TaggedRecording::Recording(r)| r.log.as_ref())
}

fn idle_speedup() -> NodeDef {
    NodeDef::style_modifier::<IdleSpeedup>(
        "camera.idleSpeedup",
        "Skip quiet parts",
        Category::Camera,
    )
    .describe("Speeds through moments where nothing happens.")
    .card(|ctx: &CardCtx<'_, IdleSpeedup>| {
        let p = &ctx.params;
        vec![Some(row(
            Icon::FastForward,
            format!(
                "{}× when quiet over {}",
                number(p.rate),
                format_duration(p.threshold_ms as f64)
            ),
        ))]
    })
    .retime(|clip: ClipValue, p: &IdleSpeedup, at| {
        let Some(log) = recording_log(&clip) else {
            return clip;
        };
        let spans = idle_spans(log, p.threshold_ms as f64, p.pad_ms as f64);
        if spans.is_empty() {
            return clip;
        }
        let segments: Vec<(f64, f64, f64)> = spans
            .into_iter()
            .map(|(from, to)| (from, to, p.rate))
            .collect();
        remap_clip(clip, at.node_id, at.fps, &segments)
    })
}

fn speed_ramp() -> NodeDef {
    NodeDef::recording_modifier::<SpeedRamp>("camera.speedRamp", "Speed ramp", Category::Camera)
        .describe("Speeds up or slows down chosen parts.")
        .card(|ctx: &CardCtx<'_, SpeedRamp>| {
            let parts = ctx.params.segments.len();
            vec![Some(
                row(Icon::Gauge, plural(parts as f64, "part")).muted_if(parts == 0),
            )]
        })
        .retime(|clip: ClipValue, p: &SpeedRamp, at| {
            let mut segments: Vec<(f64, f64, f64)> = p
                .segments
                .iter()
                .map(|s| (s.from_ms as f64, s.to_ms as f64, s.rate))
                .collect();
            segments.sort_by(|a, b| a.0.total_cmp(&b.0));
            remap_clip(clip, at.node_id, at.fps, &segments)
        })
}

fn crop() -> NodeDef {
    NodeDef::clip_modifier::<Crop>("camera.crop", "Crop", Category::Camera)
        .describe("Cuts the recording down to an area.")
        .card(|ctx: &CardCtx<'_, Crop>| {
            let p = &ctx.params;
            let pct = |v: f64| percent_text(v, f64::INFINITY);
            vec![Some(row(
                Icon::Crop,
                format!(
                    "{} × {} from {}, {}",
                    pct(p.w),
                    pct(p.h),
                    pct(p.x),
                    pct(p.y)
                ),
            ))]
        })
        .compiles(|ctx: &Ctx<'_, Crop>| {
            let p = &ctx.params;
            with_clip(ctx, |clip| {
                let props = GroupProps {
                    crop: Some(Rect {
                        x: p.x,
                        y: p.y,
                        w: p.w,
                        h: p.h,
                    }),
                    ..GroupProps::default()
                };
                wrap_clip(clip, ctx.node_id, props, None)
            })
        })
}

fn device_frame() -> NodeDef {
    NodeDef::clip_modifier::<DeviceFrame>("camera.deviceFrame", "Device frame", Category::Camera)
        .describe("Puts the video inside a laptop or phone.")
        .card(|ctx: &CardCtx<'_, DeviceFrame>| {
            let p = &ctx.params;
            vec![
                Some(row(p.device.icon(), p.device.label())),
                (p.device == Device::Laptop && p.entrance != DeviceEntrance::None)
                    .then(|| row(Icon::Sparkles, p.entrance.label())),
            ]
        })
        .compiles(|ctx: &Ctx<'_, DeviceFrame>| {
            let p = &ctx.params;
            with_clip(ctx, |clip| {
                let mut props = Map::new();
                props.insert("mode".into(), Value::from("dark"));
                if p.device == Device::Laptop {
                    props.insert(
                        "entrance".into(),
                        serde_json::to_value(p.entrance).expect("entrance serializes"),
                    );
                }
                let screened = patch_recordings(clip, &|r| RecordingProps {
                    screen: Some(true),
                    ..r
                });
                let wrapper = ElementRef {
                    component: p.device.component().to_owned(),
                    props,
                };
                wrap_clip(
                    &screened,
                    ctx.node_id,
                    GroupProps {
                        wrapper: Some(wrapper),
                        ..GroupProps::default()
                    },
                    Some(clip.duration_in_frames),
                )
            })
        })
}

fn window_frame() -> NodeDef {
    NodeDef::style_modifier::<WindowFrame>("camera.windowFrame", "Window frame", Category::Camera)
        .describe("Draws a browser window's title bar around the recording.")
        .card(|ctx: &CardCtx<'_, WindowFrame>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::AppWindow,
                    format!(
                        "{} · {}",
                        window_style_label(p.window_style),
                        humanize(frame_theme_name(p.frame_theme))
                    ),
                )),
                Some(
                    row(
                        Icon::Link,
                        if p.address_bar {
                            "Address bar"
                        } else {
                            "No address bar"
                        },
                    )
                    .muted_if(!p.address_bar),
                ),
            ]
        })
        .restyle(|props: RecordingProps, p: &WindowFrame, _| {
            with_window_frame(props, p.window_style, p.frame_theme, p.address_bar)
        })
        .host_defaults(|platform: Platform| {
            let mut values = Values::new();
            values.insert(
                "windowStyle".into(),
                serde_json::to_value(host_window_style(platform)).expect("style serializes"),
            );
            values
        })
}

fn backdrop() -> NodeDef {
    NodeDef::style_modifier::<BackdropStyle>("camera.backdrop", "Backdrop", Category::Camera)
        .describe("The background behind the floating window: a studio image, colour or gradient.")
        .card(|ctx: &CardCtx<'_, BackdropStyle>| vec![Some(ctx.params.card_row())])
        .restyle(
            |props: RecordingProps, p: &BackdropStyle, _| match p.chosen() {
                Some(chosen) => RecordingProps {
                    backdrop: chosen,
                    backdrop_chosen: Some(true),
                    ..props
                },
                None => props,
            },
        )
}

fn motion_blur() -> NodeDef {
    NodeDef::style_modifier::<MotionBlur>("camera.motionBlur", "Motion blur", Category::Camera)
        .describe("Adds blur to fast camera moves.")
        .card(|ctx: &CardCtx<'_, MotionBlur>| {
            vec![Some(toggle_row(Icon::Wind, ctx.params.enabled))]
        })
        .restyle(|props: RecordingProps, p: &MotionBlur, _| RecordingProps {
            motion_blur: p.enabled,
            ..props
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        auto_zoom(),
        apply_style(),
        manual_zoom(),
        pan(),
        follow_cursor(),
        idle_speedup(),
        speed_ramp(),
        crop(),
        device_frame(),
        window_frame(),
        backdrop(),
        motion_blur(),
    ]
}
