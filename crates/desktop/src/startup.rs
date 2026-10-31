use opsis_engine::paths::UnsupportedPlatform;
use opsis_store::db::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("The app's folders can't be found: {0}")]
    Folders(#[from] tauri::Error),
    #[error(transparent)]
    Platform(#[from] UnsupportedPlatform),
    #[error("The app's data folder can't be made: {0}")]
    DataFolder(#[from] std::io::Error),
    #[error("The app's database can't be opened: {0}")]
    Database(#[from] StoreError),
}
