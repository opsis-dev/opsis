use std::path::PathBuf;

use opsis_engine::install::install_engine;
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell, is_downloaded};
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Name the folder to install the engine into."),
    );
    let platform = ShellPlatform::current().expect("a supported platform");
    let paths = engine_paths(&dir, platform);
    if !is_downloaded(&paths) {
        std::fs::create_dir_all(&dir).expect("the engine folder");
        let mut last = String::new();
        let mut report = |update: opsis_types::engine::EngineInstallUpdate| {
            if update.message != last {
                eprintln!("{}", update.message);
                last = update.message;
            }
        };
        if let Err(error) =
            install_engine(&dir, platform, &mut report, &CancellationToken::new()).await
        {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
    let browser = find_headless_shell(&paths).expect("the installed recording browser");
    println!("{}", json!({ "browser": browser, "ffmpeg": paths.ffmpeg }));
}
