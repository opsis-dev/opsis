use opsis_types::actions::Rect;
use opsis_types::numbers::exact_eq;
use opsis_types::recording::RecordingProps;
use opsis_types::timeline::{
    Design, Ease, GroupProps, SequencerProps, TimelineItem, TimelineSpec, TimelineTrack,
    TransitionEase, TransitionRef, VisualEnvelope,
};
use serde_json::{Map, Value};

use crate::clock::Clock;
use crate::color::Color;
use crate::interpolate::{Interpolation, interpolate};
use crate::keyed::{EPSILON, clamp01, ease, envelope_at, percent, remap_frame};
use crate::node::{Node, div};
use crate::paint::Transform;
use crate::style::{Align, Display, FlexDirection, Overflow, Position, Style};
use crate::values::{Length, Num};

pub const STAGE_COLOR: &str = "#000";
pub const ERROR_COLOR: &str = "#f87171";
pub const MAIN_FONT_ROLE: &str = "df-main";
const MIN_CROP: f64 = 0.05;
const MISSING_FONT_SIZE: f64 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct At {
    pub clock: Clock,
    pub stage: Stage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Entering,
    Exiting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Presented<'a> {
    pub transition: &'a TransitionRef,
    pub side: Side,
    pub progress: f64,
    pub cut: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{component}: {message}")]
pub struct DrawError {
    pub component: String,
    pub message: String,
}

impl DrawError {
    pub fn new(component: impl Into<String>, message: impl std::fmt::Display) -> Self {
        Self {
            component: component.into(),
            message: format!("{message}"),
        }
    }
}

pub type Drawn = Result<Option<Node>, DrawError>;
pub type Nodes = Result<Vec<Node>, DrawError>;
pub type ChildAt<'a> = &'a dyn Fn(Stage) -> Result<Node, DrawError>;

pub trait SceneTracks {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn duration_in_frames(&self, scene: usize) -> u32;
    fn background(&self, scene: usize) -> Option<&str>;
    fn from(&self, scene: usize) -> u32;
    fn draw(&self, scene: usize, track: usize, at: &At) -> Nodes;
    fn track_count(&self, scene: usize) -> usize;
}

pub trait Library {
    fn element(&self, component: &str, props: &Map<String, Value>, at: &At) -> Drawn;
    fn wrapper(
        &self,
        component: &str,
        props: &Map<String, Value>,
        child: ChildAt<'_>,
        at: &At,
    ) -> Drawn;
    fn presentation(&self, presented: &Presented<'_>, child: Vec<Node>, at: &At) -> Drawn;
    fn scenes(&self, sequencer: &SequencerProps, scenes: &dyn SceneTracks, at: &At) -> Drawn;
    fn recording(&self, props: &RecordingProps, at: &At) -> Result<Node, DrawError>;
    fn color(&self, css: &str) -> Color;
}

pub fn absolute_fill() -> Style {
    Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
}

pub fn fill(children: Vec<Node>) -> Node {
    div(absolute_fill()).children(children).into()
}

fn missing(library: &dyn Library, name: &str) -> Node {
    div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .color(library.color(ERROR_COLOR))
        .font_family(vec![MAIN_FONT_ROLE.to_owned()])
        .font_size(MISSING_FONT_SIZE))
    .child(format!("Unknown element: {name}"))
    .into()
}

pub fn draw_film(
    library: &dyn Library,
    spec: &TimelineSpec,
    frame: f64,
) -> Result<Node, DrawError> {
    let at = At {
        clock: Clock::new(frame, spec.fps, f64::from(spec.duration_in_frames.max(1))),
        stage: Stage {
            width: f64::from(spec.width),
            height: f64::from(spec.height),
        },
    };
    let background = library.color(spec.background.as_deref().unwrap_or(STAGE_COLOR));
    Ok(div(absolute_fill()
        .background_color(background)
        .overflow(Overflow::Hidden)
        .font_family(vec![MAIN_FONT_ROLE.to_owned()]))
    .children(tracks(library, &spec.tracks, &at)?)
    .into())
}

pub fn tracks(library: &dyn Library, tracks: &[TimelineTrack], at: &At) -> Nodes {
    tracks
        .iter()
        .map(|track| track_fill(library, track, at))
        .collect()
}

fn track_fill(library: &dyn Library, track: &TimelineTrack, at: &At) -> Result<Node, DrawError> {
    let mut items = Vec::new();
    for item in &track.items {
        let base = item.base();
        let shown = at.clock.sequence(
            f64::from(base.from),
            Some(f64::from(base.duration_in_frames)),
        );
        if let Some(clock) = shown {
            let inner = At { clock, ..*at };
            items.push(fill(transition_wrap(library, item, &inner)?));
        }
    }
    Ok(fill(items))
}

fn item_content(library: &dyn Library, item: &TimelineItem, at: &At) -> Nodes {
    match item {
        TimelineItem::Group { props, .. } => Ok(vec![group(library, props, at)?]),
        TimelineItem::Recording { props, .. } => Ok(vec![library.recording(props, at)?]),
        TimelineItem::Element {
            base,
            component,
            props,
        } => {
            let exact = base.design.as_ref().and_then(|d| d.exact).unwrap_or(false);
            design_box(base.design.as_ref(), !exact, at, &|inner| {
                Ok(vec![
                    library
                        .element(component, props, inner)?
                        .unwrap_or_else(|| missing(library, component)),
                ])
            })
        }
    }
}

fn envelope_wrap(envelope: &VisualEnvelope, children: Vec<Node>, at: &At) -> Node {
    let frame = at.clock.frame;
    let opacity = envelope_at(envelope.opacity.as_deref(), frame, 1.0);
    let x = envelope_at(envelope.x.as_deref(), frame, 0.0) * at.stage.width;
    let y = envelope_at(envelope.y.as_deref(), frame, 0.0) * at.stage.height;
    let scale = 0.0_f64.max(envelope_at(envelope.scale.as_deref(), frame, 1.0));
    div(absolute_fill().opacity(clamp01(opacity)).transform(vec![
        Transform::Translate(Length::px(x), Length::px(y), Length::ZERO),
        Transform::Scale(Num(scale), Num(scale), Num(1.0)),
    ]))
    .children(children)
    .into()
}

const fn transition_ease(kind: Option<TransitionEase>) -> Ease {
    match kind {
        None | Some(TransitionEase::Linear) => Ease::Linear,
        Some(TransitionEase::In) => Ease::In,
        Some(TransitionEase::Out) => Ease::Out,
        Some(TransitionEase::InOut) => Ease::InOut,
    }
}

fn progress_from(transition: &TransitionRef, start: f64, frame: f64) -> Result<f64, DrawError> {
    let end = start + f64::from(transition.duration_in_frames);
    let linear = interpolate(
        frame,
        &[start, end],
        &[0.0, 1.0],
        &Interpolation::clamp_right(),
    )
    .map_err(|e| DrawError::new(&transition.kind, e))?;
    Ok(ease(transition_ease(transition.easing), linear))
}

fn active_transition(item: &TimelineItem, frame: f64) -> Result<Option<Presented<'_>>, DrawError> {
    let base = item.base();
    if let Some(enter) = &base.enter
        && frame < f64::from(enter.duration_in_frames)
    {
        return Ok(Some(Presented {
            transition: enter,
            side: Side::Entering,
            progress: progress_from(enter, 0.0, frame)?,
            cut: base.from,
        }));
    }
    if let Some(exit) = &base.exit {
        let start = f64::from(base.duration_in_frames) - f64::from(exit.duration_in_frames);
        if frame >= start {
            return Ok(Some(Presented {
                transition: exit,
                side: Side::Exiting,
                progress: progress_from(exit, start, frame)?,
                cut: (base.from + base.duration_in_frames).saturating_sub(exit.duration_in_frames),
            }));
        }
    }
    Ok(None)
}

fn transition_wrap(library: &dyn Library, item: &TimelineItem, at: &At) -> Nodes {
    let content = item_content(library, item, at)?;
    let children = match &item.base().envelope {
        Some(envelope) => vec![envelope_wrap(envelope, content, at)],
        None => content,
    };
    let Some(presented) = active_transition(item, at.clock.frame)? else {
        return Ok(children);
    };
    let fallback = children.clone();
    Ok(library
        .presentation(&presented, children, at)?
        .map_or(fallback, |node| vec![node]))
}

fn crop_scale(crop: &Rect, cover: bool) -> f64 {
    let side = if cover {
        crop.w.min(crop.h)
    } else {
        crop.w.max(crop.h)
    };
    1.0 / MIN_CROP.max(side)
}

fn crop_box(crop: &Rect, cover: bool, children: Vec<Node>) -> Node {
    let scale = crop_scale(crop, cover);
    let (w, h) = (crop.w * scale, crop.h * scale);
    let outer = if cover {
        absolute_fill().overflow(Overflow::Hidden)
    } else {
        absolute_fill()
    };
    let window = Style::default()
        .position(Position::Absolute)
        .overflow(Overflow::Hidden)
        .left(percent((1.0 - w) / 2.0))
        .top(percent((1.0 - h) / 2.0))
        .width(percent(w))
        .height(percent(h));
    let content = Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .width(percent(1.0 / w))
        .height(percent(1.0 / h))
        .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO))
        .transform(vec![
            Transform::Scale(Num(scale), Num(scale), Num(1.0)),
            Transform::Translate(percent(-crop.x), percent(-crop.y), Length::ZERO),
        ]);
    div(outer)
        .child(div(window).child(div(content).children(children)))
        .into()
}

type Inner<'a> = &'a dyn Fn(&At) -> Nodes;

fn design_box(design: Option<&Design>, fill_design: bool, at: &At, inner: Inner<'_>) -> Nodes {
    let Some(design) = design else {
        return inner(at);
    };
    let (width, height) = (at.stage.width, at.stage.height);
    let (dw, dh) = (f64::from(design.width), f64::from(design.height));
    if exact_eq(width, dw) && exact_eq(height, dh) {
        return inner(at);
    }
    let scale = (width / dw).min(height / dh);
    let (bw, bh) = if fill_design {
        ((width / scale).ceil(), (height / scale).ceil())
    } else {
        (dw, dh)
    };
    let boxed = At {
        clock: at.clock,
        stage: Stage {
            width: bw,
            height: bh,
        },
    };
    let scaled = Style::default()
        .position(Position::Relative)
        .flex_shrink(0.0)
        .width(bw)
        .height(bh)
        .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]);
    Ok(vec![
        div(absolute_fill()
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .overflow(Overflow::Hidden))
        .child(div(scaled).children(inner(&boxed)?))
        .into(),
    ])
}

fn group_clock(props: &GroupProps, outer: &Clock) -> Option<Clock> {
    let mut clock = *outer;
    if let Some(remap) = props.remap.as_ref().filter(|r| !r.is_empty()) {
        clock = clock.freeze(0.0_f64.max((remap_frame(remap, clock.frame) + EPSILON).floor()));
    }
    if let Some(every) = props.loop_every.filter(|&n| n > 0) {
        clock = clock.loop_every(f64::from(every))?;
    }
    if let Some(start) = props.start_from.filter(|&s| s > 0) {
        clock = clock.sequence(-f64::from(start), None)?;
    }
    Some(clock)
}

struct GroupScenes<'a> {
    library: &'a dyn Library,
    items: Vec<&'a TimelineItem>,
    sequencer: &'a SequencerProps,
}

impl SceneTracks for GroupScenes<'_> {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn duration_in_frames(&self, scene: usize) -> u32 {
        self.items
            .get(scene)
            .map_or(0, |i| i.base().duration_in_frames)
    }

    fn background(&self, scene: usize) -> Option<&str> {
        self.sequencer
            .backgrounds
            .get(scene)
            .and_then(Option::as_deref)
    }

    fn from(&self, scene: usize) -> u32 {
        self.items.get(scene).map_or(0, |i| i.base().from)
    }

    fn track_count(&self, scene: usize) -> usize {
        self.items
            .get(scene)
            .and_then(|i| i.group())
            .map_or(0, |g| g.tracks.len())
    }

    fn draw(&self, scene: usize, track: usize, at: &At) -> Nodes {
        let Some(group) = self.items.get(scene).and_then(|i| i.group()) else {
            return Ok(Vec::new());
        };
        let Some(clock) = at.clock.sequence(f64::from(self.from(scene)), None) else {
            return Ok(Vec::new());
        };
        let inner = At { clock, ..*at };
        group.tracks.get(track).map_or_else(
            || Ok(Vec::new()),
            |t| Ok(vec![track_fill(self.library, t, &inner)?]),
        )
    }
}

fn layers(library: &dyn Library, props: &GroupProps, at: &At) -> Nodes {
    let Some(clock) = group_clock(props, &at.clock) else {
        return Ok(Vec::new());
    };
    let inner = At { clock, ..*at };
    let drawn = match &props.sequencer {
        Some(sequencer) => {
            let scenes = GroupScenes {
                library,
                items: props.tracks.iter().flat_map(|t| t.items.iter()).collect(),
                sequencer,
            };
            vec![
                library
                    .scenes(sequencer, &scenes, &inner)?
                    .unwrap_or_else(|| missing(library, &sequencer.component)),
            ]
        }
        None => tracks(library, &props.tracks, &inner)?,
    };
    Ok(match &props.crop {
        Some(crop) => vec![crop_box(crop, props.crop_fill.unwrap_or(false), drawn)],
        None => drawn,
    })
}

fn wrapped(library: &dyn Library, props: &GroupProps, at: &At) -> Nodes {
    let Some(wrapper) = &props.wrapper else {
        return layers(library, props, at);
    };
    let child = |stage: Stage| -> Result<Node, DrawError> {
        Ok(fill(layers(library, props, &At { stage, ..*at })?))
    };
    match library.wrapper(&wrapper.component, &wrapper.props, &child, at)? {
        Some(node) => Ok(vec![node]),
        None => Ok(layers(library, props, at)?
            .into_iter()
            .chain([missing(library, &wrapper.component)])
            .collect()),
    }
}

fn group(library: &dyn Library, props: &GroupProps, at: &At) -> Result<Node, DrawError> {
    let design = props.design.map(|v| Design {
        width: v.width,
        height: v.height,
        exact: None,
    });
    let content = design_box(design.as_ref(), false, at, &|inner| {
        wrapped(library, props, inner)
    })?;
    let style = props.background.as_deref().map_or_else(absolute_fill, |b| {
        absolute_fill().background_color(library.color(b))
    });
    Ok(div(style).children(content).into())
}
