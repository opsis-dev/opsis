use opsis_types::actions::{Action, ActionKind, ActionLog, Point, Rect};
use opsis_types::recording::{CameraStyle, CameraTuning, FrameTheme, RecordingProps, WindowStyle};

use crate::backdrops::DEFAULT_BACKDROP;
use crate::node::Platform;
use crate::nodes::cursor::{default_cursor_look, default_ripple};

pub const PREVIEW_FILE_WIDTH: f64 = 960.0;
pub const DEFAULT_WINDOW_STYLE: WindowStyle = WindowStyle::Macos;
pub const DEFAULT_FRAME_THEME: FrameTheme = FrameTheme::Light;
pub const DEFAULT_ADDRESS_BAR: bool = true;

pub fn host_window_style(platform: Platform) -> WindowStyle {
    match platform {
        Platform::Mac => WindowStyle::Macos,
        Platform::Windows => WindowStyle::Windows,
        Platform::Linux => WindowStyle::Linux,
    }
}

pub fn recording_speed(props: &RecordingProps) -> f64 {
    if props.speed == 0.0 { 1.0 } else { props.speed }
}

pub fn default_recording_props(src: String, log: ActionLog, name: String) -> RecordingProps {
    RecordingProps {
        src,
        proxy_src: None,
        log,
        name,
        speed: 1.25,
        zoom: true,
        tuning: CameraTuning::default(),
        motion_blur: true,
        style: CameraStyle::Classic,
        chrome: false,
        window_style: None,
        frame_theme: None,
        address_bar: None,
        address: None,
        address_mode: None,
        backdrop: DEFAULT_BACKDROP.id().to_owned(),
        backdrop_chosen: None,
        cursor: true,
        cursor_scale: 1.0,
        cursor_look: default_cursor_look(),
        cursor_feel: None,
        ripple: Some(default_ripple()),
        extra_beats: Vec::new(),
        keystrokes: None,
        annotations: Vec::new(),
        blur_areas: None,
        look: None,
        screen: None,
        window_fit: None,
        stage_ground: None,
        window_radius: None,
        rim_glow: None,
        lift_colours: None,
        shutter_angle: None,
        bleed_blur: None,
        bleed_dim: None,
        frame_font: None,
    }
}

pub const POINTER_KINDS: [ActionKind; 5] = [
    ActionKind::Click,
    ActionKind::Type,
    ActionKind::Hover,
    ActionKind::Focus,
    ActionKind::Drag,
];

const POINT_BOX_W: f64 = 120.0;
const POINT_BOX_H: f64 = 48.0;

pub fn point_rect(point: Point, scale: f64) -> Rect {
    Rect {
        x: point.x - POINT_BOX_W / 2.0 * scale,
        y: point.y - POINT_BOX_H / 2.0 * scale,
        w: POINT_BOX_W * scale,
        h: POINT_BOX_H * scale,
    }
}

pub fn idle_spans(log: &ActionLog, threshold_ms: f64, pad_ms: f64) -> Vec<(f64, f64)> {
    let mut beats: Vec<&Action> = log.actions.iter().collect();
    beats.sort_by(|a, b| a.t.total_cmp(&b.t));
    let mut spans = Vec::new();
    let mut cursor: f64 = 0.0;
    for a in beats {
        let start = a.t_depart.unwrap_or(a.t) - pad_ms;
        if start - cursor >= threshold_ms {
            spans.push((cursor, start));
        }
        cursor = cursor.max(a.t_end.unwrap_or(a.t) + pad_ms);
    }
    let end = log.duration_ms - pad_ms;
    if end - cursor >= threshold_ms {
        spans.push((cursor, end));
    }
    spans
}

pub fn follow_cursor_beats(
    log: &ActionLog,
    scale: f64,
    every_ms: f64,
    node_id: &str,
) -> Vec<Action> {
    let mut beats = Vec::new();
    let mut next = 0.0;
    for s in log.cursor_track.iter().flatten() {
        if s.t < next {
            continue;
        }
        let mut beat = Action::new(
            format!("{node_id}:f{}", beats.len()),
            ActionKind::Focus,
            s.t,
        );
        beat.point = Some(Point { x: s.x, y: s.y });
        beat.cluster = Some(format!("{node_id}:follow"));
        beat.zoom_scale = Some(scale);
        beat.node_id = Some(node_id.to_owned());
        beats.push(beat);
        next = s.t + every_ms;
    }
    beats
}
