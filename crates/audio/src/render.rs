use crate::STEM_CHANNELS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    pub step: f64,
    pub volume: f32,
}

fn sample(samples: &[f32], frame: usize, channel: usize) -> f32 {
    samples
        .get(frame * STEM_CHANNELS + channel)
        .copied()
        .unwrap_or(0.0)
}

pub const fn frames_in(samples: &[f32]) -> usize {
    samples.len() / STEM_CHANNELS
}

pub fn fill(out: &mut [f32], channels: usize, samples: &[f32], position: f64, pace: Pace) -> f64 {
    let channels = channels.max(1);
    let total = frames_in(samples);
    let mut at = position;
    for frame in out.chunks_mut(channels) {
        let base = at.floor();
        let fraction = opsis_types::numbers::narrow(at - base);
        let inside = base >= 0.0 && base < opsis_types::numbers::float(total);
        let index = if inside {
            opsis_types::numbers::truncate::<usize>(base)
        } else {
            0
        };
        let read = |channel: usize| -> f32 {
            if !inside {
                return 0.0;
            }
            let now = sample(samples, index, channel);
            let next = if index + 1 < total {
                sample(samples, index + 1, channel)
            } else {
                now
            };
            (next - now).mul_add(fraction, now) * pace.volume
        };
        let (left, right) = (read(0), read(1));
        match frame {
            [only] => *only = f32::midpoint(left, right),
            [l, r, rest @ ..] => {
                *l = left;
                *r = right;
                rest.fill(0.0);
            }
            [] => {}
        }
        at += pace.step;
    }
    at
}
