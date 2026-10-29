use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MorphShape {
    Circle,
    Square,
    Squircle,
    Triangle,
    Diamond,
    Blob,
}

#[settings(export)]
pub struct TransitionMorphShape {
    #[setting(
        "From",
        default = MorphShape::Circle,
        help = "Shape the reveal starts as, at zero size. A raw path d also works."
    )]
    pub from: MorphShape,
    #[setting(
        "To",
        default = MorphShape::Squircle,
        help = "Shape it has become once it covers the frame."
    )]
    pub to: MorphShape,
    #[setting(
        "Horizontal origin",
        default = 0.5,
        step = 0.01,
        help = "Where the shape grows from, 0→1 of the frame width.",
        advanced
    )]
    pub origin_x: f64,
    #[setting(
        "Vertical origin",
        default = 0.5,
        step = 0.01,
        help = "Where the shape grows from, 0→1 of the frame height.",
        advanced
    )]
    pub origin_y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Overshoot",
        range(min = 0.8, max = 3),
        zero_label = "Fit the shape",
        step = 0.05,
        help = "Final size as a multiple of the frame diagonal. Leave empty to grow each shape until it covers the frame.",
        advanced,
        "type" = "number"
    )]
    pub overshoot: Option<f64>,
    #[setting(
        "Warp",
        range(min = 0, max = 100),
        step = 1,
        unit = "px",
        help = "0 keeps the silhouette crisp.",
        advanced
    )]
    pub warp: f64,
    #[setting(
        "Warp detail",
        default = 0.008,
        range(min = 0.001, max = 0.05),
        advanced
    )]
    pub warp_detail: f64,
    #[setting(
        "Variation",
        default = 7,
        range(min = 0, max = 100),
        step = 1,
        advanced
    )]
    pub seed: u32,
}

pub fn definition() -> ElementMeta {
    meta::<TransitionMorphShape>(
            element(
                ElementKind::Presentation,
                "transition-morph-shape",
                "Shape morph",
                "The next scene shows through a shape that grows and changes form until it fills the frame.",
                ElementCategory::Transition,
                24,
            )
                .look(NodeLook::of("lucide:square").morphs("lucide:circle")),
        )
}
