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

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Screen3d {
    #[default(ScreenDevice::Laptop)]
    #[schemars(
        description = "Device",
        extend(
            "x-labels" = {
                "laptop": "Laptop",
                "monitor": "Monitor",
                "phone": "Phone",
                "floating": "Floating screen"
            }
        )
    )]
    pub device: ScreenDevice,
    #[default = "#c9ced6"]
    #[schemars(
        description = "Device colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub device_color: String,
    #[default(ScreenMove::PushIn)]
    #[schemars(
        description = "Camera move",
        extend(
            "x-labels" = {
                "push-in": "Push in",
                "orbit": "Orbit",
                "tilt-reveal": "Tilt reveal",
                "rise": "Rise",
                "keyframes": "Keyframes"
            }
        )
    )]
    pub r#move: ScreenMove,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "move": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = true]
    #[schemars(
        description = "End facing the camera",
        extend("x-help" = "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device.")
    )]
    pub end_face_on: bool,
    #[default(Lighting::Studio)]
    #[schemars(
        description = "Lighting",
        extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" })
    )]
    pub lighting: Lighting,
    #[default = "#0b0b10"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Screen3d>(element(ElementKind::Wrapper, "screen-3d",
            "3D screen",
            "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
            ElementCategory::Device,
            150).look(NodeLook::of("iconoir:perspective-view")))
}
