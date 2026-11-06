use std::fmt;

use crate::bridge::ffi::{
    self, CloseView, DialogKind, EventKind, GraphView, InspectorView, MenuSectionView, NodeMove,
    PaletteEntry, ProjectEntry, SettingsView, TemplateView,
};

pub type HostResult<T> = Result<T, String>;

pub trait Host {
    fn start_pump(&self);
    fn style_sheet(&self) -> String;
    fn token(&self, name: &str) -> String;
    fn icon_svg(&self, name: &str, color: &str, stroke: f64) -> String;
    fn fonts_dir(&self) -> String;
    fn log(&self, line: &str);
    fn answer_dialog(&self, id: u64, paths: Vec<String>);
    fn set_active(&self, active: bool);

    fn examples(&self) -> Vec<ProjectEntry>;
    fn recent(&self) -> Vec<ProjectEntry>;
    fn open_project(&self, dir: &str) -> HostResult<()>;
    fn open_example(&self, dir: &str) -> HostResult<()>;
    fn create_project(&self, name: &str, template_id: &str, location: &str) -> HostResult<()>;
    fn templates(&self) -> Vec<TemplateView>;
    fn engine_missing(&self) -> bool;
    fn project_location(&self) -> String;
    fn project_folder(&self, parent: &str, name: &str) -> String;
    fn generate_project(&self, count: u32) -> HostResult<()>;
    fn close_project(&self);
    fn has_project(&self) -> bool;

    fn graph(&self) -> GraphView;
    fn palette(&self) -> Vec<PaletteEntry>;
    fn begin_gesture(&self);
    fn end_gesture(&self);
    fn move_nodes(&self, moves: Vec<NodeMove>) -> HostResult<()>;
    fn connection_problem(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> String;
    fn connect(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> HostResult<()>;
    fn remove(&self, nodes: Vec<String>, edges: Vec<String>) -> HostResult<()>;
    fn add_node(&self, node_type: &str, x: f64, y: f64) -> HostResult<String>;
    fn undo(&self) -> bool;
    fn redo(&self) -> bool;
    fn select(&self, ids: Vec<String>);
    fn selected(&self) -> String;
    fn inspector(&self) -> InspectorView;
    fn set_param(&self, id: &str, key: &str, json: &str) -> HostResult<()>;
    fn rename_node(&self, id: &str, label: &str) -> HostResult<()>;
    fn save(&self) -> HostResult<()>;

    fn settings(&self) -> SettingsView;
    fn patch_settings(&self, json: &str) -> HostResult<()>;
    fn menu(&self) -> Vec<MenuSectionView>;
    fn menu_command(&self, id: &str) -> HostResult<()>;
    fn close_allowed(&self) -> bool;
    fn request_close(&self);
    fn close_view(&self) -> CloseView;
    fn close_shown(&self);
    fn decide_close(&self, close: bool, dont_ask: bool);
    fn report_drafts(&self, held: bool);
    fn save_flushed(&self);
    fn load_layout(&self) -> String;
    fn save_layout(&self, layout: &str);
}

pub struct Session(Box<dyn Host>);

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session").finish_non_exhaustive()
    }
}

pub fn run(host: Box<dyn Host>) -> i32 {
    ffi::run_app(Box::new(Session(host)))
}

pub fn post_event(kind: EventKind) {
    ffi::post_event(kind);
}

pub fn request_dialog(
    kind: DialogKind,
    id: u64,
    title: &str,
    start: &str,
    filter: Option<(&str, &str)>,
) {
    let (name, extensions) = filter.unwrap_or_default();
    ffi::request_dialog(kind, id, title, start, name, extensions);
}

pub fn open_path(path: &str) {
    ffi::open_path(path);
}

pub fn reveal_path(path: &str) {
    ffi::reveal_path(path);
}

pub fn notify(title: &str, body: &str) {
    ffi::notify(title, body);
}

pub fn raise_editor() {
    ffi::raise_editor();
}

pub fn quit(code: i32) {
    ffi::quit_app(code);
}

pub fn qt_version() -> String {
    ffi::qt_version()
}

impl Session {
    pub(crate) fn start_pump(&self) {
        self.0.start_pump();
    }

    pub(crate) fn style_sheet(&self) -> String {
        self.0.style_sheet()
    }

    pub(crate) fn token(&self, name: &str) -> String {
        self.0.token(name)
    }

    pub(crate) fn icon_svg(&self, name: &str, color: &str, stroke: f64) -> String {
        self.0.icon_svg(name, color, stroke)
    }

    pub(crate) fn fonts_dir(&self) -> String {
        self.0.fonts_dir()
    }

    pub(crate) fn log(&self, line: &str) {
        self.0.log(line);
    }

    pub(crate) fn answer_dialog(&self, id: u64, paths: Vec<String>) {
        self.0.answer_dialog(id, paths);
    }

    pub(crate) fn set_active(&self, active: bool) {
        self.0.set_active(active);
    }

    pub(crate) fn examples(&self) -> Vec<ProjectEntry> {
        self.0.examples()
    }

    pub(crate) fn recent(&self) -> Vec<ProjectEntry> {
        self.0.recent()
    }

    pub(crate) fn open_project(&self, dir: &str) -> HostResult<()> {
        self.0.open_project(dir)
    }

    pub(crate) fn open_example(&self, dir: &str) -> HostResult<()> {
        self.0.open_example(dir)
    }

    pub(crate) fn create_project(
        &self,
        name: &str,
        template_id: &str,
        location: &str,
    ) -> HostResult<()> {
        self.0.create_project(name, template_id, location)
    }

    pub(crate) fn templates(&self) -> Vec<TemplateView> {
        self.0.templates()
    }

    pub(crate) fn engine_missing(&self) -> bool {
        self.0.engine_missing()
    }

    pub(crate) fn project_location(&self) -> String {
        self.0.project_location()
    }

    pub(crate) fn project_folder(&self, parent: &str, name: &str) -> String {
        self.0.project_folder(parent, name)
    }

    pub(crate) fn generate_project(&self, count: u32) -> HostResult<()> {
        self.0.generate_project(count)
    }

    pub(crate) fn close_project(&self) {
        self.0.close_project();
    }

    pub(crate) fn has_project(&self) -> bool {
        self.0.has_project()
    }

    pub(crate) fn graph(&self) -> GraphView {
        self.0.graph()
    }

    pub(crate) fn palette(&self) -> Vec<PaletteEntry> {
        self.0.palette()
    }

    pub(crate) fn begin_gesture(&self) {
        self.0.begin_gesture();
    }

    pub(crate) fn end_gesture(&self) {
        self.0.end_gesture();
    }

    pub(crate) fn move_nodes(&self, moves: Vec<NodeMove>) -> HostResult<()> {
        self.0.move_nodes(moves)
    }

    pub(crate) fn connection_problem(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> String {
        self.0
            .connection_problem(source, source_handle, target, target_handle)
    }

    pub(crate) fn connect(
        &self,
        source: &str,
        source_handle: &str,
        target: &str,
        target_handle: &str,
    ) -> HostResult<()> {
        self.0.connect(source, source_handle, target, target_handle)
    }

    pub(crate) fn remove(&self, nodes: Vec<String>, edges: Vec<String>) -> HostResult<()> {
        self.0.remove(nodes, edges)
    }

    pub(crate) fn add_node(&self, node_type: &str, x: f64, y: f64) -> HostResult<String> {
        self.0.add_node(node_type, x, y)
    }

    pub(crate) fn undo(&self) -> bool {
        self.0.undo()
    }

    pub(crate) fn redo(&self) -> bool {
        self.0.redo()
    }

    pub(crate) fn select(&self, ids: Vec<String>) {
        self.0.select(ids);
    }

    pub(crate) fn selected(&self) -> String {
        self.0.selected()
    }

    pub(crate) fn inspector(&self) -> InspectorView {
        self.0.inspector()
    }

    pub(crate) fn set_param(&self, id: &str, key: &str, json: &str) -> HostResult<()> {
        self.0.set_param(id, key, json)
    }

    pub(crate) fn rename_node(&self, id: &str, label: &str) -> HostResult<()> {
        self.0.rename_node(id, label)
    }

    pub(crate) fn save(&self) -> HostResult<()> {
        self.0.save()
    }

    pub(crate) fn settings(&self) -> SettingsView {
        self.0.settings()
    }

    pub(crate) fn patch_settings(&self, json: &str) -> HostResult<()> {
        self.0.patch_settings(json)
    }

    pub(crate) fn menu(&self) -> Vec<MenuSectionView> {
        self.0.menu()
    }

    pub(crate) fn menu_command(&self, id: &str) -> HostResult<()> {
        self.0.menu_command(id)
    }

    pub(crate) fn close_allowed(&self) -> bool {
        self.0.close_allowed()
    }

    pub(crate) fn request_close(&self) {
        self.0.request_close();
    }

    pub(crate) fn close_view(&self) -> CloseView {
        self.0.close_view()
    }

    pub(crate) fn close_shown(&self) {
        self.0.close_shown();
    }

    pub(crate) fn decide_close(&self, close: bool, dont_ask: bool) {
        self.0.decide_close(close, dont_ask);
    }

    pub(crate) fn report_drafts(&self, held: bool) {
        self.0.report_drafts(held);
    }

    pub(crate) fn save_flushed(&self) {
        self.0.save_flushed();
    }

    pub(crate) fn load_layout(&self) -> String {
        self.0.load_layout()
    }

    pub(crate) fn save_layout(&self, layout: &str) {
        self.0.save_layout(layout);
    }
}
