use opsis_types::timeline::{Ease, Keyframe};

pub const EPSILON: f64 = 1e-6;

pub const fn clamp(value: f64, lo: f64, hi: f64) -> f64 {
    hi.min(lo.max(value))
}

pub const fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t
}

pub fn round_to(value: f64, digits: i32) -> f64 {
    let scale = 10.0_f64.powi(digits);
    crate::clock::js_round(value * scale) / scale
}

const PERCENT_DIGITS: i32 = 6;

pub fn percent(fraction: f64) -> crate::values::Length {
    crate::values::Length::percent(round_to(fraction * 100.0, PERCENT_DIGITS))
}

pub fn sample_keyed<K, V>(
    keys: &[K],
    x: f64,
    at: impl Fn(&K) -> f64,
    value_of: impl Fn(&K) -> V,
    mix: impl Fn(&K, &K, f64) -> V,
) -> Option<V> {
    let (first, last) = (keys.first()?, keys.last()?);
    if x <= at(first) {
        return Some(value_of(first));
    }
    if x >= at(last) {
        return Some(value_of(last));
    }
    let i = keys.partition_point(|k| at(k) <= x);
    let (a, b) = (&keys[i - 1], &keys[i]);
    Some(mix(a, b, (x - at(a)) / (at(b) - at(a))))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn ease(kind: Ease, t: f64) -> f64 {
    let x = clamp01(t);
    match kind {
        Ease::In => x * x,
        Ease::Out => 1.0 - (1.0 - x) * (1.0 - x),
        Ease::InOut if x < 0.5 => 2.0 * x * x,
        Ease::InOut => {
            let rest = -2.0 * x + 2.0;
            1.0 - (rest * rest) / 2.0
        }
        Ease::Hold => 0.0,
        Ease::Linear => x,
    }
}

pub fn sort_keys(keys: &[Keyframe]) -> Vec<Keyframe> {
    let mut sorted = keys.to_vec();
    sorted.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    sorted
}

pub fn envelope_at(keys: Option<&[Keyframe]>, frame: f64, fallback: f64) -> f64 {
    let sorted = sort_keys(keys.unwrap_or_default());
    sample_keyed(
        &sorted,
        frame,
        |k| k.frame,
        |k| k.value,
        |a, b, t| lerp(a.value, b.value, ease(a.ease, t)),
    )
    .unwrap_or(fallback)
}

pub fn remap_frame(remap: &[(f64, f64)], frame: f64) -> f64 {
    let Some(last) = remap.last() else {
        return frame;
    };
    if frame > last.0 {
        return last.1 + (frame - last.0);
    }
    sample_keyed(remap, frame, |k| k.0, |k| k.1, |a, b, t| lerp(a.1, b.1, t)).unwrap_or(frame)
}
