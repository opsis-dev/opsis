use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Paper {
    #[setting("Paper colour", default = "#f5f1e8", check = colour)]
    pub color: String,
    #[setting("Grain", default = 0.18, range(min = 0, max = 0.5), step = 0.01)]
    pub grain: f64,
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
    meta::<Paper>(
        element(
            ElementKind::Component,
            "paper",
            "Paper",
            "A paper texture background.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:scroll")),
    )
}
