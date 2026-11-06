use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use opsis_session::events::MAIN_WINDOW;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

use crate::windows::{open_terminal_window, terminal_label};

const VIEW_PREFIX: &str = "view-";

pub struct TauriPlatform {
    pub app: AppHandle,
}

impl fmt::Debug for TauriPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TauriPlatform").finish_non_exhaustive()
    }
}

fn local(file: FilePath) -> Option<std::path::PathBuf> {
    file.into_path().ok()
}

fn on_destroyed(window: &WebviewWindow, closed: Closed) {
    let closed = Mutex::new(Some(closed));
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed)
            && let Some(closed) = closed.lock().expect("window close").take()
        {
            closed();
        }
    });
}

fn destroy(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label)
        && let Err(error) = window.destroy()
    {
        log::warn!("closing the {label} window: {error}");
    }
}

impl Platform for TauriPlatform {
    fn pick_folder(&self, title: &str, start: Option<&Path>, reply: PickedFolder) {
        let mut dialog = self.app.dialog().file().set_title(title);
        if let Some(dir) = start {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_folder(move |folder| {
            if reply.send(folder.and_then(local)).is_err() {
                log::debug!("the folder dialog closed after its caller left");
            }
        });
    }

    fn pick_file(&self, title: &str, reply: PickedFolder) {
        self.app
            .dialog()
            .file()
            .set_title(title)
            .pick_file(move |file| {
                if reply.send(file.and_then(local)).is_err() {
                    log::debug!("the file dialog closed after its caller left");
                }
            });
    }

    fn pick_files(&self, title: &str, filter: Option<FileFilter>, reply: PickedFiles) {
        let mut dialog = self.app.dialog().file().set_title(title);
        if let Some(filter) = &filter {
            let extensions: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
            dialog = dialog.add_filter(&filter.name, &extensions);
        }
        dialog.pick_files(move |files| {
            let files = files
                .unwrap_or_default()
                .into_iter()
                .filter_map(local)
                .collect();
            if reply.send(files).is_err() {
                log::debug!("the file dialog closed after its caller left");
            }
        });
    }

    fn open_path(&self, path: &Path) -> Result<(), String> {
        self.app
            .opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .map_err(|error| error.to_string())
    }

    fn reveal_path(&self, path: &Path) -> Result<(), String> {
        self.app
            .opener()
            .reveal_item_in_dir(path)
            .map_err(|error| error.to_string())
    }

    fn notify(&self, title: &str, body: &str) {
        if let Err(error) = self
            .app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            log::warn!("showing a notification: {error}");
        }
    }

    fn editor_focused(&self) -> bool {
        self.app
            .get_webview_window(MAIN_WINDOW)
            .and_then(|w| w.is_focused().ok())
            .unwrap_or(false)
    }

    fn raise_editor(&self) {
        if let Some(window) = self.app.get_webview_window(MAIN_WINDOW) {
            #[expect(
                clippy::or_fun_call,
                reason = "focus the window even when un-minimising it fails, so what it asks is seen"
            )]
            if let Err(error) = window.unminimize().and(window.set_focus()) {
                log::warn!("raising the editor: {error}");
            }
        }
    }

    fn exit(&self, code: i32) {
        self.app.exit(code);
    }

    fn versions(&self) -> Versions {
        Versions {
            shell: tauri::VERSION.to_owned(),
            view: tauri::webview_version().unwrap_or_default(),
        }
    }

    fn open_view_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String> {
        let label = format!("{VIEW_PREFIX}{id}");
        let url = WebviewUrl::App(format!("index.html#/view/{id}").into());
        let window = WebviewWindowBuilder::new(&self.app, &label, url)
            .title(title)
            .inner_size(1280.0, 900.0)
            .build()
            .map_err(|error| error.to_string())?;
        on_destroyed(&window, closed);
        Ok(())
    }

    fn close_view_window(&self, id: &str) {
        destroy(&self.app, &format!("{VIEW_PREFIX}{id}"));
    }

    fn open_terminal_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String> {
        let app = self.app.clone();
        let id = id.to_owned();
        let title = title.to_owned();
        tauri::async_runtime::spawn(async move {
            match open_terminal_window(&app, &id, &title).await {
                Ok(window) => on_destroyed(&window, closed),
                Err(error) => {
                    log::error!("opening the terminal window: {}", error.0);
                    closed();
                }
            }
        });
        Ok(())
    }

    fn close_terminal_window(&self, id: &str) {
        destroy(&self.app, &terminal_label(id));
    }
}
