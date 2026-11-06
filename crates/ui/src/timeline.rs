use std::collections::HashSet;

use indexmap::IndexMap;
use opsis_compile::catalog::EditorNode;
use opsis_compile::edits::timeline::Film;
use opsis_compile::time::ms_to_exact_frames;
use opsis_compile::timeline::flatten::{BarKind, FlatTimeline};
use opsis_qt::{
    TimelineAudioView, TimelineBarView, TimelineCameraKeyView, TimelineMarkView,
    TimelineMarkerView, TimelineTransitionView, TimelineView,
};
use opsis_types::actions::ActionKind;
use opsis_types::mix::ChannelKind;
use opsis_types::workflow::Workflow;

const WRAPPER_SEPARATOR: &str = " · ";

pub struct Sources<'a> {
    pub workflow: &'a Workflow,
    pub catalog: &'a IndexMap<String, EditorNode>,
    pub transitions: &'a IndexMap<String, String>,
    pub selected: &'a HashSet<String>,
}

fn node_name(src: &Sources<'_>, id: &str) -> String {
    let Some(node) = src.workflow.nodes.iter().find(|n| n.id == id) else {
        return String::new();
    };
    node.label
        .clone()
        .filter(|label| !label.is_empty())
        .or_else(|| src.catalog.get(&node.kind).map(|def| def.title.clone()))
        .unwrap_or_default()
}

fn video_tone(src: &Sources<'_>, id: &str) -> &'static str {
    let category = src
        .workflow
        .nodes
        .iter()
        .find(|n| n.id == id)
        .and_then(|node| src.catalog.get(&node.kind))
        .map(|def| def.category_label.as_str());
    match category {
        Some("Text & overlays") => "track-text",
        Some("Elements" | "Backgrounds" | "Effects" | "Transitions") => "track-graphics",
        Some("Media") => "track-media",
        _ => "track-video",
    }
}

const fn audio_tone(role: ChannelKind) -> &'static str {
    match role {
        ChannelKind::Music => "track-music",
        ChannelKind::Effect => "track-effect",
        ChannelKind::Clip => "track-clip",
    }
}

fn bars(src: &Sources<'_>, flat: &FlatTimeline) -> Vec<TimelineBarView> {
    flat.bars
        .iter()
        .map(|bar| {
            let wrappers: Vec<String> = bar
                .chain
                .iter()
                .skip(1)
                .filter_map(|link| link.node_id.as_deref())
                .filter(|id| *id != bar.node_id)
                .map(|id| node_name(src, id))
                .collect();
            let selected = src.selected.contains(&bar.node_id);
            TimelineBarView {
                key: bar.key.clone(),
                node_id: bar.node_id.clone(),
                lane: bar.lane,
                from: bar.from,
                to: bar.to,
                title: node_name(src, &bar.node_id),
                wrappers: wrappers.join(WRAPPER_SEPARATOR),
                tone: video_tone(src, &bar.node_id).to_owned(),
                selected,
                related: !selected
                    && bar.chain.iter().any(|link| {
                        link.node_id
                            .as_ref()
                            .is_some_and(|id| src.selected.contains(id))
                    }),
                recording: bar.kind == BarKind::Recording,
            }
        })
        .collect()
}

fn audio(src: &Sources<'_>, flat: &FlatTimeline) -> Vec<TimelineAudioView> {
    flat.audio
        .iter()
        .map(|bar| {
            let node_id = bar.node_id.clone().unwrap_or_default();
            TimelineAudioView {
                key: bar.key.clone(),
                title: node_name(src, &node_id),
                selected: src.selected.contains(&node_id),
                node_id,
                lane: bar.lane,
                from: bar.from,
                to: bar.to,
                tone: audio_tone(bar.role).to_owned(),
                volume: bar.volume,
                hits: bar.hits.is_some(),
                bar_key: bar.bar.as_ref().map_or_default(|b| b.key.clone()),
            }
        })
        .collect()
}

pub const fn empty() -> TimelineView {
    TimelineView {
        has_film: false,
        fps: 30,
        duration: 1.0,
        lanes: 1,
        audio_lanes: 1,
        source_id: String::new(),
        source_label: String::new(),
        bars: Vec::new(),
        audio: Vec::new(),
        marks: Vec::new(),
        markers: Vec::new(),
        transitions: Vec::new(),
        camera_keys: Vec::new(),
    }
}

pub fn view(film: Option<&Film>, src: &Sources<'_>) -> TimelineView {
    let Some(film) = film else {
        return empty();
    };
    let flat = &film.flat;
    let source_id = film.source.clone().unwrap_or_default();
    TimelineView {
        has_film: true,
        fps: flat.fps,
        duration: f64::from(flat.duration_in_frames.max(1)),
        lanes: flat.lanes.max(1),
        audio_lanes: flat.audio_lanes.max(1),
        source_label: node_name(src, &source_id),
        source_id,
        bars: bars(src, flat),
        audio: audio(src, flat),
        marks: flat
            .marks
            .iter()
            .map(|mark| TimelineMarkView {
                key: mark.key.clone(),
                frame: mark.frame,
                label: mark.label.clone(),
                chapter: mark.kind == ActionKind::Chapter,
                bar_key: mark.bar_key.clone(),
            })
            .collect(),
        markers: src
            .workflow
            .markers
            .iter()
            .map(|marker| TimelineMarkerView {
                id: marker.id.clone(),
                frame: ms_to_exact_frames(marker.at_ms, flat.fps),
                label: marker.label.clone(),
            })
            .collect(),
        transitions: flat
            .transitions
            .iter()
            .map(|mark| TimelineTransitionView {
                key: mark.key.clone(),
                from: mark.from,
                to: mark.to,
                lanes: mark.lanes.clone(),
                label: src
                    .transitions
                    .get(&mark.kind)
                    .cloned()
                    .unwrap_or_else(|| mark.kind.clone()),
            })
            .collect(),
        camera_keys: flat
            .camera_keys
            .iter()
            .filter_map(|mark| {
                let bar = flat.bars.iter().find(|bar| {
                    (bar.item_id == mark.item_id
                        || bar.chain.iter().any(|link| link.item_id == mark.item_id))
                        && mark.frame >= bar.from
                        && mark.frame <= bar.to
                })?;
                Some(TimelineCameraKeyView {
                    key: mark.key.clone(),
                    node_id: mark.node_id.clone(),
                    bar_key: bar.key.clone(),
                    frame: mark.frame,
                })
            })
            .collect(),
    }
}
