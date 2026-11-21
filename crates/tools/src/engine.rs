use std::path::Path;

use opsis_engine::install::install_engine;
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell, is_downloaded};
use opsis_types::engine::EngineInstallUpdate;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::error::ToolError;
use crate::output;

pub async fn install(folder: &Path) -> Result<(), ToolError> {
    let platform = ShellPlatform::current().map_err(|_| ToolError::NoPlatform)?;
    let paths = engine_paths(folder, platform);
    if !is_downloaded(&paths) {
        output::folder(folder)?;
        let mut last = String::new();
        let mut report = |update: EngineInstallUpdate| {
            if update.message != last {
                eprintln!("{}", update.message);
                last = update.message;
            }
        };
        install_engine(folder, platform, &mut report, &CancellationToken::new()).await?;
    }
    let browser =
        find_headless_shell(&paths).ok_or_else(|| ToolError::NoBrowser(folder.to_path_buf()))?;
    output::json(&json!({ "browser": browser }))
}
