use std::path::PathBuf;

use opsis_engine::install::install_engine;
use opsis_engine::paths::{ShellPlatform, engine_status};
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Name the folder to install the engine into."),
    );
    let platform = ShellPlatform::current().expect("a supported platform");
    if engine_status(&dir, platform).complete {
        println!("The engine in {} is complete.", dir.display());
        return;
    }
    std::fs::create_dir_all(&dir).expect("the engine folder");
    let mut last = String::new();
    let mut report = |update: opsis_types::engine::EngineInstallUpdate| {
        if update.message != last {
            println!("{}", update.message);
            last = update.message;
        }
    };
    if let Err(error) = install_engine(&dir, platform, &mut report, &CancellationToken::new()).await
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
