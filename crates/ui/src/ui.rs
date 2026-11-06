use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use indexmap::IndexMap;
use opsis_compile::catalog::{EditorNode, editor_catalog};
use opsis_compile::compile::compile_workflow;
use opsis_compile::graph::connection_problem_now;
use opsis_compile::node::Platform as NodePlatform;
use opsis_qt::{
    Choice, CloseView, EventKind, GraphView, Host, HostResult, InspectorView, MenuSectionView,
    NodeMove, PaletteEntry, ProjectEntry, ScreenArea, SettingsView, TemplateView, WindowPlacement,
};
use opsis_session::error::CommandError;
use opsis_session::events::{MAIN_WINDOW, SessionEvent};
use opsis_session::exit::{self, Exit};
use opsis_session::state::AppState;
use opsis_session::windows::{self, SavedWindow, WorkArea};
use opsis_session::{Session, compile, document, projects, run, settings, shared};
use opsis_types::actions::Viewport;
use opsis_types::edits::{EdgeEnds, NodePatch, WorkflowEdit};
use opsis_types::events::RunTargets;
use opsis_types::settings::settings_book;
use opsis_types::shell::{CloseDecision, CloseRequest, MenuCommand, Selection};
use opsis_types::workflow::{Position, WORKFLOW_FILE, Workflow};
use serde_json::{Map, Value};
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::RecvError;

use crate::platform::QtPlatform;
use crate::{graph, inspector, look};

const LAYOUT_KEY: &str = "nativeDockLayout";
const BENCH_KINDS: [&str; 4] = [
    "browser.open",
    "browser.click",
    "browser.type",
    "browser.wait",
];
const BENCH_COLUMNS: u32 = 10;
const BLANK_TEMPLATE: &str = "blank";
const RECENT_LIMIT: usize = 10;

pub struct Ui {
    session: Session,
    runtime: Runtime,
    platform: Arc<QtPlatform>,
    catalog: IndexMap<String, EditorNode>,
    showing: AtomicBool,
    closing: Arc<Mutex<Option<CloseRequest>>>,
    started: Instant,
}

impl fmt::Debug for Ui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ui")
            .field("session", &self.session)
            .finish_non_exhaustive()
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

fn failed(error: impl fmt::Display) -> String {
    error.to_string()
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

const fn event_kind(event: &SessionEvent) -> Option<EventKind> {
    match event {
        SessionEvent::ConfirmClose(_) => Some(EventKind::CloseAsked),
        SessionEvent::FlushSave(_) => Some(EventKind::FlushDrafts),
        SessionEvent::DocumentChanged(_) | SessionEvent::ProjectReloaded(_) => {
            Some(EventKind::Document)
        }
        SessionEvent::SelectionShared(_) => Some(EventKind::Selection),
        SessionEvent::SettingsChanged(_) => Some(EventKind::Settings),
        _ => None,
    }
}

impl Ui {
    pub fn new(session: Session, runtime: Runtime, platform: Arc<QtPlatform>) -> Self {
        let catalog = editor_catalog(NodePlatform::current())
            .into_iter()
            .map(|node| (node.kind.clone(), node))
            .collect();
        Self {
            session,
            runtime,
            platform,
            catalog,
            showing: AtomicBool::new(false),
            closing: Arc::new(Mutex::new(None)),
            started: Instant::now(),
        }
    }

    pub const fn session(&self) -> &Session {
        &self.session
    }

    fn state(&self) -> &AppState {
        self.session.state::<AppState>()
    }

    fn workflow(&self) -> Option<Workflow> {
        self.state()
            .editor()
            .document
            .as_ref()
            .map(|doc| doc.workflow().clone())
    }

    fn edit(&self, edit: WorkflowEdit) -> Result<Vec<String>, CommandError> {
        Ok(document::edit_workflow(&self.session, edit)?.ids)
    }

    fn show_project(&self) {
        self.showing.store(true, Ordering::Relaxed);
        opsis_qt::post_event(EventKind::Project);
    }

    fn open_dir(&self, dir: &Path) -> Result<(), CommandError> {
        self.runtime.block_on(projects::open_project(
            self.session.clone(),
            dir.to_string_lossy().into_owned(),
        ))?;
        self.show_project();
        Ok(())
    }

    fn create_in(&self, name: &str, parent: &Path) -> Result<PathBuf, CommandError> {
        self.create_from(name, BLANK_TEMPLATE, parent)
    }

    fn create_from(
        &self,
        name: &str,
        template: &str,
        parent: &Path,
    ) -> Result<PathBuf, CommandError> {
        let created = self.runtime.block_on(projects::create_project(
            self.session.clone(),
            name.to_owned(),
            template.to_owned(),
            Some(parent.to_string_lossy().into_owned()),
        ))?;
        let dir = created
            .map(|project| PathBuf::from(project.dir))
            .ok_or_else(|| CommandError("The project wasn't created.".to_owned()))?;
        self.show_project();
        Ok(dir)
    }

    fn generated(&self, count: u32) -> Result<(), CommandError> {
        let parent = self.state().paths.user_data.join("bench");
        let dir = parent.join(format!("bench-{count}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        std::fs::create_dir_all(&parent)?;
        self.create_in(&format!("Bench {count}"), &parent)?;
        {
            let mut editor = self.state().editor();
            let doc = editor
                .document
                .as_mut()
                .ok_or_else(|| CommandError("No project is open.".to_owned()))?;
            let mut previous: Option<String> = None;
            for index in 0..count {
                let column = index % BENCH_COLUMNS;
                let kind = if column == 0 {
                    BENCH_KINDS[0]
                } else {
                    BENCH_KINDS[1 + usize::try_from(column).unwrap_or(0) % (BENCH_KINDS.len() - 1)]
                };
                let edited = doc.apply(WorkflowEdit::AddNode {
                    kind: kind.to_owned(),
                    id: None,
                    label: None,
                    position: Some(Position {
                        x: f64::from(column) * 340.0,
                        y: f64::from(index / BENCH_COLUMNS) * 260.0,
                    }),
                    params: None,
                })?;
                let id = edited.ids.into_iter().next().unwrap_or_default();
                if let Some(source) = previous.replace(id.clone())
                    && column != 0
                {
                    doc.apply(WorkflowEdit::AddEdge {
                        ends: EdgeEnds {
                            source,
                            source_handle: "session".to_owned(),
                            target: id,
                            target_handle: "session".to_owned(),
                        },
                    })?;
                }
            }
        }
        document::changed(&self.session);
        Ok(())
    }

    fn group_selection(&self) -> HostResult<()> {
        let ids = shared::shared_state(&self.session).selection.selected_ids;
        if ids.len() < 2 {
            return Err("Select two or more nodes to group them.".to_owned());
        }
        let sizes = self
            .graph()
            .nodes
            .iter()
            .filter(|card| ids.contains(&card.id))
            .map(|card| {
                (
                    card.id.clone(),
                    Viewport {
                        width: opsis_types::numbers::whole(card.width),
                        height: opsis_types::numbers::whole(card.height),
                    },
                )
            })
            .collect();
        self.edit(WorkflowEdit::Group {
            ids,
            title: "Group".to_owned(),
            sizes,
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn view_key(&self) -> Option<String> {
        self.state()
            .editor()
            .dir()
            .map(|dir| opsis_engine::tools::shown(&dir))
    }
}

impl Host for Ui {
    fn start_pump(&self) {
        let mut events = self.session.subscribe();
        let closing = Arc::clone(&self.closing);
        std::thread::spawn(move || {
            loop {
                match events.blocking_recv() {
                    Ok(addressed) => {
                        if let SessionEvent::ConfirmClose(asked) = &addressed.event {
                            *closing.lock().expect("close question") = Some(asked.0.clone());
                        }
                        let mine = addressed.to.as_deref().is_none_or(|to| to == MAIN_WINDOW);
                        if let Some(kind) = event_kind(&addressed.event).filter(|_| mine) {
                            opsis_qt::post_event(kind);
                        }
                    }
                    Err(RecvError::Lagged(_)) => opsis_qt::post_event(EventKind::Document),
                    Err(RecvError::Closed) => break,
                }
            }
        });
        let _entered = self.runtime.enter();
        self.session.begin();
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
        self.state()
            .paths
            .public_dir
            .join("fonts")
            .join("bundled")
            .to_string_lossy()
            .into_owned()
    }

    fn log(&self, line: &str) {
        let elapsed = self.started.elapsed().as_secs_f64();
        log::info!(target: "views", "{elapsed:8.3}s {line}");
    }

    fn answer_dialog(&self, id: u64, paths: Vec<String>) {
        self.platform.answer(id, paths);
    }

    fn set_active(&self, active: bool) {
        self.platform.set_active(active);
    }

    fn examples(&self) -> Vec<ProjectEntry> {
        let Ok(entries) = std::fs::read_dir(self.state().paths.resources.join("examples")) else {
            return Vec::new();
        };
        let mut list: Vec<ProjectEntry> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|dir| dir.join(WORKFLOW_FILE).exists())
            .filter_map(|dir| {
                let opened = self.state().projects.open(&dir).ok()?;
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

    fn recent(&self) -> Vec<ProjectEntry> {
        self.runtime
            .block_on(projects::list_recent_projects(&self.session))
            .unwrap_or_default()
            .into_iter()
            .filter(|r| Path::new(&r.dir).join(WORKFLOW_FILE).exists())
            .take(RECENT_LIMIT)
            .map(|r| ProjectEntry {
                name: r.name,
                detail: crate::text::time_ago(r.opened_at, opsis_runner::context::now_ms()),
                dir: r.dir,
            })
            .collect()
    }

    fn open_project(&self, dir: &str) -> HostResult<()> {
        self.open_dir(Path::new(dir)).map_err(|e| e.0)
    }

    fn open_example(&self, dir: &str) -> HostResult<()> {
        let source = PathBuf::from(dir);
        let name = source.file_name().map_or_else(
            || "example".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let copy = self.state().paths.user_data.join("examples").join(&name);
        if !copy.join(WORKFLOW_FILE).exists() {
            copy_dir(&source, &copy).map_err(failed)?;
        }
        self.open_dir(&copy).map_err(|e| e.0)
    }

    fn create_project(&self, name: &str, template_id: &str, location: &str) -> HostResult<()> {
        let parent = opsis_project::files::expand_home(location, &self.state().paths.home);
        self.create_from(name, template_id, &parent)
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn templates(&self) -> Vec<TemplateView> {
        projects::list_templates()
            .into_iter()
            .map(|template| TemplateView {
                id: template.id,
                title: template.title,
                description: template.description,
                icon: plain(&template.icon),
            })
            .collect()
    }

    fn engine_missing(&self) -> bool {
        let engine = opsis_session::engine::get_engine_status(&self.session);
        engine.browser.path.is_none() || engine.browser.problem.is_some()
    }

    fn project_location(&self) -> String {
        projects::project_locations(&self.session).suggested_dir
    }

    fn project_folder(&self, parent: &str, name: &str) -> String {
        projects::project_folder(&self.session, parent, name)
    }

    fn generate_project(&self, count: u32) -> HostResult<()> {
        self.generated(count).map_err(|e| e.0)
    }

    fn close_project(&self) {
        if let Err(error) = self
            .runtime
            .block_on(document::save_project(self.session.clone()))
        {
            log::warn!("saving before closing the project: {}", error.0);
        }
        self.showing.store(false, Ordering::Relaxed);
        opsis_qt::post_event(EventKind::Project);
    }

    fn has_project(&self) -> bool {
        self.showing.load(Ordering::Relaxed)
    }

    fn graph(&self) -> GraphView {
        let state = document::document_state(&self.session);
        let Some(state) = state.filter(|_| self.has_project()) else {
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
        let wf = &state.workflow;
        let nodes = graph::cards(wf, &self.catalog);
        let edges = graph::edges(wf, &nodes);
        GraphView {
            name: wf.name.clone(),
            dir: state.dir.clone(),
            nodes,
            edges,
            revision: state.revision,
            can_undo: state.can_undo,
            can_redo: state.can_redo,
            dirty: state.revision != state.saved_revision,
        }
    }

    fn palette(&self) -> Vec<PaletteEntry> {
        let mut entries: Vec<(usize, PaletteEntry)> = self
            .catalog
            .values()
            .filter(|n| !n.hidden && n.has_card)
            .map(|n| {
                let category = plain(&n.category);
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

    fn begin_gesture(&self) {
        document::begin_gesture(&self.session);
    }

    fn end_gesture(&self) {
        document::end_gesture(&self.session);
    }

    fn move_nodes(&self, moves: Vec<NodeMove>) -> HostResult<()> {
        let Some(wf) = self.workflow() else {
            return Err("No project is open.".to_owned());
        };
        let positions = moves
            .into_iter()
            .filter_map(|m| {
                let node = wf.node(&m.id)?;
                let parent = node
                    .parent_id
                    .as_deref()
                    .and_then(|p| wf.node(p))
                    .map(|p| opsis_compile::edits::workflow::absolute_position(&wf, p));
                let (dx, dy) = parent.map_or((0.0, 0.0), |p| (p.x, p.y));
                Some((
                    m.id,
                    Position {
                        x: m.x - dx,
                        y: m.y - dy,
                    },
                ))
            })
            .collect();
        self.edit(WorkflowEdit::MoveNodes { positions })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn connection_problem(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> String {
        let Some(wf) = self.workflow() else {
            return String::new();
        };
        let results = self.state().editor().results.clone();
        let compiled = compile_workflow(&wf, Some(&results));
        connection_problem_now(&wf, &compiled, source, source_handle, target, target_handle)
            .map_or_default(|p| p.message)
    }

    fn connect(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> HostResult<()> {
        self.edit(WorkflowEdit::AddEdge {
            ends: EdgeEnds {
                source: source.to_owned(),
                source_handle: source_handle.to_owned(),
                target: target.to_owned(),
                target_handle: target_handle.to_owned(),
            },
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn remove(&self, nodes: Vec<String>, edges: Vec<String>) -> HostResult<()> {
        if nodes.is_empty() && edges.is_empty() {
            return Ok(());
        }
        if nodes.contains(&self.selected()) {
            self.select(Vec::new());
        }
        self.edit(WorkflowEdit::RemoveParts {
            node_ids: nodes,
            edge_ids: edges,
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn add_node(&self, node_type: &str, x: f64, y: f64) -> HostResult<String> {
        let id = self
            .edit(WorkflowEdit::AddNode {
                kind: node_type.to_owned(),
                id: None,
                label: None,
                position: Some(Position { x, y }),
                params: None,
            })
            .map_err(|e| e.0)?
            .into_iter()
            .next()
            .unwrap_or_default();
        self.select(vec![id.clone()]);
        Ok(id)
    }

    fn undo(&self) -> bool {
        document::undo(&self.session).is_ok()
    }

    fn redo(&self) -> bool {
        document::redo(&self.session).is_ok()
    }

    fn select(&self, ids: Vec<String>) {
        if shared::shared_state(&self.session).selection.selected_ids == ids {
            return;
        }
        let selection = Selection {
            selected_ids: ids.clone(),
            active_ids: ids.clone(),
            pinned_source: None,
        };
        if let Err(error) = shared::share_selection(&self.session, MAIN_WINDOW, selection) {
            log::warn!("sharing the selection: {}", error.0);
        }
        let focus = ids.into_iter().next();
        compile::set_compile_focus(&self.session, focus.clone(), focus);
    }

    fn selected(&self) -> String {
        shared::shared_state(&self.session)
            .selection
            .selected_ids
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    fn inspector(&self) -> InspectorView {
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
        let Some(wf) = self.workflow() else {
            return empty;
        };
        let Some(node) = wf.node(&self.selected()) else {
            return empty;
        };
        let Some(def) = self.catalog.get(&node.kind) else {
            return empty;
        };
        let category = plain(&def.category);
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

    fn set_param(&self, id: &str, key: &str, json: &str) -> HostResult<()> {
        let mut params = Map::new();
        params.insert(key.to_owned(), inspector::parsed_value(json));
        self.edit(WorkflowEdit::UpdateNode {
            id: id.to_owned(),
            patch: NodePatch {
                params: Some(params),
                ..NodePatch::default()
            },
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn rename_node(&self, id: &str, label: &str) -> HostResult<()> {
        self.edit(WorkflowEdit::UpdateNode {
            id: id.to_owned(),
            patch: NodePatch {
                label: Some(label.to_owned()),
                ..NodePatch::default()
            },
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn save(&self) -> HostResult<()> {
        self.runtime
            .block_on(document::save_project(self.session.clone()))
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn settings(&self) -> SettingsView {
        let settings = settings::get_settings(&self.session);
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
            data_dir: self.state().paths.user_data.to_string_lossy().into_owned(),
        }
    }

    fn patch_settings(&self, json: &str) -> HostResult<()> {
        let patch = serde_json::from_str(json).map_err(failed)?;
        self.runtime
            .block_on(settings::patch_settings(self.session.clone(), patch))
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn menu(&self) -> Vec<MenuSectionView> {
        crate::menu::menu(cfg!(target_os = "macos"))
    }

    fn menu_command(&self, id: &str) -> HostResult<()> {
        let command: MenuCommand =
            serde_json::from_value(Value::String(id.to_owned())).map_err(failed)?;
        match command {
            MenuCommand::ProjectSave => self.save(),
            MenuCommand::EditUndo => {
                self.undo();
                Ok(())
            }
            MenuCommand::EditRedo => {
                self.redo();
                Ok(())
            }
            MenuCommand::RunStart => self
                .runtime
                .block_on(run::run_workflow(
                    self.session.clone(),
                    RunTargets {
                        target_node_ids: None,
                        from_node_id: None,
                    },
                ))
                .map(|_| ())
                .map_err(|e| e.0),
            MenuCommand::RunCancel => {
                run::cancel_run(&self.session);
                Ok(())
            }
            MenuCommand::EditGroup => self.group_selection(),
            MenuCommand::EditLayout => {
                Err("Tidying the layout isn't in the native editor yet.".to_owned())
            }
            _ => Err(format!("The views handle {id}.")),
        }
    }

    fn close_allowed(&self) -> bool {
        self.session.state::<Exit>().allowed()
    }

    fn request_close(&self) {
        if !self.session.state::<Exit>().start_asking() {
            return;
        }
        let session = self.session.clone();
        self.runtime.spawn(async move {
            let closing = exit::confirm(&session).await;
            session.state::<Exit>().finish_asking(closing);
            if closing {
                opsis_qt::quit(0);
            }
        });
    }

    fn close_view(&self) -> CloseView {
        let asked = self.closing.lock().expect("close question").clone();
        asked.map_or_else(
            || CloseView {
                ask: false,
                busy: Vec::new(),
            },
            |asked| CloseView {
                ask: asked.ask,
                busy: asked.busy,
            },
        )
    }

    fn close_shown(&self) {
        exit::close_shown(&self.session);
    }

    fn decide_close(&self, close: bool, dont_ask: bool) {
        exit::decide_close(&self.session, CloseDecision { close, dont_ask });
    }

    fn report_drafts(&self, held: bool) {
        exit::report_drafts(&self.session, held);
    }

    fn save_flushed(&self) {
        exit::save_flushed(&self.session);
    }

    fn project_window(&self, areas: Vec<ScreenArea>) -> WindowPlacement {
        let unknown = WindowPlacement {
            known: false,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            maximized: true,
        };
        let Some(dir) = self.view_key() else {
            return unknown;
        };
        let Ok(view) = self
            .runtime
            .block_on(settings::view_state(&self.session, dir))
        else {
            return unknown;
        };
        let Some(saved) = windows::saved_window(&view) else {
            return unknown;
        };
        let areas: Vec<WorkArea> = areas
            .into_iter()
            .map(|a| WorkArea {
                x: a.x,
                y: a.y,
                width: a.width,
                height: a.height,
            })
            .collect();
        windows::visible_bounds(saved.bounds(), &areas).map_or(
            WindowPlacement {
                maximized: saved.maximized,
                ..unknown
            },
            |bounds| WindowPlacement {
                known: true,
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                maximized: saved.maximized,
            },
        )
    }

    fn save_window(&self, placement: WindowPlacement) {
        let session = self.session.clone();
        self.runtime.spawn(async move {
            let saved = SavedWindow {
                x: placement.x,
                y: placement.y,
                width: placement.width,
                height: placement.height,
                maximized: placement.maximized,
            };
            windows::save_main_window(&session, saved).await;
        });
    }

    fn load_layout(&self) -> String {
        let Some(dir) = self.view_key() else {
            return String::new();
        };
        self.runtime
            .block_on(settings::view_state(&self.session, dir))
            .ok()
            .and_then(|view| {
                view.get(LAYOUT_KEY)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_default()
    }

    fn save_layout(&self, layout: &str) {
        let Some(dir) = self.view_key() else {
            return;
        };
        let saved = self.runtime.block_on(settings::set_view_pref(
            self.session.clone(),
            dir,
            LAYOUT_KEY.to_owned(),
            Value::String(layout.to_owned()),
        ));
        if let Err(error) = saved {
            log::warn!("saving the panel layout: {}", error.0);
        }
    }
}
