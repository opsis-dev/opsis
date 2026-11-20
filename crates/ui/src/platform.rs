use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use opsis_qt::DialogKind;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};

#[derive(Debug)]
enum Reply {
    One(PickedFolder),
    Many(PickedFiles),
}

#[derive(Default)]
pub struct QtPlatform {
    headless: bool,
    next: AtomicU64,
    waiting: Mutex<HashMap<u64, Reply>>,
    active: AtomicBool,
    views: Mutex<HashMap<String, Closed>>,
    terminals: Mutex<HashMap<String, Closed>>,
}

impl std::fmt::Debug for QtPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QtPlatform")
            .field("headless", &self.headless)
            .field("next", &self.next)
            .field("waiting", &self.waiting)
            .field("active", &self.active)
            .finish_non_exhaustive()
    }
}

impl QtPlatform {
    pub fn new(headless: bool) -> Self {
        Self {
            headless,
            ..Self::default()
        }
    }

    fn ask(
        &self,
        kind: DialogKind,
        title: &str,
        start: Option<&Path>,
        filter: Option<&FileFilter>,
        reply: Reply,
    ) {
        if self.headless {
            log::info!("headless: no one to answer \"{title}\", so it is cancelled");
            let delivered = match reply {
                Reply::One(sender) => sender.send(None).is_ok(),
                Reply::Many(sender) => sender.send(Vec::new()).is_ok(),
            };
            if !delivered {
                log::debug!("a dialog was cancelled after its caller left");
            }
            return;
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.waiting.lock().expect("dialogs").insert(id, reply);
        let start = start.map_or_default(|p| p.to_string_lossy().into_owned());
        let extensions = filter.map_or_default(|f| f.extensions.join(" "));
        let named = filter.map(|f| (f.name.as_str(), extensions.as_str()));
        opsis_qt::request_dialog(kind, id, title, &start, named);
    }

    pub fn answer(&self, id: u64, paths: Vec<String>) {
        let Some(reply) = self.waiting.lock().expect("dialogs").remove(&id) else {
            return;
        };
        let mut paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        let delivered = match reply {
            Reply::One(sender) => sender.send(paths.pop()).is_ok(),
            Reply::Many(sender) => sender.send(paths).is_ok(),
        };
        if !delivered {
            log::debug!("a dialog answered after its caller left");
        }
    }

    pub fn view_window_closed(&self, id: &str) {
        let closed = self.views.lock().expect("view windows").remove(id);
        if let Some(closed) = closed {
            closed();
        }
    }

    pub fn terminal_window_closed(&self, id: &str) {
        let closed = self.terminals.lock().expect("terminal windows").remove(id);
        if let Some(closed) = closed {
            closed();
        }
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}

impl Platform for QtPlatform {
    fn pick_folder(&self, title: &str, start: Option<&Path>, reply: PickedFolder) {
        self.ask(DialogKind::Folder, title, start, None, Reply::One(reply));
    }

    fn pick_file(&self, title: &str, reply: PickedFolder) {
        self.ask(DialogKind::File, title, None, None, Reply::One(reply));
    }

    fn pick_files(&self, title: &str, filter: Option<FileFilter>, reply: PickedFiles) {
        self.ask(
            DialogKind::Files,
            title,
            None,
            filter.as_ref(),
            Reply::Many(reply),
        );
    }

    fn open_path(&self, path: &Path) -> Result<(), String> {
        opsis_qt::open_path(&path.to_string_lossy());
        Ok(())
    }

    fn reveal_path(&self, path: &Path) -> Result<(), String> {
        opsis_qt::reveal_path(&path.to_string_lossy());
        Ok(())
    }

    fn notify(&self, title: &str, body: &str) {
        if self.headless {
            log::info!("headless: {title} {body}");
            return;
        }
        opsis_qt::notify(title, body);
    }

    fn editor_focused(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn raise_editor(&self) {
        opsis_qt::raise_editor();
    }

    fn exit(&self, code: i32) {
        opsis_qt::quit(code);
    }

    fn versions(&self) -> Versions {
        Versions {
            shell: format!("Qt {}", opsis_qt::qt_version()),
            view: String::new(),
        }
    }

    fn open_view_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String> {
        self.views
            .lock()
            .expect("view windows")
            .insert(id.to_owned(), closed);
        opsis_qt::open_view_window(id, title);
        Ok(())
    }

    fn close_view_window(&self, id: &str) {
        self.views.lock().expect("view windows").remove(id);
        opsis_qt::close_view_window(id);
    }

    fn open_terminal_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String> {
        self.terminals
            .lock()
            .expect("terminal windows")
            .insert(id.to_owned(), closed);
        opsis_qt::open_terminal_window(id, title);
        Ok(())
    }

    fn close_terminal_window(&self, id: &str) {
        self.terminals.lock().expect("terminal windows").remove(id);
        opsis_qt::close_terminal_window(id);
    }

    fn headless(&self) -> bool {
        self.headless
    }
}

#[cfg(test)]
mod tests {
    use opsis_session::platform::{pick_file, pick_files, pick_folder};

    use super::*;

    #[tokio::test]
    async fn a_headless_platform_cancels_every_picker() {
        let platform = QtPlatform::new(true);
        assert_eq!(pick_folder(&platform, "Choose a folder", None).await, None);
        assert_eq!(pick_file(&platform, "Choose a file").await, None);
        assert_eq!(
            pick_files(&platform, "Choose files", None).await,
            Vec::<PathBuf>::new()
        );
        assert!(platform.headless());
    }
}
