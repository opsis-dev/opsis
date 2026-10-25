use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use indexmap::IndexMap;
use opsis_compile::asset_ref::project_ref;
use opsis_compile::mix::plan::{MixPlan, channel_identity, has_sound};
use opsis_compile::text::round_to;
use opsis_engine::files::{
    Prune, atomic_write, file_identity, prune_dir, read_json, write_via_temp,
};
use opsis_engine::hash::versioned_hash;
use opsis_types::mix::{Loudness, MasterStem, MixLevels, MixReduction, MixResult, StemLevels};
use opsis_types::project::MIX_DIR;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::ffmpeg::{MediaTools, RunOptions, ToolError};
use crate::mix::graph::{
    Job, PcmCodec, Plan, Taps, bus_job, channel_job, job_args, loudness_measure_job, master_job,
    measure_args, parse_loudnorm,
};
use crate::mix::levels::wav_levels;
use crate::mix::loudness::wav_loudness;

const MIX_VERSION: &str = "opsis-mix/1";
const HASH_LENGTH: usize = 24;
const KEEP_PLANS: usize = 12;
const UNUSED_STEM: Duration = Duration::from_secs(10 * 60);
const QUIET_PRE_DB: f64 = -100.0;
const RECENT_FILE: &str = "recent.json";
const MASTER_STEPS: [&str; 2] = ["Balancing loudness", "Measuring loudness"];

static MISSING_INPUT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)Error opening input file .*?([^/\\]+?)\.?$").expect("pattern")
});
static MISSING_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^/\\\s]+\.\w+): No such file or directory").expect("pattern"));
static RENDERING: LazyLock<Mutex<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>> =
    LazyLock::new(Default::default);

#[derive(Debug, thiserror::Error)]
pub enum MixError {
    #[error("Cancelled")]
    Cancelled,
    #[error("The sound file “{0}” is missing from the project.")]
    MissingFile(String),
    #[error("The sound couldn't be mixed. Check that every sound file still opens.")]
    Failed,
}

impl From<std::io::Error> for MixError {
    fn from(error: std::io::Error) -> Self {
        log::warn!("mixing sound: {error}");
        Self::Failed
    }
}

pub fn friendly_mix_error(error: &ToolError) -> MixError {
    let ToolError::Failed { output } = error else {
        return match error {
            ToolError::Cancelled => MixError::Cancelled,
            _ => MixError::Failed,
        };
    };
    MISSING_INPUT
        .captures(output)
        .or_else(|| MISSING_FILE.captures(output))
        .map_or(MixError::Failed, |found| {
            MixError::MissingFile(found[1].to_owned())
        })
}

pub type Resolve<'a> = &'a (dyn Fn(&str) -> Result<PathBuf, MixError> + Send + Sync);
pub type OnStep<'a> = &'a (dyn Fn(f64, &str) + Send + Sync);

pub struct MixDeps<'a> {
    pub project_dir: &'a Path,
    pub resolve: Resolve<'a>,
    pub ffmpeg: &'a dyn MediaTools,
    pub cancel: Option<&'a CancellationToken>,
    pub on_progress: Option<OnStep<'a>>,
}

impl MixDeps<'_> {
    fn dir(&self) -> PathBuf {
        self.project_dir.join(MIX_DIR)
    }

    fn options(&self) -> RunOptions<'_> {
        RunOptions {
            cancel: self.cancel,
            ..RunOptions::default()
        }
    }
}

#[derive(Clone)]
struct Stem {
    hash: String,
    file: PathBuf,
    levels: StemLevels,
    reduction: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize)]
struct StemMeta {
    levels: StemLevels,
    reduction: Option<Vec<f64>>,
}

fn sha(value: &impl Serialize) -> String {
    versioned_hash(MIX_VERSION, value, HASH_LENGTH)
}

fn identity(file: &Path) -> String {
    file_identity(file).unwrap_or_else(|_| "missing".to_owned())
}

fn reduction_from_taps(pre: &StemLevels, post: &StemLevels, makeup: f64) -> Vec<f64> {
    pre.rms
        .iter()
        .zip(&post.rms)
        .map(|(&p, &q)| {
            if p <= QUIET_PRE_DB {
                0.0
            } else {
                f64::max(0.0, round_to(p - (q - makeup), 1))
            }
        })
        .collect()
}

fn frames_of(plan: &MixPlan) -> usize {
    plan.duration_in_frames.max(0.0) as usize
}

fn shape(plan: &MixPlan) -> Plan {
    Plan {
        fps: plan.fps,
        duration_in_frames: plan.duration_in_frames,
    }
}

fn tool_error(error: ToolError) -> MixError {
    friendly_mix_error(&error)
}

struct StemJob {
    hash: String,
    job: Job,
    codec: PcmCodec,
    makeup: f64,
}

fn lock_for(file: &Path) -> Arc<tokio::sync::Mutex<()>> {
    Arc::clone(
        RENDERING
            .lock()
            .expect("stem locks")
            .entry(file.to_path_buf())
            .or_default(),
    )
}

fn temp_tap(dir: &Path, suffix: &str) -> Result<tempfile::TempPath, MixError> {
    Ok(tempfile::Builder::new()
        .prefix(".tap-")
        .suffix(suffix)
        .tempfile_in(dir)?
        .into_temp_path())
}

async fn render_stem(deps: &MixDeps<'_>, plan: &MixPlan, stem: StemJob) -> Result<Stem, MixError> {
    let dir = deps.dir();
    let file = dir.join(format!("{}.wav", stem.hash));
    let meta_file = dir.join(format!("{}.json", stem.hash));
    let lock = lock_for(&file);
    let _held = lock.lock().await;
    if file.exists()
        && let Some(meta) = read_json::<StemMeta>(&meta_file)?
    {
        return Ok(Stem {
            hash: stem.hash,
            file,
            levels: meta.levels,
            reduction: meta.reduction,
        });
    }
    std::fs::create_dir_all(&dir)?;
    let taps = match &stem.job.taps {
        Some(_) => Some((temp_tap(&dir, ".pre.wav")?, temp_tap(&dir, ".post.wav")?)),
        None => None,
    };
    let tap_files = taps.as_ref().map(|(pre, post)| Taps {
        pre: pre.to_string_lossy().into_owned(),
        post: post.to_string_lossy().into_owned(),
    });
    let frames = frames_of(plan);
    let limit = plan.duration_in_frames / plan.fps + 1.0;
    let meta = write_via_temp(&file, |temp| async move {
        let args = job_args(
            &stem.job,
            &temp.to_string_lossy(),
            stem.codec,
            tap_files.as_ref(),
            Some(limit),
        );
        deps.ffmpeg
            .run(&args, deps.options())
            .await
            .map_err(tool_error)?;
        let levels = levels_of(&temp, plan.fps, frames).await?;
        let reduction = match &taps {
            Some((pre, post)) => {
                let pre = levels_of(pre, plan.fps, frames).await?;
                let post = levels_of(post, plan.fps, frames).await?;
                Some(reduction_from_taps(&pre, &post, stem.makeup))
            }
            None => None,
        };
        Ok::<_, MixError>(StemMeta { levels, reduction })
    })
    .await?;
    atomic_write(
        &meta_file,
        serde_json::to_string(&meta)
            .expect("stem levels serialize")
            .as_bytes(),
    )?;
    Ok(Stem {
        hash: stem.hash,
        file,
        levels: meta.levels,
        reduction: meta.reduction,
    })
}

struct Ticker<'a> {
    deps: &'a MixDeps<'a>,
    done: usize,
    steps: usize,
}

impl Ticker<'_> {
    fn tick(&mut self, message: &str) {
        if let Some(progress) = self.deps.on_progress {
            progress(self.done as f64 / self.steps as f64, message);
        }
        self.done += 1;
    }
}

async fn channel_stems(
    plan: &MixPlan,
    deps: &MixDeps<'_>,
    ticker: &mut Ticker<'_>,
) -> Result<IndexMap<String, Stem>, MixError> {
    let mut stems = IndexMap::new();
    for channel in &plan.channels {
        ticker.tick("Mixing sound");
        let files: Vec<PathBuf> = channel
            .sounds
            .iter()
            .map(|s| (deps.resolve)(&s.src))
            .collect::<Result<_, _>>()?;
        let ids: Vec<String> = files.iter().map(|f| identity(f)).collect();
        let hash = format!(
            "ch-{}",
            sha(&json!({ "channel": channel_identity(plan, channel), "ids": ids }))
        );
        let makeup = if channel.strip.comp.on {
            channel.strip.comp.makeup
        } else {
            0.0
        };
        let job = channel_job(
            shape(plan),
            channel,
            files
                .iter()
                .map(|f| f.to_string_lossy().into_owned())
                .collect(),
        );
        stems.insert(
            channel.id.clone(),
            render_stem(
                deps,
                plan,
                StemJob {
                    hash,
                    job,
                    codec: PcmCodec::F32,
                    makeup,
                },
            )
            .await?,
        );
    }
    Ok(stems)
}

async fn bus_stems(
    plan: &MixPlan,
    deps: &MixDeps<'_>,
    channels: &IndexMap<String, Stem>,
    ticker: &mut Ticker<'_>,
) -> Result<IndexMap<String, Stem>, MixError> {
    let stem = |id: &str| channels.get(id).ok_or(MixError::Failed);
    let mut buses = IndexMap::new();
    for bus in &plan.buses {
        ticker.tick("Mixing sound");
        let mut shown = serde_json::to_value(bus).expect("buses serialize");
        if let Some(map) = shown.as_object_mut() {
            map.remove("name");
        }
        let stems: Vec<String> = bus
            .channels
            .iter()
            .map(|c| stem(c).map(|s| s.hash.clone()))
            .collect::<Result<_, _>>()?;
        let keys: Vec<String> = bus
            .duck
            .iter()
            .flat_map(|d| &d.keys)
            .map(|c| stem(c).map(|s| s.hash.clone()))
            .collect::<Result<_, _>>()?;
        let hash = format!(
            "bus-{}",
            sha(
                &json!({ "fps": plan.fps, "frames": plan.duration_in_frames, "bus": shown, "stems": stems, "keys": keys })
            )
        );
        let file_of = |c: &str| {
            channels
                .get(c)
                .map(|s| s.file.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let job = bus_job(shape(plan), bus, &file_of);
        buses.insert(
            bus.id.clone(),
            render_stem(
                deps,
                plan,
                StemJob {
                    hash,
                    job,
                    codec: PcmCodec::F32,
                    makeup: 0.0,
                },
            )
            .await?,
        );
    }
    Ok(buses)
}

async fn master_stem(
    plan: &MixPlan,
    deps: &MixDeps<'_>,
    buses: &IndexMap<String, Stem>,
) -> Result<Stem, MixError> {
    let files: Vec<String> = buses
        .values()
        .map(|b| b.file.to_string_lossy().into_owned())
        .collect();
    let hashes: Vec<&str> = buses.values().map(|b| b.hash.as_str()).collect();
    let hash = format!(
        "master-{}",
        sha(
            &json!({ "fps": plan.fps, "frames": plan.duration_in_frames, "master": plan.master, "buses": hashes })
        )
    );
    let measured = match loudness_measure_job(shape(plan), &plan.master, &files) {
        Some(measure) if !deps.dir().join(format!("{hash}.wav")).exists() => {
            let lines = Mutex::new(Vec::<String>::new());
            let collect = |line: &str| lines.lock().expect("log lines").push(line.to_owned());
            deps.ffmpeg
                .run(
                    &measure_args(&measure),
                    RunOptions {
                        on_log: Some(&collect),
                        ..deps.options()
                    },
                )
                .await
                .map_err(tool_error)?;
            parse_loudnorm(&lines.into_inner().expect("log lines").join("\n"))
        }
        _ => None,
    };
    let job = master_job(shape(plan), &plan.master, &files, measured.as_ref());
    let makeup = if plan.master.comp.on {
        plan.master.comp.makeup
    } else {
        0.0
    };
    render_stem(
        deps,
        plan,
        StemJob {
            hash,
            job,
            codec: PcmCodec::S16,
            makeup,
        },
    )
    .await
}

async fn measured<T: Send + 'static, E: std::fmt::Display + Send + 'static>(
    measure: impl FnOnce() -> Result<T, E> + Send + 'static,
) -> Result<T, MixError> {
    match tokio::task::spawn_blocking(measure).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => {
            log::warn!("measuring a sound stem: {error}");
            Err(MixError::Failed)
        }
        Err(error) => {
            log::warn!("measuring a sound stem stopped: {error}");
            Err(MixError::Failed)
        }
    }
}

async fn levels_of(file: &Path, fps: f64, frames: usize) -> Result<StemLevels, MixError> {
    let file = file.to_path_buf();
    measured(move || wav_levels(&file, fps, frames)).await
}

async fn loudness_of(
    deps: &MixDeps<'_>,
    plan: &MixPlan,
    master: &Stem,
) -> Result<Loudness, MixError> {
    let file = deps.dir().join(format!("{}.loudness.json", master.hash));
    if let Some(cached) = read_json::<Loudness>(&file)? {
        return Ok(cached);
    }
    let (stem, fps, frames) = (master.file.clone(), plan.fps, frames_of(plan));
    let loudness = measured(move || wav_loudness(&stem, fps, frames)).await?;
    atomic_write(
        &file,
        serde_json::to_string(&loudness)
            .expect("loudness serializes")
            .as_bytes(),
    )?;
    Ok(loudness)
}

fn remember(dir: &Path, hashes: Vec<String>) -> std::io::Result<()> {
    let index = dir.join(RECENT_FILE);
    let recent: Vec<Vec<String>> = read_json(&index)?.unwrap_or_default();
    let mut next = vec![hashes.clone()];
    next.extend(recent.into_iter().filter(|h| *h != hashes));
    next.truncate(KEEP_PLANS);
    atomic_write(&index, serde_json::to_string(&next)?.as_bytes())?;
    let keep: HashSet<String> = next.into_iter().flatten().collect();
    let keeps =
        |name: &str| name == RECENT_FILE || keep.contains(name.split('.').next().unwrap_or(name));
    prune_dir(
        dir,
        Prune {
            keep: Some(&keeps),
            newest: 0,
            unused_for: Some(UNUSED_STEM),
        },
    )
}

fn levels_by_id(stems: &IndexMap<String, Stem>) -> IndexMap<String, StemLevels> {
    stems
        .iter()
        .map(|(id, s)| (id.clone(), s.levels.clone()))
        .collect()
}

pub async fn render_mix(plan: &MixPlan, deps: &MixDeps<'_>) -> Result<MixResult, MixError> {
    if !has_sound(plan) {
        return Ok(MixResult {
            key: plan.key.clone(),
            master: None,
            levels: MixLevels::default(),
            reduction: MixReduction::default(),
            loudness: None,
        });
    }
    let steps = plan.channels.len() + plan.buses.len() + MASTER_STEPS.len();
    let mut ticker = Ticker {
        deps,
        done: 0,
        steps,
    };
    let channels = channel_stems(plan, deps, &mut ticker).await?;
    let buses = bus_stems(plan, deps, &channels, &mut ticker).await?;
    ticker.tick(MASTER_STEPS[0]);
    let master = master_stem(plan, deps, &buses).await?;
    ticker.tick(MASTER_STEPS[1]);
    let loudness = loudness_of(deps, plan, &master).await?;
    let hashes = channels
        .values()
        .chain(buses.values())
        .chain(std::iter::once(&master))
        .map(|s| s.hash.clone())
        .collect();
    remember(&deps.dir(), hashes)?;
    if let Some(progress) = deps.on_progress {
        progress(1.0, "Sound mixed");
    }
    let rel = master
        .file
        .strip_prefix(deps.project_dir)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    Ok(MixResult {
        key: plan.key.clone(),
        master: Some(MasterStem {
            src: project_ref(&rel),
            hash: master.hash.clone(),
        }),
        levels: MixLevels {
            channels: levels_by_id(&channels),
            buses: levels_by_id(&buses),
            master: Some(master.levels.clone()),
        },
        reduction: MixReduction {
            channels: channels
                .iter()
                .filter_map(|(id, s)| s.reduction.clone().map(|r| (id.clone(), r)))
                .collect(),
            master: master.reduction.clone(),
        },
        loudness: Some(loudness),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(output: &str) -> ToolError {
        ToolError::Failed {
            output: format!("ffmpeg exited with 254: [in#0 @ 0x1] {output}"),
        }
    }

    #[test]
    fn explains_mix_errors_without_naming_the_tools() {
        let cases = [
            (
                "Error opening input: No such file or directory\nError opening input file /home/me/project/assets/my song.mp3.\nError opening input files: No such file or directory",
                "The sound file “my song.mp3” is missing from the project.",
            ),
            (
                "Error opening input: No such file or directory\n/home/me/project/assets/song.mp3: No such file or directory",
                "The sound file “song.mp3” is missing from the project.",
            ),
            (
                "bad",
                "The sound couldn't be mixed. Check that every sound file still opens.",
            ),
        ];
        for (output, message) in cases {
            let error = friendly_mix_error(&failed(output)).to_string();
            assert_eq!(error, message);
            assert!(!error.to_lowercase().contains("ffmpeg"));
        }
        assert!(matches!(
            friendly_mix_error(&ToolError::Cancelled),
            MixError::Cancelled
        ));
    }

    #[test]
    fn measures_compressor_reduction_from_the_taps() {
        let pre = StemLevels {
            fps: 30.0,
            peak: Vec::new(),
            rms: vec![-110.0, -20.0, -10.0],
        };
        let post = StemLevels {
            fps: 30.0,
            peak: Vec::new(),
            rms: vec![-110.0, -24.0, -8.0],
        };
        assert_eq!(reduction_from_taps(&pre, &post, 3.0), [0.0, 7.0, 1.0]);
    }
}
