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
        Compiled,
        Run,
        Api,
    }

    #[derive(Debug, Clone, Copy)]
    struct BarView {
        api_enabled: bool,
        api_listening: bool,
        running: bool,
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
        Code,
        Other,
        Select,
        EnumList,
        DateTime,
        Timezone,
        Keys,
        PresetNumber,
        Json,
        StringList,
        NumberList,
        ColorList,
        ObjectList,
        Color,
    }

    #[derive(Debug, Clone)]
    struct ColorView {
        valid: bool,
        h: f64,
        s: f64,
        v: f64,
        a: f64,
        format: String,
    }

    #[derive(Debug, Clone)]
    struct PresetView {
        label: String,
        values_json: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TokenKind {
        Text,
        Var,
        Pause,
    }

    #[derive(Debug, Clone)]
    struct TextToken {
        kind: TokenKind,
        text: String,
    }

    #[derive(Debug, Clone, Copy)]
    struct PoseView {
        tx: f64,
        ty: f64,
        sx: f64,
        sy: f64,
        rotate: f64,
        tilt_x: f64,
        tilt_y: f64,
        opacity: f64,
        blur: f64,
        clip_top: f64,
        iris: f64,
        sweep: f64,
        mask: f64,
        dash: f64,
        split_x: f64,
        split_y: f64,
        rgb: f64,
        smear: f64,
    }

    #[derive(Debug, Clone)]
    struct MotionInfo {
        ms: u32,
        origin_x: f64,
        origin_y: f64,
        part: String,
        clip: bool,
        mask: String,
    }

    #[derive(Debug, Clone)]
    struct VariableView {
        name: String,
        value: String,
    }

    #[derive(Debug, Clone)]
    struct IssueView {
        error: bool,
        message: String,
    }

    #[derive(Debug, Clone)]
    struct ShapeView {
        label: String,
        width: u32,
        height: u32,
    }

    #[derive(Debug, Clone)]
    struct ProjectView {
        name: String,
        width: u32,
        height: u32,
        fps: u32,
        background: String,
        background_hint: String,
        shapes: Vec<ShapeView>,
        main_font: String,
        heading_font: String,
        code_font: String,
        fonts: Vec<String>,
        variables: Vec<VariableView>,
        issues: Vec<IssueView>,
        running: bool,
        selected: u32,
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
        parent: String,
        motion: String,
        problem: String,
        warning: String,
        text: String,
        badge: String,
        badge_icon: String,
        badge_tone: String,
        badge_tip: String,
        ring: String,
        progress: f64,
        progress_message: String,
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
        inputs: String,
        outputs: String,
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
        pauses: bool,
        value_json: String,
        none_label: String,
        clearable: bool,
        raw_choices: bool,
        presets: Vec<PresetView>,
        presets_label: String,
        group: String,
        min_items: u32,
        max_items: u32,
        new_item_json: String,
    }

    #[derive(Debug, Clone)]
    struct InspectorView {
        id: String,
        title: String,
        label: String,
        category: String,
        category_id: String,
        color: String,
        glyph: String,
        description: String,
        fields: Vec<FieldView>,
        renamed: bool,
        disabled: bool,
        runs: bool,
        group: bool,
        running: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
        files: Vec<FileView>,
        swatches: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct FileView {
        path: String,
        label: String,
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
        fn set_skipped(self: &Session, id: &str, skipped: bool) -> Result<()>;
        fn set_params(self: &Session, id: &str, patch_json: &str) -> Result<()>;
        fn run_node(self: &Session, id: &str, onwards: bool) -> Result<()>;
        fn ungroup(self: &Session, id: &str) -> Result<()>;
        fn delete_node(self: &Session, id: &str) -> Result<()>;
        fn reveal_file(self: &Session, path: &str) -> Result<()>;
        fn project(self: &Session) -> ProjectView;
        fn variables(self: &Session) -> Vec<VariableView>;
        fn project_edit(self: &Session, json: &str) -> Result<()>;
        fn clear_saved_results(self: &Session) -> Result<()>;
        fn copy_selection(self: &Session) -> String;
        fn bar(self: &Session) -> BarView;
        fn reveal_outputs(self: &Session) -> Result<()>;
        fn motion_info(self: &Session, motion: &str) -> MotionInfo;
        fn motion_pose(self: &Session, motion: &str, elapsed_ms: f64) -> PoseView;
        fn paste_text(self: &Session, text: &str) -> Result<()>;
        fn duplicate_selection(self: &Session) -> Result<()>;
        fn drop_files(self: &Session, paths: Vec<String>, x: f64, y: f64) -> Result<u32>;
        fn resize_node(
            self: &Session,
            id: &str,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        ) -> Result<()>;
        fn text_tokens(self: &Session, value: &str, pauses: bool) -> Vec<TextToken>;
        fn join_tokens(self: &Session, tokens: Vec<TextToken>) -> String;
        fn parse_color(self: &Session, text: &str) -> ColorView;
        fn format_color(self: &Session, h: f64, s: f64, v: f64, a: f64, format: &str) -> String;
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
