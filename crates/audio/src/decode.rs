use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use opsis_media::ffmpeg::{MediaTools, RunOptions, ToolError};

use crate::{STEM_CHANNELS, STEM_RATE};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    Tool(#[from] ToolError),
    #[error("Couldn't read the decoded sound: {0}")]
    Read(#[from] std::io::Error),
}

static DECODES: AtomicU64 = AtomicU64::new(0);

pub fn samples_from_bytes(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

pub async fn decode_stem(
    tools: &dyn MediaTools,
    file: &Path,
    scratch: &Path,
) -> Result<Vec<f32>, DecodeError> {
    tokio::fs::create_dir_all(scratch).await?;
    let raw = scratch.join(format!(
        "stem.{}.{}.raw",
        std::process::id(),
        DECODES.fetch_add(1, Ordering::Relaxed)
    ));
    let args: Vec<String> = [
        "-i",
        &file.to_string_lossy(),
        "-vn",
        "-ac",
        &STEM_CHANNELS.to_string(),
        "-ar",
        &STEM_RATE.to_string(),
        "-f",
        "f32le",
        "-c:a",
        "pcm_f32le",
        &raw.to_string_lossy(),
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    let ran = tools.run(&args, RunOptions::default()).await;
    let read = match ran {
        Ok(()) => tokio::fs::read(&raw).await.map_err(DecodeError::from),
        Err(error) => Err(error.into()),
    };
    if let Err(error) = tokio::fs::remove_file(&raw).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        log::debug!("removing the decoded stem: {error}");
    }
    Ok(samples_from_bytes(&read?))
}
