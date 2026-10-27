pub mod executors;
pub mod load;
pub mod ops;
pub mod text;

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("This file isn't an image this app can open.")]
    NotAnImage,
    #[error("Couldn't open this image: {0}")]
    Unreadable(String),
    #[error("Couldn't save the image: {0}")]
    Failed(String),
}
