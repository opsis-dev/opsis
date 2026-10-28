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
    pub duration_ms: u32,
    pub levels: Vec<WaveformLevel>,
    pub beats: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Filmstrip {
    pub src: String,
    pub image: String,
    pub count: u32,
    pub interval_ms: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub duration_ms: u32,
}
