use opsis_types::actions::{Action, ActionKind, ActionLog, Point, Rect};
use opsis_types::recording::{
    CameraProps, CameraStyle, CameraTuning, FootageChain, FrameTheme, PointerProps, RecordingProps,
    ScreenProps, ShotProps, WindowProps, WindowStyle,
};

use crate::backdrops::DEFAULT_BACKDROP;
use crate::node::Platform;
use crate::nodes::cursor::{default_cursor_look, default_ripple};

pub const DEFAULT_WINDOW_STYLE: WindowStyle = WindowStyle::Macos;
pub const DEFAULT_FRAME_THEME: FrameTheme = FrameTheme::Light;
pub const DEFAULT_ADDRESS_BAR: bool = true;
pub const DEFAULT_TAKE_SPEED: f64 = 1.25;

pub const fn host_window_style(platform: Platform) -> WindowStyle {
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
        log,
        name,
        speed: DEFAULT_TAKE_SPEED,
        camera: CameraProps {
            zoom: true,
            tuning: CameraTuning::default(),
        },
        shot: ShotProps {
            style: CameraStyle::Classic,
            backdrop: DEFAULT_BACKDROP.id().to_owned(),
            backdrop_chosen: None,
            in_device: None,
            device: None,
            window_fit: None,
            radius: None,
            rim_glow: None,
            lift_colours: None,
            bleed_blur: None,
            bleed_dim: None,
            motion_blur: true,
            shutter_angle: None,
        },
        window: WindowProps {
            chrome: false,
            style: None,
            theme: None,
            font: None,
            bar: None,
            text: None,
            mode: None,
        },
        pointer: PointerProps {
            shown: true,
            scale: 1.0,
            look: default_cursor_look(),
            feel: None,
            ripple: Some(default_ripple()),
        },
        keystrokes: None,
        annotations: Vec::new(),
        extra_beats: Vec::new(),
        screen: ScreenProps::Footage {
            src,
            chain: FootageChain::default(),
            footage_url: None,
        },
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
        x: (POINT_BOX_W / 2.0).mul_add(-scale, point.x),
        y: (POINT_BOX_H / 2.0).mul_add(-scale, point.y),
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

fn followed_track(log: &ActionLog) -> Vec<(f64, Point)> {
    if let Some(track) = &log.cursor_track {
        return track
            .iter()
            .map(|s| (s.t, Point { x: s.x, y: s.y }))
            .collect();
    }
    let Some(grid) = log.cells else {
        return Vec::new();
    };
    log.caret_track
        .iter()
        .map(|s| {
            let point = Point {
                x: grid.x(f64::from(s.col) + CELL_MIDDLE),
                y: grid.y(f64::from(s.row) + CELL_MIDDLE),
            };
            (s.t, point)
        })
        .collect()
}

const CELL_MIDDLE: f64 = 0.5;

pub fn follow_cursor_beats(
    log: &ActionLog,
    scale: f64,
    every_ms: f64,
    node_id: &str,
) -> Vec<Action> {
    let mut beats = Vec::new();
    let mut next = 0.0;
    for (t, point) in followed_track(log) {
        if t < next {
            continue;
        }
        let mut beat = Action::new(format!("{node_id}:f{}", beats.len()), ActionKind::Focus, t);
        beat.point = Some(point);
        beat.cluster = Some(format!("{node_id}:follow"));
        beat.zoom_scale = Some(scale);
        beat.node_id = Some(node_id.to_owned());
        beats.push(beat);
        next = t + every_ms;
    }
    beats
}
