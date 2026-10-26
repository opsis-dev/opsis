pub mod model;
pub mod serve;
pub mod svg;
pub mod texture;
pub mod typeface;

pub use opsis_engine::versions::PREPARED_VERSION;

#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error("Couldn't read this file.")]
    Unreadable,
    #[error("{0}")]
    Detail(String),
}
