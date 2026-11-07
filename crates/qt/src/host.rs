use std::fmt;

use crate::bridge::ffi::{
    self, BarView, BoundsView, CategoryView, CloseView, ColorView, DialogKind, EventKind,
    FilmTiles, GradientInfo, GraphView, InspectorView, KeyLineView, LoudnessView, MarkDrawnView,
    MarkPointsView, MenuSectionView, MeterView, MixerView, MotionInfo, NodeMove, PaletteEntry,
    PoseView, ProjectEntry, ProjectView, RemovalView, RulerStep, ScreenArea, SettingsView,
    SnapView, TemplateView, TextToken, TimelineView, VariableView, WindowPlacement,
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
    fn categories(&self) -> Vec<CategoryView>;
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
    fn set_skipped(&self, id: &str, skipped: bool) -> HostResult<()>;
    fn set_params(&self, id: &str, patch_json: &str) -> HostResult<()>;
    fn run_node(&self, id: &str, onwards: bool) -> HostResult<()>;
    fn ungroup(&self, id: &str) -> HostResult<()>;
    fn delete_node(&self, id: &str) -> HostResult<()>;
    fn reveal_file(&self, path: &str) -> HostResult<()>;
    fn project(&self) -> ProjectView;
    fn variables(&self) -> Vec<VariableView>;
    fn project_edit(&self, json: &str) -> HostResult<()>;
    fn clear_saved_results(&self) -> HostResult<()>;
    fn copy_selection(&self) -> String;
    fn bar(&self) -> BarView;
    fn reveal_outputs(&self) -> HostResult<()>;
    fn motion_info(&self, motion: &str) -> MotionInfo;
    fn motion_pose(&self, motion: &str, elapsed_ms: f64) -> PoseView;
    fn paste_text(&self, text: &str) -> HostResult<()>;
    fn duplicate_selection(&self) -> HostResult<()>;
    fn drop_files(&self, paths: Vec<String>, x: f64, y: f64) -> HostResult<u32>;
    fn resize_node(&self, id: &str, x: f64, y: f64, width: f64, height: f64) -> HostResult<()>;
    fn text_tokens(&self, value: &str, pauses: bool) -> Vec<TextToken>;
    fn join_tokens(&self, tokens: Vec<TextToken>) -> String;
    fn parse_color(&self, text: &str) -> ColorView;
    fn format_color(&self, h: f64, s: f64, v: f64, a: f64, format: &str) -> String;
    fn gradient_info(&self, css: &str) -> GradientInfo;
    fn import_files(&self, paths: Vec<String>) -> HostResult<Vec<String>>;
    fn file_path(&self, reference: &str) -> String;
    fn box_from(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> BoundsView;
    fn move_box(&self, origin: BoundsView, dx: f64, dy: f64) -> BoundsView;
    fn box_kept(&self, area: BoundsView) -> bool;
    fn edit_keyframes(&self, edit_json: &str) -> HostResult<()>;
    fn timeline(&self) -> TimelineView;
    fn audio_play(&self, seconds: f64, rate: f64);
    fn audio_pause(&self);
    fn audio_seek(&self, seconds: f64);
    fn audio_clock(&self) -> f64;
    fn audio_ready(&self) -> bool;
    fn audio_scrub(&self, seconds: f64);
    fn preview_mix(&self, target: &str, kind: &str, current: f64, next: f64, band_json: &str);
    fn mixer(&self) -> MixerView;
    fn meter(&self, target: &str, frame: f64) -> MeterView;
    fn loudness_at(&self, frame: f64) -> LoudnessView;
    fn edit_mix(&self, edit_json: &str) -> HostResult<()>;
    fn eq_curve(&self, eq_json: &str, highpass: f64, steps: u32) -> Vec<f64>;
    fn mix_text(&self, kind: &str, value: f64) -> String;
    fn knob_value(&self, min: f64, max: f64, step: f64, log: bool, fraction: f64) -> f64;
    fn knob_fraction(&self, min: f64, max: f64, log: bool, value: f64) -> f64;
    fn key_line(&self, bar: &str, track: &str) -> KeyLineView;
    fn bar_local(&self, bar: &str, abs: f64) -> f64;
    fn film_tiles(
        &self,
        bar: &str,
        px_per_frame: f64,
        height: f64,
        offset: f64,
        width: f64,
    ) -> FilmTiles;
    fn wave_columns(&self, bar: &str, px_per_frame: f64, offset: f64, width: u32) -> Vec<f32>;
    fn select_pinned(&self, ids: Vec<String>, source: &str);
    fn edit_timeline(&self, edit_json: &str) -> String;
    fn edit_markers(&self, edit_json: &str) -> HostResult<()>;
    fn share_playhead(&self, frame: f64, playing: bool, rate: f64, in_point: f64, out_point: f64);
    fn timeline_removal(&self) -> RemovalView;
    fn ruler_step(&self, px_per_frame: f64, fps: u32) -> RulerStep;
    fn format_time(&self, frame: f64, fps: u32, timecode: bool) -> String;
    fn snap_frame(&self, frame: f64, targets: Vec<f64>, threshold: f64) -> SnapView;
    fn snap_span(
        &self,
        from: f64,
        to: f64,
        delta: f64,
        targets: Vec<f64>,
        threshold: f64,
    ) -> SnapView;
    fn wrap_playhead(&self, position: f64, in_point: f64, out_point: f64, duration: f64) -> f64;
    fn mark_moved(&self, points: MarkPointsView, dx: f64, dy: f64) -> MarkPointsView;
    fn mark_end(&self, points: MarkPointsView, second: bool, x: f64, y: f64) -> MarkPointsView;
    fn mark_drawn(&self, kind: &str, sx: f64, sy: f64, ex: f64, ey: f64) -> MarkDrawnView;
    fn edit_gradient(&self, css: &str, edit_json: &str) -> GradientInfo;
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
    fn project_window(&self, areas: Vec<ScreenArea>) -> WindowPlacement;
    fn save_window(&self, placement: WindowPlacement);
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

    pub(crate) fn categories(&self) -> Vec<CategoryView> {
        self.0.categories()
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

    pub(crate) fn set_skipped(&self, id: &str, skipped: bool) -> HostResult<()> {
        self.0.set_skipped(id, skipped)
    }

    pub(crate) fn set_params(&self, id: &str, patch_json: &str) -> HostResult<()> {
        self.0.set_params(id, patch_json)
    }

    pub(crate) fn run_node(&self, id: &str, onwards: bool) -> HostResult<()> {
        self.0.run_node(id, onwards)
    }

    pub(crate) fn ungroup(&self, id: &str) -> HostResult<()> {
        self.0.ungroup(id)
    }

    pub(crate) fn delete_node(&self, id: &str) -> HostResult<()> {
        self.0.delete_node(id)
    }

    pub(crate) fn reveal_file(&self, path: &str) -> HostResult<()> {
        self.0.reveal_file(path)
    }

    pub(crate) fn project(&self) -> ProjectView {
        self.0.project()
    }

    pub(crate) fn variables(&self) -> Vec<VariableView> {
        self.0.variables()
    }

    pub(crate) fn project_edit(&self, json: &str) -> HostResult<()> {
        self.0.project_edit(json)
    }

    pub(crate) fn clear_saved_results(&self) -> HostResult<()> {
        self.0.clear_saved_results()
    }

    pub(crate) fn copy_selection(&self) -> String {
        self.0.copy_selection()
    }

    pub(crate) fn bar(&self) -> BarView {
        self.0.bar()
    }

    pub(crate) fn reveal_outputs(&self) -> HostResult<()> {
        self.0.reveal_outputs()
    }

    pub(crate) fn motion_info(&self, motion: &str) -> MotionInfo {
        self.0.motion_info(motion)
    }

    pub(crate) fn motion_pose(&self, motion: &str, elapsed_ms: f64) -> PoseView {
        self.0.motion_pose(motion, elapsed_ms)
    }

    pub(crate) fn paste_text(&self, text: &str) -> HostResult<()> {
        self.0.paste_text(text)
    }

    pub(crate) fn duplicate_selection(&self) -> HostResult<()> {
        self.0.duplicate_selection()
    }

    pub(crate) fn drop_files(&self, paths: Vec<String>, x: f64, y: f64) -> HostResult<u32> {
        self.0.drop_files(paths, x, y)
    }

    pub(crate) fn resize_node(
        &self,
        id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> HostResult<()> {
        self.0.resize_node(id, x, y, width, height)
    }

    pub(crate) fn text_tokens(&self, value: &str, pauses: bool) -> Vec<TextToken> {
        self.0.text_tokens(value, pauses)
    }

    pub(crate) fn join_tokens(&self, tokens: Vec<TextToken>) -> String {
        self.0.join_tokens(tokens)
    }

    pub(crate) fn parse_color(&self, text: &str) -> ColorView {
        self.0.parse_color(text)
    }

    pub(crate) fn format_color(&self, h: f64, s: f64, v: f64, a: f64, format: &str) -> String {
        self.0.format_color(h, s, v, a, format)
    }

    pub(crate) fn gradient_info(&self, css: &str) -> GradientInfo {
        self.0.gradient_info(css)
    }

    pub(crate) fn import_files(&self, paths: Vec<String>) -> HostResult<Vec<String>> {
        self.0.import_files(paths)
    }

    pub(crate) fn box_from(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> BoundsView {
        self.0.box_from(x0, y0, x1, y1)
    }

    pub(crate) fn move_box(&self, origin: BoundsView, dx: f64, dy: f64) -> BoundsView {
        self.0.move_box(origin, dx, dy)
    }

    pub(crate) fn edit_keyframes(&self, edit_json: &str) -> HostResult<()> {
        self.0.edit_keyframes(edit_json)
    }

    pub(crate) fn timeline(&self) -> TimelineView {
        self.0.timeline()
    }

    pub(crate) fn audio_play(&self, seconds: f64, rate: f64) {
        self.0.audio_play(seconds, rate);
    }

    pub(crate) fn audio_pause(&self) {
        self.0.audio_pause();
    }

    pub(crate) fn audio_seek(&self, seconds: f64) {
        self.0.audio_seek(seconds);
    }

    pub(crate) fn audio_clock(&self) -> f64 {
        self.0.audio_clock()
    }

    pub(crate) fn audio_ready(&self) -> bool {
        self.0.audio_ready()
    }

    pub(crate) fn audio_scrub(&self, seconds: f64) {
        self.0.audio_scrub(seconds);
    }

    pub(crate) fn preview_mix(
        &self,
        target: &str,
        kind: &str,
        current: f64,
        next: f64,
        band_json: &str,
    ) {
        self.0.preview_mix(target, kind, current, next, band_json);
    }

    pub(crate) fn mixer(&self) -> MixerView {
        self.0.mixer()
    }

    pub(crate) fn meter(&self, target: &str, frame: f64) -> MeterView {
        self.0.meter(target, frame)
    }

    pub(crate) fn loudness_at(&self, frame: f64) -> LoudnessView {
        self.0.loudness_at(frame)
    }

    pub(crate) fn edit_mix(&self, edit_json: &str) -> HostResult<()> {
        self.0.edit_mix(edit_json)
    }

    pub(crate) fn eq_curve(&self, eq_json: &str, highpass: f64, steps: u32) -> Vec<f64> {
        self.0.eq_curve(eq_json, highpass, steps)
    }

    pub(crate) fn mix_text(&self, kind: &str, value: f64) -> String {
        self.0.mix_text(kind, value)
    }

    pub(crate) fn knob_value(
        &self,
        min: f64,
        max: f64,
        step: f64,
        log: bool,
        fraction: f64,
    ) -> f64 {
        self.0.knob_value(min, max, step, log, fraction)
    }

    pub(crate) fn knob_fraction(&self, min: f64, max: f64, log: bool, value: f64) -> f64 {
        self.0.knob_fraction(min, max, log, value)
    }

    pub(crate) fn key_line(&self, bar: &str, track: &str) -> KeyLineView {
        self.0.key_line(bar, track)
    }

    pub(crate) fn bar_local(&self, bar: &str, abs: f64) -> f64 {
        self.0.bar_local(bar, abs)
    }

    pub(crate) fn film_tiles(
        &self,
        bar: &str,
        px_per_frame: f64,
        height: f64,
        offset: f64,
        width: f64,
    ) -> FilmTiles {
        self.0.film_tiles(bar, px_per_frame, height, offset, width)
    }

    pub(crate) fn wave_columns(
        &self,
        bar: &str,
        px_per_frame: f64,
        offset: f64,
        width: u32,
    ) -> Vec<f32> {
        self.0.wave_columns(bar, px_per_frame, offset, width)
    }

    pub(crate) fn select_pinned(&self, ids: Vec<String>, source: &str) {
        self.0.select_pinned(ids, source);
    }

    pub(crate) fn edit_timeline(&self, edit_json: &str) -> String {
        self.0.edit_timeline(edit_json)
    }

    pub(crate) fn edit_markers(&self, edit_json: &str) -> HostResult<()> {
        self.0.edit_markers(edit_json)
    }

    pub(crate) fn share_playhead(
        &self,
        frame: f64,
        playing: bool,
        rate: f64,
        in_point: f64,
        out_point: f64,
    ) {
        self.0
            .share_playhead(frame, playing, rate, in_point, out_point);
    }

    pub(crate) fn timeline_removal(&self) -> RemovalView {
        self.0.timeline_removal()
    }

    pub(crate) fn ruler_step(&self, px_per_frame: f64, fps: u32) -> RulerStep {
        self.0.ruler_step(px_per_frame, fps)
    }

    pub(crate) fn format_time(&self, frame: f64, fps: u32, timecode: bool) -> String {
        self.0.format_time(frame, fps, timecode)
    }

    pub(crate) fn snap_frame(&self, frame: f64, targets: Vec<f64>, threshold: f64) -> SnapView {
        self.0.snap_frame(frame, targets, threshold)
    }

    pub(crate) fn snap_span(
        &self,
        from: f64,
        to: f64,
        delta: f64,
        targets: Vec<f64>,
        threshold: f64,
    ) -> SnapView {
        self.0.snap_span(from, to, delta, targets, threshold)
    }

    pub(crate) fn wrap_playhead(
        &self,
        position: f64,
        in_point: f64,
        out_point: f64,
        duration: f64,
    ) -> f64 {
        self.0
            .wrap_playhead(position, in_point, out_point, duration)
    }

    pub(crate) fn box_kept(&self, area: BoundsView) -> bool {
        self.0.box_kept(area)
    }

    pub(crate) fn mark_moved(&self, points: MarkPointsView, dx: f64, dy: f64) -> MarkPointsView {
        self.0.mark_moved(points, dx, dy)
    }

    pub(crate) fn mark_end(
        &self,
        points: MarkPointsView,
        second: bool,
        x: f64,
        y: f64,
    ) -> MarkPointsView {
        self.0.mark_end(points, second, x, y)
    }

    pub(crate) fn mark_drawn(
        &self,
        kind: &str,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
    ) -> MarkDrawnView {
        self.0.mark_drawn(kind, sx, sy, ex, ey)
    }

    pub(crate) fn file_path(&self, reference: &str) -> String {
        self.0.file_path(reference)
    }

    pub(crate) fn edit_gradient(&self, css: &str, edit_json: &str) -> GradientInfo {
        self.0.edit_gradient(css, edit_json)
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

    pub(crate) fn project_window(&self, areas: Vec<ScreenArea>) -> WindowPlacement {
        self.0.project_window(areas)
    }

    pub(crate) fn save_window(&self, placement: WindowPlacement) {
        self.0.save_window(placement);
    }

    pub(crate) fn load_layout(&self) -> String {
        self.0.load_layout()
    }

    pub(crate) fn save_layout(&self, layout: &str) {
        self.0.save_layout(layout);
    }
}
