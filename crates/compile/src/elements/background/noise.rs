use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Noise {
    #[setting("Base", default = "#0b1020", check = colour)]
    pub color1: String,
    #[setting("Highlight", default = "#4f46e5", check = colour)]
    pub color2: String,
    #[setting(
        "Scale",
        default = 0.004,
        range(min = 0.0005, max = 0.02),
        step = 0.0005,
        advanced
    )]
    pub scale: f64,
    #[setting("Speed", default = 0.004, range(min = 0, max = 0.05), step = 0.001)]
    pub speed: f64,
    #[setting("Blobs", default = 5, range(min = 1, max = 12), step = 1, advanced)]
    pub blobs: u32,
    #[setting(
        "Wander",
        default = 40.0,
        range(min = 0, max = 60),
        unit = "%",
        advanced
    )]
    pub wander: f64,
    #[setting(
        "Blob size",
        default = 30.0,
        range(min = 5, max = 80),
        unit = "%",
        advanced
    )]
    pub blob_size: f64,
    #[setting("Intensity", default = 0.55, range(min = 0, max = 1))]
    pub intensity: f64,
    #[setting(
        "Softness",
        default = 40,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub softness: u32,
    #[setting("Variation", step = 1, advanced)]
    pub seed: u32,
    #[setting("Grain", default = 0.08, range(min = 0, max = 0.4))]
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
    meta::<Noise>(
        element(
            ElementKind::Component,
            "noise",
            "Noise",
            "A soft moving noise background.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("tabler:grain")),
    )
}
