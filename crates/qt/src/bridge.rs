use crate::frame::FrameRef;
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
        Media,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MixerStripView {
        id: String,
        name: String,
        kind: String,
        bus: String,
        audible: bool,
        selected: bool,
        built_in: bool,
        members: u32,
        settings_json: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MixerView {
        shown: bool,
        stale: bool,
        failed: bool,
        error: String,
        solo: bool,
        source_id: String,
        channels: Vec<MixerStripView>,
        buses: Vec<MixerStripView>,
        master: MixerStripView,
        book_json: String,
    }

    #[derive(Debug, Clone, Copy)]
    struct MeterView {
        has: bool,
        peak: f64,
        rms: f64,
        hold: f64,
        reduction: f64,
    }

    #[derive(Debug, Clone, Copy)]
    struct LoudnessView {
        momentary: f64,
        short_term: f64,
        integrated: f64,
        true_peak: f64,
        range: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct RunRowView {
        id: String,
        name: String,
        state: String,
        progress: f64,
        selected: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RunLogView {
        id: u64,
        at: i64,
        level: String,
        message: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ArtifactView {
        kind: String,
        path: String,
        file: String,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct ReplayActionView {
        at: f64,
        time: String,
        kind: String,
        label: String,
        selector: String,
        node_id: String,
    }

    #[derive(Debug, Clone)]
    struct ReplayView {
        shown: bool,
        node_id: String,
        title: String,
        description: String,
        tab: String,
        alert: String,
        video_file: String,
        image_file: String,
        reveal: String,
        trace_id: String,
        can_replay: bool,
        actions: Vec<ReplayActionView>,
    }

    #[derive(Debug, Clone)]
    struct TraceActionView {
        id: String,
        depth: u32,
        group: bool,
        failed: bool,
        title: String,
        target: String,
        node_id: String,
        start: f64,
        end: f64,
        at: String,
        length: String,
        error: String,
        logs: Vec<String>,
        has_point: bool,
        point_x: f64,
        point_y: f64,
        has_box: bool,
        box_x: f64,
        box_y: f64,
        box_width: f64,
        box_height: f64,
        before: i32,
        after: i32,
    }

    #[derive(Debug, Clone)]
    struct TraceFrameView {
        file: String,
        width: u32,
        height: u32,
    }

    #[derive(Debug, Clone)]
    struct TraceConsoleView {
        time: f64,
        at: String,
        level: String,
        text: String,
    }

    #[derive(Debug, Clone)]
    struct TraceRequestView {
        start: f64,
        at: String,
        method: String,
        status: String,
        failed: bool,
        url: String,
        kind: String,
        size: String,
        length: String,
    }

    #[derive(Debug, Clone)]
    struct TraceErrorView {
        at: String,
        text: String,
        action: i32,
    }

    #[derive(Debug, Clone)]
    struct TraceView {
        error: String,
        label: String,
        width: u32,
        height: u32,
        picked: i32,
        warnings: u32,
        actions: Vec<TraceActionView>,
        frames: Vec<TraceFrameView>,
        console: Vec<TraceConsoleView>,
        network: Vec<TraceRequestView>,
        errors: Vec<TraceErrorView>,
    }

    #[derive(Debug, Clone)]
    struct PlayerView {
        ready: bool,
        failed: bool,
        has_video: bool,
        has_audio: bool,
        sound_ready: bool,
        playing: bool,
        duration: f64,
        position: f64,
        width: u32,
        height: u32,
        serial: u64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct RunPanelView {
        running: bool,
        title: String,
        failure: String,
        failed: Vec<Choice>,
        skipped: u32,
        rows: Vec<RunRowView>,
        logs: Vec<RunLogView>,
        filtered: bool,
        filter_id: String,
        filter_type: String,
        artifacts: Vec<ArtifactView>,
    }

    #[derive(Debug, Clone, Copy)]
    struct KeyPointView {
        frame: f64,
        fraction: f64,
        value: f64,
        index: u32,
    }

    #[derive(Debug, Clone)]
    struct KeyLineView {
        shown: bool,
        node_id: String,
        label: String,
        span: f64,
        scale: f64,
        points: Vec<KeyPointView>,
        line: Vec<KeyPointView>,
        volume: f64,
    }

    #[derive(Debug, Clone, Copy)]
    struct FilmTile {
        x: f64,
        width: f64,
        index: u32,
    }

    #[derive(Debug, Clone)]
    struct FilmTiles {
        image: String,
        tile_width: u32,
        tile_height: u32,
        still: bool,
        tiles: Vec<FilmTile>,
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
        Gradient,
        Theme,
        Asset,
        Target,
        Offsets,
        Spot,
        Boxes,
        Marks,
        Envelope,
        Camera,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineBarView {
        key: String,
        node_id: String,
        lane: u32,
        from: f64,
        to: f64,
        title: String,
        wrappers: String,
        tone: String,
        selected: bool,
        related: bool,
        recording: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineAudioView {
        key: String,
        node_id: String,
        lane: u32,
        from: f64,
        to: f64,
        title: String,
        tone: String,
        volume: f64,
        selected: bool,
        hits: bool,
        bar_key: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineMarkView {
        key: String,
        frame: f64,
        label: String,
        chapter: bool,
        bar_key: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineMarkerView {
        id: String,
        frame: f64,
        label: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineTransitionView {
        key: String,
        from: f64,
        to: f64,
        lanes: Vec<u32>,
        label: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineCameraKeyView {
        key: String,
        node_id: String,
        bar_key: String,
        frame: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TimelineView {
        has_film: bool,
        fps: u32,
        duration: f64,
        lanes: u32,
        audio_lanes: u32,
        source_id: String,
        source_label: String,
        bars: Vec<TimelineBarView>,
        audio: Vec<TimelineAudioView>,
        marks: Vec<TimelineMarkView>,
        markers: Vec<TimelineMarkerView>,
        transitions: Vec<TimelineTransitionView>,
        camera_keys: Vec<TimelineCameraKeyView>,
        video_tracks: Vec<Choice>,
        audio_tracks: Vec<Choice>,
    }

    #[derive(Debug, Clone, Copy)]
    struct SnapView {
        value: f64,
        snapped: bool,
    }

    #[derive(Debug, Clone, Copy)]
    struct RulerStep {
        step: f64,
        minor: f64,
    }

    #[derive(Debug, Clone)]
    struct RemovalView {
        node_ids: Vec<String>,
        edge_ids: Vec<String>,
    }

    #[derive(Debug, Clone, Copy)]
    struct MarkPointsView {
        x: f64,
        y: f64,
        x2: f64,
        y2: f64,
    }

    #[derive(Debug, Clone, Copy)]
    struct MarkDrawnView {
        kept: bool,
        points: MarkPointsView,
    }

    #[derive(Debug, Clone, Copy)]
    struct BoundsView {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    #[derive(Debug, Clone)]
    struct GradientStopView {
        color: String,
        at: f64,
    }

    #[derive(Debug, Clone)]
    struct GradientInfo {
        valid: bool,
        linear: bool,
        angle: f64,
        stops: Vec<GradientStopView>,
        css: String,
        strip: String,
        can_add: bool,
        can_remove: bool,
        selected: i32,
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

    #[derive(Debug, Clone, PartialEq, Eq)]
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

    #[derive(Debug, Clone, PartialEq, Eq)]
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

    #[derive(Debug, Clone, PartialEq)]
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
        extra_json: String,
    }

    #[derive(Debug, Clone, PartialEq)]
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
        files: Vec<ArtifactView>,
        swatches: Vec<String>,
        can_replay: bool,
        has_trace: bool,
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
        type FrameRef;
        fn bytes(self: &FrameRef) -> &[u8];
        fn width(self: &FrameRef) -> u32;
        fn height(self: &FrameRef) -> u32;
        fn stride(self: &FrameRef) -> u32;
        fn serial(self: &FrameRef) -> u64;
        fn is_jpeg(self: &FrameRef) -> bool;

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
        fn gradient_info(self: &Session, css: &str) -> GradientInfo;
        fn import_files(self: &Session, paths: Vec<String>) -> Result<Vec<String>>;
        fn file_path(self: &Session, reference: &str) -> String;
        fn box_from(self: &Session, x0: f64, y0: f64, x1: f64, y1: f64) -> BoundsView;
        fn move_box(self: &Session, origin: BoundsView, dx: f64, dy: f64) -> BoundsView;
        fn box_kept(self: &Session, area: BoundsView) -> bool;
        fn edit_keyframes(self: &Session, edit_json: &str) -> Result<()>;
        fn timeline(self: &Session) -> TimelineView;
        fn run_panel(self: &Session) -> RunPanelView;
        fn clear_run_log(self: &Session);
        fn player_open(self: &Session, file: &str) -> u64;
        fn player_close(self: &Session, id: u64);
        fn player_view(self: &Session, id: u64) -> PlayerView;
        fn player_frame(self: &Session, id: u64) -> Box<FrameRef>;
        fn player_play(self: &Session, id: u64);
        fn player_pause(self: &Session, id: u64);
        fn player_seek(self: &Session, id: u64, seconds: f64);
        fn player_rate(self: &Session, id: u64, rate: f64);
        fn player_volume(self: &Session, id: u64, volume: f64, muted: bool);
        fn player_retry(self: &Session, id: u64);
        fn player_time(self: &Session, seconds: f64, guide: f64) -> String;
        fn show_replay(self: &Session, node_id: &str, tab: &str) -> Result<()>;
        fn replay(self: &Session, node_id: &str, tab: &str) -> ReplayView;
        fn trace(self: &Session, trace_id: &str) -> TraceView;
        fn audio_play(self: &Session, seconds: f64, rate: f64);
        fn audio_pause(self: &Session);
        fn audio_seek(self: &Session, seconds: f64);
        fn audio_clock(self: &Session) -> f64;
        fn audio_ready(self: &Session) -> bool;
        fn audio_scrub(self: &Session, seconds: f64);
        fn preview_mix(
            self: &Session,
            target: &str,
            kind: &str,
            current: f64,
            next: f64,
            band_json: &str,
        );
        fn mixer(self: &Session) -> MixerView;
        fn meter(self: &Session, target: &str, frame: f64) -> MeterView;
        fn loudness_at(self: &Session, frame: f64) -> LoudnessView;
        fn edit_mix(self: &Session, edit_json: &str) -> Result<()>;
        fn eq_curve(self: &Session, eq_json: &str, highpass: f64, steps: u32) -> Vec<f64>;
        fn mix_text(self: &Session, kind: &str, value: f64) -> String;
        fn knob_value(
            self: &Session,
            min: f64,
            max: f64,
            step: f64,
            log: bool,
            fraction: f64,
        ) -> f64;
        fn knob_fraction(self: &Session, min: f64, max: f64, log: bool, value: f64) -> f64;
        fn key_line(self: &Session, bar: &str, track: &str) -> KeyLineView;
        fn bar_local(self: &Session, bar: &str, abs: f64) -> f64;
        fn film_tiles(
            self: &Session,
            bar: &str,
            px_per_frame: f64,
            height: f64,
            offset: f64,
            width: f64,
        ) -> FilmTiles;
        fn wave_columns(
            self: &Session,
            bar: &str,
            px_per_frame: f64,
            offset: f64,
            width: u32,
        ) -> Vec<f32>;
        fn select_pinned(self: &Session, ids: Vec<String>, source: &str);
        fn edit_timeline(self: &Session, edit_json: &str) -> String;
        fn edit_markers(self: &Session, edit_json: &str) -> Result<()>;
        fn share_playhead(
            self: &Session,
            frame: f64,
            playing: bool,
            rate: f64,
            in_point: f64,
            out_point: f64,
        );
        fn timeline_removal(self: &Session) -> RemovalView;
        fn ruler_step(self: &Session, px_per_frame: f64, fps: u32) -> RulerStep;
        fn format_time(self: &Session, frame: f64, fps: u32, timecode: bool) -> String;
        fn snap_frame(self: &Session, frame: f64, targets: Vec<f64>, threshold: f64) -> SnapView;
        fn snap_span(
            self: &Session,
            from: f64,
            to: f64,
            delta: f64,
            targets: Vec<f64>,
            threshold: f64,
        ) -> SnapView;
        fn wrap_playhead(
            self: &Session,
            position: f64,
            in_point: f64,
            out_point: f64,
            duration: f64,
        ) -> f64;
        fn mark_moved(self: &Session, points: MarkPointsView, dx: f64, dy: f64) -> MarkPointsView;
        fn mark_end(
            self: &Session,
            points: MarkPointsView,
            second: bool,
            x: f64,
            y: f64,
        ) -> MarkPointsView;
        fn mark_drawn(
            self: &Session,
            kind: &str,
            sx: f64,
            sy: f64,
            ex: f64,
            ey: f64,
        ) -> MarkDrawnView;
        fn edit_gradient(self: &Session, css: &str, edit_json: &str) -> GradientInfo;
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
        fn wake(target: u64);
        fn post_edited(ids: Vec<String>);
        fn post_replay(node_id: &str, tab: &str);
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
