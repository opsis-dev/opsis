use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::elements::entry::{element, meta};
use crate::elements::shared::Lighting;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ScreenDevice {
    #[label("Laptop")]
    Laptop,
    #[label("Monitor")]
    Monitor,
    #[label("Phone")]
    Phone,
    #[label("Floating screen")]
    Floating,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ScreenMove {
    PushIn,
    Orbit,
    TiltReveal,
    Rise,
    Keyframes,
}

#[settings(export)]
pub struct Screen3d {
    #[setting(
        "Device",
        default = ScreenDevice::Laptop
    )]
    pub device: ScreenDevice,
    #[setting("Device colour", default = "#c9ced6", check = colour)]
    pub device_color: String,
    #[setting(
        "Camera move",
        default = ScreenMove::PushIn,
        labels = {
            "push-in": "Push in",
            "orbit": "Orbit",
            "tilt-reveal": "Tilt reveal",
            "rise": "Rise",
            "keyframes": "Keyframes"
        }
    )]
    pub r#move: ScreenMove,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "move": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting(
        "End facing the camera",
        default = true,
        help = "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device.",
        advanced
    )]
    pub end_face_on: bool,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b0b10", check = colour)]
    pub background: String,
    #[setting("Bezel colour", default = "#0a0a0c", check = colour, advanced)]
    pub bezel_color: String,
    #[setting(
        "Metalness",
        default = 0.6,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub metalness: f64,
    #[setting(
        "Roughness",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub roughness: f64,
    #[setting(
        "Glare",
        default = 0.16,
        range(min = 0, max = 1),
        step = 0.01,
        help = "How bright the light sweeping over the screen gets as it turns.",
        advanced
    )]
    pub glare: f64,
    #[setting(
        "Move ends at",
        default = 0.85,
        range(min = 0.1, max = 1),
        step = 0.01,
        help = "Share of the element's length the move takes; it holds still after.",
        show_if = { "move": ["push-in", "orbit", "tilt-reveal", "rise"] },
        advanced
    )]
    pub move_ends_at: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Screen3d>(element(ElementKind::Wrapper, "screen-3d",
            "3D screen",
            "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
            ElementCategory::Device,
            150).look(NodeLook::of("iconoir:perspective-view")))
}
