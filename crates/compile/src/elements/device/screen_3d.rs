use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Lighting;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ScreenDevice {
    Laptop,
    Monitor,
    Phone,
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

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Screen3d {
    #[setting(
        "Device",
        default = ScreenDevice::Laptop,
        labels = {
            "laptop": "Laptop",
            "monitor": "Monitor",
            "phone": "Phone",
            "floating": "Floating screen"
        }
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
        help = "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device."
    )]
    pub end_face_on: bool,
    #[setting(
        "Lighting",
        default = Lighting::Studio,
        labels = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b0b10", check = colour)]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Screen3d>(element(ElementKind::Wrapper, "screen-3d",
            "3D screen",
            "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
            ElementCategory::Device,
            150).look(NodeLook::of("iconoir:perspective-view")))
}
