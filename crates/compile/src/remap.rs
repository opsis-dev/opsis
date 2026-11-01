use opsis_types::numbers::exact_eq;

use crate::limits::MIN_SPEED;
use crate::text::round_to;

pub type Remap = Vec<(f64, f64)>;

fn lerp_segment((x0, y0): (f64, f64), (x1, y1): (f64, f64), x: f64) -> f64 {
    y0 + (x - x0) / (x1 - x0) * (y1 - y0)
}

pub fn speed_remap(inner_duration: u32, out_duration: u32) -> Remap {
    vec![
        (0.0, 0.0),
        (f64::from(out_duration), f64::from(inner_duration)),
    ]
}

pub fn remap_duration(remap: &[(f64, f64)]) -> f64 {
    remap.last().map_or(0.0, |(out, _)| *out)
}

pub fn remap_frame(remap: &[(f64, f64)], frame: f64) -> f64 {
    let (Some(first), Some(last)) = (remap.first(), remap.last()) else {
        return frame;
    };
    if frame > last.0 {
        return last.1 + (frame - last.0);
    }
    if frame <= first.0 {
        return first.1;
    }
    if frame >= last.0 {
        return last.1;
    }
    let i = remap.partition_point(|(out, _)| *out <= frame);
    let (a, b) = (remap[i - 1], remap[i]);
    a.1 + (b.1 - a.1) * ((frame - a.0) / (b.0 - a.0))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemapSegment {
    pub from: f64,
    pub to: f64,
    pub rate: f64,
}

const EASE_STEPS: u32 = 6;

pub fn eased_segments(segments: &[(f64, f64, f64)], ease: f64) -> Vec<(f64, f64, f64)> {
    if ease <= 0.0 {
        return segments.to_vec();
    }
    let steps = f64::from(EASE_STEPS);
    segments
        .iter()
        .flat_map(|&(from, to, rate)| {
            let e = ease.min((to - from) / 2.0).max(0.0);
            if e <= 0.0 {
                return vec![(from, to, rate)];
            }
            let step = e / steps;
            let between = |k: u32| 1.0 + (rate - 1.0) * f64::from(k) / (steps + 1.0);
            let mut parts: Vec<(f64, f64, f64)> = (0..EASE_STEPS)
                .map(|k| {
                    let at = from + step * f64::from(k);
                    (at, at + step, between(k + 1))
                })
                .collect();
            parts.push((from + e, to - e, rate));
            parts.extend((0..EASE_STEPS).map(|k| {
                let at = to - e + step * f64::from(k);
                (at, at + step, between(EASE_STEPS - k))
            }));
            parts
        })
        .collect()
}

pub fn remap_from_segments(inner_duration: f64, segments: &[RemapSegment]) -> Remap {
    let mut out = vec![(0.0, 0.0)];
    let mut out_pos = 0.0;
    let mut in_pos = 0.0;
    let mut push = |in_to: f64, rate: f64| {
        if in_to <= in_pos {
            return;
        }
        out_pos += (in_to - in_pos) / rate;
        in_pos = in_to;
        out.push((round_to(out_pos, 3), in_pos));
    };
    for s in segments {
        push(s.from.min(inner_duration), 1.0);
        push(s.to.min(inner_duration), s.rate.max(MIN_SPEED));
    }
    push(inner_duration, 1.0);
    out
}

fn before_first(remap: &[(f64, f64)], r: f64) -> f64 {
    let (o0, i0) = remap[0];
    let slope = remap
        .iter()
        .find(|(_, i)| *i > i0)
        .map_or(1.0, |next| (next.0 - o0) / (next.1 - i0));
    o0 - (i0 - r) * slope
}

fn after_last((o_n, i_n): (f64, f64), r: f64) -> f64 {
    o_n + (r - i_n)
}

fn earliest_out(remap: &[(f64, f64)], r: f64) -> f64 {
    for k in 1..remap.len() {
        let (a, ia) = remap[k - 1];
        let (b, ib) = remap[k];
        if ib >= r {
            return if exact_eq(ib, ia) {
                a
            } else {
                lerp_segment((ia, a), (ib, b), r)
            };
        }
    }
    after_last(remap[remap.len() - 1], r)
}

fn latest_out(remap: &[(f64, f64)], r: f64) -> f64 {
    for k in (1..remap.len()).rev() {
        let (a, ia) = remap[k - 1];
        let (b, ib) = remap[k];
        if ia <= r {
            return if ib <= r || exact_eq(ib, ia) {
                b
            } else {
                lerp_segment((ia, a), (ib, b), r)
            };
        }
    }
    remap[0].0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemapSide {
    Start,
    End,
}

pub fn inverse_remap(remap: &[(f64, f64)], r: f64, side: RemapSide) -> f64 {
    let Some(last) = remap.last().copied() else {
        return r;
    };
    let (o0, i0) = remap[0];
    if r < i0 {
        return before_first(remap, r);
    }
    match side {
        RemapSide::Start if exact_eq(r, i0) => o0,
        RemapSide::Start => earliest_out(remap, r),
        RemapSide::End if r >= last.1 => after_last(last, r),
        RemapSide::End => latest_out(remap, r),
    }
}
