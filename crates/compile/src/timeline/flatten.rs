use std::collections::BTreeSet;
use std::sync::Arc;

use indexmap::IndexMap;
use opsis_types::actions::{Action, ActionKind};
use opsis_types::mix::ChannelKind;
use opsis_types::numbers;
use opsis_types::numbers::whole;
use opsis_types::recording::{RecordingProps, ScreenProps};
use opsis_types::timeline::{
    AudioItem, EditHandles, GroupProps, Slot, TimelineItem, TimelineSpec, TimelineTrack,
};
use serde::Serialize;
use serde_json::Value;

use crate::asset_ref::is_project_ref;
use crate::audio::{audio_kind, channel_of, sound_length};
use crate::clip::{IMAGE, item_has_sound, video_of, video_props};
use crate::elements::media::image::Image;
use crate::limits::MAX_REPEATS;
use crate::params::typed_values;
use crate::recording::recording_speed;
use crate::remap::{Remap, RemapSide, inverse_remap, remap_frame};
use crate::time::{EPSILON, clamp, ms_to_exact_frames, recording_ms_to_frames};
use crate::timeline::camera_path::sorted_camera_keys;

#[derive(Debug, Clone)]
struct Step {
    from: f64,
    start_from: f64,
    loop_offset: f64,
    remap: Option<Arc<Remap>>,
}

#[derive(Debug, Clone)]
struct Instance {
    steps: Arc<Vec<Step>>,
    lo: f64,
    hi: f64,
}

impl Instance {
    fn root(duration: f64) -> Self {
        Self {
            steps: Arc::new(Vec::new()),
            lo: 0.0,
            hi: duration,
        }
    }

    fn to_abs(&self, content: f64, side: RemapSide) -> f64 {
        self.steps.iter().fold(content, |v, s| {
            let r = v - s.start_from + s.loop_offset;
            s.from + s.remap.as_ref().map_or(r, |m| inverse_remap(m, r, side))
        })
    }

    fn inside(&self, step: Step, lo: f64, hi: f64) -> Self {
        let mut steps = vec![step];
        steps.extend(self.steps.iter().cloned());
        Self {
            steps: Arc::new(steps),
            lo,
            hi,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChainLink {
    pub item_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handles: Option<EditHandles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    pub rate: f64,
    pub rate_end: f64,
    pub end: f64,
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BarThumbnail {
    #[serde(rename_all = "camelCase")]
    Filmstrip {
        src: String,
        speed: f64,
        trim_ms: f64,
    },
    Still {
        src: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SoundWindow {
    pub from: f64,
    pub to: f64,
    pub content_start: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum VideoTag {
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum AudioTag {
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum BarKind {
    Recording,
    Element,
}

#[derive(Debug, Clone)]
struct LocalClock {
    inst: Instance,
    item_from: f64,
    content_end: f64,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoBar {
    #[serde(rename = "type")]
    pub tag: VideoTag,
    pub key: String,
    pub item_id: String,
    pub node_id: String,
    pub kind: BarKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[specta(type = opsis_types::JsonObject)]
    pub props: Value,
    pub thumbnail: Option<BarThumbnail>,
    pub from: f64,
    pub to: f64,
    pub lane: u32,
    pub order: u32,
    pub chain: Vec<ChainLink>,
    pub rate: f64,
    pub content_start: f64,
    pub content_end: f64,
    #[serde(rename = "loop")]
    pub repeat: u32,
    pub has_sound: bool,
    pub sound: Option<SoundWindow>,
    #[serde(skip)]
    #[specta(skip)]
    clock: LocalClock,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LocalFrames {
    pub first: f64,
    pub values: Vec<f64>,
}

impl VideoBar {
    pub fn local_at(&self, abs: f64) -> f64 {
        let clock = &self.clock;
        bisect(&clock.inst, clock.item_from, clock.content_end, abs) - clock.item_from
    }

    pub fn local_frames(&self) -> LocalFrames {
        let first = self.from.floor();
        let last = self.to.ceil();
        let count = whole((last - first).max(0.0)) + 1;
        LocalFrames {
            first,
            values: (0..count)
                .map(|i| self.local_at(first + f64::from(i)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioBar {
    #[serde(rename = "type")]
    pub tag: AudioTag,
    pub key: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub channel: String,
    pub src: String,
    pub role: ChannelKind,
    pub from: f64,
    pub to: f64,
    pub lane: u32,
    pub trim_before: f64,
    #[serde(rename = "loop")]
    pub looped: bool,
    pub volume: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handles: Option<EditHandles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<AudioItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<Box<VideoBar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hits: Option<Vec<f64>>,
    #[serde(skip)]
    pub parts: Vec<Self>,
    #[serde(skip)]
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TransitionMark {
    pub key: String,
    pub item_key: String,
    pub from: f64,
    pub to: f64,
    pub kind: String,
    pub lanes: Vec<u32>,
    pub covers: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ActionMark {
    pub key: String,
    pub frame: f64,
    pub kind: ActionKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub bar_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraKeyMark {
    pub key: String,
    pub item_id: String,
    pub node_id: String,
    pub index: u32,
    pub frame: f64,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FlatTimeline {
    pub fps: u32,
    pub duration_in_frames: u32,
    pub bars: Vec<VideoBar>,
    pub lanes: u32,
    pub audio: Vec<AudioBar>,
    pub audio_lanes: u32,
    pub transitions: Vec<TransitionMark>,
    pub marks: Vec<ActionMark>,
    pub camera_keys: Vec<CameraKeyMark>,
    #[serde(skip)]
    pub hits: Vec<Hit>,
    #[serde(skip)]
    pub cuts: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Click,
    Key,
    Drag,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub frame: f64,
    pub kind: HitKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, specta::Type)]
pub struct FrameRange {
    pub from: f64,
    pub to: f64,
}

pub fn action_label(action: &Action) -> Option<String> {
    let what = action
        .label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .or_else(|| {
            action
                .text
                .as_deref()
                .map(str::trim)
                .filter(|t| !t.is_empty())
        });
    let naming =
        |verb: &str, alone: &str| what.map_or_else(|| alone.to_owned(), |w| format!("{verb} {w}"));
    Some(match action.kind {
        ActionKind::Click => naming("Click", "Click"),
        ActionKind::Type => naming("Type", "Type"),
        ActionKind::Hover => naming("Hover", "Hover"),
        ActionKind::Scroll => "Scroll".to_owned(),
        ActionKind::Navigate => naming("Go to", "Go to page"),
        ActionKind::Chapter => what.unwrap_or("Chapter").to_owned(),
        ActionKind::Key => naming("Press", "Press keys"),
        ActionKind::Drag => naming("Drag", "Drag"),
        ActionKind::Wait | ActionKind::Focus => return None,
    })
}

fn thumbnail_of(item: &TimelineItem) -> Option<BarThumbnail> {
    match item {
        TimelineItem::Recording { props, .. } => match &props.screen {
            ScreenProps::Footage { src, .. } => {
                (!src.is_empty()).then(|| BarThumbnail::Filmstrip {
                    src: src.clone(),
                    speed: recording_speed(props),
                    trim_ms: 0.0,
                })
            }
            ScreenProps::Terminal { preview, .. } => {
                preview.clone().map(|src| BarThumbnail::Filmstrip {
                    src,
                    speed: recording_speed(props),
                    trim_ms: 0.0,
                })
            }
        },
        TimelineItem::Element {
            component, props, ..
        } => {
            if let Some(video) = video_of(item) {
                return (!video.src.is_empty()).then(|| BarThumbnail::Filmstrip {
                    src: video.src.clone(),
                    speed: 1.0,
                    trim_ms: video.trim_before_ms,
                });
            }
            let src = match component.as_str() {
                IMAGE => typed_values::<Image>(props).ok().map(|image| image.src),
                _ => None,
            };
            src.filter(|src| is_project_ref(src))
                .map(|src| BarThumbnail::Still { src })
        }
        TimelineItem::Group { .. } => None,
    }
}

fn non_empty_remap(group: &GroupProps) -> Option<&Remap> {
    group.remap.as_ref().filter(|r| !r.is_empty())
}

fn group_instances(
    item: &TimelineItem,
    group: &GroupProps,
    parent: &Instance,
    abs_from: f64,
    abs_to: f64,
) -> Vec<Instance> {
    let base = item.base();
    let item_from = f64::from(base.from);
    let start_from = f64::from(group.start_from.unwrap_or(0));
    let remap = non_empty_remap(group).map(|r| Arc::new(r.clone()));
    let looped = f64::from(group.loop_every.unwrap_or(0));
    let lo = parent.lo.max(abs_from);
    let hi = parent.hi.min(abs_to);
    let inv = |r: f64, side: RemapSide| remap.as_ref().map_or(r, |m| inverse_remap(m, r, side));
    let to_outer = |o: f64, side: RemapSide| parent.to_abs(item_from + o, side);
    let step = |k: f64| Step {
        from: item_from,
        start_from,
        loop_offset: k * looped,
        remap: remap.clone(),
    };
    if looped == 0.0 {
        return vec![parent.inside(step(0.0), lo, hi)];
    }
    let duration = f64::from(base.duration_in_frames);
    let max_r = remap
        .as_ref()
        .map_or(duration, |m| remap_frame(m, duration));
    let count =
        numbers::truncate::<usize>(clamp((max_r / looped).ceil(), 1.0, f64::from(MAX_REPEATS)));
    (0..count)
        .map(|k| {
            let k = numbers::float(k);
            let start = to_outer(inv(k * looped, RemapSide::Start), RemapSide::Start);
            let end = to_outer(inv((k + 1.0) * looped, RemapSide::End), RemapSide::End);
            parent.inside(step(k), lo.max(start), hi.min(end))
        })
        .collect()
}

const BISECT_STEPS: usize = 24;

fn bisect(inst: &Instance, a: f64, b: f64, abs: f64) -> f64 {
    let (mut lo, mut hi) = (a, b);
    for _ in 0..BISECT_STEPS {
        let mid = f64::midpoint(lo, hi);
        if inst.to_abs(mid, RemapSide::Start) < abs {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
}

fn rate_at(inst: &Instance, content: f64) -> f64 {
    let d = inst.to_abs(content + 1.0, RemapSide::Start) - inst.to_abs(content, RemapSide::Start);
    if d > EPSILON { 1.0 / d } else { 1.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Timing {
    Plain,
    Held,
    Changed,
}

fn is_freeze_or_play(remap: &[(f64, f64)]) -> bool {
    remap.windows(2).all(|w| {
        let ((o0, i0), (o1, i1)) = (w[0], w[1]);
        let slope = (i1 - i0) / EPSILON.max(o1 - o0);
        slope.abs() < EPSILON || (slope - 1.0).abs() < EPSILON
    })
}

fn timing_with(timing: Timing, remap: Option<&Remap>) -> Timing {
    match remap {
        Some(remap) if timing != Timing::Changed => {
            if is_freeze_or_play(remap) {
                Timing::Held
            } else {
                Timing::Changed
            }
        }
        _ => timing,
    }
}

#[derive(Debug, Clone)]
struct Walk {
    inst: Instance,
    path: String,
    chain: Vec<ChainLink>,
    repeat: u32,
    timing: Timing,
}

struct Placed<'a> {
    item: &'a TimelineItem,
    walk: &'a Walk,
    key: String,
    abs_start: f64,
    abs_end: f64,
    from: f64,
    to: f64,
    item_from: f64,
    content_end: f64,
}

fn place<'a>(item: &'a TimelineItem, walk: &'a Walk, track: usize) -> Placed<'a> {
    let base = item.base();
    let item_from = f64::from(base.from);
    let content_end = item_from + f64::from(base.duration_in_frames);
    let abs_start = walk.inst.to_abs(item_from, RemapSide::Start);
    let abs_end = walk.inst.to_abs(content_end, RemapSide::End);
    Placed {
        item,
        walk,
        key: format!("{}/{track}:{}", walk.path, base.id),
        abs_start,
        abs_end,
        from: abs_start.max(walk.inst.lo),
        to: abs_end.min(walk.inst.hi),
        item_from,
        content_end,
    }
}

fn content_at(p: &Placed<'_>, abs: f64) -> f64 {
    bisect(&p.walk.inst, p.item_from, p.content_end, abs)
}

fn sound_window(p: &Placed<'_>, from: f64, to: f64, start_content: f64) -> Option<SoundWindow> {
    match p.walk.timing {
        Timing::Plain => Some(SoundWindow {
            from,
            to,
            content_start: start_content - p.item_from,
        }),
        Timing::Changed => None,
        Timing::Held => {
            let play_from = from.max(p.walk.inst.to_abs(start_content, RemapSide::End));
            let play_to = to.min(p.walk.inst.to_abs(p.content_end - 1.0, RemapSide::Start) + 1.0);
            (play_to - play_from >= 1.0).then(|| SoundWindow {
                from: play_from,
                to: play_to,
                content_start: content_at(p, play_from) - p.item_from,
            })
        }
    }
}

fn make_link(p: &Placed<'_>) -> ChainLink {
    let base = p.item.base();
    let inst = &p.walk.inst;
    ChainLink {
        item_id: base.id.clone(),
        node_id: base.node_id.clone(),
        handles: base.handles.clone(),
        slot: base.slot.clone(),
        rate: rate_at(inst, p.item_from),
        rate_end: rate_at(inst, p.item_from.max(p.content_end - 1.0)),
        end: if p.to < p.abs_end - EPSILON {
            content_at(p, p.to) - p.item_from
        } else {
            f64::from(base.duration_in_frames)
        },
        from: p.from,
        to: p.to,
    }
}

fn transition_mark(p: &Placed<'_>, previous_length: f64) -> Option<TransitionMark> {
    let base = p.item.base();
    let enter = base.enter.as_ref().filter(|e| e.duration_in_frames > 0)?;
    let to = p.to.min(p.walk.inst.to_abs(
        p.item_from + f64::from(enter.duration_in_frames),
        RemapSide::End,
    ));
    let shorter = (p.abs_end - p.abs_start).min(previous_length);
    Some(TransitionMark {
        key: format!("{}:enter", p.key),
        item_key: p.key.clone(),
        from: p.from,
        to,
        kind: enter.kind.clone(),
        lanes: Vec::new(),
        covers: if enter.hold == Some(true) {
            0.0
        } else {
            (to - p.from) / EPSILON.max(shorter)
        },
        slot: base.slot.clone(),
    })
}

fn leaf_bar(p: &Placed<'_>, link: ChainLink, order: u32) -> VideoBar {
    let item = p.item;
    let base = item.base();
    let start_content = if p.from > p.abs_start + EPSILON {
        content_at(p, p.from)
    } else {
        p.item_from
    };
    let end_content = if p.to < p.abs_end - EPSILON {
        content_at(p, p.to)
    } else {
        p.content_end
    };
    let has_sound = item_has_sound(item);
    let (kind, component, props) = match item {
        TimelineItem::Recording { props, .. } => (
            BarKind::Recording,
            None,
            serde_json::to_value(props).expect("props serialize"),
        ),
        TimelineItem::Element {
            component, props, ..
        } => (
            BarKind::Element,
            Some(component.clone()),
            Value::Object(props.clone()),
        ),
        TimelineItem::Group { .. } => unreachable!("groups are walked, not drawn"),
    };
    let mut chain = vec![link];
    chain.extend(p.walk.chain.iter().cloned());
    VideoBar {
        tag: VideoTag::Video,
        key: p.key.clone(),
        item_id: base.id.clone(),
        node_id: base
            .node_id
            .clone()
            .or_else(|| p.walk.chain.iter().find_map(|l| l.node_id.clone()))
            .unwrap_or_else(|| base.id.clone()),
        kind,
        component,
        props,
        thumbnail: thumbnail_of(item),
        from: p.from,
        to: p.to,
        lane: 0,
        order,
        chain,
        rate: rate_at(&p.walk.inst, start_content),
        content_start: start_content - p.item_from,
        content_end: end_content - p.item_from,
        repeat: p.walk.repeat,
        has_sound,
        sound: if has_sound {
            sound_window(p, p.from, p.to, start_content)
        } else {
            None
        },
        clock: LocalClock {
            inst: p.walk.inst.clone(),
            item_from: p.item_from,
            content_end: p.content_end,
        },
    }
}

fn recording_marks(p: &Placed<'_>, props: &RecordingProps, fps: u32) -> Vec<ActionMark> {
    let speed = recording_speed(props);
    props
        .log
        .actions
        .iter()
        .filter_map(|action| {
            let label = action_label(action)?;
            let local = recording_ms_to_frames(action.t, speed, fps);
            let frame = p.walk.inst.to_abs(p.item_from + local, RemapSide::Start);
            (frame >= p.from && frame < p.to).then(|| ActionMark {
                key: format!("{}:{}", p.key, action.id),
                frame,
                kind: action.kind,
                label,
                node_id: action.node_id.clone(),
                bar_key: p.key.clone(),
            })
        })
        .collect()
}

fn action_hits(action: &Action) -> Vec<(f64, HitKind)> {
    if action.silent == Some(true) {
        return Vec::new();
    }
    let presses = match action.kind {
        ActionKind::Click | ActionKind::Type => {
            vec![action.t_down.unwrap_or(action.t)]
        }
        _ => Vec::new(),
    };
    let chord = (action.kind == ActionKind::Key).then_some(action.t);
    let drag = if action.kind == ActionKind::Drag {
        vec![action.t_down.unwrap_or(action.t), action.t]
    } else {
        Vec::new()
    };
    presses
        .into_iter()
        .chain(action.t_second_down)
        .map(|t| (t, HitKind::Click))
        .chain(drag.into_iter().map(|t| (t, HitKind::Drag)))
        .chain(
            chord
                .into_iter()
                .chain(action.key_times.iter().flatten().copied())
                .map(|t| (t, HitKind::Key)),
        )
        .collect()
}

fn recording_hits(p: &Placed<'_>, props: &RecordingProps, fps: u32) -> Vec<Hit> {
    let speed = recording_speed(props);
    props
        .log
        .actions
        .iter()
        .flat_map(action_hits)
        .map(|(t, kind)| Hit {
            frame: p.walk.inst.to_abs(
                p.item_from + recording_ms_to_frames(t, speed, fps),
                RemapSide::Start,
            ),
            kind,
        })
        .filter(|h| h.frame >= p.from && h.frame < p.to)
        .collect()
}

fn camera_key_marks(p: &Placed<'_>) -> Vec<CameraKeyMark> {
    let base = p.item.base();
    let (Some(node_id), Some(camera)) = (&base.node_id, &base.camera) else {
        return Vec::new();
    };
    sorted_camera_keys(Some(camera))
        .iter()
        .enumerate()
        .filter_map(|(index, k)| {
            let frame = p.walk.inst.to_abs(p.item_from + k.frame, RemapSide::Start);
            (frame >= p.from - EPSILON && frame <= p.to + EPSILON).then(|| CameraKeyMark {
                key: format!("{}:camera:{index}", p.key),
                item_id: base.id.clone(),
                node_id: node_id.clone(),
                index: numbers::count(index),
                frame,
            })
        })
        .collect()
}

struct Collected {
    fps: u32,
    bars: Vec<VideoBar>,
    transitions: Vec<TransitionMark>,
    marks: Vec<ActionMark>,
    camera_keys: Vec<CameraKeyMark>,
    hits: Vec<Hit>,
    cuts: Vec<f64>,
}

fn visit_tracks(acc: &mut Collected, tracks: &[TimelineTrack], walk: &Walk) {
    for (ti, track) in tracks.iter().enumerate() {
        let mut previous_length = f64::INFINITY;
        for item in &track.items {
            let p = place(item, walk, ti);
            let before = previous_length;
            previous_length = p.abs_end - p.abs_start;
            if p.to - p.from < EPSILON {
                continue;
            }
            let entered = p
                .item
                .base()
                .enter
                .as_ref()
                .is_some_and(|e| e.duration_in_frames > 0);
            if before.is_finite() && !entered {
                acc.cuts.push(p.from);
            }
            visit_item(acc, &p, before);
        }
    }
}

fn visit_item(acc: &mut Collected, p: &Placed<'_>, previous_length: f64) {
    let link = make_link(p);
    if let Some(mark) = transition_mark(p, previous_length) {
        acc.transitions.push(mark);
    }
    acc.camera_keys.extend(camera_key_marks(p));
    match p.item {
        TimelineItem::Group { props, .. } => visit_group(acc, p, props, link),
        TimelineItem::Recording { props, .. } => {
            let order = numbers::count(acc.bars.len());
            acc.bars.push(leaf_bar(p, link, order));
            let marks = recording_marks(p, props, acc.fps);
            acc.marks.extend(marks);
            let hits = recording_hits(p, props, acc.fps);
            acc.hits.extend(hits);
        }
        TimelineItem::Element { .. } => {
            let order = numbers::count(acc.bars.len());
            acc.bars.push(leaf_bar(p, link, order));
        }
    }
}

fn visit_group(acc: &mut Collected, p: &Placed<'_>, group: &GroupProps, link: ChainLink) {
    let base = p.item.base();
    let instances = group_instances(p.item, group, &p.walk.inst, p.abs_start, p.abs_end);
    let looped = instances.len() > 1;
    let chain = if base.node_id.is_some() || base.handles.is_some() || base.slot.is_some() {
        let mut chain = vec![link];
        chain.extend(p.walk.chain.iter().cloned());
        chain
    } else {
        p.walk.chain.clone()
    };
    for (k, inst) in instances.into_iter().enumerate() {
        if inst.hi - inst.lo < EPSILON {
            continue;
        }
        let walk = Walk {
            inst,
            path: if looped {
                format!("{}#{k}", p.key)
            } else {
                p.key.clone()
            },
            chain: chain.clone(),
            repeat: if looped {
                numbers::count(k)
            } else {
                p.walk.repeat
            },
            timing: timing_with(p.walk.timing, non_empty_remap(group)),
        };
        visit_tracks(acc, &group.tracks, &walk);
    }
}

pub fn overlaps(a: FrameRange, b: FrameRange) -> bool {
    a.from < b.to - EPSILON && b.from < a.to - EPSILON
}

const fn range(from: f64, to: f64) -> FrameRange {
    FrameRange { from, to }
}

fn pack_lanes(bars: Vec<VideoBar>) -> (Vec<VideoBar>, u32) {
    let mut placed: Vec<VideoBar> = Vec::new();
    let mut lanes = 0;
    for mut bar in bars {
        let taken: BTreeSet<u32> = placed
            .iter()
            .filter(|p| overlaps(range(p.from, p.to), range(bar.from, bar.to)))
            .map(|p| p.lane)
            .collect();
        let lane = (0u32..)
            .take(taken.len() + 1)
            .find(|l| !taken.contains(l))
            .unwrap_or(0);
        bar.lane = lane;
        lanes = lanes.max(lane + 1);
        placed.push(bar);
    }
    (placed, lanes)
}

fn pack_audio(audio: Vec<AudioBar>) -> (Vec<AudioBar>, u32) {
    let mut ends: Vec<f64> = Vec::new();
    let mut last_role: Option<ChannelKind> = None;
    let mut base = 0;
    let mut placed = Vec::new();
    for mut a in audio {
        if Some(a.role) != last_role {
            base = ends.len();
            last_role = Some(a.role);
        }
        let mut lane = base;
        while lane < ends.len() && ends[lane] > a.from + EPSILON {
            lane += 1;
        }
        if lane == ends.len() {
            ends.push(a.to);
        } else {
            ends[lane] = a.to;
        }
        a.lane = numbers::count(lane);
        placed.push(a);
    }
    let lanes = numbers::count(ends.len());
    (placed, lanes)
}

fn sound_bar(a: &AudioItem, duration: u32) -> Option<AudioBar> {
    let start = f64::from(a.from);
    let from = start.max(0.0);
    let to = f64::from(duration).min(start + sound_length(a, duration));
    (to > from).then(|| AudioBar {
        tag: AudioTag::Audio,
        key: format!("audio:{}", a.id),
        id: a.id.clone(),
        node_id: a.node_id.clone(),
        channel: channel_of(a),
        src: a.src.clone(),
        role: audio_kind(a),
        from,
        to,
        lane: 0,
        trim_before: f64::from(a.trim_before.unwrap_or(0)) + (-start).max(0.0),
        looped: a.looped == Some(true),
        volume: a.volume.filter(|v| v.is_finite()).unwrap_or(1.0),
        handles: a.handles.clone(),
        item: Some(a.clone()),
        bar: None,
        hits: None,
        parts: Vec::new(),
        rate: 1.0,
    })
}

fn hits_bar(items: &[&AudioItem], duration: u32) -> Option<AudioBar> {
    let bars: Vec<AudioBar> = items
        .iter()
        .filter_map(|a| sound_bar(a, duration))
        .collect();
    let first = bars.first()?;
    let from = bars.iter().map(|b| b.from).fold(f64::INFINITY, f64::min);
    let to = bars.iter().map(|b| b.to).fold(f64::NEG_INFINITY, f64::max);
    let volume = bars.iter().map(|b| b.volume).sum::<f64>() / numbers::float(bars.len());
    Some(AudioBar {
        key: format!("hits:{}", first.channel),
        id: first.channel.clone(),
        from,
        to,
        trim_before: 0.0,
        looped: false,
        volume,
        item: None,
        hits: Some(bars.iter().map(|b| b.from).collect()),
        parts: bars.clone(),
        ..first.clone()
    })
}

fn clip_sound_bar(bar: &VideoBar, fps: u32) -> Option<AudioBar> {
    let Value::Object(props) = &bar.props else {
        return None;
    };
    let media = video_props(bar.component.as_deref()?, props)?;
    let sound = bar.sound?;
    if media.src.is_empty() {
        return None;
    }
    Some(AudioBar {
        tag: AudioTag::Audio,
        key: format!("clip:{}", bar.key),
        id: bar.key.clone(),
        node_id: Some(bar.node_id.clone()),
        channel: bar.node_id.clone(),
        src: media.src.clone(),
        role: ChannelKind::Clip,
        from: sound.from,
        to: sound.to,
        lane: 0,
        trim_before: f64::mul_add(
            sound.content_start,
            media.playback_rate,
            ms_to_exact_frames(media.trim_before_ms, fps),
        )
        .round(),
        looped: media.looped,
        volume: media.volume,
        handles: None,
        item: None,
        bar: Some(Box::new(bar.clone())),
        hits: None,
        parts: Vec::new(),
        rate: media.playback_rate,
    })
}

const fn role_order(role: ChannelKind) -> usize {
    match role {
        ChannelKind::Music => 0,
        ChannelKind::Effect => 1,
        ChannelKind::Clip => 2,
    }
}

fn audio_bars(spec: &TimelineSpec, bars: &[VideoBar]) -> Vec<AudioBar> {
    let mut hits: IndexMap<String, Vec<&AudioItem>> = IndexMap::new();
    for a in spec.audio.iter().filter(|a| a.hit == Some(true)) {
        hits.entry(channel_of(a)).or_default().push(a);
    }
    let mut out: Vec<AudioBar> = spec
        .audio
        .iter()
        .filter(|a| a.hit != Some(true))
        .filter_map(|a| sound_bar(a, spec.duration_in_frames))
        .chain(
            hits.values()
                .filter_map(|items| hits_bar(items, spec.duration_in_frames)),
        )
        .chain(bars.iter().filter_map(|b| clip_sound_bar(b, spec.fps)))
        .collect();
    out.sort_by(|x, y| {
        role_order(x.role)
            .cmp(&role_order(y.role))
            .then(x.from.total_cmp(&y.from))
    });
    out
}

fn place_transitions(transitions: Vec<TransitionMark>, bars: &[VideoBar]) -> Vec<TransitionMark> {
    transitions
        .into_iter()
        .map(|t| {
            let lanes: BTreeSet<u32> = bars
                .iter()
                .filter(|b| overlaps(range(b.from, b.to), range(t.from, t.to)))
                .map(|b| b.lane)
                .collect();
            TransitionMark {
                lanes: lanes.into_iter().collect(),
                ..t
            }
        })
        .collect()
}

pub fn flatten_spec(spec: &TimelineSpec) -> FlatTimeline {
    let mut acc = Collected {
        fps: spec.fps,
        bars: Vec::new(),
        transitions: Vec::new(),
        marks: Vec::new(),
        camera_keys: Vec::new(),
        hits: Vec::new(),
        cuts: Vec::new(),
    };
    let root = Walk {
        inst: Instance::root(f64::from(spec.duration_in_frames)),
        path: String::new(),
        chain: Vec::new(),
        repeat: 0,
        timing: Timing::Plain,
    };
    visit_tracks(&mut acc, &spec.tracks, &root);
    let (bars, lanes) = pack_lanes(acc.bars);
    let (audio, audio_lanes) = pack_audio(audio_bars(spec, &bars));
    let transitions = place_transitions(acc.transitions, &bars);
    let mut marks = acc.marks;
    marks.sort_by(|x, y| x.frame.total_cmp(&y.frame));
    let mut hits = acc.hits;
    hits.sort_by(|x, y| x.frame.total_cmp(&y.frame));
    let mut cuts = acc.cuts;
    cuts.sort_by(f64::total_cmp);
    FlatTimeline {
        fps: spec.fps,
        duration_in_frames: spec.duration_in_frames,
        bars,
        lanes,
        audio,
        audio_lanes,
        transitions,
        marks,
        camera_keys: acc.camera_keys,
        hits,
        cuts,
    }
}

pub fn node_spans(spec: &TimelineSpec) -> IndexMap<String, FrameRange> {
    let mut spans: IndexMap<String, FrameRange> = IndexMap::new();
    for bar in flatten_spec(spec).bars {
        for link in bar.chain {
            let (from, to) = (link.from.round(), link.to.round());
            let Some(node_id) = link.node_id.filter(|_| to > from) else {
                continue;
            };
            let span = spans.entry(node_id).or_insert(FrameRange { from, to });
            span.from = span.from.min(from);
            span.to = span.to.max(to);
        }
    }
    spans
}
