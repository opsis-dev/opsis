use opsis_types::numbers::{signed, whole, whole_signed};
use opsis_types::recording::RecordingProps;
use opsis_types::timeline::{
    AudioEnvelope, AudioItem, CameraPath, Design, EditHandles, GroupProps, ItemBase, Keyframe,
    SequencerProps, Slot, TimelineItem, TimelineSpec, TimelineTrack, TransitionRef, VisualEnvelope,
};
use opsis_types::values::{ClipValue, CompileValue, RecordingValue, TaggedRecording};
use opsis_types::workflow::WorkflowSettings;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::envelope::has_visual_keys;
use crate::fonts::film_fonts;
use crate::node::InputTake;
use crate::params::to_values;
use crate::remap::{RemapSide, inverse_remap};

pub const EMPTY_CLIP_FRAMES: u32 = 30;
pub const PLACEHOLDER: &str = "placeholder";
pub const STILL: &str = "still";
pub const IMAGE: &str = "image";
pub const VIDEO: &str = "video";

fn yes() -> bool {
    true
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProps {
    pub src: String,
    #[serde(default, rename = "loop")]
    pub looped: bool,
    #[serde(default = "yes")]
    pub muted: bool,
    #[serde(default = "one")]
    pub volume: f64,
    #[serde(default)]
    pub trim_before_ms: f64,
}

pub fn video_props(component: &str, props: &Map<String, Value>) -> Option<VideoProps> {
    (component == VIDEO)
        .then(|| serde_json::from_value(Value::Object(props.clone())).ok())
        .flatten()
}

pub fn video_of(item: &TimelineItem) -> Option<VideoProps> {
    match item {
        TimelineItem::Element {
            component, props, ..
        } => video_props(component, props),
        _ => None,
    }
}

pub fn item_has_sound(item: &TimelineItem) -> bool {
    video_of(item).is_some_and(|v| !v.muted)
}

pub fn sound_node_id(item: &TimelineItem) -> Option<&str> {
    if item_has_sound(item) {
        item.base().node_id.as_deref()
    } else {
        None
    }
}

pub fn group_item(base: ItemBase, props: GroupProps) -> TimelineItem {
    TimelineItem::Group {
        base,
        props: Box::new(props),
    }
}

pub fn clip_of(value: Option<&CompileValue>) -> Option<&ClipValue> {
    value.and_then(CompileValue::as_clip)
}

fn make_clip(
    duration_in_frames: u32,
    tracks: Vec<TimelineTrack>,
    audio: Vec<AudioItem>,
    recording: Option<RecordingValue>,
) -> ClipValue {
    ClipValue {
        duration_in_frames,
        tracks,
        audio,
        recording: recording.map(TaggedRecording::Recording),
    }
}

pub fn clip_recording(clip: &ClipValue) -> Option<&RecordingValue> {
    clip.recording
        .as_ref()
        .map(|TaggedRecording::Recording(r)| r)
}

pub fn empty_clip(duration_in_frames: u32) -> ClipValue {
    make_clip(duration_in_frames, Vec::new(), Vec::new(), None)
}

fn first_recording<'a>(clips: impl IntoIterator<Item = &'a ClipValue>) -> Option<RecordingValue> {
    clips.into_iter().find_map(clip_recording).cloned()
}

fn one_track(id: &str, item: TimelineItem) -> Vec<TimelineTrack> {
    vec![TimelineTrack {
        id: format!("{id}:t"),
        items: vec![item],
        layer: None,
    }]
}

#[derive(Debug, Clone, Default)]
pub struct ItemExtra {
    pub recording: Option<RecordingValue>,
    pub audio: Vec<AudioItem>,
    pub handles: Option<EditHandles>,
    pub envelope: Option<VisualEnvelope>,
    pub camera: Option<CameraPath>,
    pub design: Option<Design>,
}

enum Content {
    Recording(Box<RecordingProps>),
    Element(String, Map<String, Value>),
}

fn content_clip(
    node_id: &str,
    content: Content,
    duration_in_frames: u32,
    extra: ItemExtra,
) -> ClipValue {
    let duration = duration_in_frames.max(1);
    let mut base = ItemBase::new(format!("{node_id}:i"), 0, duration);
    base.node_id = Some(node_id.to_owned());
    base.handles = extra.handles;
    base.envelope = extra.envelope.filter(|e| has_visual_keys(Some(e)));
    base.camera = extra.camera.filter(|c| !c.keys.is_empty());
    base.design = extra.design;
    let item = match content {
        Content::Recording(props) => TimelineItem::Recording { base, props },
        Content::Element(component, props) => TimelineItem::Element {
            base,
            component,
            props,
        },
    };
    make_clip(
        duration,
        one_track(node_id, item),
        extra.audio,
        extra.recording,
    )
}

pub fn item_clip(
    node_id: &str,
    component: &str,
    props: Map<String, Value>,
    duration_in_frames: u32,
    extra: ItemExtra,
) -> ClipValue {
    content_clip(
        node_id,
        Content::Element(component.to_owned(), props),
        duration_in_frames,
        extra,
    )
}

pub fn placeholder_clip(
    node_id: &str,
    title: &str,
    subtitle: &str,
    duration_in_frames: u32,
    extra: ItemExtra,
) -> ClipValue {
    let props = PlaceholderProps {
        title: title.to_owned(),
        subtitle: subtitle.to_owned(),
    };
    item_clip(
        node_id,
        PLACEHOLDER,
        to_values(&props),
        duration_in_frames,
        extra,
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct PlaceholderProps {
    pub title: String,
    pub subtitle: String,
}

pub fn recording_clip(
    node_id: &str,
    props: RecordingProps,
    duration_in_frames: u32,
    extra: ItemExtra,
) -> ClipValue {
    content_clip(
        node_id,
        Content::Recording(Box::new(props)),
        duration_in_frames,
        extra,
    )
}

fn cut_sound_start(a: AudioItem, by: i32) -> Option<AudioItem> {
    let by = u32::try_from(by).ok().filter(|by| *by > 0);
    let Some(by) = by else { return Some(a) };
    if a.duration_in_frames.is_some_and(|d| d <= by) {
        return None;
    }
    let source_cut = whole(f64::from(by) * a.rate.unwrap_or(1.0));
    if a.length_frames.is_some_and(|l| l <= source_cut) {
        return None;
    }
    Some(AudioItem {
        from: a.from + signed(by),
        trim_before: Some(a.trim_before.unwrap_or(0) + source_cut),
        duration_in_frames: a.duration_in_frames.map(|d| d - by),
        length_frames: a.length_frames.map(|l| l - source_cut),
        ..a
    })
}

fn fit_within(a: AudioItem, end: u32) -> Option<AudioItem> {
    let by = -a.from;
    let placed = cut_sound_start(a, by)?;
    let start = u32::try_from(placed.from).ok()?;
    if start >= end {
        return None;
    }
    let room = end - start;
    Some(AudioItem {
        duration_in_frames: Some(placed.duration_in_frames.unwrap_or(room).min(room)),
        ..placed
    })
}

fn scale_keys(keys: Option<Vec<Keyframe>>, by: f64) -> Option<Vec<Keyframe>> {
    keys.map(|keys| {
        keys.into_iter()
            .map(|k| Keyframe {
                frame: (k.frame * by).round(),
                ..k
            })
            .collect()
    })
}

fn speed_sound(a: AudioItem, inner_per_outer: f64) -> AudioItem {
    let outer = |frames: u32| whole(f64::from(frames) / inner_per_outer).max(1);
    AudioItem {
        from: whole_signed(f64::from(a.from) / inner_per_outer),
        rate: Some(a.rate.unwrap_or(1.0) * inner_per_outer),
        duration_in_frames: a.duration_in_frames.map(outer),
        fade_in_frames: a
            .fade_in_frames
            .filter(|f| *f != 0)
            .map(outer)
            .or(a.fade_in_frames),
        fade_out_frames: a
            .fade_out_frames
            .filter(|f| *f != 0)
            .map(outer)
            .or(a.fade_out_frames),
        envelope: a.envelope.map(|e| AudioEnvelope {
            volume: scale_keys(e.volume, 1.0 / inner_per_outer),
            pan: scale_keys(e.pan, 1.0 / inner_per_outer),
        }),
        ..a
    }
}

fn uniform_rate(remap: &[(f64, f64)]) -> Option<f64> {
    match remap {
        [(0.0, 0.0), (x, y)] if *x > 0.0 => Some(y / x),
        _ => None,
    }
}

pub fn wrap_sounds(audio: &[AudioItem], props: &GroupProps, duration: u32) -> Vec<AudioItem> {
    let start = signed(props.start_from.unwrap_or(0));
    let mut items: Vec<AudioItem> = audio
        .iter()
        .filter_map(|a| {
            cut_sound_start(
                AudioItem {
                    from: a.from - start,
                    ..a.clone()
                },
                0,
            )
        })
        .collect();
    let looped = props.loop_every.unwrap_or(0);
    if looped > 0 {
        items = (0..duration.div_ceil(looped))
            .flat_map(|k| {
                items.iter().filter_map(move |a| {
                    let in_loop = fit_within(a.clone(), looped)?;
                    let copy = AudioItem {
                        from: in_loop.from + signed(k * looped),
                        ..in_loop
                    };
                    Some(if k == 0 {
                        copy
                    } else {
                        AudioItem {
                            id: format!("{}:r{k}", a.id),
                            handles: None,
                            ..copy
                        }
                    })
                })
            })
            .collect();
    }
    if let Some(remap) = props.remap.as_ref().filter(|r| !r.is_empty()) {
        items = match uniform_rate(remap) {
            None => items
                .into_iter()
                .map(|a| AudioItem {
                    from: whole_signed(inverse_remap(remap, f64::from(a.from), RemapSide::Start)),
                    ..a
                })
                .collect(),
            Some(rate) => items.into_iter().map(|a| speed_sound(a, rate)).collect(),
        };
    }
    items
        .into_iter()
        .filter_map(|a| fit_within(a, duration))
        .collect()
}

pub fn wrap_clip(
    clip: &ClipValue,
    node_id: &str,
    props: GroupProps,
    duration_in_frames: Option<u32>,
) -> ClipValue {
    let duration = duration_in_frames.unwrap_or(clip.duration_in_frames).max(1);
    let audio = wrap_sounds(&clip.audio, &props, duration);
    let mut base = ItemBase::new(format!("{node_id}:g"), 0, duration);
    base.node_id = Some(node_id.to_owned());
    let group = group_item(
        base,
        GroupProps {
            tracks: clip.tracks.clone(),
            ..props
        },
    );
    ClipValue {
        duration_in_frames: duration,
        tracks: one_track(node_id, group),
        audio,
        recording: clip.recording.clone(),
    }
}

pub fn with_handles(
    clip: ClipValue,
    handles: EditHandles,
    envelope: Option<VisualEnvelope>,
    camera: Option<CameraPath>,
) -> ClipValue {
    if clip.tracks.len() != 1 || clip.tracks[0].items.len() != 1 {
        return clip;
    }
    let mut clip = clip;
    let base = clip.tracks[0].items[0].base_mut();
    let mut merged = base.handles.take().unwrap_or_default();
    if handles.move_to.is_some() {
        merged.move_to = handles.move_to;
    }
    if handles.trim_start.is_some() {
        merged.trim_start = handles.trim_start;
    }
    if handles.trim_end.is_some() {
        merged.trim_end = handles.trim_end;
    }
    if handles.speed.is_some() {
        merged.speed = handles.speed;
    }
    if handles.volume.is_some() {
        merged.volume = handles.volume;
    }
    base.handles = Some(merged);
    if has_visual_keys(envelope.as_ref()) {
        base.envelope = envelope;
    }
    if camera.as_ref().is_some_and(|c| !c.keys.is_empty()) {
        base.camera = camera;
    }
    clip
}

fn shift_audio(audio: &[AudioItem], offset: u32, prefix: &str) -> Vec<AudioItem> {
    audio
        .iter()
        .map(|a| AudioItem {
            id: format!("{prefix}/{}", a.id),
            from: a.from + signed(offset),
            ..a.clone()
        })
        .collect()
}

fn overlaps(t: Option<&TransitionRef>) -> bool {
    t.is_some_and(|t| t.hold != Some(true))
}

fn share_of(clip: &ClipValue, other_side: Option<&TransitionRef>) -> u32 {
    if overlaps(other_side) {
        clip.duration_in_frames / 2
    } else {
        clip.duration_in_frames.saturating_sub(1)
    }
}

fn resolve_enters(
    clips: &[ClipValue],
    transitions: &[Option<TransitionRef>],
) -> Vec<Option<TransitionRef>> {
    let at = |i: usize| transitions.get(i).and_then(Option::as_ref);
    clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            let enter = if i > 0 { at(i) } else { None }?;
            let longest = if enter.hold == Some(true) {
                enter.duration_in_frames
            } else {
                share_of(&clips[i - 1], if i > 1 { at(i - 1) } else { None })
                    .min(share_of(clip, at(i + 1)))
            };
            let frames = longest.min(enter.duration_in_frames);
            (frames > 0).then(|| TransitionRef {
                duration_in_frames: frames,
                ..enter.clone()
            })
        })
        .collect()
}

fn hold_last_frame(item: TimelineItem, clip_frames: u32, frames: u32) -> TimelineItem {
    let TimelineItem::Group {
        mut base,
        mut props,
    } = item
    else {
        return item;
    };
    let last = f64::from(clip_frames) - 1.0;
    let end = f64::from(base.duration_in_frames) - 1.0;
    let mut remap = props.remap.take().unwrap_or_else(|| vec![(0.0, 0.0)]);
    remap.push((end, last));
    remap.push((end + f64::from(frames), last));
    props.remap = Some(remap);
    base.duration_in_frames += frames;
    TimelineItem::Group { base, props }
}

#[derive(Debug, Clone, Copy)]
struct Placement {
    from: u32,
    lead: u32,
}

fn place_clip(
    id: String,
    clip: &ClipValue,
    at: Placement,
    enter: Option<TransitionRef>,
    slot: Option<Slot>,
) -> TimelineItem {
    let mut base = ItemBase::new(id, at.from, clip.duration_in_frames + at.lead);
    base.enter = enter;
    base.slot = slot;
    let remap = (at.lead != 0).then(|| vec![(0.0, 0.0), (f64::from(at.lead), 0.0)]);
    group_item(
        base,
        GroupProps {
            tracks: clip.tracks.clone(),
            remap,
            ..GroupProps::default()
        },
    )
}

fn with_exit(item: TimelineItem, clip: &ClipValue, exit: Option<&TransitionRef>) -> TimelineItem {
    let Some(exit) = exit else { return item };
    let mut item = if exit.hold == Some(true) {
        hold_last_frame(item, clip.duration_in_frames, exit.duration_in_frames)
    } else {
        item
    };
    item.base_mut().exit = Some(exit.clone());
    item
}

fn sequence_placements(
    clips: &[ClipValue],
    enters: &[Option<TransitionRef>],
) -> (Vec<Placement>, u32) {
    let mut at = Vec::new();
    let mut end: u32 = 0;
    for (clip, enter) in clips.iter().zip(enters) {
        let frames = enter.as_ref().map_or(0, |e| e.duration_in_frames);
        let held = enter.as_ref().is_some_and(|e| e.hold == Some(true));
        let from = if held {
            end
        } else {
            end.saturating_sub(frames)
        };
        let lead = if held { frames } else { 0 };
        at.push(Placement { from, lead });
        end = from + lead + clip.duration_in_frames;
    }
    (at, end)
}

#[derive(Debug, Clone, Default)]
pub struct SequenceSlots<'a> {
    pub ports: Option<&'a [String]>,
    pub slot_node: Option<&'a str>,
}

pub fn sequence_clips(
    node_id: &str,
    clips: &[ClipValue],
    transitions: &[Option<TransitionRef>],
    slots: SequenceSlots<'_>,
) -> ClipValue {
    if clips.is_empty() {
        return empty_clip(1);
    }
    let enters = resolve_enters(clips, transitions);
    let (at, end) = sequence_placements(clips, &enters);
    let items = clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            let slot = slots.ports.and_then(|p| p.get(i)).map(|port| Slot {
                node_id: slots.slot_node.unwrap_or(node_id).to_owned(),
                port: port.clone(),
            });
            let item = place_clip(
                format!("{node_id}:s{i}"),
                clip,
                at[i],
                enters[i].clone(),
                slot,
            );
            with_exit(item, clip, enters.get(i + 1).and_then(Option::as_ref))
        })
        .collect();
    let audio = clips
        .iter()
        .enumerate()
        .flat_map(|(i, clip)| {
            shift_audio(
                &clip.audio,
                at[i].from + at[i].lead,
                &format!("{node_id}:s{i}"),
            )
        })
        .collect();
    make_clip(
        end.max(1),
        vec![TimelineTrack {
            id: format!("{node_id}:seq"),
            items,
            layer: None,
        }],
        audio,
        first_recording(clips),
    )
}

fn layer_id(node_id: &str, i: usize) -> String {
    format!("{node_id}:l{i}")
}

pub type MoveFor<'a> = &'a dyn Fn(usize) -> Option<opsis_types::timeline::ParamRef>;

pub fn stack_clips(
    node_id: &str,
    clips: &[ClipValue],
    offsets: &[u32],
    ports: Option<&[String]>,
    move_for: Option<MoveFor<'_>>,
) -> ClipValue {
    if clips.is_empty() {
        return empty_clip(1);
    }
    let offset = |i: usize| offsets.get(i).copied().unwrap_or(0);
    let tracks = clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            let mut base = ItemBase::new(
                format!("{}:g", layer_id(node_id, i)),
                offset(i),
                clip.duration_in_frames,
            );
            if let Some(m) = move_for.and_then(|f| f(i)) {
                base.handles = Some(EditHandles {
                    move_to: Some(m),
                    ..EditHandles::default()
                });
            }
            base.slot = ports.and_then(|p| p.get(i)).map(|port| Slot {
                node_id: node_id.to_owned(),
                port: port.clone(),
            });
            TimelineTrack {
                id: layer_id(node_id, i),
                items: vec![group_item(
                    base,
                    GroupProps {
                        tracks: clip.tracks.clone(),
                        ..GroupProps::default()
                    },
                )],
                layer: None,
            }
        })
        .collect();
    let duration = clips
        .iter()
        .enumerate()
        .map(|(i, c)| c.duration_in_frames + offset(i))
        .fold(0, u32::max);
    let audio = clips
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            shift_audio(&c.audio, offset(i), &layer_id(node_id, i))
                .into_iter()
                .map(move |a| AudioItem {
                    layer_track: Some(layer_id(node_id, i)),
                    ..a
                })
        })
        .collect();
    make_clip(duration, tracks, audio, first_recording(clips))
}

pub fn mark_layers(clip: ClipValue) -> ClipValue {
    ClipValue {
        tracks: clip
            .tracks
            .into_iter()
            .map(|t| TimelineTrack {
                layer: Some(true),
                ..t
            })
            .collect(),
        ..clip
    }
}

fn is_layered(clip: &ClipValue) -> bool {
    !clip.tracks.is_empty() && clip.tracks.iter().all(|t| t.layer == Some(true))
}

pub fn scene_item_count(clip: &ClipValue) -> usize {
    if is_layered(clip) {
        clip.tracks.len()
    } else {
        1
    }
}

struct Scene {
    tracks: Vec<TimelineTrack>,
    audio: Vec<AudioItem>,
    duration_in_frames: u32,
}

fn layer_start(track: &TimelineTrack) -> u32 {
    track.items.first().map_or(0, |i| i.base().from)
}

fn start_with_scene(item: &TimelineItem, by: u32) -> TimelineItem {
    let mut item = item.clone();
    let base = item.base_mut();
    base.from = base.from.saturating_sub(by);
    base.handles = base
        .handles
        .take()
        .map(|h| EditHandles { move_to: None, ..h })
        .filter(|h| *h != EditHandles::default());
    item
}

fn layers_start_together(clip: &ClipValue) -> Scene {
    let tracks: Vec<TimelineTrack> = clip
        .tracks
        .iter()
        .map(|t| TimelineTrack {
            items: t
                .items
                .iter()
                .map(|i| start_with_scene(i, layer_start(t)))
                .collect(),
            ..t.clone()
        })
        .collect();
    let audio = clip
        .audio
        .iter()
        .map(|a| {
            match clip
                .tracks
                .iter()
                .find(|t| Some(&t.id) == a.layer_track.as_ref())
            {
                Some(owner) => AudioItem {
                    from: a.from - signed(layer_start(owner)),
                    ..a.clone()
                },
                None => a.clone(),
            }
        })
        .collect();
    let longest = tracks
        .iter()
        .flat_map(|t| {
            t.items
                .iter()
                .map(|i| i.base().from + i.base().duration_in_frames)
        })
        .fold(0, u32::max);
    Scene {
        tracks,
        audio,
        duration_in_frames: clip.duration_in_frames.min(longest),
    }
}

fn scene_of(id: &str, clip: &ClipValue) -> Scene {
    if is_layered(clip) {
        return layers_start_together(clip);
    }
    let whole = group_item(
        ItemBase::new(format!("{id}:i"), 0, clip.duration_in_frames),
        GroupProps {
            tracks: clip.tracks.clone(),
            ..GroupProps::default()
        },
    );
    Scene {
        tracks: one_track(id, whole),
        audio: clip.audio.clone(),
        duration_in_frames: clip.duration_in_frames,
    }
}

pub fn scene_sequence(
    node_id: &str,
    scenes: &[(String, ClipValue)],
    gap: u32,
    sequencer: SequencerProps,
) -> ClipValue {
    let built: Vec<(String, &String, Scene)> = scenes
        .iter()
        .enumerate()
        .map(|(i, (port, clip))| {
            let id = format!("{node_id}:s{i}");
            let scene = scene_of(&id, clip);
            (id, port, scene)
        })
        .collect();
    let mut starts = Vec::new();
    let mut end = 0;
    for (_, _, scene) in &built {
        starts.push(end);
        end += scene.duration_in_frames + gap;
    }
    let duration = end.saturating_sub(gap).max(1);
    let mut items = Vec::new();
    let mut audio = Vec::new();
    for (i, (id, port, scene)) in built.into_iter().enumerate() {
        let mut base = ItemBase::new(id.clone(), starts[i], scene.duration_in_frames);
        base.slot = Some(Slot {
            node_id: node_id.to_owned(),
            port: port.clone(),
        });
        items.push(group_item(
            base,
            GroupProps {
                tracks: scene.tracks,
                ..GroupProps::default()
            },
        ));
        audio.extend(shift_audio(&scene.audio, starts[i], &id));
    }
    let mut base = ItemBase::new(format!("{node_id}:g"), 0, duration);
    base.node_id = Some(node_id.to_owned());
    let group = group_item(
        base,
        GroupProps {
            tracks: vec![TimelineTrack {
                id: format!("{node_id}:scenes"),
                items,
                layer: None,
            }],
            sequencer: Some(sequencer),
            ..GroupProps::default()
        },
    );
    make_clip(
        duration,
        one_track(node_id, group),
        audio,
        first_recording(scenes.iter().map(|(_, c)| c)),
    )
}

pub fn map_items(
    tracks: &[TimelineTrack],
    f: &dyn Fn(TimelineItem) -> TimelineItem,
) -> Vec<TimelineTrack> {
    tracks
        .iter()
        .map(|t| TimelineTrack {
            items: t
                .items
                .iter()
                .map(|item| match f(item.clone()) {
                    TimelineItem::Group { base, props } => {
                        let tracks = map_items(&props.tracks, f);
                        TimelineItem::Group {
                            base,
                            props: Box::new(GroupProps { tracks, ..*props }),
                        }
                    }
                    other => other,
                })
                .collect(),
            ..t.clone()
        })
        .collect()
}

pub fn walk_items<'a>(tracks: &'a [TimelineTrack], visit: &mut dyn FnMut(&'a TimelineItem)) {
    for track in tracks {
        for item in &track.items {
            visit(item);
            if let TimelineItem::Group { props, .. } = item {
                walk_items(&props.tracks, visit);
            }
        }
    }
}

pub fn patch_recordings(
    clip: &ClipValue,
    patch: &dyn Fn(RecordingProps) -> RecordingProps,
) -> ClipValue {
    ClipValue {
        tracks: map_items(&clip.tracks, &|item| match item {
            TimelineItem::Recording { base, props } => TimelineItem::Recording {
                base,
                props: Box::new(patch(*props)),
            },
            other => other,
        }),
        ..clip.clone()
    }
}

pub fn clip_frame_of_recording(
    tracks: &[TimelineTrack],
    content: &dyn Fn(&RecordingProps) -> f64,
) -> Option<f64> {
    for track in tracks {
        for item in &track.items {
            match item {
                TimelineItem::Recording { base, props } => {
                    return Some(f64::from(base.from) + content(props));
                }
                TimelineItem::Group { base, props } => {
                    if let Some(inner) = clip_frame_of_recording(&props.tracks, content) {
                        let local = inner - f64::from(props.start_from.unwrap_or(0));
                        let outer = match props.remap.as_ref().filter(|r| !r.is_empty()) {
                            Some(remap) => inverse_remap(remap, local, RemapSide::Start),
                            None => local,
                        };
                        return Some(f64::from(base.from) + outer);
                    }
                }
                TimelineItem::Element { .. } => {}
            }
        }
    }
    None
}

pub fn recording_take(recording: &RecordingValue) -> Option<InputTake> {
    recording.log.as_ref().map(|log| InputTake {
        duration_ms: log.duration_ms,
        cursor_track: log.cursor_track.as_ref().is_some_and(|t| !t.is_empty()),
    })
}

pub fn input_take(clip: &ClipValue) -> Option<Option<InputTake>> {
    if let Some(take) = clip_recording(clip).and_then(recording_take) {
        return Some(Some(take));
    }
    let mut placeholder = false;
    walk_items(&clip.tracks, &mut |item| {
        if matches!(item, TimelineItem::Element { component, .. } if component == PLACEHOLDER) {
            placeholder = true;
        }
    });
    if placeholder { None } else { Some(None) }
}

pub fn recording_props(clip: &ClipValue) -> Vec<&RecordingProps> {
    let mut out = Vec::new();
    walk_items(&clip.tracks, &mut |item| {
        if let Some(props) = item.recording() {
            out.push(props);
        }
    });
    out
}

pub fn clip_to_spec(clip: &ClipValue, settings: &WorkflowSettings) -> TimelineSpec {
    TimelineSpec {
        width: settings.width,
        height: settings.height,
        fps: settings.fps,
        duration_in_frames: clip.duration_in_frames.max(1),
        background: Some(
            settings
                .background
                .clone()
                .unwrap_or_else(|| settings.theme.background().to_owned()),
        ),
        tracks: clip.tracks.clone(),
        audio: clip.audio.clone(),
        assets: None,
        reframe: None,
        quality: None,
        fonts: Some(film_fonts(&settings.fonts, &clip.tracks)),
        mix: None,
    }
}
