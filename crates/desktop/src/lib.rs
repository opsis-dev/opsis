#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("The e2e feature embeds a WebDriver server; it is for debug builds only.");

mod analysis;
mod api;
pub mod bindings;
mod compile;
mod error;
mod events;
mod exit;
mod file_server;
mod fonts;
mod frames;
mod menu;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod native_menu;
mod page_log;
mod paths;
mod projects;
mod relay;
mod run;
mod settings;
mod state;
mod views;
mod windows;

use std::sync::Arc;

use tauri::{Manager, RunEvent};

use crate::analysis::Mixes;
use crate::exit::Exit;
use crate::paths::AppPaths;
use crate::run::RunHost;
use crate::state::AppState;
use crate::views::Views;
use crate::windows::WindowRuntime;

fn start(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let paths = AppPaths::new(app.handle())?;
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
    let state = AppState::new(paths, Arc::new(|line: &str| log::info!("sidecar: {line}")))?;
    settings::complete_licences(&state);
    app.manage(state);
    app.manage(file_server::start(app.handle())?);
    app.manage(RunHost::default());
    app.manage(Exit::default());
    app.manage(Views::default());
    app.manage(Mixes::default());
    app.manage(WindowRuntime::default());
    app.manage(api::Api::new(app.handle()));
    api::follow_setting(app.handle());
    #[cfg(target_os = "macos")]
    {
        app.set_menu(native_menu::build(app.handle())?)?;
        app.on_menu_event(native_menu::on_event);
    }
    windows::open_main(app.handle()).map_err(|e| e.0)?;
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
            if webview.label() == windows::MAIN_WINDOW
                && payload.event() == tauri::webview::PageLoadEvent::Started
            {
                webview.app_handle().state::<api::Api>().editor_reloaded();
            }
        })
        .build(tauri::generate_context!())
        .expect("the app starts")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<RunHost>().cancel();
                api::stop(app);
            }
        });
}
