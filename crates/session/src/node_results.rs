use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use opsis_compile::compile::CompileResult;
use opsis_compile::graph::{GraphIssue, IssueLevel, is_structural, topo_sort};
use opsis_compile::nodes::media::IMPORT_TYPE;
use opsis_compile::registry::node_def;
use opsis_compile::vars::workflow_vars;
use opsis_media::executors::{FileCheckError, check_media_file};
use opsis_media::ffmpeg::ToolError;
use opsis_project::paths::{project_paths, ref_to_path};
use opsis_runner::cache::{read_cached, write_cached};
use opsis_runner::context::now_ms;
use opsis_runner::keys::{KeyInputs, Release, hash_nodes};
use opsis_types::values::{CompileValue, NodeResult};
use opsis_types::workflow::Workflow;

use crate::Session;
use crate::engine::{engine_files, engine_tools};
use crate::state::AppState;

const MEDIA_PORT: &str = "media";
const UNREADABLE: &str = "This file can't be read as a video, an image or a sound.";
const RETRY_OPENING: Duration = Duration::from_secs(10);

#[derive(Debug, Default)]
pub struct NodeKeys {
    keyed: Mutex<Keyed>,
}

#[derive(Debug, Default)]
struct Keyed {
    dir: Option<PathBuf>,
    current: HashMap<String, String>,
    looked: HashMap<String, String>,
    checking: HashSet<String>,
    unreadable: HashMap<String, Unreadable>,
}

#[derive(Debug, Clone)]
struct Unreadable {
    message: String,
    retry_at: Option<Instant>,
}

impl Unreadable {
    fn holds(&self, now: Instant) -> bool {
        self.retry_at.is_none_or(|at| now < at)
    }
}

fn adopt(
    results: &mut IndexMap<String, NodeResult>,
    open: Option<&Path>,
    dir: &Path,
    found: impl IntoIterator<Item = (String, NodeResult)>,
) -> bool {
    if open != Some(dir) {
        return false;
    }
    results.extend(found);
    true
}

#[derive(Debug)]
pub struct Refreshed {
    pub outdated: Vec<String>,
    pub problems: Vec<GraphIssue>,
}

fn current_keys(app: &Session, dir: &Path, workflow: &Workflow) -> HashMap<String, String> {
    let state = app.state::<AppState>();
    let engine = engine_files(state);
    let version = crate::APP_VERSION.to_owned();
    let executors = crate::run::executors(app, &state.paths.views_temp);
    let vars = workflow_vars(&workflow.name, &workflow.variables);
    let sorted = topo_sort(workflow);
    let keys = hash_nodes(
        &sorted.sorted,
        &KeyInputs {
            workflow,
            vars: &vars,
            project_dir: dir,
            executors: &executors,
            release: Release {
                app: &version,
                engine: &engine.stamp,
            },
        },
    );
    sorted
        .sorted
        .iter()
        .filter(|n| !n.is_disabled() && !is_structural(n))
        .filter(|n| node_def(&n.kind).is_some_and(|d| d.executes))
        .filter_map(|n| Some((n.id.clone(), keys.get(&n.id)?.clone())))
        .collect()
}

fn saved_result(dir: &Path, key: &str) -> Option<NodeResult> {
    read_cached(&project_paths(dir).cache, key, dir).unwrap_or_else(|error| {
        log::warn!("reading the saved result {key}: {error}");
        None
    })
}

pub fn refresh(
    app: &Session,
    dir: &Path,
    workflow: &Workflow,
    results: &mut IndexMap<String, NodeResult>,
) -> Refreshed {
    let keys = current_keys(app, dir, workflow);
    let node_keys = app.state::<NodeKeys>();
    let wanted: Vec<(String, String)> = {
        let mut keyed = node_keys.keyed.lock().expect("node keys");
        if keyed.dir.as_deref() != Some(dir) {
            *keyed = Keyed {
                dir: Some(dir.to_path_buf()),
                ..Keyed::default()
            };
        }
        keyed.current.clone_from(&keys);
        keys.iter()
            .filter(|(id, key)| results.get(*id).is_none_or(|r| &r.hash != *key))
            .filter(|(id, key)| keyed.looked.get(*id) != Some(*key))
            .map(|(id, key)| (id.clone(), key.clone()))
            .collect()
    };
    let found: Vec<(String, NodeResult)> = wanted
        .iter()
        .filter_map(|(id, key)| Some((id.clone(), saved_result(dir, key)?)))
        .collect();
    let keyed = {
        let mut keyed = node_keys.keyed.lock().expect("node keys");
        keyed.looked.extend(wanted);
        (keyed.checking.clone(), keyed.unreadable.clone())
    };
    if !found.is_empty() {
        let state = app.state::<AppState>();
        let mut editor = state.editor();
        let open = editor.dir();
        if adopt(&mut editor.results, open.as_deref(), dir, found.clone()) {
            results.extend(found);
        }
    }
    let (checking, unreadable) = keyed;
    let outdated = results
        .iter()
        .filter(|(id, r)| {
            keys.get(*id)
                .is_some_and(|k| k != &r.hash && !checking.contains(k))
        })
        .map(|(id, _)| id.clone())
        .collect();
    let problems = unreadable_issues(&keys, &unreadable);
    Refreshed { outdated, problems }
}

fn unreadable_issues(
    keys: &HashMap<String, String>,
    unreadable: &HashMap<String, Unreadable>,
) -> Vec<GraphIssue> {
    keys.iter()
        .filter_map(|(id, key)| {
            Some(GraphIssue {
                level: IssueLevel::Warning,
                message: unreadable.get(key)?.message.clone(),
                node_id: Some(id.clone()),
                edge_id: None,
            })
        })
        .collect()
}

pub fn held(app: &Session, dir: &Path, results: &IndexMap<String, NodeResult>) -> Refreshed {
    let node_keys = app.state::<NodeKeys>();
    let keyed = node_keys.keyed.lock().expect("node keys");
    if keyed.dir.as_deref() != Some(dir) {
        return Refreshed {
            outdated: Vec::new(),
            problems: Vec::new(),
        };
    }
    let outdated = results
        .iter()
        .filter(|(id, r)| {
            keyed
                .current
                .get(*id)
                .is_some_and(|k| k != &r.hash && !keyed.checking.contains(k))
        })
        .map(|(id, _)| id.clone())
        .collect();
    Refreshed {
        outdated,
        problems: unreadable_issues(&keyed.current, &keyed.unreadable),
    }
}

pub fn file_problems(app: &Session) -> Vec<GraphIssue> {
    let open = app.state::<AppState>().editor().dir();
    let node_keys = app.state::<NodeKeys>();
    let keyed = node_keys.keyed.lock().expect("node keys");
    if keyed.dir.is_none() || keyed.dir != open {
        return Vec::new();
    }
    unreadable_issues(&keyed.current, &keyed.unreadable)
}

fn media_file_src(compiled: &CompileResult, id: &str) -> Option<String> {
    compiled
        .values
        .get(id)?
        .get(MEDIA_PORT)
        .and_then(CompileValue::as_media)?
        .src
        .clone()
}

pub fn check_media_files(
    app: &Session,
    dir: &Path,
    workflow: &Workflow,
    compiled: &CompileResult,
    results: &IndexMap<String, NodeResult>,
) {
    let state = app.state::<AppState>();
    let public = state.paths.public_dir.clone();
    let now = Instant::now();
    let node_keys = app.state::<NodeKeys>();
    for node in workflow.nodes.iter().filter(|n| n.kind == IMPORT_TYPE) {
        let key = {
            let keyed = node_keys.keyed.lock().expect("node keys");
            let Some(key) = keyed.current.get(&node.id).cloned() else {
                continue;
            };
            if results.get(&node.id).is_some_and(|r| r.hash == key)
                || keyed.checking.contains(&key)
                || keyed.unreadable.get(&key).is_some_and(|u| u.holds(now))
            {
                continue;
            }
            key
        };
        let Some(src) = media_file_src(compiled, &node.id) else {
            continue;
        };
        let Ok(file) = ref_to_path(dir, &public, &src) else {
            continue;
        };
        if !file.is_file() {
            continue;
        }
        let first = node_keys
            .keyed
            .lock()
            .expect("node keys")
            .checking
            .insert(key.clone());
        if !first {
            continue;
        }
        let check = FileCheck {
            app: app.clone(),
            dir: dir.to_path_buf(),
            id: node.id.clone(),
            key,
            src,
            file,
        };
        crate::runtime::spawn(check.run());
    }
}

struct FileCheck {
    app: Session,
    dir: PathBuf,
    id: String,
    key: String,
    src: String,
    file: PathBuf,
}

fn remembered(error: &FileCheckError) -> Option<Unreadable> {
    match error {
        FileCheckError::Tool(ToolError::Failed { .. }) => Some(Unreadable {
            message: UNREADABLE.to_owned(),
            retry_at: None,
        }),
        FileCheckError::Folder | FileCheckError::Unopenable(_) => Some(Unreadable {
            message: error.to_string(),
            retry_at: Some(Instant::now() + RETRY_OPENING),
        }),
        FileCheckError::Missing
        | FileCheckError::Tool(ToolError::Missing(_) | ToolError::Cancelled) => None,
    }
}

impl FileCheck {
    async fn run(self) {
        let state = self.app.state::<AppState>();
        let tools = engine_tools(state, &state.paths.views_temp);
        let started = Instant::now();
        let checked = check_media_file(&*tools, &self.file, &self.src).await;
        let keys = self.app.state::<NodeKeys>();
        let outcome = match checked {
            Ok(checked) => {
                let result = NodeResult {
                    hash: self.key.clone(),
                    outputs: IndexMap::from([(
                        MEDIA_PORT.to_owned(),
                        CompileValue::Media(checked.media),
                    )]),
                    artifacts: Vec::new(),
                    finished_at: now_ms(),
                    duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                    trace_id: None,
                    measured: vec![checked.measured],
                };
                if let Err(error) = write_cached(&project_paths(&self.dir).cache, &result) {
                    log::warn!("saving the check of {}: {error}", self.src);
                }
                Some(result)
            }
            Err(error) => {
                log::info!("checking {}: {error}", self.src);
                if let Some(unreadable) = remembered(&error) {
                    if let Some(at) = unreadable.retry_at {
                        let app = self.app.clone();
                        crate::runtime::spawn(async move {
                            tokio::time::sleep_until(at.into()).await;
                            crate::compile::request(&app);
                        });
                    }
                    keys.keyed
                        .lock()
                        .expect("node keys")
                        .unreadable
                        .insert(self.key.clone(), unreadable);
                }
                None
            }
        };
        let current = {
            let mut keyed = keys.keyed.lock().expect("node keys");
            keyed.checking.remove(&self.key);
            if outcome.is_some() {
                keyed.unreadable.remove(&self.key);
            }
            keyed.current.get(&self.id) == Some(&self.key)
        };
        let state = self.app.state::<AppState>();
        let open_here = {
            let mut editor = state.editor();
            let open = editor.dir();
            match outcome {
                Some(result) if current => adopt(
                    &mut editor.results,
                    open.as_deref(),
                    &self.dir,
                    [(self.id, result)],
                ),
                _ => open.as_deref() == Some(self.dir.as_path()),
            }
        };
        if current && open_here {
            crate::compile::request(&self.app);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use indexmap::IndexMap;
    use opsis_types::values::NodeResult;
    use serde_json::json;

    use super::adopt;

    fn result(hash: &str) -> NodeResult {
        serde_json::from_value(json!({ "hash": hash, "outputs": {}, "artifacts": [], "finishedAt": 0, "durationMs": 0 }))
            .unwrap()
    }

    #[test]
    fn keeps_results_only_for_the_project_still_open() {
        let mut results = IndexMap::new();
        let old = Path::new("/projects/old");
        let new = Path::new("/projects/new");
        assert!(!adopt(
            &mut results,
            Some(new),
            old,
            [("render".to_owned(), result("a"))]
        ));
        assert!(!adopt(
            &mut results,
            None,
            old,
            [("render".to_owned(), result("a"))]
        ));
        assert!(results.is_empty());
        assert!(adopt(
            &mut results,
            Some(new),
            new,
            [("render".to_owned(), result("b"))]
        ));
        assert_eq!(results["render"].hash, "b");
    }
}
