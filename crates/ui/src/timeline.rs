use std::collections::HashSet;

use indexmap::IndexMap;
use opsis_compile::catalog::EditorNode;
use opsis_compile::edits::timeline::Film;
use opsis_compile::envelope::{envelope_track, sample_points};
use opsis_compile::time::ms_to_exact_frames;
use opsis_compile::timeline::flatten::{AudioBar, BarKind, FlatTimeline, VideoBar};
use opsis_compile::timeline::media::{
    WAVE_FILL, WaveSpan, pick_level, tile_index, tile_start, wave_column,
};
use opsis_qt::{
    Choice, FilmTile, KeyLineView, KeyPointView, TimelineAudioView, TimelineBarView,
    TimelineCameraKeyView, TimelineMarkView, TimelineMarkerView, TimelineTransitionView,
    TimelineView,
};
use opsis_types::actions::ActionKind;
use opsis_types::analysis::{Filmstrip, Waveform};
use opsis_types::mix::ChannelKind;
use opsis_types::timeline::{EnvelopeKey, Keyframe};
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
        video_tracks: Vec::new(),
        audio_tracks: Vec::new(),
    }
}

fn track_choices(audio: bool) -> Vec<Choice> {
    opsis_compile::envelope::envelope_tracks()
        .into_iter()
        .filter(|track| track.audio == audio)
        .filter_map(|track| {
            let value = serde_json::to_value(track.key).ok()?.as_str()?.to_owned();
            Some(Choice {
                value,
                label: track.label.to_owned(),
            })
        })
        .collect()
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
        video_tracks: track_choices(false),
        audio_tracks: track_choices(true),
    }
}

const END_EPSILON: f64 = 0.001;
const MS_PER_SECOND: f64 = 1000.0;

pub struct Strip<'a> {
    pub filmstrip: &'a Filmstrip,
    pub speed: f64,
    pub trim_ms: f64,
}

pub fn film_tiles(
    bar: &VideoBar,
    fps: u32,
    strip: &Strip<'_>,
    px_per_frame: f64,
    height: f64,
    offset: f64,
    width: f64,
) -> Vec<FilmTile> {
    let film = strip.filmstrip;
    if film.tile_height == 0 || px_per_frame <= 0.0 {
        return Vec::new();
    }
    let tile_width = f64::from(film.tile_width) / f64::from(film.tile_height) * height;
    let left = bar.from * px_per_frame;
    let mut tiles = Vec::new();
    let mut x = tile_start(left, offset, tile_width);
    while x < offset + width && tile_width > 0.0 {
        let abs = (left + x + tile_width / 2.0) / px_per_frame;
        let at = abs.clamp(bar.from, (bar.to - END_EPSILON).max(bar.from));
        let local = bar.local_at(at);
        let ms =
            (local / f64::from(fps.max(1)) * strip.speed).mul_add(MS_PER_SECOND, strip.trim_ms);
        tiles.push(FilmTile {
            x,
            width: tile_width,
            index: tile_index(ms, f64::from(film.interval_ms), film.count),
        });
        x += tile_width;
    }
    tiles
}

pub fn wave_columns(
    bar: &AudioBar,
    fps: u32,
    wave: &Waveform,
    px_per_frame: f64,
    offset: f64,
    width: u32,
) -> Vec<f32> {
    let fps = f64::from(fps.max(1));
    let Some(level) = pick_level(&wave.levels, px_per_frame * fps) else {
        return Vec::new();
    };
    let span = WaveSpan {
        from: bar.from,
        to: bar.to,
        trim_before: bar.trim_before,
        looped: bar.looped,
        volume: bar.volume,
        duration_s: f64::from(wave.duration_ms) / MS_PER_SECOND,
    };
    let left = bar.from * px_per_frame;
    (0..width)
        .flat_map(|i| {
            let abs = (left + offset + f64::from(i)) / px_per_frame;
            wave_column(level, span, abs, fps, 1.0).map_or([f32::NAN, f32::NAN], |(high, low)| {
                [
                    opsis_types::numbers::narrow(high * WAVE_FILL),
                    opsis_types::numbers::narrow(low * WAVE_FILL),
                ]
            })
        })
        .collect()
}

fn has_envelope(def: Option<&EditorNode>) -> bool {
    def.and_then(|d| d.params.pointer("/properties/envelope"))
        .is_some()
}

pub const fn hidden_line() -> KeyLineView {
    KeyLineView {
        shown: false,
        node_id: String::new(),
        label: String::new(),
        span: 1.0,
        scale: 1.0,
        points: Vec::new(),
        line: Vec::new(),
        volume: -1.0,
    }
}

struct BarSpan {
    node_id: String,
    from: f64,
    to: f64,
    content_start: f64,
    rate: f64,
    scale: f64,
    volume: Option<f64>,
}

fn bar_span(flat: &FlatTimeline, bar: &str, key: EnvelopeKey) -> Option<BarSpan> {
    if let Some(video) = flat.bars.iter().find(|b| b.key == bar) {
        return Some(BarSpan {
            node_id: video.node_id.clone(),
            from: video.from,
            to: video.to,
            content_start: video.content_start,
            rate: video.rate,
            scale: 1.0,
            volume: None,
        });
    }
    let audio = flat.audio.iter().find(|b| b.key == bar)?;
    let volume = key == EnvelopeKey::Volume;
    Some(BarSpan {
        node_id: audio.node_id.clone().unwrap_or_default(),
        from: audio.from,
        to: audio.to,
        content_start: 0.0,
        rate: 1.0,
        scale: if volume { audio.volume } else { 1.0 },
        volume: volume.then_some(audio.volume),
    })
}

pub fn key_line(flat: &FlatTimeline, src: &Sources<'_>, bar: &str, track: &str) -> KeyLineView {
    let Ok(key) =
        serde_json::from_value::<EnvelopeKey>(serde_json::Value::String(track.to_owned()))
    else {
        return hidden_line();
    };
    let Some(span) = bar_span(flat, bar, key) else {
        return hidden_line();
    };
    let info = envelope_track(key);
    let range = (info.max - info.min).max(f64::EPSILON);
    let fraction = |value: f64| value.mul_add(span.scale, -info.min) / range;
    let to_abs =
        |local: f64| span.from + (local - span.content_start) / span.rate.max(f64::EPSILON);
    let node = src.workflow.nodes.iter().find(|n| n.id == span.node_id);
    let def = node.and_then(|n| src.catalog.get(&n.kind));
    if node.is_none() || !has_envelope(def) {
        return KeyLineView {
            volume: span.volume.map_or(-1.0, |v| (v - info.min) / range),
            ..hidden_line()
        };
    }
    let keys: Vec<Keyframe> = node
        .and_then(|n| n.params.get("envelope"))
        .and_then(|e| e.get(track))
        .and_then(|k| serde_json::from_value(k.clone()).ok())
        .unwrap_or_default();
    let samples = sample_points(&keys);
    let first = samples.first().map_or(info.rest, |p| p.1);
    let last = samples.last().map_or(info.rest, |p| p.1);
    let mut line = vec![KeyPointView {
        frame: span.from,
        fraction: fraction(first),
        value: first,
        index: 0,
    }];
    line.extend(samples.iter().map(|&(frame, value)| KeyPointView {
        frame: to_abs(frame),
        fraction: fraction(value),
        value,
        index: 0,
    }));
    line.push(KeyPointView {
        frame: span.to,
        fraction: fraction(last),
        value: last,
        index: 0,
    });
    KeyLineView {
        shown: true,
        node_id: span.node_id.clone(),
        label: info.label.to_owned(),
        span: range,
        scale: span.scale,
        points: keys
            .iter()
            .enumerate()
            .map(|(index, k)| KeyPointView {
                frame: to_abs(k.frame),
                fraction: fraction(k.value),
                value: k.value,
                index: u32::try_from(index).unwrap_or(u32::MAX),
            })
            .collect(),
        line,
        volume: if keys.is_empty() {
            span.volume.map_or(-1.0, |v| (v - info.min) / range)
        } else {
            -1.0
        },
    }
}

pub fn bar_local(flat: &FlatTimeline, bar: &str, abs: f64) -> f64 {
    if let Some(video) = flat.bars.iter().find(|b| b.key == bar) {
        return video.local_at(abs);
    }
    flat.audio
        .iter()
        .find(|b| b.key == bar)
        .map_or(abs, |audio| abs - audio.from)
}

#[cfg(test)]
mod tests {
    use opsis_compile::timeline::flatten::AudioTag;
    use opsis_types::analysis::WaveformLevel;
    use proptest::prelude::*;

    use super::*;

    fn sound(from: f64, to: f64) -> AudioBar {
        AudioBar {
            tag: AudioTag::Audio,
            key: "a".to_owned(),
            id: "a".to_owned(),
            node_id: None,
            channel: "music".to_owned(),
            src: "project:music.mp3".to_owned(),
            role: ChannelKind::Music,
            from,
            to,
            lane: 0,
            trim_before: 0.0,
            looped: false,
            volume: 1.0,
            handles: None,
            item: None,
            bar: None,
            hits: None,
            parts: Vec::new(),
            rate: 1.0,
        }
    }

    fn loud(rate: f64, count: usize) -> Waveform {
        Waveform {
            src: "project:music.mp3".to_owned(),
            duration_ms: 60_000,
            levels: vec![WaveformLevel {
                rate,
                min: vec![-0.8; count],
                max: vec![0.8; count],
            }],
            beats: Vec::new(),
            bpm: None,
        }
    }

    proptest! {
        #[test]
        fn wave_columns_fill_only_the_bar_and_stay_in_its_height(
            from in 0f64..300.0,
            length in 1f64..300.0,
            px_per_frame in 0.1f64..8.0,
            offset in 0f64..400.0,
            width in 1u32..400,
        ) {
            let bar = sound(from, from + length);
            let columns = wave_columns(&bar, 30, &loud(200.0, 20_000), px_per_frame, offset, width);
            prop_assert_eq!(columns.len(), usize::try_from(width).unwrap_or(0) * 2);
            for (i, pair) in columns.chunks(2).enumerate() {
                let abs = (from.mul_add(px_per_frame, offset) + f64::from(u32::try_from(i).unwrap_or(0))) / px_per_frame;
                let inside = abs >= from && abs < from + length;
                prop_assert_eq!(!pair[0].is_nan(), inside);
                if inside {
                    prop_assert!((-1.0..=1.0).contains(&pair[0]) && (-1.0..=1.0).contains(&pair[1]));
                    prop_assert!(pair[0] >= pair[1]);
                }
            }
        }
    }
}
