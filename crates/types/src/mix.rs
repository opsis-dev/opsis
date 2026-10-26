use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::edits::StripPatch;

pub const SAMPLE_RATE: u32 = 48_000;
pub const MIN_DB: f64 = -60.0;
pub const SILENCE_DB: f64 = -120.0;
pub const MAX_DB: f64 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct NumberRange {
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

pub const FADER_RANGE: NumberRange = NumberRange {
    min: MIN_DB,
    max: MAX_DB,
    default: 0.0,
};
pub const TRIM_RANGE: NumberRange = NumberRange {
    min: -24.0,
    max: 24.0,
    default: 0.0,
};
pub const PAN_RANGE: NumberRange = NumberRange {
    min: -1.0,
    max: 1.0,
    default: 0.0,
};
pub const HIGHPASS_RANGE: NumberRange = NumberRange {
    min: 20.0,
    max: 1000.0,
    default: 80.0,
};
pub const LIMITER_CEILING_RANGE: NumberRange = NumberRange {
    min: -12.0,
    max: 0.0,
    default: -1.0,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct CompRanges {
    pub threshold: NumberRange,
    pub ratio: NumberRange,
    pub attack: NumberRange,
    pub release: NumberRange,
    pub makeup: NumberRange,
}

pub const COMP_RANGES: CompRanges = CompRanges {
    threshold: NumberRange {
        min: -60.0,
        max: 0.0,
        default: -18.0,
    },
    ratio: NumberRange {
        min: 1.0,
        max: 20.0,
        default: 3.0,
    },
    attack: NumberRange {
        min: 0.1,
        max: 500.0,
        default: 10.0,
    },
    release: NumberRange {
        min: 5.0,
        max: 3000.0,
        default: 150.0,
    },
    makeup: NumberRange {
        min: 0.0,
        max: 24.0,
        default: 0.0,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuckRanges {
    pub amount: NumberRange,
    pub attack: NumberRange,
    pub release: NumberRange,
    pub ignore_shorter_ms: NumberRange,
}

pub const DUCK_RANGES: DuckRanges = DuckRanges {
    amount: NumberRange {
        min: 1.0,
        max: 24.0,
        default: 10.0,
    },
    attack: NumberRange {
        min: 5.0,
        max: 2000.0,
        default: 80.0,
    },
    release: NumberRange {
        min: 20.0,
        max: 5000.0,
        default: 400.0,
    },
    ignore_shorter_ms: NumberRange {
        min: 0.0,
        max: 5000.0,
        default: 0.0,
    },
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum EqType {
    Lowshelf,
    Peak,
    Highshelf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct EqBand {
    #[serde(rename = "type")]
    pub kind: EqType,
    pub freq: f64,
    pub gain: f64,
    pub q: f64,
}

const fn band(kind: EqType, freq: f64, gain: f64, q: f64) -> EqBand {
    EqBand {
        kind,
        freq,
        gain,
        q,
    }
}

pub fn eq_bands(
    low: [f64; 3],
    low_mid: [f64; 3],
    high_mid: [f64; 3],
    high: [f64; 3],
) -> Vec<EqBand> {
    vec![
        band(EqType::Lowshelf, low[0], low[1], low[2]),
        band(EqType::Peak, low_mid[0], low_mid[1], low_mid[2]),
        band(EqType::Peak, high_mid[0], high_mid[1], high_mid[2]),
        band(EqType::Highshelf, high[0], high[1], high[2]),
    ]
}

pub fn default_eq_bands() -> Vec<EqBand> {
    eq_bands(
        [120.0, 0.0, 0.7],
        [500.0, 0.0, 1.0],
        [2500.0, 0.0, 1.0],
        [8000.0, 0.0, 0.7],
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Eq {
    #[serde(default)]
    pub on: bool,
    #[serde(default = "default_eq_bands")]
    pub bands: Vec<EqBand>,
}

impl Default for Eq {
    fn default() -> Self {
        Self {
            on: false,
            bands: default_eq_bands(),
        }
    }
}

fn threshold() -> f64 {
    COMP_RANGES.threshold.default
}
fn ratio() -> f64 {
    COMP_RANGES.ratio.default
}
fn attack() -> f64 {
    COMP_RANGES.attack.default
}
fn release() -> f64 {
    COMP_RANGES.release.default
}
fn makeup() -> f64 {
    COMP_RANGES.makeup.default
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Comp {
    #[serde(default)]
    pub on: bool,
    #[serde(default = "threshold")]
    pub threshold: f64,
    #[serde(default = "ratio")]
    pub ratio: f64,
    #[serde(default = "attack")]
    pub attack: f64,
    #[serde(default = "release")]
    pub release: f64,
    #[serde(default = "makeup")]
    pub makeup: f64,
}

impl Default for Comp {
    fn default() -> Self {
        Self {
            on: false,
            threshold: threshold(),
            ratio: ratio(),
            attack: attack(),
            release: release(),
            makeup: makeup(),
        }
    }
}

fn highpass_freq() -> f64 {
    HIGHPASS_RANGE.default
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Highpass {
    #[serde(default)]
    pub on: bool,
    #[serde(default = "highpass_freq")]
    pub freq: f64,
}

impl Default for Highpass {
    fn default() -> Self {
        Self {
            on: false,
            freq: highpass_freq(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Strip {
    #[serde(default)]
    pub trim: f64,
    #[serde(default)]
    pub highpass: Highpass,
    #[serde(default)]
    pub eq: Eq,
    #[serde(default)]
    pub comp: Comp,
    #[serde(default)]
    pub fader: f64,
    #[serde(default)]
    pub pan: f64,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub solo: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,
}

fn duck_amount() -> f64 {
    DUCK_RANGES.amount.default
}
fn duck_attack() -> f64 {
    DUCK_RANGES.attack.default
}
fn duck_release() -> f64 {
    DUCK_RANGES.release.default
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DuckSettings {
    #[serde(default)]
    pub on: bool,
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default = "duck_amount")]
    pub amount: f64,
    #[serde(default = "duck_attack")]
    pub attack: f64,
    #[serde(default = "duck_release")]
    pub release: f64,
    #[serde(default)]
    pub ignore_shorter_ms: f64,
}

impl Default for DuckSettings {
    fn default() -> Self {
        Self {
            on: false,
            keys: Vec::new(),
            amount: duck_amount(),
            attack: duck_attack(),
            release: duck_release(),
            ignore_shorter_ms: DUCK_RANGES.ignore_shorter_ms.default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Bus {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub highpass: Highpass,
    #[serde(default)]
    pub eq: Eq,
    #[serde(default)]
    pub comp: Comp,
    #[serde(default)]
    pub fader: f64,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub solo: bool,
    #[serde(default)]
    pub duck: DuckSettings,
}

impl Bus {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            highpass: Highpass::default(),
            eq: Eq::default(),
            comp: Comp::default(),
            fader: FADER_RANGE.default,
            mute: false,
            solo: false,
            duck: DuckSettings::default(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum LoudnessTarget {
    #[default]
    Off,
    Web,
    Podcast,
    Broadcast,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LoudnessGoal {
    pub lufs: f64,
    pub true_peak: f64,
}

impl LoudnessTarget {
    pub fn target(self) -> Option<LoudnessGoal> {
        match self {
            Self::Off => None,
            Self::Web => Some(LoudnessGoal {
                lufs: -14.0,
                true_peak: -1.0,
            }),
            Self::Podcast => Some(LoudnessGoal {
                lufs: -16.0,
                true_peak: -1.0,
            }),
            Self::Broadcast => Some(LoudnessGoal {
                lufs: -23.0,
                true_peak: -2.0,
            }),
        }
    }
}

fn limiter_ceiling() -> f64 {
    LIMITER_CEILING_RANGE.default
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Limiter {
    #[serde(default)]
    pub on: bool,
    #[serde(default = "limiter_ceiling")]
    pub ceiling: f64,
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            on: false,
            ceiling: limiter_ceiling(),
        }
    }
}

fn master_comp() -> Comp {
    Comp {
        threshold: -12.0,
        ratio: 2.0,
        attack: 20.0,
        release: 250.0,
        ..Comp::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Master {
    #[serde(default)]
    pub fader: f64,
    #[serde(default)]
    pub eq: Eq,
    #[serde(default = "master_comp")]
    pub comp: Comp,
    #[serde(default)]
    pub limiter: Limiter,
    #[serde(default)]
    pub loudness: LoudnessTarget,
}

impl Default for Master {
    fn default() -> Self {
        Self {
            fader: FADER_RANGE.default,
            eq: Eq::default(),
            comp: master_comp(),
            limiter: Limiter::default(),
            loudness: LoudnessTarget::Off,
        }
    }
}

pub const MUSIC_BUS: &str = "music";
pub const EFFECTS_BUS: &str = "effects";
pub const CLIPS_BUS: &str = "clips";

pub fn default_buses() -> Vec<Bus> {
    vec![
        Bus::new(MUSIC_BUS, "Music"),
        Bus::new(EFFECTS_BUS, "Effects"),
        Bus::new(CLIPS_BUS, "Clips"),
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct Mix {
    #[serde(default)]
    pub channels: IndexMap<String, Strip>,
    #[serde(default = "default_buses")]
    pub buses: Vec<Bus>,
    #[serde(default)]
    pub master: Master,
}

impl Default for Mix {
    fn default() -> Self {
        Self {
            channels: IndexMap::new(),
            buses: default_buses(),
            master: Master::default(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ChannelKind {
    Music,
    Effect,
    Clip,
}

impl ChannelKind {
    pub fn default_bus(self) -> &'static str {
        match self {
            Self::Music => MUSIC_BUS,
            Self::Effect => EFFECTS_BUS,
            Self::Clip => CLIPS_BUS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct StripSettings {
    pub trim: f64,
    pub highpass: Highpass,
    pub eq: Eq,
    pub comp: Comp,
    pub fader: f64,
    pub pan: f64,
}

pub fn strip_settings(strip: &Strip) -> StripSettings {
    StripSettings {
        trim: strip.trim,
        highpass: strip.highpass.clone(),
        eq: strip.eq.clone(),
        comp: strip.comp.clone(),
        fader: strip.fader,
        pan: strip.pan,
    }
}

pub fn db_to_gain(db: f64) -> f64 {
    if db <= MIN_DB {
        0.0
    } else {
        10f64.powf(db / 20.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct StemLevels {
    pub fps: f64,
    pub peak: Vec<f64>,
    pub rms: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Loudness {
    pub fps: f64,
    pub momentary: Vec<f64>,
    pub short_term: Vec<f64>,
    pub integrated: Option<f64>,
    pub range: Option<f64>,
    pub true_peak: Option<f64>,
}

pub use crate::timeline::MasterStem;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
pub struct MixLevels {
    pub channels: IndexMap<String, StemLevels>,
    pub buses: IndexMap<String, StemLevels>,
    pub master: Option<StemLevels>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
pub struct MixReduction {
    pub channels: IndexMap<String, Vec<f64>>,
    pub master: Option<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MixResult {
    pub key: String,
    pub master: Option<MasterStem>,
    pub levels: MixLevels,
    pub reduction: MixReduction,
    pub loudness: Option<Loudness>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct BandBounds {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct EqBandBounds {
    pub freq: BandBounds,
    pub gain: BandBounds,
    pub q: BandBounds,
}

pub const EQ_BAND_BOUNDS: EqBandBounds = EqBandBounds {
    freq: BandBounds {
        min: 20.0,
        max: 20_000.0,
    },
    gain: BandBounds {
        min: -18.0,
        max: 18.0,
    },
    q: BandBounds {
        min: 0.1,
        max: 10.0,
    },
};

pub const LOUDNESS_TARGETS: [LoudnessTarget; 4] = [
    LoudnessTarget::Off,
    LoudnessTarget::Web,
    LoudnessTarget::Podcast,
    LoudnessTarget::Broadcast,
];

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LoudnessChoice {
    pub target: LoudnessTarget,
    pub goal: Option<LoudnessGoal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct StripPreset {
    pub id: String,
    pub label: String,
    pub strip: StripPatch,
}

fn eq_on(bands: Vec<EqBand>) -> Eq {
    Eq { on: true, bands }
}

fn comp_on(threshold: f64, ratio: f64, attack: f64, release: f64, makeup: f64) -> Comp {
    Comp {
        on: true,
        threshold,
        ratio,
        attack,
        release,
        makeup,
    }
}

fn highpass_on(freq: f64) -> Highpass {
    Highpass { on: true, freq }
}

pub fn strip_presets() -> Vec<StripPreset> {
    let preset = |id: &str, label: &str, strip: StripPatch| StripPreset {
        id: id.to_owned(),
        label: label.to_owned(),
        strip,
    };
    vec![
        preset(
            "voice",
            "Voice",
            StripPatch {
                highpass: Some(highpass_on(90.0)),
                eq: Some(eq_on(eq_bands(
                    [150.0, -2.0, 0.7],
                    [350.0, -2.0, 1.2],
                    [3000.0, 3.0, 1.0],
                    [10_000.0, 2.0, 0.7],
                ))),
                comp: Some(comp_on(-20.0, 3.0, 8.0, 120.0, 3.0)),
                ..StripPatch::default()
            },
        ),
        preset(
            "music-bed",
            "Music bed",
            StripPatch {
                highpass: Some(highpass_on(40.0)),
                eq: Some(eq_on(eq_bands(
                    [120.0, 0.0, 0.7],
                    [400.0, 0.0, 1.0],
                    [2500.0, -3.0, 1.4],
                    [9000.0, -1.0, 0.7],
                ))),
                comp: Some(comp_on(-24.0, 2.0, 30.0, 300.0, 0.0)),
                fader: Some(-6.0),
                ..StripPatch::default()
            },
        ),
        preset(
            "punchy-effects",
            "Punchy effects",
            StripPatch {
                highpass: Some(highpass_on(60.0)),
                eq: Some(eq_on(eq_bands(
                    [100.0, 2.0, 0.7],
                    [600.0, -2.0, 1.0],
                    [4000.0, 2.0, 1.0],
                    [9000.0, 1.0, 0.7],
                ))),
                comp: Some(comp_on(-16.0, 4.0, 3.0, 80.0, 2.0)),
                ..StripPatch::default()
            },
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MixerBook {
    pub sample_rate: u32,
    pub min_db: f64,
    pub silence_db: f64,
    pub max_db: f64,
    pub fader: NumberRange,
    pub trim: NumberRange,
    pub pan: NumberRange,
    pub highpass: NumberRange,
    pub limiter_ceiling: NumberRange,
    pub comp: CompRanges,
    pub duck: DuckRanges,
    pub eq_band: EqBandBounds,
    pub eq_types: Vec<EqType>,
    pub default_strip: Strip,
    pub default_master: Master,
    pub built_in_buses: Vec<String>,
    pub loudness: Vec<LoudnessChoice>,
    pub presets: Vec<StripPreset>,
}

pub fn mixer_book() -> MixerBook {
    MixerBook {
        sample_rate: SAMPLE_RATE,
        min_db: MIN_DB,
        silence_db: SILENCE_DB,
        max_db: MAX_DB,
        fader: FADER_RANGE,
        trim: TRIM_RANGE,
        pan: PAN_RANGE,
        highpass: HIGHPASS_RANGE,
        limiter_ceiling: LIMITER_CEILING_RANGE,
        comp: COMP_RANGES,
        duck: DUCK_RANGES,
        eq_band: EQ_BAND_BOUNDS,
        eq_types: vec![EqType::Lowshelf, EqType::Peak, EqType::Highshelf],
        default_strip: Strip::default(),
        default_master: Master::default(),
        built_in_buses: default_buses().into_iter().map(|b| b.id).collect(),
        loudness: LOUDNESS_TARGETS
            .iter()
            .map(|t| LoudnessChoice {
                target: *t,
                goal: t.target(),
            })
            .collect(),
        presets: strip_presets(),
    }
}
