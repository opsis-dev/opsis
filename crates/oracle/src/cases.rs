use std::path::{Path, PathBuf};

use opsis_compile::hosted::hosted;
use opsis_types::timeline::TimelineSpec;

use crate::host::{ORACLE_FOLDER, repo};

pub const CASE_WIDTH: u32 = 640;
pub const CASE_HEIGHT: u32 = 360;
pub const CASE_FPS: u32 = 30;

pub const CASE_FONTS: [&str; 8] = [
    "Inter",
    "JetBrains Mono",
    "Fraunces",
    "Instrument Serif",
    "Caveat",
    "Geist",
    "Space Grotesk",
    "Playfair Display",
];

#[derive(Debug, thiserror::Error)]
pub enum CaseError {
    #[error("reading the cases in {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("the bundled fonts have no family named {0}")]
    MissingFont(String),
}

pub fn cases_dir() -> PathBuf {
    repo().join("apps/oracle/src/cases")
}

fn entries(dir: &Path) -> Result<Vec<PathBuf>, CaseError> {
    let read = std::fs::read_dir(dir).map_err(|source| CaseError::Read {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut paths: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort();
    Ok(paths)
}

fn case_name(path: &Path) -> Option<&str> {
    (path.extension()? == "tsx")
        .then(|| path.file_stem()?.to_str())
        .flatten()
}

pub fn case_ids(dir: &Path) -> Result<Vec<String>, CaseError> {
    let mut ids = Vec::new();
    for group in entries(dir)?.into_iter().filter(|p| p.is_dir()) {
        let Some(group_name) = group.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        for file in entries(&group)? {
            if let Some(name) = case_name(&file) {
                ids.push(format!("{group_name}/{name}"));
            }
        }
    }
    Ok(ids)
}

pub fn case_spec() -> Result<TimelineSpec, CaseError> {
    let mut spec = hosted(&TimelineSpec {
        width: CASE_WIDTH,
        height: CASE_HEIGHT,
        fps: CASE_FPS,
        duration_in_frames: 1,
        background: None,
        tracks: Vec::new(),
        audio: Vec::new(),
        assets: None,
        reframe: None,
        fonts: None,
        mix: None,
    });
    let fonts = spec.fonts.as_mut().expect("the film fonts");
    for name in CASE_FONTS {
        if !fonts.families.contains_key(&name.to_lowercase()) {
            return Err(CaseError::MissingFont(name.to_owned()));
        }
    }
    fonts.preload = CASE_FONTS.iter().map(|&n| n.to_owned()).collect();
    Ok(spec)
}

pub fn case_url(base: &str, id: &str) -> String {
    format!("{base}{ORACLE_FOLDER}/index.html?case={id}")
}
