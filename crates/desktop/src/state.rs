use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use indexmap::IndexMap;
use opsis_fonts::library::FontLibrary;
use opsis_fonts::online::FontsOnline;
use opsis_project::files::ProjectFiles;
use opsis_project::watch::{LoadedFilesWatch, ProjectWatch};
use opsis_render::sidecar::Sidecar;
use opsis_store::db::{Database, StoreError};
use opsis_store::keys::{KeyError, Keyring};
use opsis_store::settings::{SettingsPatch, merged_settings};
use opsis_types::settings::AppSettings;
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;

use crate::engine::EngineChecks;
use crate::paths::AppPaths;
use crate::startup::StartupError;

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
    db: Database,
    settings: RwLock<AppSettings>,
    settings_writes: tokio::sync::Mutex<()>,
    pub keys: Result<Keyring, KeyError>,
    pub projects: ProjectFiles,
    pub fonts: FontLibrary,
    pub fonts_online: FontsOnline,
    pub engine: Mutex<EngineChecks>,
    pub sidecar: Arc<Sidecar>,
    pub editor: Mutex<Editor>,
    pub watching: Mutex<Watching>,
    pub tasks: Tasks,
}

impl AppState {
    pub async fn new(
        paths: AppPaths,
        keys: Result<Keyring, KeyError>,
        sidecar_log: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Result<Self, StartupError> {
        for dir in [&paths.user_data, &paths.views_temp, &paths.sidecar_temp] {
            std::fs::create_dir_all(dir)?;
        }
        let db = Database::open(&paths.database()).await?;
        let settings = db.read_settings().await?;
        Ok(Self {
            db,
            settings: RwLock::new(settings),
            settings_writes: tokio::sync::Mutex::new(()),
            keys,
            projects: ProjectFiles::default(),
            fonts: FontLibrary::new(&paths.user_data),
            fonts_online: FontsOnline::default(),
            engine: Mutex::default(),
            sidecar: Arc::new(Sidecar::new(paths.sidecar(), sidecar_log)),
            editor: Mutex::default(),
            watching: Mutex::default(),
            tasks: Tasks::default(),
            paths,
        })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn settings(&self) -> AppSettings {
        self.settings.read().expect("settings").clone()
    }

    pub async fn patch_settings(&self, patch: &SettingsPatch) -> Result<AppSettings, StoreError> {
        let _turn = self.settings_writes.lock().await;
        let next = merged_settings(&self.settings(), patch)?;
        self.db.write_settings(&next).await?;
        *self.settings.write().expect("settings") = next.clone();
        Ok(next)
    }

    pub fn engine_checks(&self) -> MutexGuard<'_, EngineChecks> {
        self.engine.lock().expect("engine checks")
    }

    pub fn editor(&self) -> MutexGuard<'_, Editor> {
        self.editor.lock().expect("editor state")
    }

    pub fn keys(&self) -> Result<&Keyring, &KeyError> {
        self.keys.as_ref()
    }
}
