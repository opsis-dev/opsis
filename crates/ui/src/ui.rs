use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use opsis_compile::catalog::{EditorNode, editor_catalog};
use opsis_compile::compile::compile_workflow;
use opsis_compile::edits::EditError;
use opsis_compile::edits::workflow::absolute_position;
use opsis_compile::graph::connection_problem_now;
use opsis_compile::node::Platform;
use opsis_document::Document;
use opsis_project::files::{ProjectError, ProjectFiles};
use opsis_qt::{
    Choice, EventKind, GraphView, Host, HostResult, InspectorView, NodeMove, PaletteEntry,
    ProjectEntry, SettingsView,
};
use opsis_store::db::{Database, StoreError};
use opsis_store::settings::merged_settings;
use opsis_types::edits::{EdgeEnds, NodePatch, WorkflowEdit};
use opsis_types::project::SaveResult;
use opsis_types::settings::{AppSettings, settings_book};
use opsis_types::workflow::{Position, WORKFLOW_FILE};
use serde_json::{Map, Value};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

use crate::{graph, inspector, look};

const AUTOSAVE_DELAY: Duration = Duration::from_millis(800);
const LAYOUT_FILE: &str = "layout.json";
const BENCH_KINDS: [&str; 4] = [
    "browser.open",
    "browser.click",
    "browser.type",
    "browser.wait",
];
const BENCH_COLUMNS: u32 = 10;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("No project is open.")]
    NoProject,
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Edit(#[from] EditError),
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("The settings couldn't be read: {0}")]
    Settings(#[from] serde_json::Error),
}

type UiResult<T> = Result<T, UiError>;

#[derive(Debug, Default)]
struct Editor {
    document: Option<Document>,
    selected: Option<String>,
}

struct Inner {
    runtime: Runtime,
    db: Database,
    projects: ProjectFiles,
    editor: Mutex<Editor>,
    settings: Mutex<AppSettings>,
    events: broadcast::Sender<EventKind>,
    autosave: AtomicU64,
    data: PathBuf,
    catalog: IndexMap<String, EditorNode>,
    started: Instant,
}

pub struct Ui {
    inner: Arc<Inner>,
}

impl fmt::Debug for Ui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ui")
            .field("data", &self.inner.data)
            .finish_non_exhaustive()
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn data_dir() -> PathBuf {
    std::env::var_os("OPSIS_USER_DATA").map_or_else(
        || {
            let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
            home.join(".cache").join("opsis-qt-spike").join("data")
        },
        PathBuf::from,
    )
}

fn fonts_dir() -> String {
    repo_root()
        .join("public")
        .join("fonts")
        .join("bundled")
        .to_string_lossy()
        .into_owned()
}

pub fn start() -> UiResult<Ui> {
    let started = Instant::now();
    let data = data_dir();
    std::fs::create_dir_all(data.join("projects"))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let db = runtime.block_on(Database::open(&data.join("opsis.db")))?;
    let settings = runtime.block_on(db.read_settings())?;
    let catalog = editor_catalog(Platform::current())
        .into_iter()
        .map(|node| (node.kind.clone(), node))
        .collect();
    let (events, _) = broadcast::channel(256);
    Ok(Ui {
        inner: Arc::new(Inner {
            runtime,
            db,
            projects: ProjectFiles::default(),
            editor: Mutex::new(Editor::default()),
            settings: Mutex::new(settings),
            events,
            autosave: AtomicU64::new(0),
            data,
            catalog,
            started,
        }),
    })
}

impl Inner {
    fn editor(&self) -> MutexGuard<'_, Editor> {
        self.editor.lock().expect("editor")
    }

    fn send(&self, kind: EventKind) {
        let _ = self.events.send(kind);
    }

    fn write(&self) -> UiResult<()> {
        let (dir, workflow, revision, base) = {
            let editor = self.editor();
            let Some(doc) = editor.document.as_ref() else {
                return Ok(());
            };
            if !doc.is_dirty() {
                return Ok(());
            }
            (
                doc.dir().to_path_buf(),
                doc.workflow().clone(),
                doc.revision(),
                doc.file_version().to_owned(),
            )
        };
        let saved = self.projects.save(&dir, &workflow, Some(&base))?;
        if let SaveResult::Saved { version } = saved
            && let Some(doc) = self.editor().document.as_mut()
        {
            doc.mark_saved(revision, version);
        }
        self.send(EventKind::Saved);
        Ok(())
    }
}

impl Ui {
    pub fn start_pump(&self) {
        let mut receiver = self.inner.events.subscribe();
        std::thread::spawn(move || {
            loop {
                match receiver.blocking_recv() {
                    Ok(kind) => opsis_qt::post_event(kind),
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        opsis_qt::post_event(EventKind::Document);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    pub fn log(&self, line: &str) {
        let elapsed = self.inner.started.elapsed().as_secs_f64();
        eprintln!("[opsis-qt {elapsed:8.3}s] {line}");
    }

    pub fn examples(&self) -> Vec<ProjectEntry> {
        let Ok(entries) = std::fs::read_dir(repo_root().join("examples")) else {
            return Vec::new();
        };
        let mut list: Vec<ProjectEntry> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|dir| dir.join(WORKFLOW_FILE).exists())
            .filter_map(|dir| {
                let opened = self.inner.projects.open(&dir).ok()?;
                Some(ProjectEntry {
                    name: opened.workflow.name,
                    detail: format!("{} nodes", opened.workflow.nodes.len()),
                    dir: dir.to_string_lossy().into_owned(),
                })
            })
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn recent(&self) -> Vec<ProjectEntry> {
        self.inner
            .runtime
            .block_on(self.inner.db.list_recent())
            .unwrap_or_default()
            .into_iter()
            .filter(|r| Path::new(&r.dir).join(WORKFLOW_FILE).exists())
            .map(|r| ProjectEntry {
                name: r.name,
                detail: r.dir.clone(),
                dir: r.dir,
            })
            .collect()
    }

    pub fn open_project(&self, dir: &str) -> UiResult<()> {
        self.inner.write()?;
        let dir = PathBuf::from(dir);
        let project = self.inner.projects.open(&dir)?;
        let now = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        )
        .unwrap_or_default();
        self.inner.runtime.block_on(self.inner.db.touch_recent(
            &dir.to_string_lossy(),
            &project.workflow.name,
            now,
        ))?;
        {
            let mut editor = self.inner.editor();
            editor.selected = None;
            editor.document = Some(Document::open(
                &dir,
                project.workflow,
                project.version,
                Platform::current(),
            ));
        }
        self.inner.send(EventKind::Project);
        Ok(())
    }

    pub fn open_example(&self, dir: &str) -> UiResult<()> {
        let source = PathBuf::from(dir);
        let name = source.file_name().map_or_else(
            || "example".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let copy = self.inner.data.join("projects").join(&name);
        if !copy.join(WORKFLOW_FILE).exists() {
            copy_dir(&source, &copy)?;
        }
        self.open_project(&copy.to_string_lossy())
    }

    pub fn create_project(&self, name: &str) -> UiResult<()> {
        let created =
            self.inner
                .projects
                .create(&self.inner.data.join("projects"), name, "blank")?;
        self.open_project(&created.dir.to_string_lossy())
    }

    pub fn generate_project(&self, count: u32) -> UiResult<()> {
        let name = format!("Bench {count}");
        let dir = self
            .inner
            .data
            .join("projects")
            .join(format!("bench-{count}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        let created =
            self.inner
                .projects
                .create(&self.inner.data.join("projects"), &name, "blank")?;
        self.open_project(&created.dir.to_string_lossy())?;
        let mut previous: Option<String> = None;
        for index in 0..count {
            let column = index % BENCH_COLUMNS;
            let kind = if column == 0 {
                BENCH_KINDS[0]
            } else {
                BENCH_KINDS[1 + usize::try_from(column).unwrap_or(0) % (BENCH_KINDS.len() - 1)]
            };
            let x = f64::from(index % BENCH_COLUMNS) * 340.0;
            let y = f64::from(index / BENCH_COLUMNS) * 260.0;
            let id = self.add_node_quiet(kind, x, y)?;
            if let Some(source) = previous.replace(id.clone())
                && column != 0
                && let Some(doc) = self.inner.editor().document.as_mut()
            {
                let _ = doc.apply(WorkflowEdit::AddEdge {
                    ends: EdgeEnds {
                        source,
                        source_handle: "session".to_owned(),
                        target: id,
                        target_handle: "session".to_owned(),
                    },
                });
            }
        }
        self.inner.write()?;
        self.inner.send(EventKind::Project);
        Ok(())
    }

    pub fn close_project(&self) {
        if let Err(error) = self.inner.write() {
            self.log(&format!("saving before closing: {error}"));
        }
        self.inner.editor().document = None;
        self.inner.send(EventKind::Project);
    }

    pub fn has_project(&self) -> bool {
        self.inner.editor().document.is_some()
    }

    pub fn graph(&self) -> GraphView {
        let editor = self.inner.editor();
        let Some(doc) = editor.document.as_ref() else {
            return GraphView {
                name: String::new(),
                dir: String::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                revision: 0,
                can_undo: false,
                can_redo: false,
                dirty: false,
            };
        };
        let wf = doc.workflow();
        let nodes = graph::cards(wf, &self.inner.catalog);
        let edges = graph::edges(wf, &nodes);
        let state = doc.state();
        GraphView {
            name: wf.name.clone(),
            dir: doc.dir().to_string_lossy().into_owned(),
            nodes,
            edges,
            revision: doc.revision(),
            can_undo: state.can_undo,
            can_redo: state.can_redo,
            dirty: doc.is_dirty(),
        }
    }

    pub fn palette(&self) -> Vec<PaletteEntry> {
        let mut entries: Vec<(usize, PaletteEntry)> = self
            .inner
            .catalog
            .values()
            .filter(|n| !n.hidden && n.has_card)
            .map(|n| {
                let category = serde_json::to_value(n.category)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                (
                    look::category_order(&category),
                    PaletteEntry {
                        node_type: n.kind.clone(),
                        title: n.title.clone(),
                        category: n.category_label.clone(),
                        color: look::category_color(&category).to_owned(),
                        glyph: n.look.as_ref().map_or_default(|l| l.glyph.clone()),
                        description: n.description.clone().unwrap_or_default(),
                    },
                )
            })
            .collect();
        entries.sort_by_key(|(order, _)| *order);
        entries.into_iter().map(|(_, e)| e).collect()
    }

    fn edit(&self, edit: WorkflowEdit) -> UiResult<Vec<String>> {
        let edited = {
            let mut editor = self.inner.editor();
            let doc = editor.document.as_mut().ok_or(UiError::NoProject)?;
            doc.apply(edit)?
        };
        self.changed();
        Ok(edited.ids)
    }

    fn changed(&self) {
        let turn = self.inner.autosave.fetch_add(1, Ordering::Relaxed) + 1;
        let inner = Arc::clone(&self.inner);
        self.inner.runtime.spawn(async move {
            tokio::time::sleep(AUTOSAVE_DELAY).await;
            if inner.autosave.load(Ordering::Relaxed) != turn {
                return;
            }
            let saving = Arc::clone(&inner);
            let written = tokio::task::spawn_blocking(move || saving.write()).await;
            if let Ok(Err(error)) = written {
                eprintln!("saving the project: {error}");
            }
        });
        self.inner.send(EventKind::Document);
    }

    pub fn begin_gesture(&self) {
        if let Some(doc) = self.inner.editor().document.as_mut() {
            doc.begin_gesture();
        }
    }

    pub fn end_gesture(&self) {
        if let Some(doc) = self.inner.editor().document.as_mut() {
            doc.end_gesture();
        }
        self.changed();
    }

    pub fn move_nodes(&self, moves: Vec<NodeMove>) -> UiResult<()> {
        let positions = {
            let editor = self.inner.editor();
            let wf = editor
                .document
                .as_ref()
                .ok_or(UiError::NoProject)?
                .workflow();
            moves
                .into_iter()
                .filter_map(|m| {
                    let node = wf.node(&m.id)?;
                    let parent = node
                        .parent_id
                        .as_deref()
                        .and_then(|p| wf.node(p))
                        .map(|p| absolute_position(wf, p));
                    let (dx, dy) = parent.map_or((0.0, 0.0), |p| (p.x, p.y));
                    Some((
                        m.id,
                        Position {
                            x: m.x - dx,
                            y: m.y - dy,
                        },
                    ))
                })
                .collect()
        };
        self.edit(WorkflowEdit::MoveNodes { positions })?;
        Ok(())
    }

    pub fn connection_problem(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> String {
        let editor = self.inner.editor();
        let Some(doc) = editor.document.as_ref() else {
            return String::new();
        };
        let wf = doc.workflow();
        let compiled = compile_workflow(wf, None);
        connection_problem_now(wf, &compiled, source, source_handle, target, target_handle)
            .map_or_default(|p| p.message)
    }

    pub fn connect(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> UiResult<()> {
        let problem = self.connection_problem(source, source_handle, target, target_handle);
        if !problem.is_empty() {
            return Err(UiError::Refused(problem));
        }
        self.edit(WorkflowEdit::AddEdge {
            ends: EdgeEnds {
                source: source.to_owned(),
                source_handle: source_handle.to_owned(),
                target: target.to_owned(),
                target_handle: target_handle.to_owned(),
            },
        })?;
        Ok(())
    }

    pub fn remove(&self, nodes: Vec<String>, edges: Vec<String>) -> UiResult<()> {
        if nodes.is_empty() && edges.is_empty() {
            return Ok(());
        }
        {
            let mut editor = self.inner.editor();
            if editor.selected.as_ref().is_some_and(|s| nodes.contains(s)) {
                editor.selected = None;
            }
        }
        self.edit(WorkflowEdit::RemoveParts {
            node_ids: nodes,
            edge_ids: edges,
        })?;
        self.inner.send(EventKind::Selection);
        Ok(())
    }

    fn add_node_quiet(&self, node_type: &str, x: f64, y: f64) -> UiResult<String> {
        let mut editor = self.inner.editor();
        let doc = editor.document.as_mut().ok_or(UiError::NoProject)?;
        let edited = doc.apply(WorkflowEdit::AddNode {
            kind: node_type.to_owned(),
            id: None,
            label: None,
            position: Some(Position { x, y }),
            params: None,
        })?;
        Ok(edited.ids.into_iter().next().unwrap_or_default())
    }

    pub fn add_node(&self, node_type: &str, x: f64, y: f64) -> UiResult<String> {
        let id = self.add_node_quiet(node_type, x, y)?;
        self.changed();
        self.select(&id);
        Ok(id)
    }

    pub fn undo(&self) -> bool {
        self.step(false)
    }

    pub fn redo(&self) -> bool {
        self.step(true)
    }

    fn step(&self, redo: bool) -> bool {
        let stepped = {
            let mut editor = self.inner.editor();
            let Some(doc) = editor.document.as_mut() else {
                return false;
            };
            if redo { doc.redo() } else { doc.undo() }
        };
        if stepped {
            self.changed();
            self.inner.send(EventKind::Selection);
        }
        stepped
    }

    pub fn select(&self, id: &str) {
        let next = Some(id.to_owned()).filter(|s| !s.is_empty());
        let mut editor = self.inner.editor();
        if editor.selected != next {
            editor.selected = next;
            drop(editor);
            self.inner.send(EventKind::Selection);
        }
    }

    pub fn selected(&self) -> String {
        self.inner.editor().selected.clone().unwrap_or_default()
    }

    pub fn inspector(&self) -> InspectorView {
        let editor = self.inner.editor();
        let empty = InspectorView {
            id: String::new(),
            title: String::new(),
            label: String::new(),
            category: String::new(),
            color: String::new(),
            glyph: String::new(),
            description: String::new(),
            fields: Vec::new(),
        };
        let (Some(doc), Some(id)) = (editor.document.as_ref(), editor.selected.as_ref()) else {
            return empty;
        };
        let Some(node) = doc.workflow().node(id) else {
            return empty;
        };
        let Some(def) = self.inner.catalog.get(&node.kind) else {
            return empty;
        };
        let category = serde_json::to_value(def.category)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        InspectorView {
            id: node.id.clone(),
            title: def.title.clone(),
            label: node.label.clone().unwrap_or_else(|| def.title.clone()),
            category: def.category_label.clone(),
            color: look::category_color(&category).to_owned(),
            glyph: def.look.as_ref().map_or_default(|l| l.glyph.clone()),
            description: def.description.clone().unwrap_or_default(),
            fields: inspector::fields(def, node),
        }
    }

    pub fn set_param(&self, id: &str, key: &str, json: &str) -> UiResult<()> {
        let mut params = Map::new();
        params.insert(key.to_owned(), inspector::parsed_value(json));
        self.edit(WorkflowEdit::UpdateNode {
            id: id.to_owned(),
            patch: NodePatch {
                params: Some(params),
                ..NodePatch::default()
            },
        })?;
        Ok(())
    }

    pub fn rename_node(&self, id: &str, label: &str) -> UiResult<()> {
        self.edit(WorkflowEdit::UpdateNode {
            id: id.to_owned(),
            patch: NodePatch {
                label: Some(label.to_owned()),
                ..NodePatch::default()
            },
        })?;
        Ok(())
    }

    pub fn save(&self) -> UiResult<()> {
        self.inner.write()
    }

    pub fn settings(&self) -> SettingsView {
        let settings = self.inner.settings.lock().expect("settings").clone();
        let book = settings_book();
        SettingsView {
            on_start: plain(&settings.app.on_start),
            start_options: choices(&book.start_options),
            confirm_exit: settings.app.confirm_exit,
            encode_speed: plain(&settings.media.encode_speed),
            encode_speeds: choices(&book.encode_speeds),
            default_dir: settings.projects.default_dir.clone(),
            canvas_mode: plain(&settings.editor.canvas_mode),
            canvas_modes: book
                .canvas_modes
                .iter()
                .map(|m| Choice {
                    value: plain(m),
                    label: capitalized(&plain(m)),
                })
                .collect(),
            api_enabled: settings.api.enabled,
            data_dir: self.inner.data.to_string_lossy().into_owned(),
        }
    }

    pub fn patch_settings(&self, json: &str) -> UiResult<()> {
        let patch: Map<String, Value> = serde_json::from_str(json)?;
        let next = {
            let current = self.inner.settings.lock().expect("settings").clone();
            merged_settings(&current, &patch)?
        };
        self.inner
            .runtime
            .block_on(self.inner.db.write_settings(&next))?;
        *self.inner.settings.lock().expect("settings") = next;
        self.inner.send(EventKind::Settings);
        Ok(())
    }

    pub fn load_layout(&self) -> String {
        std::fs::read_to_string(self.inner.data.join(LAYOUT_FILE)).unwrap_or_default()
    }

    pub fn save_layout(&self, layout: &str) {
        if let Err(error) = std::fs::write(self.inner.data.join(LAYOUT_FILE), layout) {
            self.log(&format!("saving the layout: {error}"));
        }
    }
}

fn plain(value: &impl serde::Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(text)) => text,
        Ok(other) => other.to_string(),
        Err(_) => String::new(),
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn choices<T: serde::Serialize>(list: &[(T, String)]) -> Vec<Choice> {
    list.iter()
        .map(|(value, label)| Choice {
            value: plain(value),
            label: label.clone(),
        })
        .collect()
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn hosted<T>(result: UiResult<T>) -> HostResult<T> {
    result.map_err(|error| error.to_string())
}

impl Host for Ui {
    fn start_pump(&self) {
        Self::start_pump(self);
    }

    fn style_sheet(&self) -> String {
        look::style_sheet()
    }

    fn token(&self, name: &str) -> String {
        look::token(name)
    }

    fn icon_svg(&self, name: &str, color: &str, stroke: f64) -> String {
        look::icon_svg(name, color, stroke)
    }

    fn fonts_dir(&self) -> String {
        fonts_dir()
    }

    fn log(&self, line: &str) {
        Self::log(self, line);
    }

    fn examples(&self) -> Vec<ProjectEntry> {
        Self::examples(self)
    }

    fn recent(&self) -> Vec<ProjectEntry> {
        Self::recent(self)
    }

    fn open_project(&self, dir: &str) -> HostResult<()> {
        hosted(Self::open_project(self, dir))
    }

    fn open_example(&self, dir: &str) -> HostResult<()> {
        hosted(Self::open_example(self, dir))
    }

    fn create_project(&self, name: &str) -> HostResult<()> {
        hosted(Self::create_project(self, name))
    }

    fn generate_project(&self, count: u32) -> HostResult<()> {
        hosted(Self::generate_project(self, count))
    }

    fn close_project(&self) {
        Self::close_project(self);
    }

    fn has_project(&self) -> bool {
        Self::has_project(self)
    }

    fn graph(&self) -> GraphView {
        Self::graph(self)
    }

    fn palette(&self) -> Vec<PaletteEntry> {
        Self::palette(self)
    }

    fn begin_gesture(&self) {
        Self::begin_gesture(self);
    }

    fn end_gesture(&self) {
        Self::end_gesture(self);
    }

    fn move_nodes(&self, moves: Vec<NodeMove>) -> HostResult<()> {
        hosted(Self::move_nodes(self, moves))
    }

    fn connection_problem(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> String {
        Self::connection_problem(self, source, source_handle, target, target_handle)
    }

    fn connect(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> HostResult<()> {
        hosted(Self::connect(
            self,
            source,
            source_handle,
            target,
            target_handle,
        ))
    }

    fn remove(&self, nodes: Vec<String>, edges: Vec<String>) -> HostResult<()> {
        hosted(Self::remove(self, nodes, edges))
    }

    fn add_node(&self, node_type: &str, x: f64, y: f64) -> HostResult<String> {
        hosted(Self::add_node(self, node_type, x, y))
    }

    fn undo(&self) -> bool {
        Self::undo(self)
    }

    fn redo(&self) -> bool {
        Self::redo(self)
    }

    fn select(&self, id: &str) {
        Self::select(self, id);
    }

    fn selected(&self) -> String {
        Self::selected(self)
    }

    fn inspector(&self) -> InspectorView {
        Self::inspector(self)
    }

    fn set_param(&self, id: &str, key: &str, json: &str) -> HostResult<()> {
        hosted(Self::set_param(self, id, key, json))
    }

    fn rename_node(&self, id: &str, label: &str) -> HostResult<()> {
        hosted(Self::rename_node(self, id, label))
    }

    fn save(&self) -> HostResult<()> {
        hosted(Self::save(self))
    }

    fn settings(&self) -> SettingsView {
        Self::settings(self)
    }

    fn patch_settings(&self, json: &str) -> HostResult<()> {
        hosted(Self::patch_settings(self, json))
    }

    fn load_layout(&self) -> String {
        Self::load_layout(self)
    }

    fn save_layout(&self, layout: &str) {
        Self::save_layout(self, layout);
    }
}
