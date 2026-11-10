use opsis_macros::{choice, settings};
use opsis_types::cards::{Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::values::{
    BrowserAction, BrowserSource, CompileValue, ImageValue, RecordingValue, TakeSource,
};
use opsis_types::workflow::{DEFAULT_TAKE_MS, DEFAULT_VIEWPORT, PortType};

use crate::card_rows::element_row;
use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, NodeDef, Phase, RECORDING_PORT, RowExt, SESSION_PORT, outputs, port,
    row,
};
use crate::params::Params;
use crate::text::{format_duration, plural};

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
    const fn row(self) -> &'static str {
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
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub establish_ms: u32,
    #[setting(
        "Keep recording after the last action",
        default = 2500,
        range(min = 0, max = 600_000),
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

#[settings(check, eq)]
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
            rec.filter(|r| r.file().is_some())
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
                log: None,
                duration_ms: DEFAULT_TAKE_MS,
                node_id: ctx.node_id.to_owned(),
                edits: Vec::new(),
                source: TakeSource::Browser(BrowserSource {
                    video: None,
                    viewport: DEFAULT_VIEWPORT,
                }),
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
        "Plays the connected actions, then takes a sharp screenshot of the page or one element. Frame image frames it; Use as clip shows it in the video.",
    )
    .executes()
    .uses_project_size()
    .inputs(vec![(SESSION_PORT, session_port())])
    .outputs(vec![("image", port(PortType::Image, "Image"))])
    .card(|ctx: &CardCtx<'_, Still>| {
        let p = &ctx.params;
        let image = image_of(ctx.result);
        vec![
            Some(match p.region {
                Region::Element => element_row(Icon::Scan, &p.target, p.target_role.as_deref()),
                region => row(Icon::Scan, region.label()),
            }),
            image
                .and_then(|i| Some((i.width.filter(|w| *w != 0)?, i.height.filter(|h| *h != 0)?)))
                .map(|(w, h)| row(Icon::Image, format!("{w} × {h}"))),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Still>| {
        let image = image_of(ctx.result).cloned().unwrap_or(ImageValue {
            src: None,
            width: None,
            height: None,
            scale: None,
            body: None,
        });
        outputs([("image", CompileValue::Image(image))])
    })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        record().look(NodeLook::of("lucide:circle-dot").morphs("lucide:circle-stop")),
        still().look(NodeLook::of("lucide:camera")),
    ]
}
