use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WaveformLevel {
    pub rate: f64,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Waveform {
    pub src: String,
    pub duration_ms: f64,
    pub levels: Vec<WaveformLevel>,
    pub beats: Vec<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<f64>)]
    pub bpm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Filmstrip {
    pub src: String,
    pub image: String,
    pub count: f64,
    pub interval_ms: f64,
    pub tile_width: f64,
    pub tile_height: f64,
    pub duration_ms: f64,
}
