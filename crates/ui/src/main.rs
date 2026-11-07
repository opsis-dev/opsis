mod clipboard;
mod colors;
mod folders;
mod graph;
mod inspector;
mod keys;
mod look;
mod media;
mod menu;
mod mixer;
mod motion;
mod platform;
mod player;
mod project;
mod replay;
mod run_panel;
mod streams;
mod text;
mod timeline;
mod ui;

use std::sync::Arc;

use opsis_session::Session;
use opsis_session::paths::AppPaths;
use opsis_store::keys::Keyring;

use crate::platform::QtPlatform;
use crate::ui::Ui;

fn tidy_earlier_runs(paths: &AppPaths) {
    let alive = &opsis_engine::temp::process_alive;
    let ended = opsis_engine::children::end_children_of_gone_apps(&paths.temp_root, alive);
    if !ended.is_empty() {
        log::info!(
            "ended {} programs left running by earlier runs: {ended:?}",
            ended.len()
        );
    }
    let cleared = opsis_engine::temp::clear_stale_child_temps(&paths.temp_root, alive);
    if !cleared.is_empty() {
        log::info!(
            "removed {} temp folders left by earlier runs",
            cleared.len()
        );
    }
}

fn start() -> Result<Ui, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let paths = AppPaths::new(&folders::host_folders())?;
    tidy_earlier_runs(&paths);
    let platform = Arc::new(QtPlatform::default());
    let keys = Keyring::platform();
    let session = runtime.block_on(Session::start(paths, keys, platform.clone()))?;
    Ok(Ui::new(session, runtime, platform))
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let ui = match start() {
        Ok(ui) => ui,
        Err(error) => {
            log::error!("Opsis couldn't start: {error}");
            std::process::exit(1);
        }
    };
    let session = ui.session().clone();
    let code = opsis_qt::run(Box::new(ui));
    session.end();
    std::process::exit(code);
}
