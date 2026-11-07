use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use indexmap::IndexMap;
use opsis_compile::canvas::{snap_to_grid, staggered};
use opsis_compile::catalog::{EditorCatalog, EditorNode, editor_bundle, editor_catalog};
use opsis_compile::compile::compile_workflow;
use opsis_compile::graph::{IssueLevel, connection_problem_now};
use opsis_compile::inspector::PickerTarget;
use opsis_compile::layout::auto_layout;
use opsis_compile::node::Platform as NodePlatform;
use opsis_compile::nodes::media::IMPORT_TYPE;
use opsis_compile::timeline::flatten::BarThumbnail;
use opsis_compile::timeline::view;
use opsis_qt::{
    AboutView, BarView, BoundsView, CatalogRowView, CategoryView, Choice, CloseView, ColorView,
    EngineView, EventKind, FilmTiles, FontCatalogView, FontChoicesView, FontPickView,
    FontReviewView, FontRowView, GradientInfo, GraphView, Host, HostResult, InspectorView,
    InstalledFontView, KeyLineView, LicenceView, LoudnessView, MarkDrawnView, MarkPointsView,
    MenuSectionView, MeterView, MixerView, MotionInfo, NodeMove, PaletteEntry, PlayerView,
    PoseView, ProjectEntry, ProjectView, RemovalView, ReplayView, RulerStep, RunPanelView,
    ScreenArea, SettingsView, SnapView, TemplateView, TextToken, TimelineView, TraceView,
    VariableView, WindowPlacement,
};
use opsis_session::analysis::{self, AnalysisRequest, AnalysisResult};
use opsis_session::document::DiskChoice;
use opsis_session::error::CommandError;
use opsis_session::events::{MAIN_WINDOW, SessionEvent};
use opsis_session::exit::{self, Exit};
use opsis_session::run::RunHost;
use opsis_session::state::AppState;
use opsis_session::windows::{self, SavedWindow, WorkArea};
use opsis_session::{Session, compile, document, projects, run, settings, shared};
use opsis_stream::picker::PickRequest;
use opsis_stream::view::ViewInput;
use opsis_types::actions::{MouseButton, Viewport};
use opsis_types::edits::{EdgeEnds, NodePatch, WorkflowEdit};
use opsis_types::events::RunTargets;
use opsis_types::project::OUT_DIR;
use opsis_types::settings::settings_book;
use opsis_types::shell::{CloseDecision, CloseRequest, MenuCommand, ReplayTab, Selection};
use opsis_types::terminal::{PointerButton, PointerMove, TerminalInput};
use opsis_types::workflow::{Position, WORKFLOW_FILE, Workflow};
use serde::Deserialize;
use serde_json::{Map, Value};
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::RecvError;

use crate::platform::QtPlatform;
use crate::project::Extras;
use crate::{
    clipboard, colors, font_views, graph, inspector, keys, live, look, media, mixer, motion,
    player, project, replay, run_panel, streams, terminal_window, timeline,
};

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
    bundle: EditorCatalog,
    showing: AtomicBool,
    closing: Arc<Mutex<Option<CloseRequest>>>,
    compiled: Arc<Mutex<graph::Compiled>>,
    media: media::MediaCache,
    mixes: mixer::Mixes,
    audio: Arc<Mutex<opsis_audio::output::Output>>,
    previews: mixer::Previews,
    players: player::Players,
    view_windows: streams::ViewWindows,
    live: live::Live,
    engine_install: Arc<Mutex<EngineInstall>>,
    font_work: Arc<Mutex<FontWork>>,
    terminal_windows: terminal_window::TerminalWindows,
    terminal_inputs: Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TerminalInput>>>,
    view_inputs: Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<ViewInput>>>,
    started: Instant,
}

#[derive(Debug, Default)]
struct FontWork {
    review: Option<(opsis_fonts::library::InspectedFonts, bool)>,
    catalog: Option<Result<Vec<opsis_fonts::online::FontCatalogEntry>, String>>,
    downloading: String,
}

#[derive(Debug, Default)]
struct EngineInstall {
    installing: bool,
    progress: Option<f64>,
    message: String,
    error: String,
    choosing: bool,
    browser_error: String,
}

impl Drop for Ui {
    fn drop(&mut self) {
        self.players.close_all();
        self.audio.lock().expect("sound output").close();
    }
}

impl fmt::Debug for Ui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ui")
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

fn port_labels(ports: &IndexMap<String, opsis_compile::catalog::CatalogPort>) -> String {
    if ports.is_empty() {
        return "Nothing".to_owned();
    }
    ports
        .values()
        .map(|port| port.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn plain(value: &impl serde::Serialize) -> String {
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
        SessionEvent::ProjectReloaded(_) => Some(EventKind::Document),
        SessionEvent::SelectionShared(_) => Some(EventKind::Selection),
        SessionEvent::SettingsChanged(_) => Some(EventKind::Settings),
        SessionEvent::RunViewChanged(_) | SessionEvent::RunStateReset(_) => Some(EventKind::Run),
        SessionEvent::ApiStatusChanged(_) => Some(EventKind::Api),
        SessionEvent::EngineChanged(_) => Some(EventKind::Engine),
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
            bundle: editor_bundle(NodePlatform::current()),
            compiled: Arc::new(Mutex::new(graph::Compiled::default())),
            media: media::MediaCache::default(),
            mixes: mixer::Mixes::default(),
            audio: Arc::new(Mutex::new(opsis_audio::output::Output::new())),
            previews: mixer::Previews::default(),
            players: player::Players::default(),
            view_windows: streams::ViewWindows::default(),
            live: live::Live::default(),
            engine_install: Arc::default(),
            font_work: Arc::default(),
            terminal_windows: terminal_window::TerminalWindows::default(),
            terminal_inputs: Mutex::default(),
            view_inputs: Mutex::default(),
            started: Instant::now(),
        }
    }

    pub const fn session(&self) -> &Session {
        &self.session
    }

    fn state(&self) -> &AppState {
        self.session.state::<AppState>()
    }

    fn resolve(&self, dir: &Path, reference: &str) -> String {
        let public = self.state().paths.public_dir.clone();
        opsis_project::paths::ref_to_path(dir, &public, reference)
            .map_or_default(|path| path.to_string_lossy().into_owned())
    }

    fn load_media(&self, key: String, request: AnalysisRequest) {
        if !self.media.claim(&key) {
            return;
        }
        let session = self.session.clone();
        let cache = self.media.clone();
        self.runtime.spawn(async move {
            let waveform = matches!(request, AnalysisRequest::Waveform { .. });
            match analysis::analyze(&session, request).await {
                Ok(AnalysisResult::Filmstrip { filmstrip }) => {
                    cache.put_strip(&key, Some(filmstrip));
                }
                Ok(AnalysisResult::Waveform { waveform }) => cache.put_wave(&key, Some(waveform)),
                _ if waveform => cache.put_wave(&key, None),
                _ => cache.put_strip(&key, None),
            }
            opsis_qt::post_event(EventKind::Media);
        });
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

    fn issues(
        &self,
        wf: &Workflow,
        results: &IndexMap<String, opsis_types::values::NodeResult>,
    ) -> Vec<opsis_qt::IssueView> {
        let compiled = self.compiled.lock().expect("latest compile");
        if compiled.seq == 0 {
            return project::issues(wf, results);
        }
        project::issue_views(compiled.issues.iter().cloned())
    }

    fn tidy_layout(&self) -> HostResult<()> {
        let Some(wf) = self.workflow() else {
            return Ok(());
        };
        let shapes = {
            let compiled = self.compiled.lock().expect("latest compile");
            let run = run::run_state(&self.session).view.progress;
            graph::shapes(&graph::cards(&wf, &self.catalog, &compiled, &run))
        };
        let positions = auto_layout(&wf, &shapes);
        if positions.is_empty() {
            return Ok(());
        }
        self.edit(WorkflowEdit::MoveNodes { positions })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn paste(&self, subgraph: clipboard::Subgraph) -> HostResult<()> {
        let ids = self
            .edit(WorkflowEdit::Paste {
                nodes: subgraph.nodes,
                edges: subgraph.edges,
                offset: clipboard::PASTE_OFFSET,
            })
            .map_err(|e| e.0)?;
        Host::select(self, ids);
        Ok(())
    }

    fn watch_live(&self) {
        let live = self.live.clone();
        let session = self.session.clone();
        let slot = streams::FrameSlot::new(Box::new(move |header, jpeg| {
            let auto_open = || live_opens(&session);
            live.framed(header, jpeg, &auto_open);
        }));
        let earlier = run::watch_live(&self.session, MAIN_WINDOW, Arc::new(slot));
        let auto_open = || live_opens(&self.session);
        for noted in earlier {
            self.live
                .noted(&noted.capture_id, &noted.node_id, &noted.event, &auto_open);
        }
    }

    fn font_file(&self, reference: &str) -> Option<PathBuf> {
        let paths = &self.state().paths;
        let dir = self
            .state()
            .editor()
            .dir()
            .unwrap_or_else(|| paths.user_data.clone());
        reference
            .strip_prefix(opsis_compile::asset_ref::LIBRARY_SCHEME)
            .map(|rel| paths.fonts_dir().join(rel))
            .or_else(|| opsis_project::paths::ref_to_path(&dir, &paths.public_dir, reference).ok())
    }

    fn licence_text(&self, reference: Option<&str>) -> String {
        let Some(reference) = reference else {
            return "No licence file came with this font.".to_owned();
        };
        self.font_file(reference)
            .and_then(|file| std::fs::read_to_string(file).ok())
            .unwrap_or_else(|| "This licence file couldn't be read.".to_owned())
    }

    fn sample_file(&self, def: &opsis_types::fonts::FontFamilyDef) -> String {
        let italic = |face: &opsis_types::fonts::FontFaceDef| {
            matches!(face.style, opsis_types::fonts::FontStyle::Italic)
        };
        def.faces
            .iter()
            .find(|f| !italic(f) && f.weight == "400")
            .or_else(|| def.faces.iter().find(|f| !italic(f)))
            .or_else(|| def.faces.first())
            .and_then(|f| self.font_file(&f.file))
            .map_or_default(|p| p.to_string_lossy().into_owned())
    }

    fn font_row(&self, def: &opsis_types::fonts::FontFamilyDef) -> FontRowView {
        let italic = |face: &opsis_types::fonts::FontFaceDef| {
            matches!(face.style, opsis_types::fonts::FontStyle::Italic)
        };
        let sample = self.sample_file(def);
        FontRowView {
            id: def.id.clone(),
            family: def.family.clone(),
            faces: font_views::describe_faces(
                def.faces.iter().map(|f| (f.weight.as_str(), italic(f))),
            ),
            source: match def.source {
                opsis_types::fonts::FontSource::Bundled => "Built in",
                opsis_types::fonts::FontSource::Online => "Downloaded",
                opsis_types::fonts::FontSource::File => "From files",
                opsis_types::fonts::FontSource::Installed => "From this computer",
            }
            .to_owned(),
            licensed: def.license_file.is_some(),
            sample,
        }
    }

    fn fetch_catalog(&self) {
        let (session, work) = (self.session.clone(), Arc::clone(&self.font_work));
        self.runtime.spawn(async move {
            let outcome = opsis_session::fonts::font_catalog(&session)
                .await
                .map_err(|e| e.0);
            work.lock().expect("font work").catalog = Some(outcome);
            opsis_qt::post_event(EventKind::Fonts);
        });
    }

    fn send_terminal(&self, id: &str, input: TerminalInput) {
        let mut inputs = self.terminal_inputs.lock().expect("terminal inputs");
        let sender = inputs.entry(id.to_owned()).or_insert_with(|| {
            let (sender, mut queue) = tokio::sync::mpsc::unbounded_channel();
            let (session, id) = (self.session.clone(), id.to_owned());
            self.runtime.spawn(async move {
                while let Some(input) = queue.recv().await {
                    if let Err(error) =
                        opsis_session::terminal_live::terminal_input(&session, id.clone(), input)
                            .await
                    {
                        log::warn!("sending input to a terminal: {}", error.0);
                    }
                }
            });
            sender
        });
        if sender.send(input).is_err() {
            log::warn!("a terminal's input queue ended");
        }
    }

    fn send_view(&self, id: &str, input: ViewInput) {
        let mut inputs = self.view_inputs.lock().expect("view inputs");
        let sender = inputs.entry(id.to_owned()).or_insert_with(|| {
            let (sender, mut queue) = tokio::sync::mpsc::unbounded_channel();
            let (session, id) = (self.session.clone(), id.to_owned());
            self.runtime.spawn(async move {
                while let Some(input) = queue.recv().await {
                    if let Err(error) =
                        opsis_session::views::view_input(&session, id.clone(), input).await
                    {
                        log::warn!("sending input to a view: {}", error.0);
                    }
                }
            });
            sender
        });
        if sender.send(input).is_err() {
            log::warn!("a view's input queue ended");
        }
    }

    fn view_key(&self) -> Option<String> {
        self.state()
            .editor()
            .dir()
            .map(|dir| opsis_engine::tools::shown(&dir))
    }
}

fn render_mix(
    session: &Session,
    handle: &tokio::runtime::Handle,
    mixes: &mixer::Mixes,
    audio: &Arc<Mutex<opsis_audio::output::Output>>,
    previews: &mixer::Previews,
    plan: opsis_compile::mix::plan::MixPlan,
) {
    if plan.channels.is_empty() || !mixes.want(&plan.key) {
        return;
    }
    let Some(dir) = session.state::<AppState>().editor().dir() else {
        return;
    };
    let session = session.clone();
    let mixes = mixes.clone();
    let audio = Arc::clone(audio);
    let previews = previews.clone();
    handle.spawn(async move {
        let key = plan.key.clone();
        let request = AnalysisRequest::Mix {
            project_dir: dir.to_string_lossy().into_owned(),
            plan: Box::new(plan),
        };
        let outcome = match analysis::analyze(&session, request).await {
            Ok(AnalysisResult::Mix { mix }) => Ok(*mix),
            Ok(AnalysisResult::Skipped) => return,
            Ok(AnalysisResult::Error { error }) => Err(error),
            Ok(_) => Err("The mix came back as something else.".to_owned()),
            Err(error) => Err(error.0),
        };
        let stem = outcome
            .as_ref()
            .ok()
            .and_then(|mix| mix.master.as_ref())
            .map(|master| master.src.clone());
        mixes.finish(&key, outcome);
        opsis_qt::post_event(EventKind::Media);
        if let Some(src) = stem {
            load_stem(&session, &dir, &src, &audio).await;
            previews.clear();
            audio
                .lock()
                .expect("sound output")
                .set_preview(opsis_audio::output::Preview::default());
        }
    });
}

async fn load_stem(
    session: &Session,
    dir: &Path,
    src: &str,
    audio: &Arc<Mutex<opsis_audio::output::Output>>,
) {
    let state = session.state::<AppState>();
    let public = state.paths.public_dir.clone();
    let Ok(file) = opsis_project::paths::ref_to_path(dir, &public, src) else {
        return;
    };
    let tools = opsis_session::engine::engine_tools(state);
    let scratch = opsis_project::paths::project_paths(dir).analysis;
    match opsis_audio::decode::decode_stem(tools.as_ref(), &file, &scratch).await {
        Ok(samples) => audio
            .lock()
            .expect("sound output")
            .set_samples(Some(Arc::new(samples))),
        Err(error) => log::warn!("decoding the mix for playback: {error}"),
    }
}

fn live_opens(session: &Session) -> bool {
    settings::get_settings(session).live.auto_open
}

fn snap_targets(frames: &[f64]) -> Vec<view::SnapTarget> {
    frames
        .iter()
        .map(|&frame| view::SnapTarget {
            frame,
            kind: view::SnapKind::Edge,
        })
        .collect()
}

impl Host for Ui {
    fn start_pump(&self) {
        let mut events = self.session.subscribe();
        let closing = Arc::clone(&self.closing);
        let compiled = Arc::clone(&self.compiled);
        let mixes = self.mixes.clone();
        let audio = Arc::clone(&self.audio);
        let previews = self.previews.clone();
        let view_windows = self.view_windows.clone();
        let live = self.live.clone();
        let engine_install = Arc::clone(&self.engine_install);
        let handle = self.runtime.handle().clone();
        let session = self.session.clone();
        std::thread::spawn(move || {
            loop {
                match events.blocking_recv() {
                    Ok(addressed) => {
                        if let SessionEvent::ConfirmClose(asked) = &addressed.event {
                            *closing.lock().expect("close question") = Some(asked.0.clone());
                        }
                        if let SessionEvent::Compiled(value) = &addressed.event {
                            match graph::Compiled::deserialize(&value.0) {
                                Ok(latest) => {
                                    let plan = latest.plan.clone();
                                    *compiled.lock().expect("latest compile") = latest;
                                    opsis_qt::post_event(EventKind::Compiled);
                                    if let Some(plan) = plan {
                                        render_mix(
                                            &session, &handle, &mixes, &audio, &previews, plan,
                                        );
                                    }
                                }
                                Err(error) => log::warn!("reading the compile: {error}"),
                            }
                        }
                        let mine = addressed.to.as_deref().is_none_or(|to| to == MAIN_WINDOW);
                        if let SessionEvent::ReplayRequested(asked) = &addressed.event
                            && mine
                            && let Some(node) = asked.node_id.as_deref()
                        {
                            opsis_qt::post_replay(node, &plain(&asked.tab));
                        }
                        if let SessionEvent::DiskChangeAsked(asked) = &addressed.event
                            && mine
                        {
                            opsis_qt::post_disk_change(asked.readable);
                        }
                        if matches!(addressed.event, SessionEvent::ProjectReloaded(_)) && mine {
                            opsis_qt::notify(
                                "The project file changed on disk, so the editor loaded it.",
                                "Undo brings back the previous version.",
                            );
                        }
                        if let SessionEvent::EngineInstallProgress(progress) = &addressed.event {
                            let event = &progress.0;
                            let mut install = engine_install.lock().expect("engine install");
                            install.progress = event.update.progress;
                            install.message.clone_from(&event.update.message);
                            if let Some(error) = &event.error {
                                install.error.clone_from(error);
                            }
                            drop(install);
                            opsis_qt::post_event(EventKind::Engine);
                        }
                        if let SessionEvent::LiveNoted(noted) = &addressed.event
                            && mine
                        {
                            let auto_open = || live_opens(&session);
                            live.noted(&noted.capture_id, &noted.node_id, &noted.event, &auto_open);
                        }
                        if let SessionEvent::ViewEventFired(fired) = &addressed.event
                            && view_windows.noted(&fired.id, &fired.event)
                        {
                            let session = session.clone();
                            let id = fired.id.clone();
                            handle.spawn(async move {
                                if let Err(error) =
                                    opsis_session::views::view_choose_files(session, id).await
                                {
                                    log::warn!("choosing files for a view: {}", error.0);
                                }
                            });
                        }
                        if let SessionEvent::DocumentChanged(changed) = &addressed.event
                            && mine
                        {
                            opsis_qt::post_edited(changed.edited.clone());
                        }
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
        self.watch_live();
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
        {
            let mut audio = self.audio.lock().expect("sound output");
            audio.pause();
            audio.set_samples(None);
        }
        self.players.close_all();
        self.live.reset();
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
        let run = run::run_state(&self.session).view.progress;
        let nodes = {
            let compiled = self.compiled.lock().expect("latest compile");
            graph::cards(wf, &self.catalog, &compiled, &run)
        };
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
                        category_id: category.clone(),
                        title: n.title.clone(),
                        category: n.category_label.clone(),
                        color: look::category_color(&category).to_owned(),
                        glyph: n.look.as_ref().map_or_default(|l| l.glyph.clone()),
                        description: n.description.clone().unwrap_or_default(),
                        inputs: port_labels(&n.inputs),
                        outputs: port_labels(&n.outputs),
                    },
                )
            })
            .collect();
        entries.sort_by_key(|(order, _)| *order);
        entries.into_iter().map(|(_, e)| e).collect()
    }

    fn categories(&self) -> Vec<CategoryView> {
        let mut seen: Vec<CategoryView> = Vec::new();
        for entry in self.palette() {
            if seen.iter().all(|c| c.id != entry.category_id) {
                seen.push(CategoryView {
                    icon: look::category_icon(&entry.category_id).to_owned(),
                    id: entry.category_id,
                    label: entry.category,
                    color: entry.color,
                });
            }
        }
        seen
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
                position: Some(snap_to_grid(Position { x, y })),
                params: None,
            })
            .map_err(|e| e.0)?
            .into_iter()
            .next()
            .unwrap_or_default();
        self.select(vec![id.clone()]);
        Ok(id)
    }

    fn resize_node(&self, id: &str, x: f64, y: f64, width: f64, height: f64) -> HostResult<()> {
        let wf = self
            .workflow()
            .ok_or_else(|| "No project is open.".to_owned())?;
        let node = wf.node(id).ok_or_else(|| "That node is gone.".to_owned())?;
        let parent = node
            .parent_id
            .as_deref()
            .and_then(|p| wf.node(p))
            .map_or(Position { x: 0.0, y: 0.0 }, |p| {
                opsis_compile::edits::workflow::absolute_position(&wf, p)
            });
        let whole = opsis_types::numbers::whole;
        let shift = Position {
            x: x - parent.x - node.position.x,
            y: y - parent.y - node.position.y,
        };
        let children: IndexMap<String, Position> = wf
            .nodes
            .iter()
            .filter(|child| child.parent_id.as_deref() == Some(id))
            .map(|child| {
                (
                    child.id.clone(),
                    Position {
                        x: child.position.x - shift.x,
                        y: child.position.y - shift.y,
                    },
                )
            })
            .collect();
        document::begin_gesture(&self.session);
        let moved = if children.is_empty() {
            Ok(Vec::new())
        } else {
            self.edit(WorkflowEdit::MoveNodes {
                positions: children,
            })
        };
        let resized = moved.and_then(|_| {
            self.edit(WorkflowEdit::UpdateNode {
                id: id.to_owned(),
                patch: NodePatch {
                    position: Some(Position {
                        x: x - parent.x,
                        y: y - parent.y,
                    }),
                    size: Some(Viewport {
                        width: whole(width),
                        height: whole(height),
                    }),
                    ..NodePatch::default()
                },
            })
        });
        document::end_gesture(&self.session);
        resized.map(|_| ()).map_err(|e| e.0)
    }

    fn drop_files(&self, paths: Vec<String>, x: f64, y: f64) -> HostResult<u32> {
        let Some(dir) = self.state().editor().dir() else {
            return Ok(0);
        };
        let assets = self
            .runtime
            .block_on(projects::import_assets(
                self.session.clone(),
                dir.to_string_lossy().into_owned(),
                Some(paths),
            ))
            .map_err(|e| format!("Couldn't add the files: {}", e.0))?;
        let mut added = Vec::new();
        for (index, asset) in assets.iter().enumerate() {
            let mut params = Map::new();
            params.insert("path".to_owned(), Value::String(asset.clone()));
            let ids = self
                .edit(WorkflowEdit::AddNode {
                    kind: IMPORT_TYPE.to_owned(),
                    id: None,
                    label: None,
                    position: Some(staggered(Position { x, y }, index)),
                    params: Some(params),
                })
                .map_err(|e| e.0)?;
            added.extend(ids);
        }
        if !added.is_empty() {
            self.select(added);
        }
        Ok(u32::try_from(assets.len()).unwrap_or(u32::MAX))
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

    fn selected_ids(&self) -> Vec<String> {
        shared::shared_state(&self.session).selection.selected_ids
    }

    fn inspector(&self) -> InspectorView {
        let empty = InspectorView {
            id: String::new(),
            title: String::new(),
            label: String::new(),
            category: String::new(),
            category_id: String::new(),
            color: String::new(),
            glyph: String::new(),
            description: String::new(),
            fields: Vec::new(),
            renamed: false,
            disabled: false,
            runs: false,
            group: false,
            running: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            files: Vec::new(),
            swatches: Vec::new(),
            can_replay: false,
            has_trace: false,
        };
        let Some(wf) = self.workflow() else {
            return empty;
        };
        let selected = shared::shared_state(&self.session)
            .selection
            .selected_ids
            .len();
        if selected > 1 {
            return empty;
        }
        let Some(node) = wf.node(&self.selected()) else {
            return empty;
        };
        let Some(def) = self.catalog.get(&node.kind) else {
            return empty;
        };
        let category = plain(&def.category);
        let (errors, warnings) = {
            let compiled = self.compiled.lock().expect("latest compile");
            let of = |level: IssueLevel| -> Vec<String> {
                compiled
                    .issues
                    .iter()
                    .filter(|i| i.level == level && i.node_id.as_deref() == Some(&node.id))
                    .map(|i| i.message.clone())
                    .collect()
            };
            (of(IssueLevel::Error), of(IssueLevel::Warning))
        };
        let dir = self.state().editor().dir();
        let resolve = |reference: &str| {
            dir.as_ref()
                .map_or_default(|dir| self.resolve(dir, reference))
        };
        let files = self
            .state()
            .editor()
            .results
            .get(&node.id)
            .map_or_default(|result| {
                result
                    .artifacts
                    .iter()
                    .map(|a| run_panel::artifact(&resolve, a))
                    .collect()
            });
        let replayable = self.replay(&node.id, "replay");
        InspectorView {
            id: node.id.clone(),
            title: def.title.clone(),
            label: node.label.clone().unwrap_or_else(|| def.title.clone()),
            category: def.category_label.clone(),
            category_id: category.clone(),
            color: look::category_color(&category).to_owned(),
            glyph: def.look.as_ref().map_or_default(|l| l.glyph.clone()),
            description: def.description.clone().unwrap_or_default(),
            fields: {
                let compiled = self.compiled.lock().expect("latest compile");
                let context = compiled.inspector.as_ref().filter(|c| c.node_id == node.id);
                inspector::fields(
                    def,
                    node,
                    &inspector::FormCtx {
                        inspector: context,
                        catalog: &self.bundle,
                        workflow: &wf,
                        playhead: shared::shared_state(&self.session).playhead.and_then(|p| {
                            compiled
                                .flat
                                .as_ref()
                                .and_then(|f| graph::playhead_in_node(f, &node.id, p.frame))
                        }),
                    },
                )
            },
            renamed: node
                .label
                .as_deref()
                .is_some_and(|l| !l.is_empty() && l != def.title),
            disabled: node.disabled.unwrap_or(false),
            runs: def.executes || def.is_virtual,
            group: opsis_compile::graph::is_group_node(node),
            running: self.session.state::<RunHost>().is_running(),
            errors,
            warnings,
            files,
            swatches: self
                .compiled
                .lock()
                .expect("latest compile")
                .inspector
                .as_ref()
                .filter(|c| c.node_id == node.id)
                .map_or_default(|c| c.swatches.clone()),
            can_replay: replayable.can_replay,
            has_trace: !replayable.trace_id.is_empty() && dir.is_some(),
        }
    }

    fn set_param(&self, id: &str, key: &str, json: &str) -> HostResult<()> {
        let mut params = Map::new();
        let value = inspector::parsed_value(json);
        if let Some((top, rest)) = key.split_once('/') {
            let wf = self
                .workflow()
                .ok_or_else(|| "No project is open.".to_owned())?;
            let node = wf.node(id).ok_or_else(|| "That node is gone.".to_owned())?;
            let current = node
                .params
                .get(top)
                .cloned()
                .or_else(|| {
                    self.catalog
                        .get(&node.kind)
                        .and_then(|d| d.defaults.get(top).cloned())
                })
                .unwrap_or(Value::Null);
            params.insert(top.to_owned(), inspector::set_at(current, rest, value));
        } else {
            params.insert(key.to_owned(), value);
        }
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

    fn project(&self) -> ProjectView {
        let Some(wf) = self.workflow() else {
            return project::empty();
        };
        let results = self.state().editor().results.clone();
        let selected = shared::shared_state(&self.session)
            .selection
            .selected_ids
            .len();
        project::view(
            &wf,
            Extras {
                issues: self.issues(&wf, &results),
                running: self.session.state::<RunHost>().is_running(),
                selected: u32::try_from(selected).unwrap_or(u32::MAX),
            },
        )
    }

    fn variables(&self) -> Vec<VariableView> {
        self.workflow().map_or_default(|wf| {
            opsis_compile::vars::workflow_vars(&wf.name, &wf.variables)
                .into_iter()
                .map(|(name, value)| VariableView { name, value })
                .collect()
        })
    }

    fn project_edit(&self, json: &str) -> HostResult<()> {
        let edit: WorkflowEdit = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.edit(edit).map(|_| ()).map_err(|e| e.0)
    }

    fn clear_saved_results(&self) -> HostResult<()> {
        let Some(dir) = self.state().editor().dir() else {
            return Ok(());
        };
        projects::clear_cache(&self.session, &dir.to_string_lossy()).map_err(|e| e.0)
    }

    fn motion_info(&self, motion: &str) -> MotionInfo {
        motion::info(motion)
    }

    fn motion_pose(&self, motion: &str, elapsed_ms: f64) -> PoseView {
        motion::pose(motion, elapsed_ms)
    }

    fn bar(&self) -> BarView {
        let api = opsis_session::api::api_status(&self.session);
        BarView {
            api_enabled: api.enabled,
            api_listening: api.listening,
            running: self.session.state::<RunHost>().is_running(),
        }
    }

    fn reveal_outputs(&self) -> HostResult<()> {
        let Some(dir) = self.state().editor().dir() else {
            return Ok(());
        };
        std::fs::create_dir_all(dir.join(OUT_DIR)).map_err(|e| e.to_string())?;
        settings::reveal_in_folder(
            &self.session,
            &dir.to_string_lossy(),
            Some(OUT_DIR.to_owned()),
        )
        .map_err(|e| e.0)
    }

    fn copy_selection(&self) -> String {
        let ids = shared::shared_state(&self.session).selection.selected_ids;
        self.workflow()
            .and_then(|wf| clipboard::selected(&wf, &ids))
            .map_or_default(|subgraph| clipboard::copy_text(&subgraph))
    }

    fn paste_text(&self, text: &str) -> HostResult<()> {
        match clipboard::read(text) {
            clipboard::Pasted::Foreign => Ok(()),
            clipboard::Pasted::Unreadable => Err(clipboard::UNREADABLE.to_owned()),
            clipboard::Pasted::Nodes(subgraph) => self.paste(subgraph),
        }
    }

    fn duplicate_selection(&self) -> HostResult<()> {
        let ids = shared::shared_state(&self.session).selection.selected_ids;
        match self
            .workflow()
            .and_then(|wf| clipboard::selected(&wf, &ids))
        {
            Some(subgraph) => self.paste(subgraph),
            None => Ok(()),
        }
    }

    fn text_tokens(&self, value: &str, pauses: bool) -> Vec<TextToken> {
        project::text_tokens(value, pauses)
    }

    fn join_tokens(&self, tokens: Vec<TextToken>) -> String {
        project::join_tokens(&tokens)
    }

    fn parse_color(&self, text: &str) -> ColorView {
        colors::view(text)
    }

    fn format_color(&self, h: f64, s: f64, v: f64, a: f64, format: &str) -> String {
        colors::text(h, s, v, a, format)
    }

    fn gradient_info(&self, css: &str) -> GradientInfo {
        colors::gradient(css)
    }

    fn box_from(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> BoundsView {
        colors::drawn_box(x0, y0, x1, y1)
    }

    fn move_box(&self, origin: BoundsView, dx: f64, dy: f64) -> BoundsView {
        colors::moved(origin, dx, dy)
    }

    fn select_pinned(&self, ids: Vec<String>, source: &str) {
        let pinned = (!source.is_empty()).then(|| source.to_owned());
        let single = (ids.len() == 1).then(|| ids[0].clone());
        let selection = Selection {
            selected_ids: ids.clone(),
            active_ids: ids,
            pinned_source: pinned.clone(),
        };
        if let Err(error) = shared::share_selection(&self.session, MAIN_WINDOW, selection) {
            log::warn!("sharing the selection: {}", error.0);
        }
        compile::set_compile_focus(&self.session, pinned.or_else(|| single.clone()), single);
    }

    fn film_tiles(
        &self,
        bar: &str,
        px_per_frame: f64,
        height: f64,
        offset: f64,
        width: f64,
    ) -> FilmTiles {
        let empty = FilmTiles {
            image: String::new(),
            tile_width: 0,
            tile_height: 0,
            still: false,
            tiles: Vec::new(),
        };
        let Some(dir) = self.state().editor().dir() else {
            return empty;
        };
        let dir_text = dir.to_string_lossy().into_owned();
        let editor = self.state().editor();
        let Some(film) = editor.film.as_ref() else {
            return empty;
        };
        let Some(video) = film.flat.bars.iter().find(|b| b.key == bar) else {
            return empty;
        };
        match &video.thumbnail {
            Some(BarThumbnail::Still { src }) => FilmTiles {
                image: self.resolve(&dir, src),
                still: true,
                ..empty
            },
            Some(BarThumbnail::Filmstrip {
                src,
                speed,
                trim_ms,
            }) => {
                let key = media::key(&dir_text, "filmstrip", src);
                let Some(filmstrip) = self.media.filmstrip(&key) else {
                    self.load_media(
                        key,
                        AnalysisRequest::Filmstrip {
                            project_dir: dir_text,
                            src: src.clone(),
                        },
                    );
                    return empty;
                };
                let tiles = timeline::film_tiles(
                    video,
                    film.flat.fps,
                    &timeline::Strip {
                        filmstrip: &filmstrip,
                        speed: *speed,
                        trim_ms: *trim_ms,
                    },
                    px_per_frame,
                    height,
                    offset,
                    width,
                );
                FilmTiles {
                    image: self.resolve(&dir, &filmstrip.image),
                    tile_width: filmstrip.tile_width,
                    tile_height: filmstrip.tile_height,
                    still: false,
                    tiles,
                }
            }
            None => empty,
        }
    }

    fn wave_columns(&self, bar: &str, px_per_frame: f64, offset: f64, width: u32) -> Vec<f32> {
        let Some(dir) = self.state().editor().dir() else {
            return Vec::new();
        };
        let dir_text = dir.to_string_lossy().into_owned();
        let editor = self.state().editor();
        let Some(film) = editor.film.as_ref() else {
            return Vec::new();
        };
        let Some(audio) = film.flat.audio.iter().find(|b| b.key == bar) else {
            return Vec::new();
        };
        let key = media::key(&dir_text, "waveform", &audio.src);
        let Some(wave) = self.media.waveform(&key) else {
            self.load_media(
                key,
                AnalysisRequest::Waveform {
                    project_dir: dir_text,
                    src: audio.src.clone(),
                },
            );
            return Vec::new();
        };
        timeline::wave_columns(audio, film.flat.fps, &wave, px_per_frame, offset, width)
    }

    fn key_line(&self, bar: &str, track: &str) -> KeyLineView {
        let Some(workflow) = self.workflow() else {
            return timeline::hidden_line();
        };
        let selected = std::collections::HashSet::new();
        let editor = self.state().editor();
        let Some(film) = editor.film.as_ref() else {
            return timeline::hidden_line();
        };
        timeline::key_line(
            &film.flat,
            &timeline::Sources {
                workflow: &workflow,
                catalog: &self.catalog,
                transitions: &self.bundle.transitions,
                selected: &selected,
            },
            bar,
            track,
        )
    }

    fn bar_local(&self, bar: &str, abs: f64) -> f64 {
        let editor = self.state().editor();
        editor
            .film
            .as_ref()
            .map_or(abs, |film| timeline::bar_local(&film.flat, bar, abs))
    }

    fn mixer(&self) -> MixerView {
        let Some(workflow) = self.workflow() else {
            return mixer::hidden();
        };
        let selected: std::collections::HashSet<String> = shared::shared_state(&self.session)
            .selection
            .selected_ids
            .into_iter()
            .collect();
        let source_id = self
            .state()
            .editor()
            .film
            .as_ref()
            .and_then(|film| film.source.clone())
            .unwrap_or_default();
        let compiled = self.compiled.lock().expect("latest compile");
        mixer::view(
            compiled.plan.as_ref(),
            &compiled.mix,
            compiled.solo,
            &mixer::Sources {
                workflow: &workflow,
                catalog: &self.catalog,
                selected: &selected,
                source_id,
            },
            &self.mixes,
        )
    }

    fn meter(&self, target: &str, frame: f64) -> MeterView {
        mixer::meter(self.mixes.latest().as_deref(), target, frame)
    }

    fn loudness_at(&self, frame: f64) -> LoudnessView {
        mixer::loudness(self.mixes.latest().as_deref(), frame)
    }

    fn edit_mix(&self, edit_json: &str) -> HostResult<()> {
        let edit: opsis_types::edits::MixEdit =
            serde_json::from_str(edit_json).map_err(|e| e.to_string())?;
        self.edit(WorkflowEdit::Mix { edit })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn eq_curve(&self, eq_json: &str, highpass: f64, steps: u32) -> Vec<f64> {
        let eq: opsis_types::mix::Eq = serde_json::from_str(eq_json).unwrap_or_default();
        opsis_compile::mix::eq::curve(
            &eq.bands,
            eq.on,
            (highpass > 0.0).then_some(highpass),
            steps,
        )
    }

    fn mix_text(&self, kind: &str, value: f64) -> String {
        use opsis_compile::mix::controls as c;
        match kind {
            "db" => c::format_db(value),
            "level" => c::format_level_db(value),
            "signed" => c::format_signed_db(value),
            "hz" => c::format_hz(value),
            "ms" => c::format_ms(value),
            "pan" => c::format_pan(value),
            _ => c::format_reading(value, -70.0, false),
        }
    }

    fn knob_value(&self, min: f64, max: f64, step: f64, log: bool, fraction: f64) -> f64 {
        opsis_compile::mix::controls::fraction_to_value(
            opsis_compile::mix::controls::KnobRange {
                min,
                max,
                step,
                log,
            },
            fraction,
        )
    }

    fn knob_fraction(&self, min: f64, max: f64, log: bool, value: f64) -> f64 {
        opsis_compile::mix::controls::value_to_fraction(
            opsis_compile::mix::controls::KnobRange {
                min,
                max,
                step: 0.0,
                log,
            },
            value,
        )
    }

    fn audio_play(&self, seconds: f64, rate: f64) {
        self.audio.lock().expect("sound output").play(seconds, rate);
    }

    fn audio_pause(&self) {
        self.audio.lock().expect("sound output").pause();
    }

    fn audio_seek(&self, seconds: f64) {
        self.audio.lock().expect("sound output").seek(seconds);
    }

    fn audio_clock(&self) -> f64 {
        self.audio
            .lock()
            .expect("sound output")
            .clock()
            .unwrap_or(-1.0)
    }

    fn audio_ready(&self) -> bool {
        self.audio.lock().expect("sound output").has_samples()
    }

    fn audio_scrub(&self, seconds: f64) {
        self.audio.lock().expect("sound output").scrub(seconds);
    }

    fn preview_mix(&self, target: &str, kind: &str, current: f64, next: f64, band_json: &str) {
        let frame = shared::shared_state(&self.session)
            .playhead
            .map_or(0.0, |p| p.frame);
        let fraction = mixer::share_of(self.mixes.latest().as_deref(), target, frame);
        let band = serde_json::from_str::<opsis_types::mix::EqBand>(band_json).ok();
        let total = self.previews.change(
            format!("{target}:{kind}"),
            fraction,
            kind,
            current,
            next,
            band,
        );
        self.audio.lock().expect("sound output").set_preview(total);
    }

    fn run_panel(&self) -> RunPanelView {
        let Some(workflow) = self.workflow() else {
            return run_panel::empty();
        };
        let selection = shared::shared_state(&self.session).selection.selected_ids;
        let selected = (selection.len() == 1).then(|| selection[0].clone());
        let dir = self.state().editor().dir();
        let resolve = |reference: &str| {
            dir.as_ref()
                .map_or_default(|dir| self.resolve(dir, reference))
        };
        let outdated = self
            .compiled
            .lock()
            .expect("latest compile")
            .outdated
            .clone();
        run_panel::view(
            &run::run_state(&self.session),
            &run_panel::Sources {
                workflow: &workflow,
                catalog: &self.catalog,
                outdated: &outdated,
                selected: selected.as_deref(),
                resolve: &resolve,
            },
        )
    }

    fn clear_run_log(&self) {
        run::clear_run_log(&self.session);
    }

    fn player_open(&self, file: &str) -> u64 {
        self.players
            .open(&self.session, self.runtime.handle(), file)
    }

    fn player_close(&self, id: u64) {
        self.players.close(id);
    }

    fn player_view(&self, id: u64) -> PlayerView {
        self.players.view(id)
    }

    fn player_frame(&self, id: u64) -> opsis_qt::FrameRef {
        self.players.frame(id)
    }

    fn player_play(&self, id: u64) {
        self.players.play(&self.session, id);
    }

    fn player_pause(&self, id: u64) {
        self.players.pause(id);
    }

    fn player_seek(&self, id: u64, seconds: f64) {
        self.players.seek(&self.session, id, seconds);
    }

    fn player_rate(&self, id: u64, rate: f64) {
        self.players.set_rate(&self.session, id, rate);
    }

    fn player_volume(&self, id: u64, volume: f64, muted: bool) {
        self.players.set_volume(id, volume, muted);
    }

    fn player_retry(&self, id: u64) {
        self.players.retry(&self.session, self.runtime.handle(), id);
    }

    fn player_time(&self, seconds: f64, guide: f64) -> String {
        player::clock_text(seconds, guide)
    }

    fn show_replay(&self, node_id: &str, tab: &str) -> HostResult<()> {
        let tab = if tab == "trace" {
            ReplayTab::Trace
        } else {
            ReplayTab::Replay
        };
        shared::show_replay(&self.session, Some(node_id.to_owned()), tab).map_err(|e| e.0)
    }

    fn replay(&self, node_id: &str, tab: &str) -> ReplayView {
        let Some(wf) = self.workflow() else {
            return replay::hidden();
        };
        let Some(node) = wf.node(node_id) else {
            return replay::hidden();
        };
        let def = self.catalog.get(&node.kind);
        let title = node
            .label
            .clone()
            .filter(|l| !l.is_empty())
            .or_else(|| def.map(|d| d.title.clone()))
            .unwrap_or_else(|| node_id.to_owned());
        let capture = def.is_some_and(|d| plain(&d.category) == "Capture");
        let state = run::run_state(&self.session);
        let progress = &state.view.progress;
        let editor = self.state().editor();
        let result = progress
            .results
            .get(node_id)
            .or_else(|| editor.results.get(node_id));
        let dir = editor.dir();
        let resolve = |reference: &str| {
            dir.as_ref()
                .map_or_default(|dir| self.resolve(dir, reference))
        };
        let has_result = result.is_some();
        let trace = progress.traces.get(node_id).map(String::as_str);
        let mut view = replay::view(&replay::Sources {
            node_id,
            title,
            capture,
            result,
            error: progress.errors.get(node_id).map(String::as_str),
            trace,
            tab,
            resolve: &resolve,
        });
        view.can_replay = view.can_replay && (has_result || trace.is_some());
        if dir.is_none() {
            view.trace_id.clear();
        }
        view
    }

    fn view_attach(&self, id: &str) -> u64 {
        let (target, slot) = self.view_windows.slot(id);
        if let Err(error) =
            opsis_session::views::attach_view(self.session.clone(), id.to_owned(), Arc::new(slot))
        {
            log::warn!("showing a view: {}", error.0);
        }
        target
    }

    fn view_state(&self, id: &str) -> opsis_qt::StreamViewState {
        self.view_windows.state(id)
    }

    fn view_frame(&self, id: &str) -> opsis_qt::FrameRef {
        self.view_windows.frame(id)
    }

    fn view_mouse(
        &self,
        id: &str,
        kind: u8,
        x: f64,
        y: f64,
        button: u32,
        buttons: u32,
        clicks: i32,
        modifiers: u32,
    ) {
        let modifiers = keys::dom_modifiers(modifiers, cfg!(target_os = "macos"));
        let buttons = i64::from(buttons & 0x1f);
        let click_count = i64::from(clicks.max(1));
        let button = match button {
            1 => MouseButton::Left,
            2 => MouseButton::Right,
            4 => MouseButton::Middle,
            8 => MouseButton::Back,
            16 => MouseButton::Forward,
            _ => MouseButton::None,
        };
        let input = match kind {
            0 => ViewInput::MouseDown {
                x,
                y,
                button,
                buttons,
                click_count,
                modifiers,
            },
            1 => ViewInput::MouseUp {
                x,
                y,
                button,
                buttons,
                click_count,
                modifiers,
            },
            _ => ViewInput::MouseMove {
                x,
                y,
                buttons,
                modifiers,
            },
        };
        self.send_view(id, input);
    }

    fn view_wheel(&self, id: &str, x: f64, y: f64, dx: f64, dy: f64, modifiers: u32) {
        self.send_view(
            id,
            ViewInput::Wheel {
                x,
                y,
                delta_x: dx,
                delta_y: dy,
                modifiers: keys::dom_modifiers(modifiers, cfg!(target_os = "macos")),
            },
        );
    }

    fn view_key(&self, id: &str, down: bool, key: u32, text: &str, modifiers: u32, repeat: bool) {
        let keypad = modifiers & 0x2000_0000 != 0;
        let modifiers = keys::dom_modifiers(modifiers, cfg!(target_os = "macos"));
        let dom = keys::dom_key(key, text, keypad);
        let input = if down {
            ViewInput::KeyDown {
                text: keys::typed_text(&dom, text, modifiers),
                key: dom.key,
                code: dom.code,
                key_code: dom.key_code,
                location: dom.location,
                modifiers,
                repeat,
            }
        } else {
            ViewInput::KeyUp {
                key: dom.key,
                code: dom.code,
                key_code: dom.key_code,
                location: dom.location,
                modifiers,
            }
        };
        self.send_view(id, input);
    }

    fn view_text(&self, id: &str, text: &str) {
        self.send_view(
            id,
            ViewInput::Text {
                text: text.to_owned(),
            },
        );
    }

    fn view_navigate(&self, id: &str, url: &str) {
        let (session, id, url) = (self.session.clone(), id.to_owned(), url.to_owned());
        self.runtime.spawn(async move {
            if let Err(error) = opsis_session::views::view_navigate(&session, id, url).await {
                log::warn!("going to an address in a view: {}", error.0);
            }
        });
    }

    fn view_go(&self, id: &str, steps: i64) {
        let (session, id) = (self.session.clone(), id.to_owned());
        self.runtime.spawn(async move {
            if let Err(error) = opsis_session::views::view_go(&session, id, steps).await {
                log::warn!("going back or forward in a view: {}", error.0);
            }
        });
    }

    fn view_reload(&self, id: &str) {
        let (session, id) = (self.session.clone(), id.to_owned());
        self.runtime.spawn(async move {
            if let Err(error) = opsis_session::views::view_reload(&session, id).await {
                log::warn!("reloading a view: {}", error.0);
            }
        });
    }

    fn view_switch_tab(&self, id: &str, tab: &str) {
        let (session, id, tab) = (self.session.clone(), id.to_owned(), tab.to_owned());
        self.runtime.spawn(async move {
            if let Err(error) = opsis_session::views::view_switch_tab(&session, id, tab).await {
                log::warn!("switching a view's tab: {}", error.0);
            }
        });
    }

    fn view_answer(&self, id: &str, accept: bool, prompt: &str) {
        self.view_windows.answered(id);
        let (session, id, prompt) = (self.session.clone(), id.to_owned(), prompt.to_owned());
        self.runtime.spawn(async move {
            if let Err(error) =
                opsis_session::views::view_answer_dialog(&session, id, accept, prompt).await
            {
                log::warn!("answering a page's dialog: {}", error.0);
            }
        });
    }

    fn view_resize(&self, id: &str, width: u32, height: u32, dpr: f64) {
        let (session, id) = (self.session.clone(), id.to_owned());
        self.runtime.spawn(async move {
            let size = opsis_stream::view::ViewSize { width, height, dpr };
            if let Err(error) = opsis_session::views::view_resize(&session, id, size).await {
                log::warn!("resizing a view: {}", error.0);
            }
        });
    }

    fn pick_target(&self, node_id: &str, key: &str) {
        let picker = self
            .compiled
            .lock()
            .expect("latest compile")
            .inspector
            .as_ref()
            .filter(|c| c.node_id == node_id)
            .map(|c| c.picker.clone());
        let page = match picker {
            Some(PickerTarget::Page(page)) => page,
            Some(PickerTarget::Problem { message }) => {
                opsis_qt::notify(&message, "");
                return;
            }
            None => {
                opsis_qt::notify("The page to pick from isn't ready yet.", "");
                return;
            }
        };
        opsis_qt::notify(
            "A browser window opened. Click the element, or Shift-click a spot.",
            "",
        );
        let (session, node, key) = (self.session.clone(), node_id.to_owned(), key.to_owned());
        self.runtime.spawn(async move {
            let request = PickRequest {
                url: page.url,
                viewport: page.viewport,
            };
            match opsis_session::views::pick_element(session.clone(), request).await {
                Ok(Some(chosen)) => {
                    let mut params = Map::new();
                    params.insert(key.clone(), Value::String(chosen.target.clone()));
                    params.insert(
                        format!("{key}Role"),
                        chosen
                            .role
                            .filter(|r| !r.is_empty())
                            .map_or(Value::Null, Value::String),
                    );
                    let edit = WorkflowEdit::UpdateNode {
                        id: node,
                        patch: NodePatch {
                            params: Some(params),
                            ..NodePatch::default()
                        },
                    };
                    match document::edit_workflow(&session, edit) {
                        Ok(_) => opsis_qt::notify(&format!("Picked “{}”", chosen.target), ""),
                        Err(error) => opsis_qt::notify("Couldn't pick the element", &error.0),
                    }
                }
                Ok(None) => {}
                Err(error) => opsis_qt::notify("Couldn't pick the element", &error.0),
            }
        });
    }

    fn live_target(&self) -> u64 {
        self.live.target()
    }

    fn live_view(&self) -> opsis_qt::LiveView {
        let workflow = self.workflow();
        let name_of = |node: &str| {
            workflow
                .as_ref()
                .and_then(|wf| wf.node(node))
                .and_then(|n| {
                    n.label
                        .clone()
                        .filter(|l| !l.is_empty())
                        .or_else(|| self.catalog.get(&n.kind).map(|d| d.title.clone()))
                })
                .unwrap_or_else(|| "Live recording".to_owned())
        };
        self.live.view(&name_of)
    }

    fn live_frame(&self, capture_id: &str) -> opsis_qt::FrameRef {
        self.live.frame(capture_id)
    }

    fn live_focus(&self, capture_id: &str) {
        self.live.focus(capture_id);
    }

    fn live_pip(&self, open: bool) {
        self.live.set_pip(open);
    }

    fn terminal_attach(&self, id: &str) -> u64 {
        let (target, slot) = self.terminal_windows.slot(id);
        if let Err(error) =
            opsis_session::terminal_live::attach_terminal(&self.session, id, Arc::new(slot))
        {
            log::warn!("showing a terminal: {}", error.0);
        }
        target
    }

    fn terminal_view(&self, id: &str) -> opsis_qt::TerminalWindowView {
        self.terminal_windows.view(id)
    }

    fn terminal_frame(&self, id: &str) -> opsis_qt::FrameRef {
        self.terminal_windows.frame(id)
    }

    fn terminal_key(&self, id: &str, key: u32, text: &str, modifiers: u32) {
        let dom = keys::dom_key(key, text, modifiers & 0x2000_0000 != 0);
        let bits = keys::dom_modifiers(modifiers, cfg!(target_os = "macos"));
        let input = terminal_window::key_input(&dom, text, bits);
        if let Some(input) = input {
            self.send_terminal(id, input);
        }
    }

    fn terminal_paste(&self, id: &str, text: &str) {
        self.send_terminal(
            id,
            TerminalInput::Paste {
                text: text.to_owned(),
            },
        );
    }

    fn terminal_pointer(
        &self,
        id: &str,
        action: u8,
        button: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) {
        let grid = self.terminal_windows.grid(id);
        let (col, row) = terminal_window::cell_at(x, y, width, height, &grid);
        let button = match button {
            1 => Some(PointerButton::Left),
            2 => Some(PointerButton::Right),
            4 => Some(PointerButton::Middle),
            _ => None,
        };
        let action = match action {
            0 => PointerMove::Press,
            1 => PointerMove::Release,
            _ => PointerMove::Move,
        };
        self.send_terminal(
            id,
            TerminalInput::Pointer {
                action,
                button,
                col,
                row,
            },
        );
    }

    fn terminal_wheel(&self, id: &str, x: f64, y: f64, width: f64, height: f64, notches: i32) {
        let grid = self.terminal_windows.grid(id);
        let (col, row) = terminal_window::cell_at(x, y, width, height, &grid);
        self.send_terminal(id, TerminalInput::Wheel { col, row, notches });
    }

    fn terminal_focus(&self, id: &str, gained: bool) {
        self.send_terminal(id, TerminalInput::Focus { gained });
    }

    fn terminal_finish(&self, id: &str, save: bool) -> HostResult<()> {
        opsis_session::terminal_live::finish_terminal(&self.session, id, save).map_err(|e| e.0)
    }

    fn terminal_closed(&self, id: &str) {
        self.terminal_inputs
            .lock()
            .expect("terminal inputs")
            .remove(id);
        self.terminal_windows.forget(id);
        self.platform.terminal_window_closed(id);
    }

    fn active_nodes(&self, frame: f64) -> Vec<String> {
        self.compiled
            .lock()
            .expect("latest compile")
            .film
            .as_ref()
            .map_or_default(|film| film.active_at(frame))
    }

    fn selection_start(&self) -> f64 {
        let selection = shared::shared_state(&self.session).selection;
        let [only] = selection.selected_ids.as_slice() else {
            return -1.0;
        };
        if selection.pinned_source.is_some() {
            return -1.0;
        }
        self.compiled
            .lock()
            .expect("latest compile")
            .film
            .as_ref()
            .and_then(|film| film.spans.get(only))
            .map_or(-1.0, |span| span.from)
    }

    fn resolve_disk_change(&self, choice: &str) -> HostResult<()> {
        let choice = match choice {
            "load" => DiskChoice::Load,
            "keepMine" => DiskChoice::KeepMine,
            _ => DiskChoice::Wait,
        };
        self.runtime
            .block_on(document::resolve_disk_change(self.session.clone(), choice))
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn view_closed(&self, id: &str) {
        self.view_inputs.lock().expect("view inputs").remove(id);
        self.view_windows.forget(id);
        self.platform.view_window_closed(id);
        opsis_session::views::view_close(&self.session, id);
    }

    fn trace(&self, trace_id: &str) -> TraceView {
        let Some(dir) = self.state().editor().dir() else {
            return replay::failed_trace("No project is open.".to_owned());
        };
        let resolve = |reference: &str| self.resolve(&dir, reference);
        match settings::read_trace(&dir.to_string_lossy(), trace_id) {
            Ok(trace) => replay::trace_view(&trace, &resolve),
            Err(error) => replay::failed_trace(error.0),
        }
    }

    fn timeline(&self) -> TimelineView {
        let Some(workflow) = self.workflow() else {
            return timeline::empty();
        };
        let selected: std::collections::HashSet<String> = shared::shared_state(&self.session)
            .selection
            .selected_ids
            .into_iter()
            .collect();
        let film = self.state().editor().film.clone();
        timeline::view(
            film.as_ref(),
            &timeline::Sources {
                workflow: &workflow,
                catalog: &self.catalog,
                transitions: &self.bundle.transitions,
                selected: &selected,
            },
        )
    }

    fn edit_timeline(&self, edit_json: &str) -> String {
        let edit: opsis_types::edits::TimelineEdit = match serde_json::from_str(edit_json) {
            Ok(edit) => edit,
            Err(error) => return error.to_string(),
        };
        match document::edit_timeline(&self.session, edit) {
            Ok(reply) => reply.error.unwrap_or_default(),
            Err(error) => error.0,
        }
    }

    fn edit_markers(&self, edit_json: &str) -> HostResult<()> {
        let edit: opsis_types::edits::MarkerEdit =
            serde_json::from_str(edit_json).map_err(|e| e.to_string())?;
        self.edit(WorkflowEdit::Markers { edit })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn share_playhead(&self, frame: f64, playing: bool, rate: f64, in_point: f64, out_point: f64) {
        let (fps, duration) = self.state().editor().film.as_ref().map_or((30, 1), |film| {
            (film.flat.fps, film.flat.duration_in_frames)
        });
        let playhead = opsis_types::shell::Playhead {
            frame,
            playing,
            rate,
            fps,
            duration_in_frames: duration,
            in_point: (in_point >= 0.0).then_some(in_point),
            out_point: (out_point >= 0.0).then_some(out_point),
        };
        if let Err(error) = shared::share_playhead(&self.session, MAIN_WINDOW, playhead) {
            log::debug!("sharing the playhead: {}", error.0);
        }
    }

    fn timeline_removal(&self) -> RemovalView {
        let selected = shared::shared_state(&self.session).selection.selected_ids;
        document::timeline_removal(&self.session, &selected).map_or_else(
            || RemovalView {
                node_ids: Vec::new(),
                edge_ids: Vec::new(),
            },
            |removal| RemovalView {
                node_ids: removal.node_ids,
                edge_ids: removal.edge_ids,
            },
        )
    }

    fn ruler_step(&self, px_per_frame: f64, fps: u32) -> RulerStep {
        let (step, minor) = view::ruler_step(px_per_frame, fps, view::MIN_STEP_PX);
        RulerStep { step, minor }
    }

    fn format_time(&self, frame: f64, fps: u32, timecode: bool) -> String {
        view::format_time(frame, fps, timecode)
    }

    fn snap_frame(&self, frame: f64, targets: Vec<f64>, threshold: f64) -> SnapView {
        let snapped = view::snap_frame(frame, &snap_targets(&targets), threshold);
        SnapView {
            value: snapped.frame,
            snapped: snapped.target.is_some(),
        }
    }

    fn snap_span(
        &self,
        from: f64,
        to: f64,
        delta: f64,
        targets: Vec<f64>,
        threshold: f64,
    ) -> SnapView {
        let snapped = view::snap_span(from, to, delta, &snap_targets(&targets), threshold);
        SnapView {
            value: snapped.delta,
            snapped: snapped.target.is_some(),
        }
    }

    fn wrap_playhead(&self, position: f64, in_point: f64, out_point: f64, duration: f64) -> f64 {
        let range = view::loop_of(
            (in_point >= 0.0).then_some(in_point),
            (out_point >= 0.0).then_some(out_point),
            duration,
        );
        view::wrap_in_loop(position, range)
    }

    fn edit_keyframes(&self, edit_json: &str) -> HostResult<()> {
        let edit: opsis_types::edits::KeyframeEdit =
            serde_json::from_str(edit_json).map_err(|e| e.to_string())?;
        self.edit(WorkflowEdit::Keyframes { edit })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn box_kept(&self, area: BoundsView) -> bool {
        colors::kept(area)
    }

    fn mark_moved(&self, points: MarkPointsView, dx: f64, dy: f64) -> MarkPointsView {
        colors::mark_shifted(points, dx, dy)
    }

    fn mark_end(&self, points: MarkPointsView, second: bool, x: f64, y: f64) -> MarkPointsView {
        colors::mark_end(points, second, x, y)
    }

    fn mark_drawn(&self, kind: &str, sx: f64, sy: f64, ex: f64, ey: f64) -> MarkDrawnView {
        colors::mark_drawn(kind, sx, sy, ex, ey)
    }

    fn file_path(&self, reference: &str) -> String {
        let Some(dir) = self.state().editor().dir() else {
            return String::new();
        };
        let public = self.state().paths.public_dir.clone();
        opsis_project::paths::ref_to_path(&dir, &public, reference)
            .map_or_default(|path| path.to_string_lossy().into_owned())
    }

    fn import_files(&self, paths: Vec<String>) -> HostResult<Vec<String>> {
        let dir = self
            .state()
            .editor()
            .dir()
            .ok_or_else(|| "No project is open.".to_owned())?;
        self.runtime
            .block_on(projects::import_assets(
                self.session.clone(),
                dir.to_string_lossy().into_owned(),
                Some(paths),
            ))
            .map_err(|e| format!("Couldn't add the file: {}", e.0))
    }

    fn edit_gradient(&self, css: &str, edit_json: &str) -> GradientInfo {
        colors::edit(css, edit_json)
    }

    fn set_params(&self, id: &str, patch_json: &str) -> HostResult<()> {
        let params: Map<String, Value> =
            serde_json::from_str(patch_json).map_err(|e| e.to_string())?;
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

    fn set_skipped(&self, id: &str, skipped: bool) -> HostResult<()> {
        self.edit(WorkflowEdit::UpdateNode {
            id: id.to_owned(),
            patch: NodePatch {
                disabled: Some(skipped),
                ..NodePatch::default()
            },
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn run_node(&self, id: &str, onwards: bool) -> HostResult<()> {
        let targets = if onwards {
            RunTargets {
                target_node_ids: None,
                from_node_id: Some(id.to_owned()),
            }
        } else {
            RunTargets {
                target_node_ids: Some(vec![id.to_owned()]),
                from_node_id: None,
            }
        };
        self.runtime
            .block_on(run::run_workflow(self.session.clone(), targets))
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn ungroup(&self, id: &str) -> HostResult<()> {
        self.edit(WorkflowEdit::Ungroup { id: id.to_owned() })
            .map(|_| ())
            .map_err(|e| e.0)
    }

    fn delete_node(&self, id: &str) -> HostResult<()> {
        Host::select(self, Vec::new());
        self.edit(WorkflowEdit::RemoveNodes {
            ids: vec![id.to_owned()],
        })
        .map(|_| ())
        .map_err(|e| e.0)
    }

    fn reveal_file(&self, path: &str) -> HostResult<()> {
        let Some(dir) = self.state().editor().dir() else {
            return Ok(());
        };
        settings::reveal_in_folder(&self.session, &dir.to_string_lossy(), Some(path.to_owned()))
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
        let api = opsis_session::api::api_status(&self.session);
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
            trace_retention: plain(&settings.traces.retention),
            trace_retentions: choices(&book.trace_retention),
            keep_last: settings.traces.keep_last,
            keep_last_min: book.keep_last_traces.min,
            keep_last_max: book.keep_last_traces.max,
            live_fps: settings.live.fps,
            live_fps_min: book.live_fps.min,
            live_fps_max: book.live_fps.max,
            live_auto_open: settings.live.auto_open,
            api_listening: api.listening,
            api_error: api.error.unwrap_or_default(),
            api_address: api.address,
            setup_skipped: settings.app.setup_skipped,
            default_dir_fallback: if settings.projects.default_dir.is_empty() {
                projects::project_locations(&self.session).default_dir
            } else {
                String::new()
            },
        }
    }

    fn engine_view(&self) -> EngineView {
        let status = opsis_session::engine::get_engine_status(&self.session);
        let install = self.engine_install.lock().expect("engine install");
        EngineView {
            complete: status.complete,
            checking: status.browser.checking,
            path: status.browser.path.unwrap_or_default(),
            version: status.browser.version.unwrap_or_default(),
            problem: status.browser.problem.unwrap_or_default(),
            warning: status.browser.warning.unwrap_or_default(),
            download_dir: status.download_dir,
            installing: install.installing,
            progress: install.progress.unwrap_or(-1.0),
            message: install.message.clone(),
            error: install.error.clone(),
            choosing: install.choosing,
            browser_error: install.browser_error.clone(),
        }
    }

    fn install_engine(&self) {
        {
            let mut install = self.engine_install.lock().expect("engine install");
            if install.installing {
                return;
            }
            *install = EngineInstall {
                installing: true,
                ..EngineInstall::default()
            };
        }
        opsis_qt::post_event(EventKind::Engine);
        let (session, install) = (self.session.clone(), Arc::clone(&self.engine_install));
        self.runtime.spawn(async move {
            let outcome = opsis_session::engine::install_engine(session).await;
            let mut state = install.lock().expect("engine install");
            state.installing = false;
            if let Err(error) = outcome {
                state.error = error.0;
            }
            drop(state);
            opsis_qt::post_event(EventKind::Engine);
        });
    }

    fn choose_browser(&self) {
        {
            let mut install = self.engine_install.lock().expect("engine install");
            install.choosing = true;
            install.browser_error.clear();
        }
        opsis_qt::post_event(EventKind::Engine);
        let (session, install) = (self.session.clone(), Arc::clone(&self.engine_install));
        self.runtime.spawn(async move {
            let outcome = opsis_session::engine::choose_browser(session).await;
            let mut state = install.lock().expect("engine install");
            state.choosing = false;
            if let Err(error) = outcome {
                state.browser_error = error.0;
            }
            drop(state);
            opsis_qt::post_event(EventKind::Engine);
        });
    }

    fn about(&self) -> AboutView {
        let env = settings::get_environment(&self.session);
        AboutView {
            app: env.versions.app,
            development: !env.is_packaged,
            shell: env.versions.tauri,
            platform: format!("{} / {}", env.platform, env.arch),
            browser: env
                .engine
                .browser
                .path
                .unwrap_or_else(|| "Not set up".to_owned()),
            downloads: env.engine.download_dir,
            user_data: env.user_data,
        }
    }

    fn licences(&self) -> Vec<LicenceView> {
        settings::list_licences(&self.session).map_or_default(|entries| {
            entries
                .into_iter()
                .map(|entry| LicenceView {
                    title: entry.title,
                    summary: entry.summary,
                    badge: String::new(),
                    text: entry.text.unwrap_or_default(),
                    file: entry.file.unwrap_or_default(),
                })
                .collect()
        })
    }

    fn font_licences(&self, library: bool) -> Vec<LicenceView> {
        let defs: Vec<opsis_types::fonts::FontFamilyDef> = if library {
            self.runtime
                .block_on(opsis_session::fonts::list_fonts(&self.session))
                .map_or_default(|choices| choices.into_iter().map(|c| c.def).collect())
        } else {
            opsis_compile::fonts::bundled_fonts().to_vec()
        };
        defs.into_iter()
            .filter(|def| !def.hidden)
            .map(|def| LicenceView {
                title: def.family.clone(),
                summary: def
                    .license
                    .clone()
                    .unwrap_or_else(|| "Licence not listed".to_owned()),
                badge: if matches!(def.source, opsis_types::fonts::FontSource::Online) {
                    "Downloaded".to_owned()
                } else {
                    String::new()
                },
                text: self.licence_text(def.license_file.as_deref()),
                file: String::new(),
            })
            .collect()
    }

    fn open_licence_file(&self, file: &str) -> HostResult<()> {
        settings::open_licence_file(&self.session, file).map_err(|e| e.0)
    }

    fn font_library(&self) -> Vec<FontRowView> {
        self.runtime
            .block_on(opsis_session::fonts::list_fonts(&self.session))
            .map_or_default(|choices| choices.iter().map(|c| self.font_row(&c.def)).collect())
    }

    fn remove_font(&self, id: &str) -> HostResult<()> {
        self.runtime
            .block_on(opsis_session::fonts::remove_font(
                &self.session,
                id.to_owned(),
            ))
            .map_err(|e| e.0)
    }

    fn add_font_files(&self) {
        let (session, work) = (self.session.clone(), Arc::clone(&self.font_work));
        self.runtime.spawn(async move {
            match opsis_session::fonts::inspect_font_files(session, None).await {
                Ok(Some(fonts)) if fonts.families.is_empty() => {
                    opsis_qt::notify("No fonts found in those files.", "");
                }
                Ok(Some(fonts)) => {
                    work.lock().expect("font work").review = Some((fonts, false));
                    opsis_qt::post_event(EventKind::Fonts);
                }
                Ok(None) => {}
                Err(error) => opsis_qt::notify(&error.0, ""),
            }
        });
    }

    fn installed_fonts(&self) -> Vec<InstalledFontView> {
        let mut families = self
            .runtime
            .block_on(opsis_session::fonts::installed_font_families())
            .unwrap_or_default();
        families.sort_by_key(|f| f.family.to_lowercase());
        families
            .into_iter()
            .map(|f| InstalledFontView {
                styles: opsis_compile::text::plural(f64::from(f.faces), "style"),
                family: f.family,
            })
            .collect()
    }

    fn inspect_installed(&self, family: &str) {
        let (session, work, family) = (
            self.session.clone(),
            Arc::clone(&self.font_work),
            family.to_owned(),
        );
        self.runtime.spawn(async move {
            match opsis_session::fonts::inspect_installed_fonts(&session, vec![family]).await {
                Ok(fonts) => {
                    work.lock().expect("font work").review = Some((fonts, true));
                    opsis_qt::post_event(EventKind::Fonts);
                }
                Err(error) => opsis_qt::notify(&error.0, ""),
            }
        });
    }

    fn font_review(&self) -> FontReviewView {
        let work = self.font_work.lock().expect("font work");
        let Some((fonts, installed)) = work.review.as_ref() else {
            return FontReviewView {
                shown: false,
                token: String::new(),
                summary: String::new(),
                note: String::new(),
                families: Vec::new(),
            };
        };
        FontReviewView {
            shown: true,
            token: fonts.token.clone(),
            summary: font_views::found_text(fonts.families.len(), fonts.skipped, fonts.unreadable),
            note: font_views::licence_note(fonts.license.as_deref(), *installed),
            families: fonts
                .families
                .iter()
                .map(|f| FontRowView {
                    id: f.family.clone(),
                    family: f.family.clone(),
                    faces: font_views::describe_faces(f.faces.iter().map(|c| {
                        (
                            c.weight.as_str(),
                            matches!(c.style, opsis_types::fonts::FontStyle::Italic),
                        )
                    })),
                    source: String::new(),
                    licensed: fonts.license.is_some(),
                    sample: String::new(),
                })
                .collect(),
        }
    }

    fn add_reviewed(&self, families: Vec<String>) -> HostResult<String> {
        let token = self
            .font_work
            .lock()
            .expect("font work")
            .review
            .as_ref()
            .map(|(fonts, _)| fonts.token.clone())
            .ok_or_else(|| "There are no fonts to add.".to_owned())?;
        let added = self
            .runtime
            .block_on(opsis_session::fonts::add_inspected_fonts(
                &self.session,
                token,
                families,
            ))
            .map_err(|e| e.0)?;
        self.drop_review();
        let names: Vec<String> = added.into_iter().map(|d| d.family).collect();
        Ok(font_views::added_text(&names))
    }

    fn drop_review(&self) {
        let review = self.font_work.lock().expect("font work").review.take();
        if let Some((fonts, _)) = review {
            opsis_session::fonts::release_inspected_fonts(&self.session, &fonts.token);
        }
        opsis_qt::post_event(EventKind::Fonts);
    }

    fn load_font_catalog(&self) {
        self.font_work.lock().expect("font work").catalog = None;
        opsis_qt::post_event(EventKind::Fonts);
        self.fetch_catalog();
    }

    fn font_catalog(&self) -> FontCatalogView {
        let work = self.font_work.lock().expect("font work");
        let (status, error, entries) = match &work.catalog {
            None => ("loading", String::new(), Vec::new()),
            Some(Err(error)) => ("error", error.clone(), Vec::new()),
            Some(Ok(entries)) => (
                "ready",
                String::new(),
                entries
                    .iter()
                    .map(|e| CatalogRowView {
                        id: e.id.clone(),
                        family: e.family.clone(),
                        detail: font_views::catalog_detail(e),
                        added: e.added,
                    })
                    .collect(),
            ),
        };
        FontCatalogView {
            status: status.to_owned(),
            error,
            busy: work.downloading.clone(),
            entries,
        }
    }

    fn font_choices(&self, value: &str) -> FontChoicesView {
        let project = self
            .workflow()
            .map_or_default(|wf| wf.settings.fonts.families);
        let choices = self
            .runtime
            .block_on(opsis_session::fonts::font_choices(
                &self.session,
                value.to_owned(),
                project,
            ))
            .ok();
        let picks = |list: &[opsis_types::fonts::FontChoice]| -> Vec<FontPickView> {
            list.iter()
                .map(|c| FontPickView {
                    id: c.def.id.clone(),
                    family: c.def.family.clone(),
                    sample: self.sample_file(&c.def),
                    selected: c.selected,
                })
                .collect()
        };
        choices.map_or_else(
            || FontChoicesView {
                project: Vec::new(),
                built_in: Vec::new(),
                library: Vec::new(),
                current_sample: String::new(),
            },
            |c| FontChoicesView {
                project: picks(&c.project),
                built_in: picks(&c.built_in),
                library: picks(&c.library),
                current_sample: c
                    .current
                    .as_ref()
                    .map_or_default(|current| self.sample_file(&current.def)),
            },
        )
    }

    fn use_library_font(&self, id: &str) -> HostResult<String> {
        let dir = self
            .state()
            .editor()
            .dir()
            .ok_or_else(|| "Open a project to use this font.".to_owned())?;
        let copied = self
            .runtime
            .block_on(opsis_session::fonts::use_font_in_project(
                &self.session,
                dir.to_string_lossy().into_owned(),
                id.to_owned(),
            ))
            .map_err(|e| e.0)?;
        let family = copied.family.clone();
        self.edit(WorkflowEdit::AddProjectFont { font: copied })
            .map_err(|e| e.0)?;
        Ok(family)
    }

    fn download_font(&self, id: &str) {
        {
            let mut work = self.font_work.lock().expect("font work");
            if !work.downloading.is_empty() {
                return;
            }
            id.clone_into(&mut work.downloading);
        }
        opsis_qt::post_event(EventKind::Fonts);
        let (session, work, id) = (
            self.session.clone(),
            Arc::clone(&self.font_work),
            id.to_owned(),
        );
        self.runtime.spawn(async move {
            match opsis_session::fonts::download_font(&session, id).await {
                Ok(def) if def.license_file.is_some() => {
                    opsis_qt::notify(&format!("Added {}", def.family), "");
                }
                Ok(def) => opsis_qt::notify(
                    &format!("Added {}, but its licence file didn't download", def.family),
                    "Check that you're allowed to use this font in videos.",
                ),
                Err(error) => opsis_qt::notify(&error.0, ""),
            }
            let outcome = opsis_session::fonts::font_catalog(&session)
                .await
                .map_err(|e| e.0);
            let mut state = work.lock().expect("font work");
            state.downloading.clear();
            state.catalog = Some(outcome);
            drop(state);
            opsis_qt::post_event(EventKind::Fonts);
        });
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
            MenuCommand::EditLayout => self.tidy_layout(),
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
