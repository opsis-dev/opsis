use std::future::Future;
use std::path::{Path, PathBuf};

use opsis_compile::text::{js_round, number, round_to};
use opsis_compile::time::{EPSILON, even_px};
use opsis_engine::files::{atomic_write, file_identity, read_json, write_via_temp};
use opsis_engine::hash::versioned_hash;
use opsis_engine::json::js_json;
use opsis_project::paths::{
    ProjectPaths, WebAddress, is_inside, project_paths, ref_to_path, to_project_ref,
};
use opsis_types::analysis::{Filmstrip, Waveform, WaveformLevel};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::codecs::strings;
use crate::ffmpeg::{MediaTools, RunOptions, ToolError};

const ANALYSIS_VERSION: &str = "opsis-analysis/1";
const HASH_LENGTH: usize = 24;
const DECODE_RATE: f64 = 8000.0;
const MAX_TILES: f64 = 100.0;
const MIN_TILE_MS: f64 = 500.0;
const TILE_HEIGHT: f64 = 72.0;
const FLOAT_BYTES: usize = 4;

const LEVEL_RATES: [f64; 3] = [200.0, 50.0, 10.0];
const ONSETS_PER_SECOND: f64 = 100.0;
const ONSETS_PER_MINUTE: f64 = 60.0 * ONSETS_PER_SECOND;
const ONSET_LOG_GAIN: f64 = 100.0;
const MIN_BEAT_FRAMES: usize = 400;
const MIN_BPM: f64 = 60.0;
const MAX_BPM: f64 = 180.0;
const PRIOR_BPM: f64 = 120.0;
const PRIOR_WIDTH_OCTAVES: f64 = 0.9;
const PRIOR_FLOOR: f64 = 0.85;
const PRIOR_WEIGHT: f64 = 0.15;
const MIN_PERIODICITY: f64 = 1.8;
const PEAK_SEARCH_FRAMES: usize = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beats {
    pub beats: Vec<f64>,
    pub bpm: Option<f64>,
}

fn round3(value: f64) -> f64 {
    round_to(value, 3)
}

fn windows(count: usize, size: f64, total: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..count).map(move |i| {
        (
            (i as f64 * size).floor() as usize,
            (((i + 1) as f64 * size).floor() as usize).min(total),
        )
    })
}

pub fn waveform_levels(samples: &[f32], rate: f64) -> Vec<WaveformLevel> {
    LEVEL_RATES
        .iter()
        .map(|&r| {
            let per = rate / r;
            let count = (samples.len() as f64 / per).ceil() as usize;
            let (mut min, mut max) = (Vec::with_capacity(count), Vec::with_capacity(count));
            for (a, b) in windows(count, per, samples.len()) {
                let (mut lo, mut hi) = (0f64, 0f64);
                for &v in &samples[a.min(b)..b] {
                    lo = lo.min(f64::from(v));
                    hi = hi.max(f64::from(v));
                }
                min.push(round3(lo));
                max.push(round3(hi));
            }
            WaveformLevel { rate: r, min, max }
        })
        .collect()
}

fn onset_envelope(samples: &[f32], rate: f64) -> Vec<f32> {
    let hop = js_round(rate / ONSETS_PER_SECOND) as usize;
    let frames = samples.len() / hop;
    let energy: Vec<f32> = (0..frames)
        .map(|f| {
            let squares: f64 = samples[f * hop..(f + 1) * hop]
                .iter()
                .map(|&s| f64::from(s) * f64::from(s))
                .sum();
            (ONSET_LOG_GAIN * (squares / hop as f64).sqrt()).ln_1p() as f32
        })
        .collect();
    let mut onset = vec![0f32; frames];
    for f in 1..frames {
        onset[f] = f64::max(0.0, f64::from(energy[f]) - f64::from(energy[f - 1])) as f32;
    }
    onset
}

fn tempo_prior(lag: usize) -> f64 {
    let bpm = ONSETS_PER_MINUTE / lag as f64;
    let octaves = (bpm / PRIOR_BPM).log2() / PRIOR_WIDTH_OCTAVES;
    PRIOR_FLOOR + PRIOR_WEIGHT * (-0.5 * octaves.powi(2)).exp()
}

fn beat_period(onset: &[f32]) -> Option<usize> {
    let min_lag = (ONSETS_PER_MINUTE / MAX_BPM).floor() as usize;
    let max_lag = (ONSETS_PER_MINUTE / MIN_BPM).ceil() as usize;
    let mut correlations = Vec::new();
    let (mut best_lag, mut best) = (0usize, 0f64);
    for lag in min_lag..=max_lag {
        let ac: f64 = (lag..onset.len())
            .map(|f| f64::from(onset[f]) * f64::from(onset[f - lag]))
            .sum();
        correlations.push(ac);
        let score = ac * tempo_prior(lag);
        if score > best {
            best = score;
            best_lag = lag;
        }
    }
    let mut sorted = correlations.clone();
    sorted.sort_by(f64::total_cmp);
    let median = sorted[sorted.len() / 2];
    if best_lag == 0 || median <= 0.0 || correlations[best_lag - min_lag] < median * MIN_PERIODICITY
    {
        return None;
    }
    Some(best_lag)
}

fn beat_phase(onset: &[f32], period: usize) -> usize {
    let (mut phase, mut phase_score) = (0, -1.0);
    for p in 0..period {
        let score: f64 = onset
            .iter()
            .skip(p)
            .step_by(period)
            .map(|&o| f64::from(o))
            .sum();
        if score > phase_score {
            phase_score = score;
            phase = p;
        }
    }
    phase
}

pub fn detect_beats(samples: &[f32], rate: f64) -> Beats {
    let none = Beats {
        beats: Vec::new(),
        bpm: None,
    };
    let onset = onset_envelope(samples, rate);
    if onset.len() < MIN_BEAT_FRAMES {
        return none;
    }
    let mean = onset.iter().map(|&o| f64::from(o)).sum::<f64>() / onset.len() as f64;
    if mean <= EPSILON {
        return none;
    }
    let Some(period) = beat_period(&onset) else {
        return none;
    };
    let mut beats = Vec::new();
    let mut f = beat_phase(&onset, period);
    while f < onset.len() {
        let mut peak = f;
        let last = (onset.len() - 1).min(f + PEAK_SEARCH_FRAMES);
        for k in f.saturating_sub(PEAK_SEARCH_FRAMES)..=last {
            if onset[k] > onset[peak] {
                peak = k;
            }
        }
        beats.push(round3(peak as f64 / ONSETS_PER_SECOND));
        f += period;
    }
    Beats {
        beats,
        bpm: Some(js_round(ONSETS_PER_MINUTE / period as f64)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error(transparent)]
    Tool(#[from] ToolError),
    #[error(transparent)]
    WebAddress(#[from] WebAddress),
    #[error("This file has no picture.")]
    NoPicture,
    #[error("{} isn't in the project.", .0.display())]
    Outside(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct AnalysisDeps<'a> {
    pub project_dir: &'a Path,
    pub public_dir: &'a Path,
    pub tools: &'a dyn MediaTools,
    pub cancel: Option<&'a CancellationToken>,
}

impl AnalysisDeps<'_> {
    fn options(&self) -> RunOptions<'_> {
        RunOptions {
            cancel: self.cancel,
            ..RunOptions::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedWaveform {
    duration_ms: f64,
    levels: Vec<WaveformLevel>,
    beats: Vec<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    bpm: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedFilmstrip {
    image: String,
    count: f64,
    interval_ms: f64,
    tile_width: f64,
    tile_height: f64,
    duration_ms: f64,
}

fn id_for(file: &Path) -> std::io::Result<String> {
    Ok(versioned_hash(
        ANALYSIS_VERSION,
        &json!({ "file": file.to_string_lossy(), "identity": file_identity(file)? }),
        HASH_LENGTH,
    ))
}

fn is_node_cache_file(paths: &ProjectPaths, file: &Path) -> bool {
    is_inside(&paths.cache, file)
        && file.parent() != Some(paths.cache.as_path())
        && !is_inside(&paths.analysis, file)
        && !is_inside(&paths.mix, file)
}

fn filmstrip_dir(project_dir: &Path, file: &Path) -> PathBuf {
    let paths = project_paths(project_dir);
    match file.parent() {
        Some(parent) if is_node_cache_file(&paths, file) => parent.to_path_buf(),
        _ => paths.analysis,
    }
}

async fn cached_analysis<T, F, Fut>(
    file: &Path,
    dir: &Path,
    name: &str,
    ready: impl FnOnce(&str) -> bool,
    make: F,
) -> Result<T, AnalysisError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<T, AnalysisError>>,
{
    let id = id_for(file)?;
    let saved = dir.join(format!("{id}.{name}.json"));
    if ready(&id)
        && let Some(cached) = read_json(&saved)?
    {
        return Ok(cached);
    }
    tokio::fs::create_dir_all(dir).await?;
    let value = make(id).await?;
    atomic_write(
        &saved,
        js_json(&value).map_err(std::io::Error::other)?.as_bytes(),
    )?;
    Ok(value)
}

fn float_samples(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<FLOAT_BYTES>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect()
}

async fn decode_waveform(raw: &Path) -> Result<SavedWaveform, AnalysisError> {
    let bytes = tokio::fs::read(raw).await?;
    let measured = tokio::task::spawn_blocking(move || {
        let samples = float_samples(&bytes);
        let Beats { beats, bpm } = detect_beats(&samples, DECODE_RATE);
        SavedWaveform {
            duration_ms: js_round(samples.len() as f64 / DECODE_RATE * 1000.0),
            levels: waveform_levels(&samples, DECODE_RATE),
            beats,
            bpm,
        }
    })
    .await
    .map_err(std::io::Error::other)?;
    Ok(measured)
}

async fn remove_file(file: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_file(file).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        done => done,
    }
}

pub async fn analyze_waveform(
    src: &str,
    deps: &AnalysisDeps<'_>,
) -> Result<Waveform, AnalysisError> {
    let file = &ref_to_path(deps.project_dir, deps.public_dir, src)?;
    let dir = &project_paths(deps.project_dir).analysis;
    let saved = cached_analysis(
        file,
        dir,
        "waveform",
        |_| true,
        |id| async move {
            let raw = dir.join(format!("{id}.waveform.{}.raw", std::process::id()));
            let input = file.to_string_lossy();
            let output = raw.to_string_lossy();
            let rate = number(DECODE_RATE);
            let args = strings(&[
                "-i",
                &input,
                "-vn",
                "-ac",
                "1",
                "-ar",
                &rate,
                "-f",
                "f32le",
                "-c:a",
                "pcm_f32le",
                &output,
            ]);
            let decoded = match deps.tools.run(&args, deps.options()).await {
                Ok(()) => decode_waveform(&raw).await,
                Err(e) => Err(e.into()),
            };
            remove_file(&raw).await?;
            decoded
        },
    )
    .await?;
    Ok(Waveform {
        src: src.to_owned(),
        duration_ms: saved.duration_ms,
        levels: saved.levels,
        beats: saved.beats,
        bpm: saved.bpm,
    })
}

pub async fn analyze_filmstrip(
    src: &str,
    deps: &AnalysisDeps<'_>,
) -> Result<Filmstrip, AnalysisError> {
    let file = &ref_to_path(deps.project_dir, deps.public_dir, src)?;
    let dir = &filmstrip_dir(deps.project_dir, file);
    let image_of = |id: &str| dir.join(format!("{id}.filmstrip.jpg"));
    let saved = cached_analysis(
        file,
        dir,
        "filmstrip",
        |id| image_of(id).exists(),
        |id| async move {
            let probe = deps.tools.probe(file).await?;
            let video = probe.video.ok_or(AnalysisError::NoPicture)?;
            let duration_ms = probe.duration_ms.max(1.0);
            let interval_ms = MIN_TILE_MS.max((duration_ms / MAX_TILES).ceil());
            let count = (duration_ms / interval_ms).ceil().max(1.0);
            let tile_width = even_px(video.width / video.height * TILE_HEIGHT).max(2.0);
            let image = image_of(&id);
            let filter = format!(
                "fps=fps={}:start_time=0,scale={}:{},tile={}x1",
                number(1000.0 / interval_ms),
                number(tile_width),
                number(TILE_HEIGHT),
                number(count)
            );
            let input = file.to_string_lossy();
            write_via_temp(&image, |temp| async move {
                let args = strings(&[
                    "-i",
                    &input,
                    "-an",
                    "-vf",
                    &filter,
                    "-frames:v",
                    "1",
                    "-q:v",
                    "5",
                    &temp.to_string_lossy(),
                ]);
                deps.tools
                    .run(&args, deps.options())
                    .await
                    .map_err(AnalysisError::from)
            })
            .await?;
            let image_ref = to_project_ref(deps.project_dir, &image)
                .ok_or_else(|| AnalysisError::Outside(image.clone()))?;
            Ok(SavedFilmstrip {
                image: image_ref,
                count,
                interval_ms,
                tile_width,
                tile_height: TILE_HEIGHT,
                duration_ms,
            })
        },
    )
    .await?;
    Ok(Filmstrip {
        src: src.to_owned(),
        image: saved.image,
        count: saved.count,
        interval_ms: saved.interval_ms,
        tile_width: saved.tile_width,
        tile_height: saved.tile_height,
        duration_ms: saved.duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 8000.0;

    fn clicks(bpm: f64, seconds: f64, offset: f64) -> Vec<f32> {
        let mut seed: u64 = 1;
        let mut samples: Vec<f32> = (0..(RATE * seconds) as usize)
            .map(|_| {
                seed = (seed * 16_807) % 2_147_483_647;
                ((seed as f64 / 2_147_483_647.0 - 0.5) * 0.05) as f32
            })
            .collect();
        let gap = 60.0 / bpm;
        let mut t = offset;
        while t < seconds {
            let start = (t * RATE).round() as usize;
            for i in 0..200 {
                if let Some(s) = samples.get_mut(start + i) {
                    *s += ((i as f64 / 3.0).sin() * (-(i as f64) / 60.0).exp()) as f32;
                }
            }
            t += gap;
        }
        samples
    }

    #[test]
    fn finds_the_tempo_and_the_beats_of_a_steady_click() {
        let found = detect_beats(&clicks(120.0, 12.0, 0.25), RATE);
        assert_eq!(found.bpm, Some(120.0));
        assert!(found.beats.len() > 20);
        assert!(
            found
                .beats
                .windows(2)
                .all(|w| ((w[1] - w[0]) - 0.5).abs() < 0.05),
            "{:?}",
            found.beats
        );
        assert!(((found.beats[0] % 0.5) - 0.25).abs() < 0.05);
    }

    #[test]
    fn finds_no_beat_in_silence_or_in_a_short_sound() {
        let none = Beats {
            beats: Vec::new(),
            bpm: None,
        };
        assert_eq!(detect_beats(&vec![0.0; (RATE * 12.0) as usize], RATE), none);
        assert_eq!(detect_beats(&clicks(120.0, 2.0, 0.25), RATE), none);
    }

    #[test]
    fn keeps_the_lowest_and_highest_sample_of_each_step() {
        let levels = waveform_levels(&[0.5, -0.25, 0.125, -1.0], 400.0);
        assert_eq!(
            levels.iter().map(|l| l.rate).collect::<Vec<_>>(),
            [200.0, 50.0, 10.0]
        );
        assert_eq!(
            levels[0],
            WaveformLevel {
                rate: 200.0,
                min: vec![-0.25, -1.0],
                max: vec![0.5, 0.125]
            }
        );
        assert_eq!(
            levels[1],
            WaveformLevel {
                rate: 50.0,
                min: vec![-1.0],
                max: vec![0.5]
            }
        );
    }
}
