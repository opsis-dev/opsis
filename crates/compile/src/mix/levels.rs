use indexmap::IndexMap;
use opsis_types::mix::{Bus, ChannelKind, Mix, Strip, db_to_gain};
use opsis_types::timeline::{AudioItem, Duck, TimelineItem, TimelineSpec, TimelineTrack};
use serde_json::Value;

use crate::audio::{audio_kind, channel_of, sound_length};
use crate::clip::{map_items, sound_node_id, video_of, walk_items};
use crate::time::{ms_to_exact_frames, ms_to_frames};
use crate::timeline::flatten::FlatTimeline;

pub fn strip_of<'a>(mix: &'a Mix, channel: &str) -> std::borrow::Cow<'a, Strip> {
    match mix.channels.get(channel) {
        Some(strip) => std::borrow::Cow::Borrowed(strip),
        None => std::borrow::Cow::Owned(Strip::default()),
    }
}

pub fn bus_of<'a>(mix: &'a Mix, channel: &str, kind: ChannelKind) -> &'a Bus {
    let strip = strip_of(mix, channel);
    let find = |id: &str| mix.buses.iter().find(|b| b.id == id);
    strip
        .bus
        .as_deref()
        .and_then(find)
        .or_else(|| find(kind.default_bus()))
        .unwrap_or(&mix.buses[0])
}

pub fn solo_active(mix: &Mix) -> bool {
    mix.channels.values().any(|s| s.solo) || mix.buses.iter().any(|b| b.solo)
}

pub fn audible(mix: &Mix, channel: &str, kind: ChannelKind) -> bool {
    let strip = strip_of(mix, channel);
    let bus = bus_of(mix, channel, kind);
    if strip.mute || bus.mute {
        return false;
    }
    !solo_active(mix) || strip.solo || bus.solo
}

fn channel_gain(mix: &Mix, channel: &str, kind: ChannelKind) -> f64 {
    if !audible(mix, channel, kind) {
        return 0.0;
    }
    let strip = strip_of(mix, channel);
    let bus = bus_of(mix, channel, kind);
    db_to_gain(strip.trim + strip.fader) * db_to_gain(bus.fader) * db_to_gain(mix.master.fader)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelInfo {
    pub id: String,
    pub kind: ChannelKind,
    pub keep_music: bool,
    pub length_frames: f64,
}

fn played_frames(a: &AudioItem) -> f64 {
    if a.looped == Some(true) {
        f64::INFINITY
    } else {
        a.length_frames
            .or(a.duration_in_frames)
            .map_or(f64::INFINITY, f64::from)
    }
}

pub fn channels_in(spec: &TimelineSpec, flat: &FlatTimeline) -> Vec<ChannelInfo> {
    let playing: std::collections::HashSet<&str> =
        flat.audio.iter().map(|a| a.channel.as_str()).collect();
    let mut out: IndexMap<String, ChannelInfo> = IndexMap::new();
    for a in &spec.audio {
        let id = channel_of(a);
        if !playing.contains(id.as_str()) {
            continue;
        }
        let prev = out.get(&id);
        let info = ChannelInfo {
            id: id.clone(),
            kind: audio_kind(a),
            keep_music: prev.is_none_or(|p| p.keep_music) && a.keep_music == Some(true),
            length_frames: prev.map_or(0.0, |p| p.length_frames).max(played_frames(a)),
        };
        out.insert(id, info);
    }
    walk_items(&spec.tracks, &mut |item| {
        if let Some(id) = sound_node_id(item).filter(|id| playing.contains(id)) {
            out.insert(
                id.to_owned(),
                ChannelInfo {
                    id: id.to_owned(),
                    kind: ChannelKind::Clip,
                    keep_music: false,
                    length_frames: f64::INFINITY,
                },
            );
        }
    });
    out.into_values().collect()
}

pub fn duck_keys_for(mix: &Mix, bus: &Bus, channels: &[ChannelInfo], fps: u32) -> Vec<String> {
    if !bus.duck.on {
        return Vec::new();
    }
    let shortest = ms_to_exact_frames(bus.duck.ignore_shorter_ms, fps);
    channels
        .iter()
        .filter(|c| {
            let from = &bus_of(mix, &c.id, c.kind).id;
            !c.keep_music
                && c.length_frames >= shortest
                && bus.duck.keys.contains(from)
                && *from != bus.id
        })
        .filter(|c| audible(mix, &c.id, c.kind))
        .map(|c| c.id.clone())
        .collect()
}

fn merged(mut spans: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    spans.retain(|(a, b)| b > a);
    spans.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut out: Vec<(f64, f64)> = Vec::new();
    for (a, b) in spans {
        match out.last_mut() {
            Some(last) if a <= last.1 => last.1 = last.1.max(b),
            _ => out.push((a, b)),
        }
    }
    out
}

fn clip_sounds(flat: &FlatTimeline) -> impl Iterator<Item = (&str, f64, f64)> {
    flat.audio
        .iter()
        .filter(|a| a.role == ChannelKind::Clip)
        .map(|a| (a.channel.as_str(), a.from, a.to))
}

pub fn duck_spans<'a>(
    spec: &TimelineSpec,
    clips: impl IntoIterator<Item = (&'a str, f64, f64)>,
    keys: &[String],
) -> Vec<(f64, f64)> {
    let sounds = spec
        .audio
        .iter()
        .filter(|a| a.keep_music != Some(true) && keys.contains(&channel_of(a)))
        .map(|a| {
            let from = f64::from(a.from);
            (from, from + sound_length(a, spec.duration_in_frames))
        });
    let clips = clips
        .into_iter()
        .filter(|(channel, _, _)| keys.iter().any(|k| k == channel))
        .map(|(_, from, to)| (from, to));
    merged(sounds.chain(clips).collect())
}

pub fn apply_mix_levels(
    spec: &TimelineSpec,
    mix: Option<&Mix>,
    flat: &FlatTimeline,
) -> TimelineSpec {
    let default = Mix::default();
    let mix = mix.unwrap_or(&default);
    let channels = channels_in(spec, flat);
    let duck_for: IndexMap<&str, Duck> = mix
        .buses
        .iter()
        .filter_map(|bus| {
            let keys = duck_keys_for(mix, bus, &channels, spec.fps);
            (!keys.is_empty()).then(|| {
                (
                    bus.id.as_str(),
                    Duck {
                        level: db_to_gain(-bus.duck.amount),
                        fade_frames: ms_to_frames(bus.duck.attack, spec.fps),
                        spans: duck_spans(spec, clip_sounds(flat), &keys),
                        keys,
                    },
                )
            })
        })
        .collect();
    let audio = spec
        .audio
        .iter()
        .map(|a| {
            let id = channel_of(a);
            let kind = audio_kind(a);
            AudioItem {
                gain: Some(channel_gain(mix, &id, kind)),
                duck: duck_for.get(bus_of(mix, &id, kind).id.as_str()).cloned(),
                ..a.clone()
            }
        })
        .collect();
    let tracks = map_items(&spec.tracks, &|item| {
        let Some(id) = sound_node_id(&item).map(str::to_owned) else {
            return item;
        };
        let gain = channel_gain(mix, &id, ChannelKind::Clip);
        let volume = video_of(&item).map_or(1.0, |v| v.volume);
        match item {
            TimelineItem::Element {
                base,
                component,
                mut props,
            } => {
                props.insert("volume".into(), Value::from(volume * gain));
                if gain == 0.0 {
                    props.insert("muted".into(), Value::Bool(true));
                }
                TimelineItem::Element {
                    base,
                    component,
                    props,
                }
            }
            other => other,
        }
    });
    TimelineSpec {
        audio,
        tracks,
        ..spec.clone()
    }
}

pub fn with_master_stem(spec: &TimelineSpec, src: &str, hash: &str) -> TimelineSpec {
    let tracks: Vec<TimelineTrack> = map_items(&spec.tracks, &|item| match item {
        TimelineItem::Element {
            base,
            component,
            mut props,
        } if item_has_sound_parts(&component, &props) => {
            props.insert("muted".into(), Value::Bool(true));
            TimelineItem::Element {
                base,
                component,
                props,
            }
        }
        other => other,
    });
    TimelineSpec {
        tracks,
        audio: Vec::new(),
        mix: Some(opsis_types::timeline::MasterStem {
            src: src.to_owned(),
            hash: hash.to_owned(),
        }),
        ..spec.clone()
    }
}

fn item_has_sound_parts(component: &str, props: &serde_json::Map<String, Value>) -> bool {
    crate::clip::video_props(component, props).is_some_and(|v| !v.muted)
}
