use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use opsis_qt::DialogKind;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};

const NOT_YET: &str = "This window isn't in the native editor yet.";

#[derive(Debug)]
enum Reply {
    One(PickedFolder),
    Many(PickedFiles),
}

#[derive(Debug, Default)]
pub struct QtPlatform {
    next: AtomicU64,
    waiting: Mutex<HashMap<u64, Reply>>,
    active: AtomicBool,
}

impl QtPlatform {
    fn ask(
        &self,
        kind: DialogKind,
        title: &str,
        start: Option<&Path>,
        filter: Option<&FileFilter>,
        reply: Reply,
    ) {
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

    fn open_view_window(&self, _id: &str, _title: &str, closed: Closed) -> Result<(), String> {
        closed();
        Err(NOT_YET.to_owned())
    }

    fn close_view_window(&self, _id: &str) {}

    fn open_terminal_window(&self, _id: &str, _title: &str, closed: Closed) -> Result<(), String> {
        closed();
        Err(NOT_YET.to_owned())
    }

    fn close_terminal_window(&self, _id: &str) {}
}
