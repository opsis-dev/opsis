#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands take State, AppHandle and WebviewWindow by value, the only way the command macro passes them"
)]

#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("The e2e feature embeds a WebDriver server; it is for debug builds only.");

pub mod bindings;
mod commands;
mod events;
mod exit;
mod file_server;
mod forward;
mod frames;
#[cfg_attr(
    not(target_os = "macos"),
    expect(dead_code, reason = "only macOS shows the native menu")
)]
mod native_menu;
mod page_log;
mod platform;
mod windows;

use std::sync::Arc;

use opsis_session::Session;
use opsis_session::paths::{AppPaths, HostFolders};
use opsis_store::keys::{KeyError, Keyring};
use tauri::{AppHandle, Manager, RunEvent};

use crate::platform::TauriPlatform;
use crate::windows::WindowRuntime;

pub fn session_of(app: &AppHandle) -> Session {
    app.state::<Session>().inner().clone()
}

fn host_folders(app: &tauri::App) -> Result<HostFolders, tauri::Error> {
    let path = app.path();
    Ok(HostFolders {
        config: path.config_dir()?,
        resources: path.resource_dir()?,
        home: path.home_dir()?,
    })
}

fn start(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let paths = AppPaths::new(&host_folders(app)?)?;
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
        app.on_menu_event(|app, event| native_menu::on_event(app, &event));
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
    let platform = Arc::new(TauriPlatform { app: app.clone() });
    let session = Session::start(paths, keys, platform).await?;
    app.manage(session.clone());
    let files = file_server::start(app)?;
    app.manage(files);
    app.manage(WindowRuntime::default());
    forward::start(app, &session);
    session.begin();
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
            if matches!(event, RunEvent::Exit)
                && let Some(session) = app.try_state::<Session>()
            {
                session.end();
            }
        });
}
