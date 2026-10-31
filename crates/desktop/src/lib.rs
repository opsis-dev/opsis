#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("The e2e feature embeds a WebDriver server; it is for debug builds only.");

mod analysis;
mod api;
pub mod bindings;
mod compile;
mod document;
mod engine;
mod error;
mod events;
mod exit;
mod file_server;
mod fonts;
mod frames;
mod gradients;
mod menu;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod native_menu;
mod node_results;
mod page_log;
mod paths;
mod projects;
mod run;
mod settings;
mod shared;
mod startup;
mod state;
mod views;
mod windows;

use opsis_store::keys::{KeyError, Keyring};
use tauri::{AppHandle, Manager, RunEvent};

use crate::analysis::Mixes;
use crate::exit::Exit;
use crate::paths::AppPaths;
use crate::run::RunHost;
use crate::state::AppState;
use crate::views::Views;
use crate::windows::WindowRuntime;

fn start(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let paths = AppPaths::new(app.handle())?;
    let ended = opsis_engine::children::end_children_of_gone_apps(
        &paths.temp_root,
        &opsis_engine::temp::process_alive,
    );
    if !ended.is_empty() {
        log::info!(
            "ended {} programs left running by earlier runs: {ended:?}",
            ended.len()
        );
    }
    let cleared = opsis_engine::temp::clear_stale_child_temps(
        &paths.temp_root,
        &opsis_engine::temp::process_alive,
    );
    if !cleared.is_empty() {
        log::info!(
            "removed {} temp folders left by earlier runs",
            cleared.len()
        );
    }
    #[cfg(target_os = "macos")]
    {
        app.set_menu(native_menu::build(app.handle())?)?;
        app.on_menu_event(native_menu::on_event);
    }
    let keys = Keyring::platform();
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = open(&handle, paths, keys).await {
            log::error!("Opsis couldn't start: {error}");
            handle.exit(1);
        }
    });
    Ok(())
}

async fn open(
    app: &AppHandle,
    paths: AppPaths,
    keys: Result<Keyring, KeyError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new(paths, keys).await?;
    let files = file_server::start(app)?;
    app.manage(state);
    app.manage(files);
    app.manage(RunHost::default());
    app.manage(shared::Shared::default());
    app.manage(Exit::default());
    app.manage(Views::default());
    app.manage(Mixes::default());
    app.manage(node_results::NodeKeys::default());
    app.manage(WindowRuntime::default());
    app.manage(api::Api::new(app));
    api::follow_setting(app);
    #[cfg(unix)]
    exit::close_on_signals(app);
    engine::verify_on_start(app);
    windows::open_main(app).map_err(|e| e.0)?;
    Ok(())
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let builder = bindings::builder();
    let invoke = builder.invoke_handler();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init());
    #[cfg(feature = "e2e")]
    let app = app.plugin(tauri_plugin_wdio_webdriver::init());
    app.invoke_handler(invoke)
        .setup(move |app| {
            builder.mount_events(app);
            start(app)
        })
        .on_window_event(windows::on_window_event)
        .on_page_load(|webview, payload| {
            log::info!(target: "page", "[{}] {:?} {}", webview.label(), payload.event(), payload.url());
        })
        .build(tauri::generate_context!())
        .expect("the app starts")
        .run(|app, event| {
            if let RunEvent::Exit = event
                && let Some(state) = app.try_state::<AppState>()
            {
                app.state::<RunHost>().cancel();
                let ended = opsis_engine::children::end_remembered_children();
                if !ended.is_empty() {
                    log::info!("ended the browsers still open: {ended:?}");
                }
                api::stop(app);
                opsis_engine::temp::remove_child_temp(&state.paths.views_temp);
            }
        });
}
