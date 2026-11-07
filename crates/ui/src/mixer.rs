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

const MIN_POWER: f64 = 1e-9;
const HIGHPASS_Q: f64 = 0.7;

#[derive(Debug, Clone, Default)]
struct Contribution {
    gain_db: f64,
    pan: f64,
    filters: Vec<opsis_audio::output::PreviewFilter>,
}

#[derive(Debug, Default)]
struct PreviewState {
    contributions: IndexMap<String, Contribution>,
    baselines: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Default, Clone)]
pub struct Previews(Arc<Mutex<PreviewState>>);

pub fn share_of(result: Option<&MixResult>, target: &str, frame: f64) -> f64 {
    let Some(result) = result else {
        return 1.0;
    };
    let Some(master) = result.levels.master.as_ref() else {
        return 1.0;
    };
    let levels = if let Some(id) = target.strip_prefix("ch:") {
        result.levels.channels.get(id)
    } else if let Some(id) = target.strip_prefix("bus:") {
        result.levels.buses.get(id)
    } else {
        return 1.0;
    };
    levels.map_or(1.0, |l| {
        opsis_compile::mix::controls::share(&master.rms, &l.rms, frame)
    })
}

pub fn gain_contribution(fraction: f64, delta_db: f64) -> f64 {
    let ratio = fraction
        .mul_add(10f64.powf(delta_db / 10.0), 1.0 - fraction)
        .max(MIN_POWER);
    10.0 * ratio.log10()
}

impl Previews {
    pub fn clear(&self) {
        let mut state = self.0.lock().expect("preview");
        state.contributions.clear();
        state.baselines.clear();
    }

    pub fn change(
        &self,
        key: String,
        fraction: f64,
        kind: &str,
        current: f64,
        next: f64,
        band: Option<opsis_types::mix::EqBand>,
    ) -> opsis_audio::output::Preview {
        let mut state = self.0.lock().expect("preview");
        let baseline = *state.baselines.entry(key.clone()).or_insert(current);
        let delta = next - baseline;
        let contribution = if kind == "pan" {
            Contribution {
                pan: delta * fraction,
                ..Contribution::default()
            }
        } else if kind == "highpass" {
            Contribution {
                filters: vec![opsis_audio::output::PreviewFilter {
                    kind: opsis_audio::output::FilterKind::Highpass,
                    freq: next,
                    gain: 0.0,
                    q: HIGHPASS_Q,
                }],
                ..Contribution::default()
            }
        } else if let Some(band) = band {
            Contribution {
                filters: vec![opsis_audio::output::PreviewFilter {
                    kind: opsis_audio::output::FilterKind::Band(band.kind),
                    freq: band.freq,
                    gain: delta * fraction,
                    q: band.q,
                }],
                ..Contribution::default()
            }
        } else {
            Contribution {
                gain_db: gain_contribution(fraction, delta),
                ..Contribution::default()
            }
        };
        state.contributions.insert(key, contribution);
        let mut total = opsis_audio::output::Preview::default();
        for c in state.contributions.values() {
            total.gain_db += c.gain_db;
            total.pan += c.pan;
            total.filters.extend(c.filters.iter().copied());
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::gain_contribution;

    proptest! {
        #[test]
        fn a_strip_moves_the_mix_by_its_share(delta in -60f64..12.0, fraction in 0f64..1.0, other in -60f64..12.0) {
            prop_assert!((gain_contribution(1.0, delta) - delta).abs() < 1e-6 || delta < -89.0);
            prop_assert!(gain_contribution(0.0, delta).abs() < 1e-9);
            let (low, high) = if delta <= other { (delta, other) } else { (other, delta) };
            prop_assert!(gain_contribution(fraction, low) <= gain_contribution(fraction, high) + 1e-12);
        }
    }
}
