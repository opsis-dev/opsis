use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::oneshot;

pub type PickedFolder = oneshot::Sender<Option<PathBuf>>;
pub type PickedFiles = oneshot::Sender<Vec<PathBuf>>;
pub type Closed = Box<dyn FnOnce() + Send>;

#[derive(Debug, Clone, Default)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Versions {
    pub shell: String,
    pub view: String,
}

pub trait Platform: Send + Sync {
    fn pick_folder(&self, title: &str, start: Option<&Path>, reply: PickedFolder);
    fn pick_file(&self, title: &str, reply: PickedFolder);
    fn pick_files(&self, title: &str, filter: Option<FileFilter>, reply: PickedFiles);
    fn open_path(&self, path: &Path) -> Result<(), String>;
    fn reveal_path(&self, path: &Path) -> Result<(), String>;
    fn notify(&self, title: &str, body: &str);
    fn editor_focused(&self) -> bool;
    fn raise_editor(&self);
    fn exit(&self, code: i32);
    fn versions(&self) -> Versions;
    fn open_view_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String>;
    fn close_view_window(&self, id: &str);
    fn open_terminal_window(&self, id: &str, title: &str, closed: Closed) -> Result<(), String>;
    fn close_terminal_window(&self, id: &str);
}

pub trait FrameSender: Send + Sync + std::fmt::Debug {
    fn id(&self) -> u32;
    fn send(&self, frame: Vec<u8>) -> bool;
}

pub type FrameChannel = Arc<dyn FrameSender>;

pub async fn pick_folder(
    platform: &dyn Platform,
    title: &str,
    start: Option<&Path>,
) -> Option<PathBuf> {
    let (reply, answer) = oneshot::channel();
    platform.pick_folder(title, start, reply);
    answer.await.ok().flatten()
}

pub async fn pick_file(platform: &dyn Platform, title: &str) -> Option<PathBuf> {
    let (reply, answer) = oneshot::channel();
    platform.pick_file(title, reply);
    answer.await.ok().flatten()
}

pub async fn pick_files(
    platform: &dyn Platform,
    title: &str,
    filter: Option<FileFilter>,
) -> Vec<PathBuf> {
    let (reply, answer) = oneshot::channel();
    platform.pick_files(title, filter, reply);
    answer.await.unwrap_or_default()
}
