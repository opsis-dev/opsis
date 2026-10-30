use opsis_macros::{choice, settings};
use opsis_types::actions::Bounds;
use opsis_types::cards::{Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::recording::WindowStyle;
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit, VisualEnvelope};
use opsis_types::values::{BrowserAction, CompileValue, ImageValue, RecordingValue};
use opsis_types::workflow::{DEFAULT_TAKE_MS, DEFAULT_VIEWPORT, PortType};
use serde::{Deserialize, Serialize};

use crate::card_rows::element_row;
use crate::clip::{ItemExtra, STILL, item_clip, placeholder_clip};
use crate::errors::SettingsError;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Phase, RECORDING_PORT, RowExt, SESSION_PORT,
    outputs, port, row,
};
use crate::nodes::camera::FrameChoice;
use crate::params::Params;
use crate::params::to_values;
use crate::text::{format_duration, plural};
use crate::time::ms_to_frames;

pub const STILL_TYPE: &str = "capture.still";

pub const RECORD_TYPE: &str = "capture.record";

#[choice]
pub enum Cursor {
    #[label("Smooth")]
    Synthetic,
    #[label("As on screen")]
    Baked,
    #[label("None")]
    None,
}

#[choice]
pub enum Timing {
    #[default]
    #[label("Real time")]
    Real,
    #[label("Smooth (slower)")]
    Smooth,
}

#[choice]
pub enum FramePicture {
    #[default]
    #[label("Standard")]
    Standard,
    #[label("Lossless")]
    Lossless,
    #[label("Full detail")]
    FullDetail,
}

impl Cursor {
    fn row(self) -> &'static str {
        match self {
            Self::Synthetic => "Smooth cursor",
            Self::Baked => "Cursor as on screen",
            Self::None => "No cursor",
        }
    }
}

#[settings]
pub struct Record {
    #[setting("Frame rate", default = 30, "enum" = [25, 30, 60], unit = "fps")]
    pub fps: u32,
    #[setting(
        "Timing",
        help = "Smooth gives every frame its own picture, however busy the computer is. It takes longer to record than the take lasts."
    )]
    pub timing: Timing,
    #[setting(
        "Frame pictures",
        help = "Standard matches real-time takes. Lossless keeps every pixel exact and takes about 70% longer to record. Full detail uses every pixel of a high-density screen: sharper, with twice the width and height, but about four times as long to record.",
        show_if = { "timing": ["smooth"] },
        advanced
    )]
    pub pictures: FramePicture,
    #[setting(
        "Recording quality",
        default = 90,
        range(min = 10, max = 100),
        step = 5,
        unit = "%",
        advanced
    )]
    pub quality: u32,
    #[setting(
        "Pause before the first action",
        default = 700,
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms"
    )]
    pub establish_ms: u32,
    #[setting(
        "Keep recording after the last action",
        default = 2500,
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms",
        help = "Gives the camera time to settle at the end."
    )]
    pub end_tail_ms: u32,
    #[setting(
        "Cursor",
        default = Cursor::Synthetic
    )]
    pub cursor: Cursor,
    #[setting("Quick mode: show action labels on the video", advanced)]
    pub show_actions: bool,
    #[setting(
        "Action labels shown for",
        default = 600,
        range(min = 100, max = 10000),
        step = 50,
        unit = "ms",
        show_if = { "showActions": ["true"] },
        advanced
    )]
    pub action_label_ms: u32,
    #[setting("Cursor colour", default = "#111418", widget = "color", check = colour, show_if = { "cursor": ["baked"] }, advanced)]
    pub cursor_fill: String,
    #[setting("Cursor outline", default = "#ffffff", widget = "color", check = colour, show_if = { "cursor": ["baked"] }, advanced)]
    pub cursor_outline: String,
    #[setting("Cursor size", default = 1.0, range(min = 0.5, max = 3), step = 0.1, unit = "×", show_if = { "cursor": ["baked"] }, advanced)]
    pub cursor_scale: f64,
    #[setting("Click ring", default = true, show_if = { "cursor": ["baked"] }, advanced)]
    pub click_ring: bool,
    #[setting(
        "Click ring colour",
        default = "rgba(15, 23, 42, 0.55)",
        widget = "color",
        check = colour,
        show_if = { "cursor": ["baked"], "clickRing": ["true"] },
        advanced
    )]
    pub ring_color: String,
    #[setting(
        "Don't record at the same time as",
        help = "Takes that share this name run one after another, e.g. when they change the same account.",
        advanced
    )]
    pub mutates: String,
}

#[choice]
pub enum Region {
    #[label("Visible part")]
    Viewport,
    #[label("Whole page")]
    FullPage,
    #[label("One element")]
    Element,
}

#[choice(export)]
pub enum StillLook {
    #[label("Window")]
    Window,
    #[label("Cut-out")]
    Cutout,
    #[label("Full bleed")]
    Bleed,
}

#[choice(export)]
pub enum StillAlign {
    #[default]
    #[label("The top")]
    Top,
    #[label("The middle")]
    Middle,
}

#[settings(check)]
pub struct Still {
    #[setting(
        "Capture",
        default = Region::Viewport
    )]
    pub region: Region,
    #[setting("Element", widget = "target", show_if = { "region": ["element"] })]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "Space around",
        default = 24,
        range(min = 0, max = 400),
        unit = "px",
        show_if = { "region": ["element"] },
        advanced
    )]
    pub padding: u32,
    #[setting(
        "Style",
        default = StillLook::Window
    )]
    pub look: StillLook,
    #[setting("Backdrop", default = "glaze", widget = "backdrop")]
    pub backdrop: String,
    #[setting(
        "Length",
        default = 3000,
        range(min = 100, max = 600000),
        step = 50,
        unit = "ms"
    )]
    pub duration_ms: u32,
    #[setting(
        "Keyframes",
        help = "Change opacity, position and size over time. Edit them here or on the timeline.",
        widget = "envelope",
        advanced
    )]
    pub envelope: VisualEnvelope,
    #[setting(
        "Window size",
        default = 86,
        range(min = 30, max = 100),
        unit = "%",
        help = "How much of the frame the window fills.",
        show_if = { "look": ["window"] },
        advanced
    )]
    pub window_fit: u32,
    #[setting(
        "Cut-out size",
        default = 80,
        range(min = 30, max = 100),
        unit = "%",
        help = "How much of the frame the element fills.",
        show_if = { "look": ["cutout"] },
        advanced
    )]
    pub cutout_fit: u32,
    #[setting("Shadow", default = 45, range(min = 0, max = 100), unit = "%", show_if = { "look": ["cutout"] }, advanced)]
    pub shadow: u32,
    #[setting(
        "Title bar",
        show_if = { "look": ["window"] },
        advanced
    )]
    pub frame: FrameChoice,
    #[setting(
        "Push in",
        default = 2.0,
        range(min = 0, max = 30),
        step = 0.5,
        unit = "%",
        help = "How much closer the picture slowly moves.",
        advanced
    )]
    pub push_in: f64,
    #[setting(
        "Push-in time",
        default = 4000,
        range(min = 100, max = 600000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub push_in_ms: u32,
    #[setting(
        "Keep in view",
        help = "Which part of a tall picture shows.",
        show_if = { "look": ["bleed"] },
        advanced
    )]
    pub align: StillAlign,
}

impl Still {
    pub fn fit(&self) -> f64 {
        let percent = if self.look == StillLook::Cutout {
            self.cutout_fit
        } else {
            self.window_fit
        };
        f64::from(percent) / 100.0
    }
}

impl Params for Still {
    fn check(&self) -> Result<(), SettingsError> {
        if self.region == Region::Element && self.target.trim().is_empty() {
            return Err(SettingsError::rule("Pick an element"));
        }
        Ok(())
    }
}

fn recording_of(ctx_result: Option<&opsis_types::values::NodeResult>) -> Option<&RecordingValue> {
    ctx_result
        .and_then(|r| r.output(RECORDING_PORT))
        .and_then(CompileValue::as_recording)
}

fn image_of(result: Option<&opsis_types::values::NodeResult>) -> Option<&ImageValue> {
    result
        .and_then(|r| r.output("image"))
        .and_then(CompileValue::as_image)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct StillShot {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Bounds>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StillProps {
    pub src: String,
    pub look: StillLook,
    pub backdrop: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<StillShot>,
    pub fit: f64,
    pub shadow: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<WindowStyle>,
    pub push_in: f64,
    pub push_in_frames: u32,
    pub align: StillAlign,
}

fn shot_of(image: &ImageValue) -> Option<StillShot> {
    let (width, height) = (
        image.width.filter(|w| *w != 0)?,
        image.height.filter(|h| *h != 0)?,
    );
    Some(StillShot {
        width,
        height,
        scale: image.scale.unwrap_or(1.0),
        body: image.body,
    })
}

fn session_port() -> crate::node::PortSpec {
    port(PortType::BrowserSession, "Browser")
}

fn record() -> NodeDef {
    NodeDef::of::<Record>(
        RECORD_TYPE,
        "Record take",
        Category::Capture,
        Phase::Capture,
    )
    .describe("Plays all connected browser actions in one go and records the screen.")
    .executes()
    .inputs(vec![(SESSION_PORT, session_port())])
    .outputs(vec![(
        RECORDING_PORT,
        port(PortType::Recording, "Recording"),
    )])
    .mutates(|p: &Record| (!p.mutates.is_empty()).then(|| p.mutates.clone()))
    .card(|ctx: &CardCtx<'_, Record>| {
        let p = &ctx.params;
        let rec = recording_of(ctx.result);
        let steps = ctx.inputs.steps.get(SESSION_PORT);
        vec![
            Some(row(
                Icon::Video,
                match p.timing {
                    Timing::Real => format!("{} fps", p.fps),
                    Timing::Smooth => format!("{} fps · Smooth", p.fps),
                },
            )),
            Some(row(Icon::MousePointer, p.cursor.row())),
            steps
                .filter(|s| !s.contains(&BrowserAction::StartRecording))
                .map(|_| row(Icon::Clapperboard, "Starts with the first action").tone(Tone::Muted)),
            rec.filter(|r| r.src.is_some())
                .and_then(|r| r.log.as_ref().map(|log| (r, log)))
                .map(|(r, log)| {
                    row(
                        Icon::Clock,
                        format!(
                            "{} · {}",
                            format_duration(r.duration_ms),
                            plural(numbers::float(log.actions.len()), "action")
                        ),
                    )
                }),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Record>| {
        let recording = recording_of(ctx.result)
            .cloned()
            .unwrap_or_else(|| RecordingValue {
                src: None,
                log: None,
                duration_ms: DEFAULT_TAKE_MS,
                viewport: DEFAULT_VIEWPORT,
                node_id: ctx.node_id.to_owned(),
                style: Vec::new(),
            });
        outputs([(RECORDING_PORT, CompileValue::Recording(recording))])
    })
}

fn still() -> NodeDef {
    NodeDef::of::<Still>(
        STILL_TYPE,
        "Screenshot",
        Category::Capture,
        Phase::Capture,
    )
    .describe(
        "Plays the connected actions, then takes a sharp screenshot of the page or one element.",
    )
    .executes()
    .uses_project_size()
    .inputs(vec![(SESSION_PORT, session_port())])
    .outputs(vec![
        ("image", port(PortType::Image, "Image")),
        (CLIP_PORT, port(PortType::Clip, "Clip")),
    ])
    .card(|ctx: &CardCtx<'_, Still>| {
        let p = &ctx.params;
        let shot = image_of(ctx.result).and_then(shot_of);
        vec![
            Some(match p.region {
                Region::Element => element_row(Icon::Scan, &p.target, p.target_role.as_deref()),
                region => row(Icon::Scan, region.label()),
            }),
            shot.map(|s| row(Icon::Image, format!("{} × {}", s.width, s.height))),
            Some(row(
                Icon::Clock,
                format_duration(f64::from(ctx.params.duration_ms)),
            )),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Still>| {
        let p = &ctx.params;
        let image = image_of(ctx.result).cloned().unwrap_or(ImageValue {
            src: None,
            width: None,
            height: None,
            scale: None,
            body: None,
        });
        let frames = ms_to_frames(f64::from(p.duration_ms), ctx.settings.fps);
        let extra = ItemExtra {
            handles: Some(EditHandles {
                trim_end: Some(ParamRef::new(ctx.node_id, "durationMs", ParamUnit::Ms)),
                ..EditHandles::default()
            }),
            envelope: Some(p.envelope.clone()),
            ..ItemExtra::default()
        };
        let clip = match &image.src {
            Some(src) => {
                let props = to_values(&StillProps {
                    src: src.clone(),
                    look: p.look,
                    backdrop: p.backdrop.clone(),
                    shot: shot_of(&image),
                    fit: p.fit(),
                    shadow: f64::from(p.shadow) / 100.0,
                    frame: p.frame.style(),
                    push_in: 1.0 + p.push_in / 100.0,
                    push_in_frames: ms_to_frames(f64::from(p.push_in_ms), ctx.settings.fps),
                    align: p.align,
                });
                item_clip(ctx.node_id, STILL, props, frames, extra)
            }
            None => placeholder_clip(
                ctx.node_id,
                "Screenshot not taken yet",
                "Run to take it",
                frames,
                extra,
            ),
        };
        outputs([
            ("image", CompileValue::Image(image)),
            (CLIP_PORT, CompileValue::Clip(clip)),
        ])
    })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        record().look(NodeLook::of("lucide:circle-dot").morphs("lucide:circle-stop")),
        still().look(NodeLook::of("lucide:camera")),
    ]
}
