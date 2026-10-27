use indexmap::IndexMap;
use opsis_types::json::sorted_keys;
use opsis_types::mix::{
    Bus, ChannelKind, Comp, Eq, Highpass, Master, Mix, Strip, StripSettings, strip_settings,
};
use opsis_types::numbers::whole;
use opsis_types::timeline::{AudioEnvelope, TimelineSpec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::mix::levels::{ChannelInfo, audible, bus_of, channels_in, duck_keys_for, strip_of};
use crate::time::at_least_one_frame;
use crate::timeline::flatten::FlatTimeline;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SoundPlacement {
    pub src: String,
    pub at: u32,
    pub offset: u32,
    pub length: u32,
    #[serde(rename = "loop")]
    pub looped: bool,
    pub rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    pub volume: f64,
    pub fade_in: u32,
    pub fade_out: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<AudioEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelPlan {
    pub id: String,
    pub kind: ChannelKind,
    pub bus: String,
    pub strip: Strip,
    pub sounds: Vec<SoundPlacement>,
    pub audible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct BusProcessing {
    pub highpass: Highpass,
    pub eq: Eq,
    pub comp: Comp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct BusDuck {
    pub keys: Vec<String>,
    pub amount: f64,
    pub attack: f64,
    pub release: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct BusPlan {
    pub id: String,
    pub name: String,
    pub processing: BusProcessing,
    pub fader: f64,
    pub channels: Vec<String>,
    pub duck: Option<BusDuck>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MixPlan {
    pub fps: u32,
    pub duration_in_frames: u32,
    pub channels: Vec<ChannelPlan>,
    pub buses: Vec<BusPlan>,
    pub master: Master,
    pub key: String,
}

fn sound_placements(flat: &FlatTimeline) -> IndexMap<String, Vec<SoundPlacement>> {
    let mut by_channel: IndexMap<String, Vec<SoundPlacement>> = IndexMap::new();
    let sounds = flat.audio.iter().flat_map(|a| {
        if a.parts.is_empty() {
            std::slice::from_ref(a)
        } else {
            a.parts.as_slice()
        }
    });
    for a in sounds {
        let item = a.item.as_ref();
        by_channel
            .entry(a.channel.clone())
            .or_default()
            .push(SoundPlacement {
                src: a.src.clone(),
                at: whole(a.from),
                offset: whole(a.trim_before),
                length: at_least_one_frame(a.to - a.from),
                looped: a.looped,
                rate: item.and_then(|i| i.rate).unwrap_or(1.0),
                pitch: item.and_then(|i| i.pitch),
                volume: a.volume,
                fade_in: item.and_then(|i| i.fade_in_frames).unwrap_or(0),
                fade_out: item.and_then(|i| i.fade_out_frames).unwrap_or(0),
                envelope: item.and_then(|i| i.envelope.clone()),
            });
    }
    by_channel
}

fn bus_plans(
    mix: &Mix,
    channels: &[ChannelPlan],
    placed: &[ChannelInfo],
    fps: u32,
) -> Vec<BusPlan> {
    mix.buses
        .iter()
        .map(|bus: &Bus| {
            let keys = duck_keys_for(mix, bus, placed, fps);
            BusPlan {
                id: bus.id.clone(),
                name: bus.name.clone(),
                processing: BusProcessing {
                    highpass: bus.highpass.clone(),
                    eq: bus.eq.clone(),
                    comp: bus.comp.clone(),
                },
                fader: bus.fader,
                channels: channels
                    .iter()
                    .filter(|c| c.bus == bus.id && c.audible)
                    .map(|c| c.id.clone())
                    .collect(),
                duck: (!keys.is_empty()).then_some(BusDuck {
                    keys,
                    amount: bus.duck.amount,
                    attack: bus.duck.attack,
                    release: bus.duck.release,
                }),
            }
        })
        .collect()
}

pub fn lookup_key(value: &impl Serialize) -> String {
    let text = serde_json::to_string(&sorted_keys(
        serde_json::to_value(value).expect("value serializes"),
    ))
    .expect("json");
    format!("{:016x}", xxhash_rust::xxh3::xxh3_64(text.as_bytes()))
}

pub fn build_mix_plan(spec: &TimelineSpec, mix: Option<&Mix>, flat: &FlatTimeline) -> MixPlan {
    let default = Mix::default();
    let mix = mix.unwrap_or(&default);
    let mut sounds = sound_placements(flat);
    let placed = channels_in(spec, flat);
    let channels: Vec<ChannelPlan> = placed
        .iter()
        .map(|c| ChannelPlan {
            id: c.id.clone(),
            kind: c.kind,
            bus: bus_of(mix, &c.id, c.kind).id.clone(),
            strip: strip_of(mix, &c.id).into_owned(),
            sounds: sounds.shift_remove(&c.id).unwrap_or_default(),
            audible: audible(mix, &c.id, c.kind),
        })
        .collect();
    let buses = bus_plans(mix, &channels, &placed, spec.fps);
    let mut plan = MixPlan {
        fps: spec.fps,
        duration_in_frames: spec.duration_in_frames,
        channels,
        buses,
        master: mix.master.clone(),
        key: String::new(),
    };
    let mut body = serde_json::to_value(&plan).expect("plan serializes");
    if let Value::Object(map) = &mut body {
        map.remove("key");
    }
    plan.key = lookup_key(&body);
    plan
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelIdentity<'a> {
    pub fps: u32,
    pub duration_in_frames: u32,
    pub sounds: &'a [SoundPlacement],
    pub strip: StripSettings,
}

pub fn channel_identity<'a>(plan: &MixPlan, channel: &'a ChannelPlan) -> ChannelIdentity<'a> {
    ChannelIdentity {
        fps: plan.fps,
        duration_in_frames: plan.duration_in_frames,
        sounds: &channel.sounds,
        strip: strip_settings(&channel.strip),
    }
}

pub fn has_sound(plan: &MixPlan) -> bool {
    plan.channels.iter().any(|c| !c.sounds.is_empty())
}
