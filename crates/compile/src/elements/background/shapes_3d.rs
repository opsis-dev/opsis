use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, Lighting};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Shapes3d {
    #[default = 14]
    #[schemars(description = "Shapes", range(min = 3, max = 40), extend("x-step" = 1))]
    pub count: u32,
    #[default(vec!["#6366f1".into(), "#a78bfa".into(), "#f472b6".into(), "#38bdf8".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 8))]
    pub palette: Vec<Colour>,
    #[default = 1]
    #[schemars(description = "Arrangement", range(min = 1, max = 100), extend("x-step" = 1))]
    pub seed: u32,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera"
        )
    )]
    pub camera: CameraPath,
    #[default(Lighting::Studio)]
    #[schemars(
        description = "Lighting",
        extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" })
    )]
    pub lighting: Lighting,
    #[default = "#0b1020"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
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
