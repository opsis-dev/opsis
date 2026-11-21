use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{path}: {source}")]
    File {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("standard input: {0}")]
    Input(std::io::Error),
    #[error("the answer couldn't be written as JSON: {0}")]
    Answer(serde_json::Error),
    #[error("that isn't a workflow: {0}")]
    NotAWorkflow(serde_json::Error),
    #[error("the async runtime couldn't start: {0}")]
    Runtime(std::io::Error),
    #[error(transparent)]
    Install(#[from] opsis_engine::install::InstallError),
    #[error("this computer's platform has no recording browser")]
    NoPlatform,
    #[error("the recording browser isn't in {0} after installing it")]
    NoBrowser(PathBuf),
    #[error("{failed} of {total} elements didn't render:\n{list}")]
    Stills {
        failed: usize,
        total: usize,
        list: String,
    },
}
