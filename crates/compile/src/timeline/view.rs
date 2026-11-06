use std::collections::BTreeMap;

use crate::time::clamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    Playhead,
    Edge,
    Marker,
    Action,
    Beat,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapTarget {
    pub frame: f64,
    pub kind: SnapKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snapped {
    pub frame: f64,
    pub target: Option<SnapTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnappedSpan {
    pub delta: f64,
    pub target: Option<SnapTarget>,
}

pub fn snap_frame(frame: f64, targets: &[SnapTarget], threshold: f64) -> Snapped {
    let mut best: Option<SnapTarget> = None;
    let mut distance = threshold;
    for target in targets {
        let away = (target.frame - frame).abs();
        if away <= distance {
            distance = away;
            best = Some(*target);
        }
    }
    best.map_or(
        Snapped {
            frame,
            target: None,
        },
        |target| Snapped {
            frame: target.frame,
            target: Some(target),
        },
    )
}

pub fn snap_span(
    from: f64,
    to: f64,
    delta: f64,
    targets: &[SnapTarget],
    threshold: f64,
) -> SnappedSpan {
    let mut best: Option<(f64, SnapTarget, f64)> = None;
    for edge in [from, to] {
        let snapped = snap_frame(edge + delta, targets, threshold);
        let Some(target) = snapped.target else {
            continue;
        };
        let distance = (snapped.frame - (edge + delta)).abs();
        if best.is_none_or(|(_, _, closest)| distance < closest) {
            best = Some((snapped.frame - edge, target, distance));
        }
    }
    best.map_or(
        SnappedSpan {
            delta,
            target: None,
        },
        |(delta, target, _)| SnappedSpan {
            delta,
            target: Some(target),
        },
    )
}

const FRAME_STEPS: [f64; 4] = [1.0, 2.0, 5.0, 10.0];
const FRAME_TICKS: f64 = 5.0;
const SECOND_STEPS: [(f64, f64); 11] = [
    (0.5, 2.0),
    (1.0, 5.0),
    (2.0, 5.0),
    (5.0, 5.0),
    (10.0, 5.0),
    (15.0, 5.0),
    (30.0, 5.0),
    (60.0, 5.0),
    (120.0, 5.0),
    (300.0, 5.0),
    (600.0, 5.0),
];
const BEYOND_WIDEST_TICKS: f64 = 10.0;
pub const MIN_STEP_PX: f64 = 72.0;

pub fn ruler_step(px_per_frame: f64, fps: u32, min_px: f64) -> (f64, f64) {
    let fps = f64::from(fps);
    for step in FRAME_STEPS {
        if step * px_per_frame >= min_px && step < fps / 2.0 {
            return (step, (step / FRAME_TICKS).max(1.0));
        }
    }
    let (seconds, ticks) = SECOND_STEPS
        .into_iter()
        .find(|(seconds, _)| seconds * fps * px_per_frame >= min_px)
        .unwrap_or((SECOND_STEPS[SECOND_STEPS.len() - 1].0, BEYOND_WIDEST_TICKS));
    let step = seconds * fps;
    (step, step / ticks)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loop {
    pub from: f64,
    pub to: f64,
}

pub fn play_range(
    in_point: Option<f64>,
    out_point: Option<f64>,
    duration: f64,
) -> Option<(f64, f64)> {
    if (in_point.is_none() && out_point.is_none()) || duration < 2.0 {
        return None;
    }
    let last = duration - 1.0;
    let in_frame = clamp(in_point.unwrap_or(0.0), 0.0, last - 1.0);
    Some((
        in_frame,
        clamp(out_point.unwrap_or(last), in_frame + 1.0, last),
    ))
}

pub fn loop_of(in_point: Option<f64>, out_point: Option<f64>, duration: f64) -> Loop {
    play_range(in_point, out_point, duration).map_or(
        Loop {
            from: 0.0,
            to: duration,
        },
        |(from, out)| Loop {
            from,
            to: out + 1.0,
        },
    )
}

pub fn wrap_in_loop(position: f64, range: Loop) -> f64 {
    if position >= range.from && position < range.to {
        return position;
    }
    let length = range.to - range.from;
    let wrapped = range.from + (position - range.from).rem_euclid(length);
    if wrapped >= range.to {
        range.from
    } else {
        wrapped
    }
}

pub fn active_at(spans: &BTreeMap<String, (f64, f64)>, frame: f64) -> Vec<String> {
    spans
        .iter()
        .filter(|(_, (from, to))| frame >= *from && frame < *to)
        .map(|(id, _)| id.clone())
        .collect()
}

pub fn clamp_frame(frame: f64, total: f64) -> f64 {
    clamp(frame.round(), 0.0, (total - 1.0).max(0.0))
}

pub const MIN_PX_PER_FRAME: f64 = 0.01;
pub const MAX_PX_PER_FRAME: f64 = 24.0;

pub const fn clamp_zoom(px_per_frame: f64) -> f64 {
    clamp(px_per_frame, MIN_PX_PER_FRAME, MAX_PX_PER_FRAME)
}

pub fn format_time(frame: f64, fps: u32, timecode: bool) -> String {
    let whole = frame.round().max(0.0);
    let rate = f64::from(fps.max(1));
    if timecode {
        let seconds = (whole / rate).floor();
        let hours = (seconds / 3600.0).floor();
        let minutes = ((seconds % 3600.0) / 60.0).floor();
        return format!(
            "{hours:02}:{minutes:02}:{:02}:{:02}",
            seconds % 60.0,
            whole % rate
        );
    }
    let tenths = (whole / rate * 10.0).round();
    let minutes = (tenths / 600.0).floor();
    let rest = minutes.mul_add(-600.0, tenths);
    let text = if rest % 10.0 == 0.0 {
        format!("{:02}", rest / 10.0)
    } else {
        format!("{:04.1}", rest / 10.0)
    };
    format!("{minutes}:{text}")
}
