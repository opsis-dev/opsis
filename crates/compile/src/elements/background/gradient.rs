use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum GradientShape {
    #[label("Straight")]
    Linear,
    #[label("Round")]
    Radial,
    #[label("Around the centre")]
    Conic,
}

#[settings(export)]
pub struct Gradient {
    #[setting("From", default = "#1e1b4b", check = colour)]
    pub from: String,
    #[setting("To", default = "#0f172a", check = colour)]
    pub to: String,
    #[setting("Angle", default = 135, range(min = 0, max = 360), step = 1)]
    pub angle: u32,
    #[setting("Grain", default = 0.08, range(min = 0, max = 0.4), step = 0.01)]
    pub grain: f64,
    #[setting(
        "Shape",
        default = GradientShape::Linear,
        advanced
    )]
    pub shape: GradientShape,
    #[setting(
        "Grain size",
        default = 1.0,
        range(min = 0.5, max = 4),
        unit = "x",
        advanced
    )]
    pub grain_size: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Gradient>(
        element(
            ElementKind::Component,
            "gradient",
            "Gradient",
            "A gradient background.",
            ElementCategory::Background,
            90,
        )
        .look(NodeLook::of("lucide:rainbow")),
    )
}
