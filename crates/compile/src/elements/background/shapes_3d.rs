use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, Lighting};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Sphere,
    Ring,
    Box,
}

#[settings(export)]
pub struct Shapes3d {
    #[setting("Shapes", default = 14, range(min = 3, max = 40), step = 1)]
    pub count: u32,
    #[setting(
        "Colours",
        default = vec!["#6366f1".into(), "#a78bfa".into(), "#f472b6".into(), "#38bdf8".into()],
        length(min = 1, max = 8)
    )]
    pub palette: Vec<Colour>,
    #[setting(
        "Arrangement",
        default = 1,
        range(min = 1, max = 100),
        step = 1,
        advanced
    )]
    pub seed: u32,
    #[setting("Speed", default = 1.0, range(min = 0, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        advanced
    )]
    pub camera: CameraPath,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b1020", check = colour)]
    pub background: String,
    #[setting("Depth", default = 1.0, range(min = 0, max = 2), unit = "x", advanced)]
    pub depth: f64,
    #[setting("Shape size", default = 1.0, range(min = 0.25, max = 3), unit = "x")]
    pub size: f64,
    #[setting("Drift", default = 0.45, range(min = 0, max = 1.5), advanced)]
    pub drift: f64,
    #[setting("Spin", default = 1.0, range(min = 0, max = 4), unit = "x", advanced)]
    pub spin: f64,
    #[setting("Roughness", default = 0.4, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting("Metalness", default = 0.1, range(min = 0, max = 1), advanced)]
    pub metalness: f64,
    #[setting(
        "Kinds",
        default = vec![ShapeKind::Sphere, ShapeKind::Ring, ShapeKind::Box],
        length(min = 1, max = 3),
        widget = "multiselect",
        labels = { "sphere": "Spheres", "ring": "Rings", "box": "Boxes" },
        check = { "rule": "unique", "message": "Pick each shape once" },
        advanced
    )]
    pub kinds: Vec<ShapeKind>,
}

pub fn definition() -> ElementMeta {
    meta::<Shapes3d>(
        element(
            ElementKind::Component,
            "shapes-3d",
            "Floating shapes",
            "Soft 3D spheres, rings and rounded boxes drifting slowly.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:torus")),
    )
}
