use opsis_types::timeline::{AudioEnvelope, Ease, Keyframe, VisualEnvelope};

use crate::time::clamp;

pub fn ease(kind: Ease, t: f64) -> f64 {
    let x = clamp(t, 0.0, 1.0);
    match kind {
        Ease::In => x * x,
        Ease::Out => 1.0 - (1.0 - x) * (1.0 - x),
        Ease::InOut if x < 0.5 => 2.0 * x * x,
        Ease::InOut => 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0,
        Ease::Hold => 0.0,
        Ease::Linear => x,
    }
}

pub fn sort_keys(keys: &[Keyframe]) -> Vec<Keyframe> {
    let mut sorted = keys.to_vec();
    sorted.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    sorted
}

fn sorted_envelope_at(sorted: &[Keyframe], frame: f64, fallback: f64) -> f64 {
    let (Some(first), Some(last)) = (sorted.first(), sorted.last()) else {
        return fallback;
    };
    if frame <= first.frame {
        return first.value;
    }
    if frame >= last.frame {
        return last.value;
    }
    let i = sorted.partition_point(|k| k.frame <= frame);
    let (a, b) = (&sorted[i - 1], &sorted[i]);
    a.value + (b.value - a.value) * ease(a.ease, (frame - a.frame) / (b.frame - a.frame))
}

pub fn envelope_at(keys: Option<&[Keyframe]>, frame: f64, fallback: f64) -> f64 {
    sorted_envelope_at(&sort_keys(keys.unwrap_or_default()), frame, fallback)
}

const STEPS_PER_SEGMENT: usize = 8;
const HOLD_STEP_FRAMES: f64 = 1e-3;

pub fn sample_points(keys: &[Keyframe]) -> Vec<(f64, f64)> {
    let sorted = sort_keys(keys);
    let mut out = Vec::new();
    for (i, a) in sorted.iter().enumerate() {
        out.push((a.frame, a.value));
        let Some(b) = sorted.get(i + 1) else { continue };
        if b.frame <= a.frame {
            continue;
        }
        match a.ease {
            Ease::Hold => out.push((b.frame - HOLD_STEP_FRAMES, a.value)),
            Ease::Linear => {}
            _ => {
                for s in 1..STEPS_PER_SEGMENT {
                    let frame = a.frame + (b.frame - a.frame) * s as f64 / STEPS_PER_SEGMENT as f64;
                    out.push((frame, sorted_envelope_at(&sorted, frame, a.value)));
                }
            }
        }
    }
    out
}

fn any_keys<'a>(lists: impl IntoIterator<Item = &'a Option<Vec<Keyframe>>>) -> bool {
    lists
        .into_iter()
        .any(|k| k.as_ref().is_some_and(|k| !k.is_empty()))
}

pub fn has_visual_keys(env: Option<&VisualEnvelope>) -> bool {
    env.is_some_and(|e| any_keys([&e.opacity, &e.x, &e.y, &e.scale]))
}

pub fn has_audio_keys(env: Option<&AudioEnvelope>) -> bool {
    env.is_some_and(|e| any_keys([&e.volume, &e.pan]))
}
