use std::fmt;

use opsis_media::ffmpeg::{MediaTools, ToolError};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum FilmError {
    #[error("{0}")]
    Encode(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("{0}")]
    Draw(String),
}

impl From<ToolError> for FilmError {
    fn from(error: ToolError) -> Self {
        match error {
            ToolError::Cancelled => Self::Cancelled,
            ToolError::Failed { output } => Self::Encode(output),
            other @ ToolError::Missing(_) => Self::Encode(other.to_string()),
        }
    }
}

pub struct FilmRun<'a> {
    pub tools: &'a dyn MediaTools,
    pub cancel: &'a CancellationToken,
    pub progress: &'a (dyn Fn(f64) + Send + Sync),
    pub note: &'a (dyn Fn(String) + Send + Sync),
}

impl fmt::Debug for FilmRun<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilmRun")
            .field("cancel", &self.cancel)
            .finish_non_exhaustive()
    }
}
