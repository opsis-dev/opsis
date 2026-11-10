use opsis_qt::{MorphPathView, MorphStep};
use opsis_types::numbers::{float, truncate};

use crate::look::{self, BakedMorph};

const STIFFNESS: f64 = 420.0;
const DAMPING: f64 = 30.0;
const SUBSTEP_S: f64 = 1.0 / 240.0;
const SETTLE_POSITION: f64 = 0.001;
const SETTLE_SPEED: f64 = 0.02;

pub fn step(position: f64, velocity: f64, target: f64, dt_ms: f64) -> MorphStep {
    let seconds = (dt_ms / 1000.0).max(0.0);
    let steps = (seconds / SUBSTEP_S).ceil().max(1.0);
    let each = seconds / steps;
    let (mut x, mut v) = (position, velocity);
    for _ in 0..truncate::<u32>(steps) {
        let pull = DAMPING.mul_add(-v, STIFFNESS * (target - x));
        v = pull.mul_add(each, v);
        x = v.mul_add(each, x);
    }
    if (target - x).abs() < SETTLE_POSITION && v.abs() < SETTLE_SPEED {
        return MorphStep {
            position: target,
            velocity: 0.0,
            settled: true,
        };
    }
    MorphStep {
        position: x,
        velocity: v,
        settled: false,
    }
}

pub fn frame(glyph: &str, morph: &str, position: f64) -> Vec<MorphPathView> {
    look::baked_morph(glyph, morph).map_or_else(Vec::new, |baked| frame_of(baked, position))
}

fn frame_of(baked: &BakedMorph, position: f64) -> Vec<MorphPathView> {
    let Some(last) = baked.frames.len().checked_sub(1) else {
        return Vec::new();
    };
    let at = ((position - baked.from) / baked.step).clamp(0.0, float(last));
    let below = truncate::<usize>(at).min(last);
    let above = (below + 1).min(last);
    let mix = at - float(below);
    baked.frames[below]
        .iter()
        .zip(&baked.frames[above])
        .zip(&baked.closed)
        .map(|((from, to), closed)| MorphPathView {
            points: from
                .iter()
                .zip(to)
                .map(|(a, b)| a + (b - a) * mix)
                .collect(),
            closed: *closed,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const FRAME_MS: [f64; 4] = [4.0, 8.0, 16.0, 33.0];
    const MAX_FRAMES: u32 = 1000;

    fn settle_time(position: f64, velocity: f64, target: f64, dt_ms: f64) -> Option<f64> {
        let mut state = MorphStep {
            position,
            velocity,
            settled: false,
        };
        for frames in 1..=MAX_FRAMES {
            state = step(state.position, state.velocity, target, dt_ms);
            if state.settled {
                return Some(dt_ms * f64::from(frames));
            }
        }
        None
    }

    fn baked() -> Vec<&'static BakedMorph> {
        [
            ("lucide:film", "lucide:clapperboard"),
            ("lucide:eye", "lucide:eye-off"),
            ("lucide:laptop", "lucide:smartphone"),
        ]
        .into_iter()
        .map(|(glyph, morph)| look::baked_morph(glyph, morph).expect("a baked morph"))
        .collect()
    }

    #[test]
    fn every_frame_keeps_the_morphs_paths_and_points() {
        for morph in baked() {
            let points = morph.frames[0][0].len();
            assert!(points > 0);
            for paths in &morph.frames {
                assert_eq!(paths.len(), morph.closed.len());
                assert!(paths.iter().all(|path| path.len() == points));
            }
        }
    }

    proptest! {
        #[test]
        fn the_spring_settles_on_its_target(
            position in -0.1f64..1.1,
            velocity in -14.0f64..14.0,
            to_one in any::<bool>(),
            dt in prop::sample::select(FRAME_MS.to_vec()),
        ) {
            let target = if to_one { 1.0 } else { 0.0 };
            let took = settle_time(position, velocity, target, dt);
            prop_assert!(took.is_some_and(|ms| ms < 1500.0), "{took:?}");
        }

        #[test]
        fn a_morph_from_rest_overshoots_by_a_few_percent_at_most(
            dt in prop::sample::select(FRAME_MS.to_vec()),
            to_one in any::<bool>(),
        ) {
            let (start, target) = if to_one { (0.0, 1.0) } else { (1.0, 0.0) };
            let mut state = MorphStep { position: start, velocity: 0.0, settled: false };
            let mut furthest: f64 = 0.0;
            while !state.settled {
                state = step(state.position, state.velocity, target, dt);
                furthest = furthest.max((state.position - start).abs());
            }
            prop_assert!(furthest < 1.05, "{furthest}");
        }

        #[test]
        fn a_frame_lies_between_the_baked_frames_around_it(
            which in 0usize..3,
            position in -0.2f64..1.2,
        ) {
            let morph = baked()[which];
            let drawn = frame_of(morph, position);
            let at = ((position - morph.from) / morph.step).clamp(0.0, float(morph.frames.len() - 1));
            let below = &morph.frames[truncate::<usize>(at.floor())];
            let above = &morph.frames[truncate::<usize>(at.ceil())];
            for (k, path) in drawn.iter().enumerate() {
                prop_assert_eq!(path.closed, morph.closed[k]);
                for (i, value) in path.points.iter().enumerate() {
                    let (a, b) = (below[k][i], above[k][i]);
                    prop_assert!(*value >= a.min(b) - 1e-9 && *value <= a.max(b) + 1e-9);
                }
            }
        }

        #[test]
        fn a_frame_on_a_baked_sample_is_that_sample(which in 0usize..3, index in 0usize..23) {
            let morph = baked()[which];
            let index = index.min(morph.frames.len() - 1);
            let drawn = frame_of(morph, morph.step.mul_add(float(index), morph.from));
            for (k, path) in drawn.iter().enumerate() {
                for (value, sample) in path.points.iter().zip(&morph.frames[index][k]) {
                    prop_assert!((value - sample).abs() < 1e-9);
                }
            }
        }
    }
}
