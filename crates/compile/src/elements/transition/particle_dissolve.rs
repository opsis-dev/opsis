use opsis_macros::{choice, settings};
use opsis_scene::node::Node;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{paired, value};
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

const MAX_STAGGER: f64 = 0.92;

const fn axis(direction: ParticleDissolveDirection) -> (f64, f64) {
    match direction {
        ParticleDissolveDirection::Reveal => (0.0, 0.0),
        ParticleDissolveDirection::Up => (0.0, -1.0),
        ParticleDissolveDirection::Down => (0.0, 1.0),
        ParticleDissolveDirection::Left => (-1.0, 0.0),
        ParticleDissolveDirection::Right => (1.0, 0.0),
    }
}

pub fn present(
    props: &ParticleDissolve,
    side: Transition,
    _: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let (dx, dy) = axis(props.direction);
    let radial = if props.direction == ParticleDissolveDirection::Reveal {
        1.0
    } else {
        0.0
    };
    Ok(paired(
        "particle-dissolve",
        vec![
            value("progress", vec![side.progress]),
            value("particleSize", vec![f64::from(props.particle_size)]),
            value("scatter", vec![props.scatter]),
            value("shimmer", vec![props.shimmer]),
            value("aberration", vec![props.aberration]),
            value("stagger", vec![props.stagger.clamp(0.0, MAX_STAGGER)]),
            value("dir", vec![dx, dy]),
            value("radial", vec![radial]),
            value("grey", vec![1.0 - props.dust_color / 100.0]),
            value("ragged", vec![props.raggedness]),
        ],
        side.entering,
        side.cut,
        side.children,
    ))
}
