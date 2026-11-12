use opsis_scene::random::random;

pub const DEFAULT_STEP: f64 = 3.0;

pub fn quantised_frame(frame: f64, step: f64) -> f64 {
    (frame / step).floor() * step
}

pub fn step_index(frame: f64, step: f64) -> f64 {
    (frame / step).floor()
}

pub fn random_between(seed: &str, lo: f64, hi: f64) -> f64 {
    random(seed).mul_add(hi - lo, lo)
}

pub fn stepped_ramp(frame: f64, from: f64, to: f64, step: f64, ease: Option<&dyn Fn(f64) -> f64>) -> f64 {
    let f = quantised_frame(frame, step);
    if f <= from {
        return 0.0;
    }
    if f >= to {
        return 1.0;
    }
    let t = (f - from) / (to - from);
    ease.map_or(t, |e| e(t))
}
