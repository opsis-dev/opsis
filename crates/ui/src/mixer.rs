use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use opsis_compile::catalog::EditorNode;
use opsis_compile::mix::controls::{level_at, sample_at};
use opsis_compile::mix::plan::MixPlan;
use opsis_qt::{LoudnessView, MeterView, MixerStripView, MixerView};
use opsis_types::mix::{ChannelKind, Mix, MixResult, mixer_book};
use opsis_types::workflow::Workflow;

const HOLD_SECONDS: f64 = 1.5;
const CHANNEL_FALLBACK: &str = "Sound";

#[derive(Debug, Default)]
struct MixState {
    requested: String,
    result: Option<Arc<MixResult>>,
    error: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Mixes(Arc<Mutex<MixState>>);

impl Mixes {
    pub fn want(&self, key: &str) -> bool {
        let mut state = self.0.lock().expect("mix state");
        if state.requested == key {
            return false;
        }
        key.clone_into(&mut state.requested);
        true
    }

    pub fn finish(&self, key: &str, outcome: Result<MixResult, String>) {
        let mut state = self.0.lock().expect("mix state");
        if state.requested != key {
            return;
        }
        match outcome {
            Ok(result) => {
                state.result = Some(Arc::new(result));
                state.error = None;
            }
            Err(error) => state.error = Some(error),
        }
    }

    pub fn latest(&self) -> Option<Arc<MixResult>> {
        self.0.lock().expect("mix state").result.clone()
    }

    fn status(&self) -> (bool, Option<String>) {
        let state = self.0.lock().expect("mix state");
        let outdated = state
            .result
            .as_ref()
            .is_none_or(|r| r.key != state.requested);
        (outdated && state.error.is_none(), state.error.clone())
    }
}

pub struct Sources<'a> {
    pub workflow: &'a Workflow,
    pub catalog: &'a IndexMap<String, EditorNode>,
    pub selected: &'a HashSet<String>,
    pub source_id: String,
}

fn node_name(src: &Sources<'_>, id: &str) -> String {
    src.workflow
        .nodes
        .iter()
        .find(|n| n.id == id)
        .and_then(|node| {
            node.label
                .clone()
                .filter(|label| !label.is_empty())
                .or_else(|| src.catalog.get(&node.kind).map(|def| def.title.clone()))
        })
        .unwrap_or_else(|| CHANNEL_FALLBACK.to_owned())
}

const fn kind_name(kind: ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Music => "music",
        ChannelKind::Effect => "effect",
        ChannelKind::Clip => "clip",
    }
}

fn settings<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned())
}

fn empty_strip() -> MixerStripView {
    MixerStripView {
        id: String::new(),
        name: String::new(),
        kind: String::new(),
        bus: String::new(),
        audible: true,
        selected: false,
        built_in: false,
        members: 0,
        settings_json: "{}".to_owned(),
    }
}

pub fn hidden() -> MixerView {
    MixerView {
        shown: false,
        stale: false,
        failed: false,
        error: String::new(),
        solo: false,
        source_id: String::new(),
        channels: Vec::new(),
        buses: Vec::new(),
        master: empty_strip(),
        book_json: settings(&mixer_book()),
    }
}

pub fn view(
    plan: Option<&MixPlan>,
    mix: &Mix,
    solo: bool,
    src: &Sources<'_>,
    mixes: &Mixes,
) -> MixerView {
    let book = mixer_book();
    let book_json = settings(&book);
    let Some(plan) = plan.filter(|p| !p.channels.is_empty()) else {
        return MixerView {
            shown: false,
            stale: false,
            failed: false,
            error: String::new(),
            solo,
            source_id: src.source_id.clone(),
            channels: Vec::new(),
            buses: Vec::new(),
            master: empty_strip(),
            book_json,
        };
    };
    let (stale, error) = mixes.status();
    MixerView {
        shown: true,
        stale,
        failed: error.is_some(),
        error: error.unwrap_or_default(),
        solo,
        source_id: src.source_id.clone(),
        channels: plan
            .channels
            .iter()
            .map(|channel| MixerStripView {
                id: channel.id.clone(),
                name: node_name(src, &channel.id),
                kind: kind_name(channel.kind).to_owned(),
                bus: channel.bus.clone(),
                audible: channel.audible,
                selected: src.selected.contains(&channel.id),
                built_in: false,
                members: 0,
                settings_json: settings(&channel.strip),
            })
            .collect(),
        buses: mix
            .buses
            .iter()
            .map(|bus| MixerStripView {
                id: bus.id.clone(),
                name: bus.name.clone(),
                kind: "bus".to_owned(),
                bus: bus.id.clone(),
                audible: true,
                selected: false,
                built_in: book.built_in_buses.contains(&bus.id),
                members: u32::try_from(plan.channels.iter().filter(|c| c.bus == bus.id).count())
                    .unwrap_or(u32::MAX),
                settings_json: settings(bus),
            })
            .collect(),
        master: MixerStripView {
            id: "master".to_owned(),
            name: "Master".to_owned(),
            kind: "master".to_owned(),
            settings_json: settings(&mix.master),
            ..empty_strip()
        },
        book_json,
    }
}

pub fn meter(result: Option<&MixResult>, target: &str, frame: f64) -> MeterView {
    let none = MeterView {
        has: false,
        peak: f64::NAN,
        rms: f64::NAN,
        hold: f64::NAN,
        reduction: f64::NAN,
    };
    let Some(result) = result else {
        return none;
    };
    let (levels, reduction) = if target == "master" {
        (
            result.levels.master.as_ref(),
            result.reduction.master.as_deref(),
        )
    } else if let Some(id) = target.strip_prefix("ch:") {
        (
            result.levels.channels.get(id),
            result.reduction.channels.get(id).map(Vec::as_slice),
        )
    } else if let Some(id) = target.strip_prefix("bus:") {
        (result.levels.buses.get(id), None)
    } else {
        (None, None)
    };
    let reduction = reduction.map_or(f64::NAN, |values| sample_at(values, frame).unwrap_or(0.0));
    let Some(levels) = levels else {
        return MeterView { reduction, ..none };
    };
    let hold = opsis_types::numbers::truncate::<usize>((levels.fps * HOLD_SECONDS).round());
    level_at(&levels.peak, &levels.rms, frame, hold).map_or(
        MeterView { reduction, ..none },
        |level| MeterView {
            has: true,
            peak: level.peak,
            rms: level.rms,
            hold: level.hold,
            reduction,
        },
    )
}

pub fn loudness(result: Option<&MixResult>, frame: f64) -> LoudnessView {
    let loudness = result.and_then(|r| r.loudness.as_ref());
    LoudnessView {
        momentary: loudness
            .and_then(|l| sample_at(&l.momentary, frame))
            .unwrap_or(f64::NAN),
        short_term: loudness
            .and_then(|l| sample_at(&l.short_term, frame))
            .unwrap_or(f64::NAN),
        integrated: loudness.and_then(|l| l.integrated).unwrap_or(f64::NAN),
        true_peak: loudness.and_then(|l| l.true_peak).unwrap_or(f64::NAN),
        range: loudness.and_then(|l| l.range).unwrap_or(f64::NAN),
    }
}
