use indexmap::IndexMap;
use opsis_macros::settings;
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::recording::{FootageChain, RecordingProps};
use opsis_types::timeline::TimelineItem;
use opsis_types::values::{ClipValue, CompileValue, StyleEntry};
use opsis_types::workflow::PortType;
use serde_json::Value;

use crate::clip::{IMAGE, VIDEO, map_items};
use crate::errors::SettingsError;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, GRADE_PORT, InputTake, NodeDef, Phase, RECORDING_PORT,
    RowExt, missing, outputs, port, row,
};
use crate::nodes::grade::{grade_changes_nothing, grade_step_title};
use crate::nodes::image::{RedactImage, RedactStyle, hidden_area};
use crate::params::Params;
use crate::text::{format_duration, plural};

pub const REDACT_TAKE_TYPE: &str = "effect.redactTake";
pub const GRADE_FOOTAGE_TYPE: &str = "effect.gradeFootage";

#[settings]
pub struct Area {
    #[setting(range(min = 0, max = 1))]
    pub x: f64,
    #[setting(range(min = 0, max = 1))]
    pub y: f64,
    #[setting(range(min = 0, max = 1))]
    pub width: f64,
    #[setting(range(min = 0, max = 1))]
    pub height: f64,
}

#[settings(check)]
pub struct RedactTake {
    #[setting(
        "Areas to hide",
        default = hidden_area(),
        length(min = 1, max = 8),
        widget = "boxes",
        time_key = "frameMs",
        help = "Drag on the picture to mark each part to hide. Move the slider to draw on a later moment of the take. The areas stay on the page while the camera zooms and pans."
    )]
    pub areas: Vec<Area>,
    #[setting("Frame to draw on", range(min = 0, max = 3600000), unit = "ms", hidden)]
    pub frame_ms: f64,
    #[setting("Hide with")]
    pub style: RedactStyle,
    #[setting(
        "Pixel size",
        default = 16,
        range(min = 2, max = 256),
        unit = "px",
        show_if = { "style": ["pixelate"] }
    )]
    pub block_size: u32,
    #[setting(
        "Blur",
        default = 16.0,
        range(min = 1, max = 100),
        step = 0.5,
        unit = "px",
        show_if = { "style": ["blur"] }
    )]
    pub blur: f64,
    #[setting(
        "Colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "style": ["solid"] }
    )]
    pub color: String,
    #[setting(
        "Corner radius",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "style": ["solid"] },
        advanced
    )]
    pub radius: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "From",
        range(min = 0, max = 3600000),
        unit = "ms",
        zero_label = "Start of the take",
        advanced
    )]
    pub from_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Until",
        range(min = 0, max = 3600000),
        unit = "ms",
        zero_label = "End of the take",
        advanced
    )]
    pub until_ms: Option<u32>,
}

impl Params for RedactTake {
    fn check(&self) -> Result<(), SettingsError> {
        match (self.from_ms, self.until_ms) {
            (Some(from), Some(until)) if until <= from => {
                Err(SettingsError::rule("Until must come after From"))
            }
            _ => Ok(()),
        }
    }
}

impl RedactTake {
    pub fn picture(&self) -> RedactImage {
        RedactImage {
            areas: self.areas.clone(),
            style: self.style,
            block_size: self.block_size,
            blur: self.blur,
            color: self.color.clone(),
            radius: self.radius,
        }
    }
}

fn times_text(from_ms: Option<u32>, until_ms: Option<u32>) -> Option<String> {
    match (from_ms, until_ms) {
        (None, None) => None,
        (from, None) => Some(format!(
            "From {}",
            format_duration(f64::from(from.unwrap_or(0)))
        )),
        (None, Some(until)) => Some(format!("Until {}", format_duration(f64::from(until)))),
        (Some(from), Some(until)) => Some(format!(
            "{} → {}",
            format_duration(f64::from(from)),
            format_duration(f64::from(until))
        )),
    }
}

fn after_take_row(take: Option<InputTake>, from_ms: Option<u32>) -> Option<CardRow> {
    let (take, from) = (take?, from_ms?);
    (f64::from(from) >= take.duration_ms).then(|| {
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

fn params_of<P: serde::Serialize>(params: &P) -> serde_json::Map<String, Value> {
    match serde_json::to_value(params) {
        Ok(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    }
}

fn redact_take() -> NodeDef {
    NodeDef::recording_modifier::<RedactTake>(REDACT_TAKE_TYPE, "Redact take", Category::Effects)
        .describe("Hides parts of the browser window, such as names, email addresses and keys, behind large pixels, a blur or a solid colour. The areas stay on the page while the camera moves.")
        .card(|ctx: &CardCtx<'_, RedactTake>| {
            let p = &ctx.params;
            let take = ctx.inputs.takes.get(RECORDING_PORT).copied().flatten();
            vec![
                Some(row(
                    Icon::EyeOff,
                    format!("{} {}", plural(numbers::float(p.areas.len()), "area"), p.style.word()),
                )),
                times_text(p.from_ms, p.until_ms).map(|t| row(Icon::Clock, t)),
                after_take_row(take, p.from_ms),
            ]
        })
        .restyle(|props: RecordingProps, p: &RedactTake, node_id| {
            let mut footage = props.footage.clone();
            footage.redact.push(StyleEntry {
                node_id: node_id.to_owned(),
                kind: REDACT_TAKE_TYPE.to_owned(),
                params: params_of(p),
            });
            RecordingProps { footage, ..props }
        })
}

#[settings]
pub struct GradeFootage {}

const GRADED_COMPONENTS: [&str; 2] = [VIDEO, IMAGE];

fn graded(footage: &FootageChain, grade: &[StyleEntry]) -> FootageChain {
    FootageChain {
        grade: footage.grade.iter().chain(grade).cloned().collect(),
        redact: footage.redact.clone(),
    }
}

fn element_footage(props: &serde_json::Map<String, Value>) -> FootageChain {
    props
        .get("footage")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn with_grade(clip: &ClipValue, grade: &[StyleEntry]) -> ClipValue {
    ClipValue {
        tracks: map_items(&clip.tracks, &|item| match item {
            TimelineItem::Recording { base, props } => {
                let footage = graded(&props.footage, grade);
                TimelineItem::Recording {
                    base,
                    props: Box::new(RecordingProps { footage, ..*props }),
                }
            }
            TimelineItem::Element {
                base,
                component,
                mut props,
            } if GRADED_COMPONENTS.contains(&component.as_str()) => {
                let footage = graded(&element_footage(&props), grade);
                props.insert(
                    "footage".into(),
                    serde_json::to_value(footage).unwrap_or(Value::Null),
                );
                TimelineItem::Element {
                    base,
                    component,
                    props,
                }
            }
            other => other,
        }),
        ..clip.clone()
    }
}

fn grade_footage() -> NodeDef {
    NodeDef::of::<GradeFootage>(GRADE_FOOTAGE_TYPE, "Grade footage", Category::Effects, Phase::Compile)
        .describe("Applies a colour grade to the footage in a clip: the page in each recording, and each video and image. The window and backdrop keep their colours; to grade the whole picture, use Apply grade on an exported file.")
        .inputs(vec![
            (CLIP_PORT, port(PortType::Clip, "Clip")),
            (GRADE_PORT, port(PortType::Grade, "Grade")),
        ])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, GradeFootage>| {
            let entries = ctx.inputs.grade.get(GRADE_PORT);
            let titles: Vec<String> = entries.into_iter().flatten().map(|e| grade_step_title(&e.kind)).collect();
            vec![
                Some(if titles.is_empty() {
                    missing(Icon::Palette, "Connect a grade")
                } else {
                    row(Icon::Palette, format!("{}: {}", plural(numbers::float(titles.len()), "step"), titles.join(", ")))
                }),
                entries.is_some_and(|e| !e.is_empty() && grade_changes_nothing(e)).then(|| missing(Icon::TriangleAlert, "The grade changes nothing")),
            ]
        })
        .compile(|ctx: &Ctx<'_, GradeFootage>| {
            let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
                return Ok(Some(IndexMap::default()));
            };
            let grade = ctx
                .input(GRADE_PORT)
                .and_then(CompileValue::as_grade)
                .map_or_default(|g| g.entries.clone());
            Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(with_grade(clip, &grade)))])))
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        redact_take().look(NodeLook::of("mynaui:incognito")),
        grade_footage().look(NodeLook::of("lucide:contrast")),
    ]
}
