use crate::host::Session;

#[cxx::bridge(namespace = "opsis::bridge")]
pub mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum EventKind {
        Document,
        Selection,
        Settings,
        Project,
        Saved,
        CloseAsked,
        FlushDrafts,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MenuItemKind {
        Command,
        Role,
        Separator,
    }

    #[derive(Debug, Clone)]
    struct MenuItemView {
        kind: MenuItemKind,
        id: String,
        label: String,
        shortcut: String,
    }

    #[derive(Debug, Clone)]
    struct MenuSectionView {
        label: String,
        items: Vec<MenuItemView>,
    }

    #[derive(Debug, Clone)]
    struct CloseView {
        ask: bool,
        busy: Vec<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DialogKind {
        Folder,
        File,
        Files,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum CardKind {
        Card,
        Group,
        Note,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FieldKind {
        Text,
        Textarea,
        Number,
        Switch,
        Choice,
        Other,
    }

    #[derive(Debug, Clone)]
    struct Choice {
        value: String,
        label: String,
    }

    #[derive(Debug, Clone)]
    struct ProjectEntry {
        name: String,
        dir: String,
        detail: String,
    }

    #[derive(Debug, Clone, Copy)]
    struct ScreenArea {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    }

    #[derive(Debug, Clone, Copy)]
    struct WindowPlacement {
        known: bool,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        maximized: bool,
    }

    #[derive(Debug, Clone)]
    struct TemplateView {
        id: String,
        title: String,
        description: String,
        icon: String,
    }

    #[derive(Debug, Clone)]
    struct PortView {
        id: String,
        label: String,
        kind: String,
        color: String,
        optional: bool,
        y: f64,
    }

    #[derive(Debug, Clone)]
    struct RowView {
        icon: String,
        text: String,
        muted: bool,
    }

    #[derive(Debug, Clone)]
    struct CardView {
        id: String,
        node_type: String,
        kind: CardKind,
        title: String,
        category: String,
        color: String,
        glyph: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        inputs: Vec<PortView>,
        outputs: Vec<PortView>,
        rows: Vec<RowView>,
        disabled: bool,
        problem: String,
        text: String,
    }

    #[derive(Debug, Clone)]
    struct EdgeView {
        id: String,
        source: String,
        source_handle: String,
        target: String,
        target_handle: String,
        color: String,
        step: f64,
    }

    #[derive(Debug, Clone)]
    struct GraphView {
        name: String,
        dir: String,
        nodes: Vec<CardView>,
        edges: Vec<EdgeView>,
        revision: u64,
        can_undo: bool,
        can_redo: bool,
        dirty: bool,
    }

    #[derive(Debug, Clone)]
    struct CategoryView {
        id: String,
        label: String,
        color: String,
        icon: String,
    }

    #[derive(Debug, Clone)]
    struct PaletteEntry {
        node_type: String,
        category_id: String,
        title: String,
        category: String,
        color: String,
        glyph: String,
        description: String,
    }

    #[derive(Debug, Clone)]
    struct NodeMove {
        id: String,
        x: f64,
        y: f64,
    }

    #[derive(Debug, Clone)]
    struct FieldView {
        key: String,
        label: String,
        help: String,
        kind: FieldKind,
        text: String,
        number: f64,
        minimum: f64,
        maximum: f64,
        step: f64,
        decimals: u32,
        unit: String,
        zero_label: String,
        scale: f64,
        checked: bool,
        choices: Vec<Choice>,
        advanced: bool,
    }

    #[derive(Debug, Clone)]
    struct InspectorView {
        id: String,
        title: String,
        label: String,
        category: String,
        color: String,
        glyph: String,
        description: String,
        fields: Vec<FieldView>,
    }

    #[derive(Debug, Clone)]
    struct SettingsView {
        on_start: String,
        start_options: Vec<Choice>,
        confirm_exit: bool,
        encode_speed: String,
        encode_speeds: Vec<Choice>,
        default_dir: String,
        canvas_mode: String,
        canvas_modes: Vec<Choice>,
        api_enabled: bool,
        data_dir: String,
    }

    extern "Rust" {
        type Session;

        fn start_pump(self: &Session);
        fn style_sheet(self: &Session) -> String;
        fn token(self: &Session, name: &str) -> String;
        fn icon_svg(self: &Session, name: &str, color: &str, stroke: f64) -> String;
        fn fonts_dir(self: &Session) -> String;
        fn log(self: &Session, line: &str);
        fn answer_dialog(self: &Session, id: u64, paths: Vec<String>);
        fn set_active(self: &Session, active: bool);

        fn examples(self: &Session) -> Vec<ProjectEntry>;
        fn recent(self: &Session) -> Vec<ProjectEntry>;
        fn open_project(self: &Session, dir: &str) -> Result<()>;
        fn open_example(self: &Session, dir: &str) -> Result<()>;
        fn create_project(
            self: &Session,
            name: &str,
            template_id: &str,
            location: &str,
        ) -> Result<()>;
        fn templates(self: &Session) -> Vec<TemplateView>;
        fn engine_missing(self: &Session) -> bool;
        fn project_location(self: &Session) -> String;
        fn project_folder(self: &Session, parent: &str, name: &str) -> String;
        fn generate_project(self: &Session, count: u32) -> Result<()>;
        fn close_project(self: &Session);
        fn has_project(self: &Session) -> bool;

        fn graph(self: &Session) -> GraphView;
        fn palette(self: &Session) -> Vec<PaletteEntry>;
        fn categories(self: &Session) -> Vec<CategoryView>;
        fn begin_gesture(self: &Session);
        fn end_gesture(self: &Session);
        fn move_nodes(self: &Session, moves: Vec<NodeMove>) -> Result<()>;
        fn connection_problem(
            self: &Session,
            source: &str,
            source_handle: &str,
            target: &str,
            target_handle: &str,
        ) -> String;
        fn connect(
            self: &Session,
            source: &str,
            source_handle: &str,
            target: &str,
            target_handle: &str,
        ) -> Result<()>;
        fn remove(self: &Session, nodes: Vec<String>, edges: Vec<String>) -> Result<()>;
        fn add_node(self: &Session, node_type: &str, x: f64, y: f64) -> Result<String>;
        fn undo(self: &Session) -> bool;
        fn redo(self: &Session) -> bool;
        fn select(self: &Session, ids: Vec<String>);
        fn selected(self: &Session) -> String;
        fn inspector(self: &Session) -> InspectorView;
        fn set_param(self: &Session, id: &str, key: &str, json: &str) -> Result<()>;
        fn rename_node(self: &Session, id: &str, label: &str) -> Result<()>;
        fn save(self: &Session) -> Result<()>;

        fn settings(self: &Session) -> SettingsView;
        fn patch_settings(self: &Session, json: &str) -> Result<()>;
        fn menu(self: &Session) -> Vec<MenuSectionView>;
        fn menu_command(self: &Session, id: &str) -> Result<()>;
        fn close_allowed(self: &Session) -> bool;
        fn request_close(self: &Session);
        fn close_view(self: &Session) -> CloseView;
        fn close_shown(self: &Session);
        fn decide_close(self: &Session, close: bool, dont_ask: bool);
        fn report_drafts(self: &Session, held: bool);
        fn save_flushed(self: &Session);
        fn project_window(self: &Session, areas: Vec<ScreenArea>) -> WindowPlacement;
        fn save_window(self: &Session, placement: WindowPlacement);
        fn load_layout(self: &Session) -> String;
        fn save_layout(self: &Session, layout: &str);
    }

    unsafe extern "C++" {
        include!("opsis-qt/cpp/bridge_ffi.h");

        fn run_app(session: Box<Session>) -> i32;
        fn post_event(kind: EventKind);
        fn request_dialog(
            kind: DialogKind,
            id: u64,
            title: &str,
            start: &str,
            filter_name: &str,
            filter_extensions: &str,
        );
        fn open_path(path: &str);
        fn reveal_path(path: &str);
        fn notify(title: &str, body: &str);
        fn raise_editor();
        fn quit_app(code: i32);
        fn qt_version() -> String;
    }
}
