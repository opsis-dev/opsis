use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::notify::{self, RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use opsis_compile::asset_ref::{file_version_key, stays_in_project};
use opsis_engine::tools::shown;
use opsis_types::project::{
    FileVersions, META_DIR, MISSING_FILE, ProjectFileChange, ProjectFilesChange,
};
use opsis_types::workflow::WORKFLOW_FILE;

use crate::files::{ProjectFiles, parse_workflow_text, read_project_file};
use crate::paths::to_posix_rel;
use crate::stamps::file_stamp;

const DEBOUNCE: Duration = Duration::from_millis(150);

fn real(dir: &Path) -> PathBuf {
    std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf())
}

fn changed_paths(dir: &Path, result: DebounceEventResult) -> Vec<PathBuf> {
    match result {
        Ok(events) => events.into_iter().map(|e| e.path).collect(),
        Err(error) => {
            log::warn!("watching {}: {error}", dir.display());
            Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct ProjectWatch {
    _debouncer: Debouncer<RecommendedWatcher>,
}

pub fn watch_project(
    files: &ProjectFiles,
    dir: &Path,
    on_change: impl Fn(ProjectFileChange) + Send + 'static,
) -> notify::Result<ProjectWatch> {
    let files = files.clone();
    let folder = dir.to_path_buf();
    let mut announced = files.written_version(dir);
    let mut debouncer = new_debouncer(DEBOUNCE, move |result: DebounceEventResult| {
        let touched = changed_paths(&folder, result)
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == WORKFLOW_FILE));
        let current = match touched.then(|| read_project_file(&folder)) {
            Some(Ok(Some(current))) => current,
            Some(Err(error)) => return log::warn!("watching {}: {error}", folder.display()),
            _ => return,
        };
        if Some(&current.version) == files.written_version(&folder).as_ref()
            || Some(&current.version) == announced.as_ref()
        {
            return;
        }
        announced = Some(current.version.clone());
        on_change(ProjectFileChange {
            dir: shown(&folder),
            workflow: parse_workflow_text(&current.text).ok(),
            version: current.version,
        });
    })?;
    debouncer
        .watcher()
        .watch(dir, RecursiveMode::NonRecursive)?;
    Ok(ProjectWatch {
        _debouncer: debouncer,
    })
}

fn is_app_made(key: &str) -> bool {
    key == META_DIR || key.starts_with(&format!("{META_DIR}/"))
}

fn folder_of(key: &str) -> String {
    key.rsplit_once('/')
        .map_or(String::new(), |(folder, _)| folder.to_owned())
}

fn watched_refs(refs: &[String]) -> HashMap<String, Vec<String>> {
    let mut by_key: HashMap<String, Vec<String>> = HashMap::new();
    for r in refs.iter().filter(|r| stays_in_project(r)) {
        if let Some(key) = file_version_key(r).filter(|key| !is_app_made(key)) {
            by_key.entry(key).or_default().push(r.clone());
        }
    }
    by_key
}

#[derive(Debug)]
pub struct LoadedFilesWatch {
    debouncer: Debouncer<RecommendedWatcher>,
    dir: PathBuf,
    loaded: Arc<Mutex<HashMap<String, Vec<String>>>>,
    folders: HashSet<String>,
}

impl LoadedFilesWatch {
    pub fn start(
        dir: &Path,
        on_change: impl Fn(ProjectFilesChange) + Send + 'static,
    ) -> notify::Result<Self> {
        let loaded: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
        let keys = Arc::clone(&loaded);
        let folder = dir.to_path_buf();
        let real_dir = real(dir);
        let debouncer = new_debouncer(DEBOUNCE, move |result: DebounceEventResult| {
            let loaded = keys.lock().expect("loaded files").clone();
            let changed: HashSet<String> = changed_paths(&folder, result)
                .iter()
                .filter_map(|p| to_posix_rel(&real_dir, p))
                .filter(|key| loaded.contains_key(key))
                .collect();
            if changed.is_empty() {
                return;
            }
            let files: FileVersions = changed
                .into_iter()
                .flat_map(|key| {
                    let stamp =
                        file_stamp(&folder.join(&key)).unwrap_or_else(|| MISSING_FILE.to_owned());
                    loaded[&key]
                        .iter()
                        .map(move |r| (r.clone(), stamp.clone()))
                        .collect::<Vec<_>>()
                })
                .collect();
            on_change(ProjectFilesChange {
                dir: shown(&folder),
                files,
            });
        })?;
        Ok(Self {
            debouncer,
            dir: dir.to_path_buf(),
            loaded,
            folders: HashSet::new(),
        })
    }

    pub fn follow(&mut self, refs: &[String]) {
        let refs = watched_refs(refs);
        let folders: HashSet<String> = refs.keys().map(|k| folder_of(k)).collect();
        *self.loaded.lock().expect("loaded files") = refs;
        for gone in self.folders.difference(&folders) {
            if let Err(error) = self.debouncer.watcher().unwatch(&self.dir.join(gone)) {
                log::warn!("watching {}: {error}", self.dir.join(gone).display());
            }
        }
        let mut watched = HashSet::new();
        for folder in folders {
            let full = self.dir.join(&folder);
            let ok = self.folders.contains(&folder)
                || (full.is_dir()
                    && self
                        .debouncer
                        .watcher()
                        .watch(&full, RecursiveMode::NonRecursive)
                        .inspect_err(|error| log::warn!("watching {}: {error}", full.display()))
                        .is_ok());
            if ok {
                watched.insert(folder);
            }
        }
        self.folders = watched;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{Receiver, channel};

    use super::*;

    const WAIT: Duration = Duration::from_secs(3);
    const SETTLE: Duration = Duration::from_millis(500);

    fn watching(
        names: &[&str],
    ) -> (
        tempfile::TempDir,
        LoadedFilesWatch,
        Receiver<ProjectFilesChange>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::create_dir_all(dir.path().join(".opsis/cache")).unwrap();
        for name in names {
            std::fs::write(dir.path().join(name), "one").unwrap();
        }
        let (send, changes) = channel();
        let watch = LoadedFilesWatch::start(dir.path(), move |c| send.send(c).unwrap()).unwrap();
        (dir, watch, changes)
    }

    #[test]
    fn reports_a_loaded_file_replaced_in_place_with_its_new_stamp() {
        let (dir, mut watch, changes) = watching(&["assets/anim.lottie", "assets/other.png"]);
        watch.follow(&[
            "project:assets/anim.lottie".into(),
            "project:./assets/anim.lottie".into(),
        ]);
        std::fs::write(dir.path().join("assets/other.png"), "two").unwrap();
        std::fs::write(dir.path().join("assets/anim.lottie"), "three!").unwrap();
        let change = changes.recv_timeout(WAIT).unwrap();
        assert_eq!(
            change.files.keys().collect::<Vec<_>>(),
            ["project:./assets/anim.lottie", "project:assets/anim.lottie"]
        );
        assert!(change.files["project:assets/anim.lottie"].starts_with("6:"));
        assert!(change.files["project:./assets/anim.lottie"].starts_with("6:"));
    }

    #[test]
    fn reports_a_deleted_file_as_missing_and_stops_once_it_isnt_loaded() {
        let (dir, mut watch, changes) = watching(&["assets/a.wav"]);
        watch.follow(&["project:assets/a.wav".into()]);
        std::fs::remove_file(dir.path().join("assets/a.wav")).unwrap();
        let change = changes.recv_timeout(WAIT).unwrap();
        assert_eq!(
            change.files,
            FileVersions::from([("project:assets/a.wav".into(), MISSING_FILE.into())])
        );
        watch.follow(&[]);
        std::fs::write(dir.path().join("assets/a.wav"), "back").unwrap();
        assert!(changes.recv_timeout(SETTLE).is_err());
    }

    #[test]
    fn leaves_the_apps_own_files_alone() {
        let (dir, mut watch, changes) = watching(&[".opsis/cache/take.mp4"]);
        watch.follow(&["project:.opsis/cache/take.mp4".into()]);
        std::fs::write(dir.path().join(".opsis/cache/take.mp4"), "two").unwrap();
        assert!(changes.recv_timeout(SETTLE).is_err());
    }

    #[test]
    fn reports_outside_changes_to_the_project_file_but_not_its_own_saves() {
        let parent = tempfile::tempdir().unwrap();
        let files = ProjectFiles::default();
        let created = files.create(parent.path(), "Watched", "blank").unwrap();
        let (send, changes) = channel();
        let _watch = watch_project(&files, &created.dir, move |c| send.send(c).unwrap()).unwrap();
        let mut ours = created.workflow.clone();
        ours.name = "Ours".into();
        files
            .save(&created.dir, &ours, Some(&created.version))
            .unwrap();
        assert!(changes.recv_timeout(SETTLE).is_err());
        let mut theirs = ours.clone();
        theirs.name = "Theirs".into();
        let text = crate::files::workflow_text(&theirs);
        std::fs::write(created.dir.join(WORKFLOW_FILE), &text).unwrap();
        let change = changes.recv_timeout(WAIT).unwrap();
        assert_eq!(change.workflow.map(|w| w.name), Some("Theirs".into()));
        assert_eq!(change.version, crate::files::version_of(&text));
    }
}
