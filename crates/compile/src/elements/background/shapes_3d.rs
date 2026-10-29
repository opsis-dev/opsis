use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, Lighting};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Shapes3d {
    #[setting("Shapes", default = 14, range(min = 3, max = 40), step = 1)]
    pub count: u32,
    #[setting(
        "Colours",
        default = vec!["#6366f1".into(), "#a78bfa".into(), "#f472b6".into(), "#38bdf8".into()],
        length(min = 1, max = 8)
    )]
    pub palette: Vec<Colour>,
    #[setting("Arrangement", default = 1, range(min = 1, max = 100), step = 1)]
    pub seed: u32,
    #[setting("Speed", default = 1.0, range(min = 0, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera"
    )]
    pub camera: CameraPath,
    #[setting(
        "Lighting",
        default = Lighting::Studio,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b1020", check = colour)]
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
