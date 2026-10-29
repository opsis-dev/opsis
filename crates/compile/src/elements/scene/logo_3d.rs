use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;

use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Logo3d {
    #[setting(
        "Logo file",
        widget = "asset",
        accept = ".svg,.png,.jpg,.jpeg,.webp",
        zero_label = "Sample logo",
        help = "A drawn logo is given depth; a picture of a logo goes on a card."
    )]
    pub src: String,
    #[setting("Depth", default = 0.25, range(min = 0.05, max = 1), step = 0.05)]
    pub depth: f64,
    #[setting("Colour", widget = "color", zero_label = "The logo's own colours")]
    pub color: String,
    #[setting("Metal", default = 0.3, range(min = 0, max = 1), step = 0.05, advanced)]
    pub metalness: f64,
    #[setting(
        "Motion",
        default = Motion::Spin
    )]
    pub motion: Motion,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "motion": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3))]
    pub zoom: f64,
    #[setting("Matte", default = 0.35, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting("Bevel", default = 0.08, range(min = 0, max = 0.3), advanced)]
    pub bevel: f64,
    #[setting("Rounded corners", default = 0.12, range(min = 0, max = 0.5), advanced)]
    pub corner_radius: f64,
    #[setting(
        "Tilt",
        default = 52,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        advanced
    )]
    pub tilt: u32,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
    #[setting("Turn the other way", advanced)]
    pub reverse: bool,
    #[setting(
        "Lens",
        default = 30.0,
        range(min = 10, max = 70),
        unit = "°",
        advanced
    )]
    pub lens: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Logo3d>(
        element(
            ElementKind::Component,
            "logo-3d",
            "3D logo",
            "Your logo in 3D, spinning, tilting in or floating, with studio lighting.",
            ElementCategory::Scene,
            150,
        )
        .look(NodeLook::of("lucide:rotate-3d").moves(NodeMotion::Turn)),
    )
}
