use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, UNIX_EPOCH};

use opsis_types::engine::CheckedTool;
use opsis_types::settings::EngineSection;
use sha2::{Digest, Sha256};
use tokio::process::Command;

use crate::downloads::downloads;
use crate::ffmpeg::bundled_ffmpeg;
use crate::paths::{EngineMissing, ShellPlatform, engine_paths, find_headless_shell};

const VERSION_TIMEOUT: Duration = Duration::from_secs(15);
const HASH_BUFFER: usize = 1 << 20;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, thiserror::Error)]
pub enum ToolProblem {
    #[error("There's no file at {0}.")]
    NotFound(String),
    #[error("{0} isn't a program.")]
    NotAFile(String),
    #[error("{path} couldn't be started ({reason}).")]
    NotRunnable { path: String, reason: String },
    #[error("{0} didn't answer in time.")]
    TimedOut(String),
    #[error("{path} isn't the {expected}.")]
    Unrecognized { path: String, expected: String },
    #[error("This {tool} is missing what Opsis needs: {missing}.")]
    Missing { tool: String, missing: String },
    #[error("{0}")]
    Failed(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineFiles {
    pub browser: Option<PathBuf>,
    pub ffmpeg: Option<PathBuf>,
    pub stamp: String,
}

impl EngineFiles {
    pub fn from_checked(engine: &EngineSection, usable: bool) -> Self {
        let browser = engine.browser.as_ref();
        Self {
            browser: browser.filter(|_| usable).map(|c| PathBuf::from(&c.path)),
            ffmpeg: bundled_ffmpeg().ok(),
            stamp: browser.map_or_else(String::new, |c| c.sha256.clone()),
        }
    }

    pub fn downloaded(dir: &Path) -> Self {
        let Ok(platform) = ShellPlatform::current() else {
            return Self::default();
        };
        let paths = engine_paths(dir, platform);
        Self {
            browser: find_headless_shell(&paths).map(Path::to_path_buf),
            ffmpeg: bundled_ffmpeg().ok(),
            stamp: String::new(),
        }
    }

    pub fn browser(&self) -> Result<&Path, EngineMissing> {
        self.browser.as_deref().ok_or(EngineMissing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    pub size: u64,
    pub modified_ms: i64,
}

pub fn file_stamp(path: &Path) -> std::io::Result<FileStamp> {
    let meta = std::fs::metadata(path)?;
    let modified = meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Ok(FileStamp {
        size: meta.len(),
        modified_ms: modified,
    })
}

pub fn file_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn same_stamp(checked: &CheckedTool, stamp: FileStamp) -> bool {
    checked.size == stamp.size && checked.modified_ms == stamp.modified_ms
}

pub fn shown(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn require_program(path: &Path) -> Result<(), ToolProblem> {
    match std::fs::metadata(path) {
        Err(_) => Err(ToolProblem::NotFound(shown(path))),
        Ok(meta) if !meta.is_file() => Err(ToolProblem::NotAFile(shown(path))),
        Ok(_) => Ok(()),
    }
}

pub async fn tool_output(path: &Path, args: &[&str]) -> Result<String, ToolProblem> {
    require_program(path)?;
    let mut command = Command::new(path);
    command.args(args).stdin(Stdio::null()).kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let run = command.output();
    let output = tokio::time::timeout(VERSION_TIMEOUT, run)
        .await
        .map_err(|_| ToolProblem::TimedOut(shown(path)))?
        .map_err(|e| ToolProblem::NotRunnable {
            path: shown(path),
            reason: e.to_string(),
        })?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(ToolProblem::NotRunnable {
            path: shown(path),
            reason: text
                .lines()
                .next()
                .unwrap_or("it stopped with an error")
                .to_owned(),
        });
    }
    Ok(text)
}

fn version_like(word: &str) -> bool {
    let parts: Vec<&str> = word.split('.').collect();
    parts.len() >= 2
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

pub fn browser_version(text: &str) -> Option<String> {
    let line = text.lines().find(|l| l.contains("Chrom"))?;
    line.split_whitespace()
        .find(|w| version_like(w))
        .map(str::to_owned)
}

pub fn ffmpeg_version(text: &str) -> Option<String> {
    let first = text.lines().next()?;
    let rest = first.strip_prefix("ffmpeg version ")?;
    rest.split_whitespace().next().map(str::to_owned)
}

pub fn browser_warning(version: &str) -> Option<String> {
    let pinned = &downloads().tested_version;
    (version != pinned).then(|| {
        format!(
            "This recording browser is version {version}; Opsis is tested with {pinned}. Recordings and exports may not match."
        )
    })
}

pub fn unrecognized(path: &Path, expected: &str) -> ToolProblem {
    ToolProblem::Unrecognized {
        path: shown(path),
        expected: expected.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_browser_version_from_any_chromium_build() {
        assert_eq!(
            browser_version("Chromium 157.0.8080.0\n").as_deref(),
            Some("157.0.8080.0")
        );
        assert_eq!(
            browser_version("Google Chrome for Testing 157.0.8080.0 \n").as_deref(),
            Some("157.0.8080.0")
        );
        assert_eq!(browser_version("ffmpeg version 7.0.2"), None);
    }

    #[test]
    fn reads_the_version_only_from_ffmpeg() {
        let text = "ffmpeg version 7.0.2-static https://johnvansickle.com/ffmpeg/  Copyright (c) 2000-2024\n";
        assert_eq!(ffmpeg_version(text).as_deref(), Some("7.0.2-static"));
        assert_eq!(ffmpeg_version("ffplay version 7.0.2-static"), None);
        assert_eq!(ffmpeg_version("Chromium 157.0.8080.0"), None);
    }

    #[test]
    fn warns_only_about_a_browser_other_than_the_tested_one() {
        assert_eq!(browser_warning(&downloads().tested_version), None);
        assert!(browser_warning("1.0.0.0").is_some_and(|w| w.contains("1.0.0.0")));
    }

    #[test]
    fn hashes_and_stamps_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tool");
        std::fs::write(&file, b"abc").unwrap();
        assert_eq!(
            file_sha256(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let stamp = file_stamp(&file).unwrap();
        assert_eq!(stamp.size, 3);
        let checked = CheckedTool {
            path: shown(&file),
            version: String::new(),
            sha256: String::new(),
            size: stamp.size,
            modified_ms: stamp.modified_ms,
        };
        assert!(same_stamp(&checked, stamp));
        assert!(!same_stamp(&checked, FileStamp { size: 4, ..stamp }));
    }

    #[test]
    fn refuses_a_missing_file_and_a_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            require_program(&dir.path().join("none")),
            Err(ToolProblem::NotFound(_))
        ));
        assert!(matches!(
            require_program(dir.path()),
            Err(ToolProblem::NotAFile(_))
        ));
    }
}
