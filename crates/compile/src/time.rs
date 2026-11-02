use opsis_types::numbers;
use opsis_types::numbers::{whole, whole_signed};

pub const EPSILON: f64 = 1e-6;

pub fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t
}

pub fn even_px(value: f64) -> f64 {
    (value / 2.0).round() * 2.0
}

pub const fn clamp(value: f64, lo: f64, hi: f64) -> f64 {
    hi.min(lo.max(value))
}

pub fn ms_to_seconds(ms: f64) -> f64 {
    ms / 1000.0
}

pub fn ms_to_exact_frames(ms: f64, fps: u32) -> f64 {
    ms_to_seconds(ms) * f64::from(fps)
}

pub fn ms_to_signed_frames(ms: f64, fps: u32) -> i32 {
    whole_signed(ms_to_exact_frames(ms, fps))
}

pub fn whole_frames(frames: f64, min: u32) -> u32 {
    whole(frames).max(min)
}

pub fn at_least_one_frame(frames: f64) -> u32 {
    whole_frames(frames, 1)
}

pub fn ms_to_frames(ms: f64, fps: u32) -> u32 {
    whole_frames(ms_to_exact_frames(ms, fps), 1)
}

pub fn ms_to_frames_from(ms: f64, fps: u32, min: u32) -> u32 {
    whole_frames(ms_to_exact_frames(ms, fps), min)
}

pub fn frames_to_ms(frames: impl Into<f64>, fps: u32) -> f64 {
    (frames.into() * 1000.0 / f64::from(fps)).round()
}

pub fn recording_ms_to_frames(ms: f64, speed: f64, fps: u32) -> f64 {
    ms / 1000.0 / speed * f64::from(fps)
}

pub fn frames_to_recording_ms(frames: f64, speed: f64, fps: u32) -> f64 {
    frames / f64::from(fps) * speed * 1000.0
}

const fn clock(total_seconds: u64) -> (u64, u64, u64) {
    (
        total_seconds / 3600,
        total_seconds % 3600 / 60,
        total_seconds % 60,
    )
}

pub fn chapter_time(ms: f64) -> String {
    let (h, m, s) = clock(numbers::truncate::<u64>((ms / 1000.0).floor().max(0.0)));
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
