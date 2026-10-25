use opsis_types::actions::Bounds;
use opsis_types::cards::{Icon, Tone};
use opsis_types::timeline::{EditHandles, ParamRef, ParamUnit, VisualEnvelope};
use opsis_types::values::{CompileValue, ImageValue, RecordingValue};
use opsis_types::workflow::{DEFAULT_TAKE_MS, DEFAULT_VIEWPORT, PortType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use smart_default::SmartDefault;

use crate::card::element_row;
use crate::clip::{ItemExtra, item_clip, placeholder_clip};
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Phase, RECORDING_PORT, RowExt, SESSION_PORT,
    outputs, port, row,
};
use crate::params::Params;
use crate::text::{format_duration, plural};
use crate::time::ms_to_frames;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Cursor {
    Synthetic,
    Baked,
    None,
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Record {
    #[default = 30]
    #[schemars(description = "Frame rate", extend("enum" = [25, 30, 60], "x-unit" = "fps"))]
    pub fps: i64,
    #[default = 90]
    #[schemars(
        description = "Recording quality",
        range(min = 10, max = 100),
        extend("x-step" = 5, "x-unit" = "%", "x-advanced" = true)
    )]
    pub quality: i64,
    #[default = 700]
    #[schemars(description = "Pause before the first action", range(min = 0, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub establish_ms: i64,
    #[default = 2500]
    #[schemars(
        description = "Keep recording after the last action",
        range(min = 0, max = 600000),
        extend("x-step" = 50, "x-unit" = "ms", "x-help" = "Gives the camera time to settle at the end.")
    )]
    pub end_tail_ms: i64,
    #[default(Cursor::Synthetic)]
    #[schemars(description = "Cursor", extend("x-labels" = { "synthetic": "Smooth", "baked": "As on screen", "none": "None" }))]
    pub cursor: Cursor,
    #[schemars(description = "Quick mode: show action labels on the video")]
    pub show_actions: bool,
    #[schemars(
        description = "Don't record at the same time as",
        extend(
            "x-help" = "Takes that share this name run one after another, e.g. when they change the same account.",
            "x-advanced" = true
        )
    )]
    pub mutates: String,
}

impl Params for Record {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    Viewport,
    FullPage,
    Element,
}

impl Region {
    fn label(self) -> &'static str {
        match self {
            Self::Viewport => "Visible part",
            Self::FullPage => "Whole page",
            Self::Element => "One element",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum StillLook {
    Window,
    Cutout,
    Bleed,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Still {
    #[default(Region::Viewport)]
    #[schemars(
        description = "Capture",
        extend("x-labels" = { "viewport": "Visible part", "full-page": "Whole page", "element": "One element" })
    )]
    pub region: Region,
    #[schemars(description = "Element", extend("x-widget" = "target", "x-show-if" = { "region": ["element"] }))]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Role of the picked element", extend("x-hidden" = true))]
    pub target_role: Option<String>,
    #[default = 24]
    #[schemars(
        description = "Space around",
        range(min = 0, max = 400),
        extend("x-unit" = "px", "x-show-if" = { "region": ["element"] })
    )]
    pub padding: i64,
    #[default(StillLook::Window)]
    #[schemars(description = "Style", extend("x-labels" = { "window": "Window", "cutout": "Cut-out", "bleed": "Full bleed" }))]
    pub look: StillLook,
    #[default = "glaze"]
    #[schemars(description = "Backdrop", extend("x-widget" = "backdrop"))]
    pub backdrop: String,
    #[default = 3000]
    #[schemars(description = "Length", range(min = 100, max = 600000), extend("x-step" = 50, "x-unit" = "ms"))]
    pub duration_ms: i64,
    #[schemars(
        description = "Keyframes",
        extend(
            "x-help" = "Change opacity, position and size over time. Edit them here or on the timeline.",
            "x-widget" = "envelope"
        )
    )]
    pub envelope: VisualEnvelope,
}

impl Params for Still {
    fn check(&self) -> Result<(), String> {
        if self.region == Region::Element && self.target.trim().is_empty() {
            return Err("Pick an element".to_owned());
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

struct Shot {
    width: f64,
    height: f64,
    scale: f64,
    body: Option<Bounds>,
}

fn shot_of(image: &ImageValue) -> Option<Shot> {
    let (width, height) = (
        image.width.filter(|w| *w != 0.0)?,
        image.height.filter(|h| *h != 0.0)?,
    );
    Some(Shot {
        width,
        height,
        scale: image.scale.unwrap_or(1.0),
        body: image.body,
    })
}

fn shot_value(shot: &Shot) -> Value {
    let mut value = json!({ "width": shot.width, "height": shot.height, "scale": shot.scale });
    if let Some(body) = &shot.body {
        value["body"] = serde_json::to_value(body).expect("bounds serialize");
    }
    value
}

fn session_port() -> crate::node::PortSpec {
    port(PortType::BrowserSession, "Browser")
}

const START_RECORDING: &str = "browser.startRecording";

fn record() -> NodeDef {
    NodeDef::of::<Record>(
        "capture.record",
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
            Some(row(Icon::Video, format!("{} fps", p.fps))),
            Some(row(Icon::MousePointer, p.cursor.row())),
            steps
                .filter(|s| !s.iter().any(|t| t == START_RECORDING))
                .map(|_| row(Icon::Clapperboard, "Starts with the first action").tone(Tone::Muted)),
            rec.filter(|r| r.src.is_some())
                .and_then(|r| r.log.as_ref().map(|log| (r, log)))
                .map(|(r, log)| {
                    row(
                        Icon::Clock,
                        format!(
                            "{} · {}",
                            format_duration(r.duration_ms),
                            plural(log.actions.len() as f64, "action")
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
                proxy_src: None,
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
        "capture.still",
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
            shot.map(|s| {
                row(
                    Icon::Image,
                    format!("{} × {}", s.width.round(), s.height.round()),
                )
            }),
            Some(row(
                Icon::Clock,
                format_duration(ctx.params.duration_ms as f64),
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
        let frames = ms_to_frames(p.duration_ms as f64, ctx.settings.fps);
        let extra = ItemExtra {
            handles: Some(EditHandles {
                trim_end: Some(ParamRef {
                    node_id: ctx.node_id.to_owned(),
                    param: "durationMs".into(),
                    unit: ParamUnit::Ms,
                    scale: None,
                    index: None,
                    linked: None,
                }),
                ..EditHandles::default()
            }),
            envelope: Some(p.envelope.clone()),
            ..ItemExtra::default()
        };
        let clip = match &image.src {
            Some(src) => {
                let mut props = Map::new();
                props.insert("src".into(), Value::from(src.clone()));
                props.insert(
                    "look".into(),
                    serde_json::to_value(p.look).expect("look serializes"),
                );
                props.insert("backdrop".into(), Value::from(p.backdrop.clone()));
                if let Some(shot) = shot_of(&image) {
                    props.insert("shot".into(), shot_value(&shot));
                }
                item_clip(ctx.node_id, "core/still", props, frames, extra)
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
    vec![record(), still()]
}
