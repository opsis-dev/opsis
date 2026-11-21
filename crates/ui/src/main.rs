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

use clap::Parser;
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

fn absolute_folder(folder: &str) -> Result<PathBuf, String> {
    std::path::absolute(folder).map_err(|e| format!("{folder}: {e}"))
}

#[derive(Debug, Parser)]
#[command(name = "opsis", version)]
#[doc = "Opsis turns recorded websites and terminals into finished videos."]
struct Cli {
    #[arg(long, value_name = "PROJECT FOLDER", value_parser = absolute_folder)]
    #[doc = "Run with no window on this project, serving the Local API for agents until a signal ends it"]
    headless: Option<PathBuf>,
}

fn headless_folder<I, T>(args: I) -> Result<Option<PathBuf>, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Ok(Cli::try_parse_from(args)?.headless)
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
    let headless = headless_folder(std::env::args_os()).unwrap_or_else(|error| error.exit());
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

    fn parsed(list: &[&str]) -> Result<Option<PathBuf>, clap::Error> {
        headless_folder(std::iter::once("opsis").chain(list.iter().copied()))
    }

    #[test]
    fn starts_with_a_window_when_given_nothing() {
        assert_eq!(parsed(&[]).unwrap(), None);
    }

    #[test]
    fn starts_headless_on_the_given_folder_made_absolute() {
        let found = parsed(&["--headless", "demo"]).unwrap().unwrap();
        assert!(found.is_absolute());
        assert!(found.ends_with("demo"));
    }

    #[test]
    fn refuses_anything_else() {
        for list in [
            &["--headless"][..],
            &["demo"],
            &["--headless", "a", "b"],
            &["--window"],
        ] {
            assert!(parsed(list).is_err(), "{list:?}");
        }
    }

    #[test]
    fn answers_help_and_version() {
        assert_eq!(
            parsed(&["--help"]).unwrap_err().kind(),
            clap::error::ErrorKind::DisplayHelp
        );
        assert_eq!(
            parsed(&["--version"]).unwrap_err().kind(),
            clap::error::ErrorKind::DisplayVersion
        );
    }
}
