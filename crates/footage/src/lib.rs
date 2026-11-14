pub mod chain;
pub mod decoder;
pub mod footage;
pub mod serve;
pub mod terminal;

#[derive(Debug, thiserror::Error)]
pub enum FootageError {
    #[error("No footage named {0} in this film.")]
    Unknown(String),
    #[error("Couldn't start reading the video: {0}")]
    Start(std::io::Error),
    #[error("Couldn't read the video: {0}")]
    Decode(String),
    #[error("The video has no frames.")]
    NoFrames,
    #[error("The video stopped sending frames for 30 seconds.")]
    Stalled,
    #[error("Couldn't save a frame: {0}")]
    Encode(String),
    #[error("Couldn't draw the terminal take: {0}")]
    Terminal(String),
}
