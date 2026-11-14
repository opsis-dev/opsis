use opsis_scene::easing::Easing;
use opsis_scene::keyed::{lerp, sample_keyed};
use opsis_types::actions::{Rect, Viewport};
use opsis_types::recording::{CameraEasing, CameraTuning};

use crate::elements::camera::click_log::{ClickEvent, ClickLog, design_scale_for, rect_center};
use crate::recording::point_rect;

pub const S_MAX: f64 = 1.74;
const ZOOM_MIN: f64 = 1.18;
const FIRST_SHOT_MAX: f64 = 1.54;
const PAD_PX: f64 = 80.0;
const MENU_ROOM_PX: f64 = 110.0;
const TALL_TARGET_PX: f64 = 90.0;
const TALL_MENU_ROOM_PX: f64 = 24.0;
const CLICK_PULL: f64 = 0.2;
const VERTICAL_OFF_CENTER_SHARE: f64 = 0.65;
const TARGET_SHARE: f64 = 0.55;
const NARROW_TARGET_SHARE: f64 = 0.86;
const OFF_CENTER_X: f64 = 0.26;
const CLICK_EDGE_MARGIN: f64 = 0.04;
const CLICK_EDGE_MARGIN_MAX_SHARE: f64 = 0.4;
const GROUP_GAP_S: f64 = 2.2;
const GROUP_DIST_FRAC: f64 = 0.28;
const STICKY_GAP_S: f64 = 2.8;
const STICKY_DIST_FRAC: f64 = 0.22;
const LEAD_FALLBACK_S: f64 = 0.75;
const HOLD_MIN_S: f64 = 1.3;
const TYPE_TAIL_S: f64 = 0.4;
const CAMERA_LEAD_S: f64 = 0.035;
const BASE_BEAT_S: f64 = 0.6;
const MAX_PAN_PX: f64 = 500.0;
const TRACK_MIN_S: f64 = 0.5;
const KEY_EPSILON_S: f64 = 1e-4;
const KEY_MERGE_S: f64 = 0.2;
const NARROW_WINDOW_S: f64 = 1.2;
const NARROW_REST_EXP: f64 = 0.8;
const NARROW_REST_AIM_Y: f64 = 0.5;
const SAME_SCALE: f64 = 0.008;
const SAME_CENTER: f64 = 0.01;
const MOVE_BASE_MS: f64 = 480.0;
const MOVE_MS_PER_SCALE: f64 = 420.0;
const MOVE_MS_PER_PX: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPose {
    pub scale: f64,
    pub cx: f64,
    pub cy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomState {
    pub scale: f64,
    pub translate_x: f64,
    pub translate_y: f64,
}

pub const BASE_POSE: CameraPose = CameraPose {
    scale: 1.0,
    cx: 0.5,
    cy: 0.48,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraKey {
    pub t: f64,
    pub pose: CameraPose,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tuning {
    pub camera: CameraTuning,
    pub narrow: Option<f64>,
}

struct Tuned {
    min_scale: f64,
    first_shot_max: f64,
    pad_px: f64,
    target_share: f64,
    off_centre: f64,
    join_gap_s: f64,
    sticky_gap_s: f64,
    max_pan_px: f64,
}

fn tuned(t: &CameraTuning) -> Tuned {
    Tuned {
        min_scale: t.min_scale.unwrap_or(ZOOM_MIN),
        first_shot_max: t.first_max_scale.unwrap_or(FIRST_SHOT_MAX),
        pad_px: t.room_px.unwrap_or(PAD_PX),
        target_share: t.target_share.unwrap_or(TARGET_SHARE),
        off_centre: t.off_centre.unwrap_or(OFF_CENTER_X),
        join_gap_s: t.join_within_s.unwrap_or(GROUP_GAP_S),
        sticky_gap_s: t.stay_zoomed_within_s.unwrap_or(STICKY_GAP_S),
        max_pan_px: t.longest_pan_px.unwrap_or(MAX_PAN_PX),
    }
}

pub const fn js_clamp(value: f64, low: f64, high: f64) -> f64 {
    high.min(low.max(value))
}

pub const fn clamp01(value: f64) -> f64 {
    js_clamp(value, 0.0, 1.0)
}

const fn easing_of(easing: CameraEasing) -> Option<Easing> {
    match easing {
        CameraEasing::Smooth => Some(Easing::Bezier(0.2, 0.2, 0.15, 1.0)),
        CameraEasing::Gentle => Some(Easing::Bezier(0.45, 0.0, 0.55, 1.0)),
        CameraEasing::Snappy => Some(Easing::Bezier(0.1, 0.9, 0.2, 1.0)),
        CameraEasing::Linear => None,
    }
}

pub fn camera_ease(t: f64, easing: CameraEasing) -> f64 {
    let t = clamp01(t);
    easing_of(easing).map_or(t, |e| e.at(t))
}

pub fn interpolate_pose(a: CameraPose, b: CameraPose, t: f64, easing: CameraEasing) -> CameraPose {
    let p = camera_ease(t, easing);
    CameraPose {
        scale: lerp(a.scale, b.scale, p),
        cx: lerp(a.cx, b.cx, p),
        cy: lerp(a.cy, b.cy, p),
    }
}

fn poses_nearly_equal(a: CameraPose, b: CameraPose) -> bool {
    (a.scale - b.scale).abs() < SAME_SCALE
        && (a.cx - b.cx).abs() < SAME_CENTER
        && (a.cy - b.cy).abs() < SAME_CENTER
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Move {
    In,
    Out,
    Pan,
}

const fn limits(kind: Move) -> (f64, f64, f64) {
    match kind {
        Move::In => (1.0, 550.0, 1250.0),
        Move::Out => (1.25, 650.0, 1450.0),
        Move::Pan => (1.0, 700.0, 1400.0),
    }
}

pub fn pose_travel_px(from: CameraPose, to: CameraPose, vp: Viewport) -> f64 {
    let (w, h) = (f64::from(vp.width), f64::from(vp.height));
    ((to.cx - from.cx) * w).hypot((to.cy - from.cy) * h) / design_scale_for(w)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film times each move with separate multiplies and adds"
)]
fn move_duration(from: CameraPose, to: CameraPose, kind: Move, vp: Viewport, speed: f64) -> f64 {
    let ds = (to.scale - from.scale).abs();
    let dpx = pose_travel_px(from, to, vp);
    let ms = MOVE_BASE_MS + MOVE_MS_PER_SCALE * ds + MOVE_MS_PER_PX * dpx;
    let (stretch, min_ms, max_ms) = limits(kind);
    js_clamp(ms * stretch, min_ms, max_ms) * speed / 1000.0
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places the camera with separate multiplies and adds"
)]
pub fn pose_to_css(pose: CameraPose, chrome_frac: f64, fit: f64) -> ZoomState {
    let effective = fit * pose.scale;
    let window_cy = chrome_frac + pose.cy * (1.0 - chrome_frac);
    let half = 1.0 / (2.0 * pose.scale);
    let focus_x = js_clamp(pose.cx, half, 1.0 - half);
    let focus_y = js_clamp(window_cy, half, 1.0 - half);
    ZoomState {
        scale: effective,
        translate_x: effective * (0.5 - focus_x),
        translate_y: effective * (0.5 - focus_y),
    }
}

fn narrow_factor(tuning: &Tuning) -> f64 {
    tuning.narrow.unwrap_or(1.0)
}

pub fn is_narrow(tuning: &Tuning) -> bool {
    narrow_factor(tuning) > 1.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    pub cx: f64,
    pub cy: f64,
}

pub fn cursor_aim(log: &ClickLog, t: f64) -> Option<Aim> {
    let track = log.cursor_track.as_deref().filter(|t| !t.is_empty())?;
    let from = (t - NARROW_WINDOW_S) * 1000.0;
    let to = (t + NARROW_WINDOW_S) * 1000.0;
    let lo = track.partition_point(|s| s.t < from);
    let (mut x, mut y, mut n) = (0.0, 0.0, 0.0);
    for s in track.iter().skip(lo).take_while(|s| s.t <= to) {
        x += s.x;
        y += s.y;
        n += 1.0;
    }
    if n == 0.0 {
        let near = &track[lo.min(track.len() - 1)];
        x = near.x;
        y = near.y;
        n = 1.0;
    }
    Some(Aim {
        cx: x / n / f64::from(log.viewport.width),
        cy: y / n / f64::from(log.viewport.height),
    })
}

fn zoom_ceiling(tuning: &Tuning) -> f64 {
    tuning.camera.s_max.unwrap_or(S_MAX) * narrow_factor(tuning)
}

pub fn narrow_pose(pose: CameraPose, tuning: &Tuning, aim: Option<Aim>) -> CameraPose {
    if !is_narrow(tuning) {
        return pose;
    }
    let rest = narrow_factor(tuning).powf(NARROW_REST_EXP);
    let zoomed = clamp01((pose.scale - rest) / (zoom_ceiling(tuning) - rest));
    let rest_aim = aim.unwrap_or(Aim {
        cx: BASE_POSE.cx,
        cy: BASE_POSE.cy,
    });
    CameraPose {
        scale: pose.scale.max(rest),
        cx: lerp(rest_aim.cx, pose.cx, zoomed),
        cy: lerp(
            lerp(BASE_POSE.cy, rest_aim.cy, NARROW_REST_AIM_Y),
            pose.cy,
            zoomed,
        ),
    }
}

fn apart_frac(a: &ClickEvent, b: &ClickEvent, vp: Viewport) -> f64 {
    let d = (b.x - a.x).hypot(b.y - a.y);
    d / f64::from(vp.width).hypot(f64::from(vp.height))
}

fn gap_s(a: &ClickEvent, b: &ClickEvent) -> f64 {
    (b.t_ms - a.t_ms) / 1000.0
}

fn clusterize(clicks: &[ClickEvent], vp: Viewport, t: &Tuned) -> Vec<Vec<ClickEvent>> {
    let list: Vec<&ClickEvent> = clicks.iter().filter(|c| c.zoom != Some(false)).collect();
    let mut groups: Vec<Vec<ClickEvent>> = Vec::new();
    for (i, c) in list.iter().enumerate() {
        let starts = i == 0 || {
            let prev = list[i - 1];
            apart_frac(prev, c, vp) > GROUP_DIST_FRAC
                || prev.cluster != c.cluster
                || (c.cluster.is_none() && gap_s(prev, c) > t.join_gap_s)
        };
        match groups.last_mut() {
            Some(group) if !starts => group.push((*c).clone()),
            _ => groups.push(vec![(*c).clone()]),
        }
    }
    groups
}

fn stickify(groups: Vec<Vec<ClickEvent>>, vp: Viewport, t: &Tuned) -> Vec<Vec<ClickEvent>> {
    if groups.len() <= 1 {
        return groups;
    }
    let explicit = groups.iter().any(|g| g.iter().any(|c| c.cluster.is_some()));
    let mut iter = groups.into_iter();
    let mut out: Vec<Vec<ClickEvent>> = iter.next().into_iter().collect();
    for next in iter {
        let Some(prev) = out.last_mut() else {
            out.push(next);
            continue;
        };
        let (Some(last), Some(first)) = (prev.last(), next.first()) else {
            out.push(next);
            continue;
        };
        let other_cluster = explicit
            && last.cluster.is_some()
            && first.cluster.is_some()
            && last.cluster != first.cluster;
        let close =
            gap_s(last, first) <= t.sticky_gap_s && apart_frac(last, first, vp) <= STICKY_DIST_FRAC;
        if !other_cluster && close {
            prev.extend(next);
        } else {
            out.push(next);
        }
    }
    out
}

struct Subject {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film sizes the subject with separate multiplies and adds"
)]
fn subject_of(c: &ClickEvent, width: f64, pad_px: f64) -> Subject {
    let k = design_scale_for(width);
    let rect: Rect = c.rect.unwrap_or_else(|| point_rect(c.point(), k));
    let menu_room = (if rect.h >= TALL_TARGET_PX * k {
        TALL_MENU_ROOM_PX
    } else {
        MENU_ROOM_PX
    }) * k;
    let center = rect_center(rect);
    let lifted_y = center.y + menu_room / 2.0;
    Subject {
        x: lerp(center.x, c.x, CLICK_PULL),
        y: lerp(lifted_y, c.y, CLICK_PULL),
        w: rect.w + 2.0 * pad_px * k,
        h: rect.h + menu_room + 2.0 * pad_px * k,
    }
}

fn scale_for(
    c: &ClickEvent,
    subject: &Subject,
    vp: Viewport,
    s_max: f64,
    narrow: bool,
    t: &Tuned,
) -> f64 {
    let least = t.min_scale.min(s_max);
    if let Some(scale) = c.zoom_scale {
        return js_clamp(scale, least, s_max);
    }
    let share = if narrow {
        NARROW_TARGET_SHARE
    } else {
        t.target_share
    };
    let by_width = (share * f64::from(vp.width)) / subject.w;
    if narrow {
        return js_clamp(by_width, least, s_max);
    }
    let by_height = (t.target_share * f64::from(vp.height)) / subject.h;
    js_clamp(by_width.min(by_height), least, s_max)
}

fn in_view(v: f64, half: f64) -> f64 {
    js_clamp(v, half, 1.0 - half)
}

fn keeping_click(aimed: f64, click: f64, half: f64) -> f64 {
    let margin = CLICK_EDGE_MARGIN.min(half * CLICK_EDGE_MARGIN_MAX_SHARE);
    in_view(
        js_clamp(aimed, click - half + margin, click + half - margin),
        half,
    )
}

fn off_center(focus: f64, click: f64, offset: f64, scale: f64) -> f64 {
    let half = 1.0 / (2.0 * scale);
    let side = if focus >= 0.5 { 1.0 } else { -1.0 };
    keeping_click(in_view(focus - (offset * side) / scale, half), click, half)
}

fn frame_for(c: &ClickEvent, vp: Viewport, s_max: f64, narrow: bool, t: &Tuned) -> CameraPose {
    let (w, h) = (f64::from(vp.width), f64::from(vp.height));
    let subject = subject_of(c, w, t.pad_px);
    let scale = scale_for(c, &subject, vp, s_max, narrow, t);
    let focus_x = clamp01(subject.x / w);
    let focus_y = clamp01(subject.y / h);
    if narrow {
        let half = 1.0 / (2.0 * scale);
        return CameraPose {
            scale,
            cx: keeping_click(in_view(focus_x, half), clamp01(c.x / w), half),
            cy: keeping_click(in_view(focus_y, half), clamp01(c.y / h), half),
        };
    }
    CameraPose {
        scale,
        cx: off_center(focus_x, clamp01(c.x / w), t.off_centre, scale),
        cy: off_center(
            focus_y,
            clamp01(c.y / h),
            t.off_centre * VERTICAL_OFF_CENTER_SHARE,
            scale,
        ),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn depart_s(c: &ClickEvent) -> f64 {
    c.t_depart_ms.unwrap_or(c.t_ms - LEAD_FALLBACK_S * 1000.0) / 1000.0
}

fn normalize_keys(mut keys: Vec<CameraKey>) -> Vec<CameraKey> {
    keys.sort_by(|a, b| a.t.total_cmp(&b.t));
    let mut out: Vec<CameraKey> = Vec::with_capacity(keys.len());
    for k in keys {
        let Some(prev) = out.last_mut() else {
            out.push(k);
            continue;
        };
        if k.t + KEY_EPSILON_S < prev.t {
            continue;
        }
        let replaces = k.t - prev.t < KEY_EPSILON_S
            || (k.t - prev.t < KEY_MERGE_S && poses_nearly_equal(prev.pose, k.pose));
        if replaces {
            *prev = k;
        } else {
            out.push(k);
        }
    }
    out
}

struct Shot {
    group: Vec<ClickEvent>,
    poses: Vec<CameraPose>,
    t_in: f64,
}

struct Plan {
    vp: Viewport,
    speed: f64,
    max_pan_px: f64,
    hold_min: f64,
    type_tail: f64,
    base_beat: f64,
    end_s: f64,
}

fn last_t(keys: &[CameraKey]) -> f64 {
    keys.last().map_or(0.0, |k| k.t)
}

fn hold_until(c: &ClickEvent, plan: &Plan) -> f64 {
    c.type_end_ms
        .map_or(c.t_ms / 1000.0 + plan.hold_min, |end| {
            (c.t_ms / 1000.0 + plan.hold_min).max(end / 1000.0 + plan.type_tail)
        })
}

fn movement(t: f64, from: CameraPose, to: CameraPose, duration: f64) -> Vec<CameraKey> {
    vec![
        CameraKey { t, pose: from },
        CameraKey {
            t: t + duration,
            pose: to,
        },
    ]
}

fn zoom_in(from: CameraPose, shot: &Shot, after: f64, plan: &Plan) -> Vec<CameraKey> {
    let to = shot.poses[0];
    movement(
        shot.t_in.max(after),
        from,
        to,
        move_duration(from, to, Move::In, plan.vp, plan.speed),
    )
}

fn route_via_base(
    from: CameraPose,
    to: CameraPose,
    t_pan: f64,
    hold_end: f64,
    plan: &Plan,
) -> Vec<CameraKey> {
    let back = move_duration(from, BASE_POSE, Move::Out, plan.vp, plan.speed);
    let back_start = js_clamp(t_pan - back, hold_end, t_pan);
    let mut keys = movement(back_start, from, BASE_POSE, back);
    keys.extend(movement(
        (back_start + back).max(t_pan),
        BASE_POSE,
        to,
        move_duration(BASE_POSE, to, Move::In, plan.vp, plan.speed),
    ));
    keys
}

fn pan_within(shot: &Shot, start: f64, plan: &Plan) -> Vec<CameraKey> {
    let mut keys: Vec<CameraKey> = Vec::new();
    let mut cur = start;
    for j in 1..shot.group.len() {
        let (from, to) = (shot.poses[j - 1], shot.poses[j]);
        let t_pan = cur.max(depart_s(&shot.group[j]) - CAMERA_LEAD_S);
        let hold_end = cur.max(hold_until(&shot.group[j - 1], plan));
        if pose_travel_px(from, to, plan.vp) > plan.max_pan_px {
            keys.extend(route_via_base(from, to, t_pan, hold_end, plan));
        } else {
            keys.extend(movement(
                t_pan,
                from,
                to,
                move_duration(from, to, Move::Pan, plan.vp, plan.speed),
            ));
        }
        cur = last_t(&keys);
    }
    keys
}

fn trail_out(
    shot: &Shot,
    next: Option<&Shot>,
    cur: f64,
    plan: &Plan,
) -> (Vec<CameraKey>, CameraPose) {
    let pose = shot.poses[shot.poses.len() - 1];
    let hold_end = cur.max(hold_until(&shot.group[shot.group.len() - 1], plan));
    let out = move_duration(pose, BASE_POSE, Move::Out, plan.vp, plan.speed);
    let Some(next) = next else {
        let clip_end = cur.max(plan.end_s);
        let start = cur.max(hold_end.min(clip_end - out - plan.base_beat));
        let keys = if start + out <= clip_end {
            let mut keys = movement(start, pose, BASE_POSE, out);
            keys.push(CameraKey {
                t: clip_end,
                pose: BASE_POSE,
            });
            keys
        } else {
            vec![CameraKey { t: clip_end, pose }]
        };
        return (keys, BASE_POSE);
    };
    let far_apart = pose_travel_px(pose, next.poses[0], plan.vp) > plan.max_pan_px;
    if next.t_in >= hold_end + out || far_apart {
        let start = cur.max(hold_end.min(next.t_in - out));
        return (movement(start, pose, BASE_POSE, out), BASE_POSE);
    }
    (
        vec![CameraKey {
            t: cur.max(next.t_in),
            pose,
        }],
        pose,
    )
}

fn plan_shots(log: &ClickLog, tuning: &Tuning) -> Vec<Shot> {
    let narrow = is_narrow(tuning);
    let s_max = zoom_ceiling(tuning);
    let t = tuned(&tuning.camera);
    let first_max = t.first_shot_max.min(s_max);
    let vp = log.viewport;
    stickify(clusterize(&log.clicks, vp, &t), vp, &t)
        .into_iter()
        .enumerate()
        .map(|(i, group)| {
            let poses = group
                .iter()
                .map(|c| {
                    let most = if i == 0 && c.zoom_scale.is_none() && !narrow {
                        first_max
                    } else {
                        s_max
                    };
                    frame_for(c, vp, most, narrow, &t)
                })
                .collect();
            let t_in = (depart_s(&group[0]) - CAMERA_LEAD_S).max(0.0);
            Shot { group, poses, t_in }
        })
        .collect()
}

fn track_plan(log: &ClickLog, speed: f64, tuning: &Tuning) -> Plan {
    let c = &tuning.camera;
    Plan {
        vp: log.viewport,
        speed: speed * c.pace.unwrap_or(1.0),
        max_pan_px: tuned(c).max_pan_px,
        hold_min: c.hold_min_s.unwrap_or(HOLD_MIN_S) * speed,
        type_tail: c.hold_after_typing_s.unwrap_or(TYPE_TAIL_S) * speed,
        base_beat: c.rest_at_end_s.unwrap_or(BASE_BEAT_S) * speed,
        end_s: log.duration_ms / 1000.0,
    }
}

pub fn build_camera_track(log: &ClickLog, speed: f64, tuning: &Tuning) -> Vec<CameraKey> {
    let shots = plan_shots(log, tuning);
    let start = CameraKey {
        t: 0.0,
        pose: BASE_POSE,
    };
    let plan = track_plan(log, speed, tuning);
    if shots.is_empty() {
        return vec![
            start,
            CameraKey {
                t: plan.end_s.max(TRACK_MIN_S),
                pose: BASE_POSE,
            },
        ];
    }
    let mut keys = vec![start];
    let mut pose = BASE_POSE;
    for (i, shot) in shots.iter().enumerate() {
        let mut moves = zoom_in(pose, shot, last_t(&keys), &plan);
        let panned = pan_within(shot, last_t(&moves), &plan);
        moves.extend(panned);
        let (trail, after) = trail_out(shot, shots.get(i + 1), last_t(&moves), &plan);
        keys.extend(moves);
        keys.extend(trail);
        pose = after;
    }
    normalize_keys(keys)
}

pub fn sample_track(keys: &[CameraKey], t: f64, tuning: &Tuning) -> CameraPose {
    let easing = tuning.camera.easing.unwrap_or(CameraEasing::Smooth);
    sample_keyed(
        keys,
        t,
        |k| k.t,
        |k| k.pose,
        |a, b, p| interpolate_pose(a.pose, b.pose, p, easing),
    )
    .unwrap_or(BASE_POSE)
}

pub fn log_time_s(frame: f64, fps: f64, speed: f64) -> f64 {
    frame / fps * speed
}

pub fn pose_at(
    keys: &[CameraKey],
    frame: f64,
    fps: f64,
    speed: f64,
    tuning: &Tuning,
) -> CameraPose {
    sample_track(keys, log_time_s(frame, fps, speed), tuning)
}

pub fn zoom_at(
    log: &ClickLog,
    keys: &[CameraKey],
    time_s: f64,
    tuning: &Tuning,
    chrome_frac: f64,
    fit: f64,
) -> ZoomState {
    let aim = if is_narrow(tuning) {
        cursor_aim(log, time_s)
    } else {
        None
    };
    let pose = narrow_pose(sample_track(keys, time_s, tuning), tuning, aim);
    pose_to_css(pose, chrome_frac, fit)
}
