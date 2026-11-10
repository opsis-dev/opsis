use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ParticleDissolveDirection {
    Reveal,
    Up,
    Down,
    Left,
    Right,
}

#[settings(export)]
pub struct ParticleDissolve {
    #[setting("Grain size", default = 4, range(min = 1, max = 16), step = 1)]
    pub particle_size: u32,
    #[setting("Scatter", default = 0.08, range(min = 0.01, max = 0.4), step = 0.01)]
    pub scatter: f64,
    #[setting(
        "Shimmer",
        default = 0.6,
        range(min = 0, max = 1.5),
        step = 0.05,
        advanced
    )]
    pub shimmer: f64,
    #[setting(
        "Aberration",
        default = 0.5,
        range(min = 0, max = 1.5),
        step = 0.05,
        advanced
    )]
    pub aberration: f64,
    #[setting("Direction", default = ParticleDissolveDirection::Reveal)]
    pub direction: ParticleDissolveDirection,
    #[setting(
        "Stagger",
        default = 0.55,
        range(min = 0, max = 0.9),
        step = 0.05,
        advanced
    )]
    pub stagger: f64,
    #[setting(
        "Dust colour kept",
        default = 0.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub dust_color: f64,
    #[setting("Ragged edge", default = 0.45, range(min = 0, max = 1), advanced)]
    pub raggedness: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ParticleDissolve>(
            element(
                ElementKind::Presentation,
                "particle-dissolve",
                "Particle dissolve",
                "The frame is ground into drifting grey dust whose specks carry real fragments of the picture, and the next scene coalesces back out of the same dust.",
                ElementCategory::Transition,
                36,
            )
                .look(NodeLook::of("lucide:sparkles")),
        )
}
