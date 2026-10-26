use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use indexmap::IndexMap;
use opsis_fonts::library::FontLibrary;
use opsis_fonts::online::FontsOnline;
use opsis_media::executors::Tools;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_project::files::ProjectFiles;
use opsis_project::watch::{LoadedFilesWatch, ProjectWatch};
use opsis_render::sidecar::Sidecar;
use opsis_store::db::Database;
use opsis_store::keys::Keyring;
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;

use crate::paths::AppPaths;

#[derive(Default)]
pub struct Editor {
    pub dir: Option<PathBuf>,
    pub workflow: Option<Workflow>,
    pub results: IndexMap<String, NodeResult>,
    pub selected: Option<String>,
    pub inspected: Option<String>,
}

#[derive(Default)]
pub struct Watching {
    pub dir: Option<PathBuf>,
    pub project: Option<ProjectWatch>,
    pub files: Option<LoadedFilesWatch>,
    pub refs: Option<(PathBuf, Vec<String>)>,
}

#[derive(Default)]
pub struct Tasks(Mutex<HashMap<String, usize>>);

impl Tasks {
    pub fn begin(&self, label: &str) {
        *self
            .0
            .lock()
            .expect("tasks")
            .entry(label.to_owned())
            .or_default() += 1;
    }

    pub fn end(&self, label: &str) {
        let mut tasks = self.0.lock().expect("tasks");
        match tasks.get_mut(label) {
            Some(count) if *count > 1 => *count -= 1,
            _ => {
                tasks.remove(label);
            }
        }
    }

    pub fn labels(&self) -> Vec<String> {
        self.0.lock().expect("tasks").keys().cloned().collect()
    }
}

pub struct AppState {
    pub paths: AppPaths,
    pub(crate) db: Mutex<Database>,
    pub keys: Result<Keyring, String>,
    pub projects: ProjectFiles,
    pub fonts: FontLibrary,
    pub fonts_online: FontsOnline,
    pub tools: Tools,
    pub sidecar: Arc<Sidecar>,
    pub editor: Mutex<Editor>,
    pub watching: Mutex<Watching>,
    pub tasks: Tasks,
}

impl AppState {
    pub fn new(
        paths: AppPaths,
        sidecar_log: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Result<Self, String> {
        std::fs::create_dir_all(&paths.user_data).map_err(|e| e.to_string())?;
        let db = Database::open(&paths.database()).map_err(|e| e.to_string())?;
        let engine = opsis_engine::paths::engine_paths(&paths.engine_dir(), paths.platform);
        Ok(Self {
            db: Mutex::new(db),
            keys: Keyring::platform().map_err(|e| e.to_string()),
            projects: ProjectFiles::default(),
            fonts: FontLibrary::new(&paths.user_data),
            fonts_online: FontsOnline::default(),
            tools: Arc::new(Ffmpeg::new(&engine)),
            sidecar: Arc::new(Sidecar::new(paths.sidecar(), sidecar_log)),
            editor: Mutex::default(),
            watching: Mutex::default(),
            tasks: Tasks::default(),
            paths,
        })
    }

    pub fn db(&self) -> MutexGuard<'_, Database> {
        self.db.lock().expect("app database")
    }

    pub fn editor(&self) -> MutexGuard<'_, Editor> {
        self.editor.lock().expect("editor state")
    }

    pub fn keys(&self) -> Result<&Keyring, String> {
        self.keys.as_ref().map_err(Clone::clone)
    }
}
