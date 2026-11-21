use std::future::Future;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::ToolError;

pub fn repository() -> PathBuf {
    opsis_golden::repo()
}

pub fn shown(path: &Path) -> String {
    path.strip_prefix(repository())
        .unwrap_or(path)
        .display()
        .to_string()
}

pub fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result<(), ToolError> {
    let failed = |source| ToolError::File {
        path: path.to_path_buf(),
        source,
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(failed)?;
    }
    std::fs::write(path, contents).map_err(failed)
}

pub fn folder(path: &Path) -> Result<(), ToolError> {
    std::fs::create_dir_all(path).map_err(|source| ToolError::File {
        path: path.to_path_buf(),
        source,
    })
}

pub fn json(value: &impl Serialize) -> Result<(), ToolError> {
    println!(
        "{}",
        serde_json::to_string(value).map_err(ToolError::Answer)?
    );
    Ok(())
}

pub fn with_runtime(work: impl Future<Output = Result<(), ToolError>>) -> Result<(), ToolError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(ToolError::Runtime)?
        .block_on(work)
}
