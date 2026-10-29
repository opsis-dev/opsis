use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ParticleDissolveDirection {
    Reveal,
    Up,
    Down,
    Left,
    Right,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleDissolve {
    #[setting("Grain size", default = 4, range(min = 1, max = 16), step = 1)]
    pub particle_size: u32,
    #[setting("Scatter", default = 0.08, range(min = 0.01, max = 0.4), step = 0.01)]
    pub scatter: f64,
    #[setting("Shimmer", default = 0.6, range(min = 0, max = 1.5), step = 0.05)]
    pub shimmer: f64,
    #[setting("Aberration", default = 0.5, range(min = 0, max = 1.5), step = 0.05)]
    pub aberration: f64,
    #[setting("Direction", default = ParticleDissolveDirection::Reveal)]
    pub direction: ParticleDissolveDirection,
    #[setting("Stagger", default = 0.55, range(min = 0, max = 0.9), step = 0.05)]
    pub stagger: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ParticleDissolve>(
            element(
                ElementKind::Presentation,
                "particle-dissolve",
                "Particle dissolve",
                "The frame is ground into drifting grey dust whose specks carry real fragments of the picture, and the next scene coalesces back out of the same dust.",
                ElementCategory::Transition,
                84,
            )
                .look(NodeLook::of("lucide:sparkles").moves(NodeMotion::Dissolve)),
        )
}
