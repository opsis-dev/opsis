use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum GrainDissolveShape {
    Wave,
    Dots,
    Truchet,
    Corners,
    Ripple,
    Blob,
    Sphere,
}

#[settings(export)]
pub struct GrainDissolve {
    #[setting("Shape", default = GrainDissolveShape::Blob)]
    pub shape: GrainDissolveShape,
    #[setting("Noise", default = 0.3, range(min = 0, max = 1), step = 0.05, advanced)]
    pub noise: f64,
    #[setting(
        "Zoom",
        default = 2.0,
        range(min = 0.5, max = 4),
        step = 0.25,
        advanced
    )]
    pub zoom: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()],
        length(min = 2, max = 7)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Scene blur",
        default = 10,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub scene_blur: u32,
    #[setting(
        "Rotation",
        default = 0.0,
        range(min = 0, max = 360),
        unit = "°",
        advanced
    )]
    pub rotation: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GrainDissolve>(
            element(
                ElementKind::Presentation,
                "grain-dissolve",
                "Grain dissolve",
                "A grain-gradient shader transition that disintegrates the outgoing scene into soft grainy shapes and condenses the next scene out of the noise.",
                ElementCategory::Transition,
                76,
            )
                .look(NodeLook::of("tabler:grain")),
        )
}
