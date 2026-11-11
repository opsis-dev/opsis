use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use opsis_compile::mix::plan::MixPlan;
use opsis_media::analysis::{AnalysisDeps, analyze_filmstrip, analyze_waveform};
use opsis_media::mix::render::{MixDeps, MixError, render_mix};
use opsis_project::paths::ref_to_path;
use opsis_types::analysis::{Filmstrip, Waveform};
use opsis_types::mix::MixResult;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Session;
use crate::engine::engine_tools;
use crate::error::CommandResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AnalysisRequest {
    Waveform {
        project_dir: String,
        src: String,
    },
    Filmstrip {
        project_dir: String,
        src: String,
    },
    Mix {
        project_dir: String,
        plan: Box<MixPlan>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AnalysisResult {
    Waveform { waveform: Waveform },
    Filmstrip { filmstrip: Filmstrip },
    Mix { mix: Box<MixResult> },
    Skipped,
    Error { error: String },
}

#[derive(Debug, Default)]
pub struct Mixes {
    latest: Mutex<HashMap<PathBuf, u64>>,
    next: AtomicU64,
    one_at_a_time: tokio::sync::Mutex<()>,
}

fn failed(error: impl std::fmt::Display) -> AnalysisResult {
    AnalysisResult::Error {
        error: error.to_string(),
    }
}

async fn mix(state: &AppState, mixes: &Mixes, project: &Path, plan: &MixPlan) -> AnalysisResult {
    let ticket = mixes.next.fetch_add(1, Ordering::Relaxed);
    mixes
        .latest
        .lock()
        .expect("mix tickets")
        .insert(project.to_path_buf(), ticket);
    let _turn = mixes.one_at_a_time.lock().await;
    if mixes.latest.lock().expect("mix tickets").get(project) != Some(&ticket) {
        return AnalysisResult::Skipped;
    }
    let public = state.paths.public_dir.clone();
    let tools = engine_tools(state, &state.paths.views_temp);
    let resolve = move |reference: &str| {
        ref_to_path(project, &public, reference).map_err(|_| MixError::Failed)
    };
    let deps = MixDeps {
        project_dir: project,
        resolve: &resolve,
        ffmpeg: tools.as_ref(),
        cancel: None,
        on_progress: None,
    };
    match render_mix(plan, &deps).await {
        Ok(result) => AnalysisResult::Mix {
            mix: Box::new(result),
        },
        Err(error) => failed(error),
    }
}
pub async fn analyze(app: &Session, request: AnalysisRequest) -> CommandResult<AnalysisResult> {
    let state = app.state::<AppState>();
    let mixes = app.state::<Mixes>();
    let tools = engine_tools(state, &state.paths.views_temp);
    Ok(match request {
        AnalysisRequest::Waveform { project_dir, src } => analyze_waveform(
            &src,
            &AnalysisDeps {
                project_dir: Path::new(&project_dir),
                public_dir: &state.paths.public_dir,
                tools: tools.as_ref(),
                cancel: None,
            },
        )
        .await
        .map_or_else(failed, |waveform| AnalysisResult::Waveform { waveform }),
        AnalysisRequest::Filmstrip { project_dir, src } => analyze_filmstrip(
            &src,
            &AnalysisDeps {
                project_dir: Path::new(&project_dir),
                public_dir: &state.paths.public_dir,
                tools: tools.as_ref(),
                cancel: None,
            },
        )
        .await
        .map_or_else(failed, |filmstrip| AnalysisResult::Filmstrip { filmstrip }),
        AnalysisRequest::Mix { project_dir, plan } => {
            mix(state, mixes, Path::new(&project_dir), &plan).await
        }
    })
}
