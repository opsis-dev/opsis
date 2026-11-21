#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod agent_panel;
mod clipboard;
mod colors;
mod font_views;
mod graph;
mod inspector;
mod keys;
mod live;
mod look;
mod markdown;
mod media;
mod menu;
mod mixer;
mod morph;
mod motion;
mod palette;
mod platform;
mod player;
mod project;
mod replay;
mod run_panel;
mod script_panel;
mod streams;
mod terminal_window;
mod text;
mod timeline;
mod ui;

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use opsis_persistence::keys::Keyring;
use opsis_session::Session;
use opsis_session::paths::AppPaths;

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

const USAGE: &str = "Usage: opsis [--headless <project folder>]";

fn headless_folder(args: &[OsString]) -> Result<Option<PathBuf>, String> {
    match args {
        [] => Ok(None),
        [flag, folder] if flag == "--headless" => std::path::absolute(folder)
            .map(Some)
            .map_err(|e| format!("{}: {e}", PathBuf::from(folder).display())),
        _ => Err(USAGE.to_owned()),
    }
}

fn start(headless: Option<PathBuf>) -> Result<Ui, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let paths = AppPaths::new(&opsis_folders::host_folders())?;
    tidy_earlier_runs(&paths);
    let platform = Arc::new(QtPlatform::new(headless.is_some()));
    let keys = Keyring::platform();
    let session = runtime.block_on(Session::start(paths, keys, platform.clone()))?;
    Ok(Ui::new(session, runtime, platform, headless))
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let headless = match headless_folder(&args) {
        Ok(headless) => headless,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let ui = match start(headless) {
        Ok(ui) => ui,
        Err(error) => {
            log::error!("Opsis couldn't start: {error}");
            std::process::exit(1);
        }
    };
    let session = ui.session().clone();
    let code = opsis_qt::run(Box::new(ui));
    session.end();
    opsis_session::updates::finish(&session);
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn starts_with_a_window_when_given_nothing() {
        assert_eq!(headless_folder(&[]), Ok(None));
    }

    #[test]
    fn starts_headless_on_the_given_folder_made_absolute() {
        let found = headless_folder(&args(&["--headless", "demo"]))
            .unwrap()
            .unwrap();
        assert!(found.is_absolute());
        assert!(found.ends_with("demo"));
    }

    #[test]
    fn refuses_anything_else_with_the_usage() {
        for list in [
            &["--headless"][..],
            &["demo"],
            &["--headless", "a", "b"],
            &["--window"],
        ] {
            assert_eq!(
                headless_folder(&args(list)),
                Err(USAGE.to_owned()),
                "{list:?}"
            );
        }
    }
}
