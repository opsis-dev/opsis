use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use smart_default::SmartDefault;

use crate::elements::choices::SpeedLinesMode;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CrtScreen {
    #[default = 1.0]
    #[schemars(description = "Curvature", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub curvature: f64,
    #[default = 1.0]
    #[schemars(description = "Scanlines", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub scanlines: f64,
    #[default = 2]
    #[schemars(description = "Mask pitch", range(min = 1, max = 6), extend("x-step" = 1))]
    pub mask_scale: u32,
    #[default = 1.0]
    #[schemars(description = "Vignette", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub vignette: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct VhsFilter {
    #[default = 1.0]
    #[schemars(description = "Chroma bleed", range(min = 0, max = 3), extend("x-step" = 0.1))]
    pub bleed: f64,
    #[default = 1.0]
    #[schemars(description = "Tape wobble", range(min = 0, max = 3), extend("x-step" = 0.1))]
    pub wobble: f64,
    #[default = 1.0]
    #[schemars(description = "Noise", range(min = 0, max = 3), extend("x-step" = 0.1))]
    pub noise: f64,
    #[default = 1.0]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct HalftonePrint {
    #[default = 10]
    #[schemars(description = "Dot size", range(min = 4, max = 32), extend("x-step" = 1))]
    pub dot_size: u32,
    #[schemars(description = "Screen angle", range(min = 0, max = 90), extend("x-step" = 1))]
    pub angle: u32,
    #[default = 1.2]
    #[schemars(description = "Misregistration", range(min = 0, max = 6), extend("x-step" = 0.2))]
    pub misregistration: f64,
    #[default = "#f4efe4"]
    #[schemars(description = "Paper", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub paper_tint: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CameraLens {
    #[default = 0.5]
    #[schemars(description = "Corner softness", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.5]
    #[schemars(description = "Bloom", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub bloom: f64,
    #[default = 0.5]
    #[schemars(description = "Aberration", range(min = 0, max = 3), extend("x-step" = 0.1))]
    pub aberration: f64,
    #[default = 0.5]
    #[schemars(description = "Vignette", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub vignette: f64,
    #[default = 0.05]
    #[schemars(description = "Distortion", range(min = 0, max = 0.6), extend("x-step" = 0.01))]
    pub distortion: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Polaroid {
    #[default = "first light"]
    #[schemars(description = "Caption")]
    pub caption: String,
    #[schemars(description = "Caption starts", range(min = 0, max = 60), extend("x-step" = 1))]
    pub caption_at: u32,
    #[default = 652]
    #[schemars(description = "Card width", range(min = 320, max = 900), extend("x-step" = 4))]
    pub width: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Caption size", range(min = 12, max = 56), extend("x-zero-label" = "Fit", "x-step" = 1, "x-unit" = "px", "x-help" = "Leave empty to size the caption with the card.", "type" = "integer"))]
    pub caption_size: Option<u32>,
    #[default = 0.1]
    #[schemars(description = "Time per pose", range(min = 0.0333, max = 0.2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub step: f64,
    #[default = "#fdfcf8"]
    #[schemars(description = "Frame colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub frame_color: String,
    #[default = "#26242c"]
    #[schemars(description = "Caption colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub caption_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TvPowerOff {
    #[default = 18]
    #[schemars(description = "Duration", range(min = 8, max = 30), extend("x-step" = 1))]
    pub duration_in_frames: u32,
    #[default = 1.0]
    #[schemars(description = "Surge", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub gain: f64,
    #[default = 1.0]
    #[schemars(description = "Afterglow", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub afterglow: f64,
    #[default = "#d8ecff"]
    #[schemars(description = "Phosphor", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub phosphor: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeedLines {
    #[default(SpeedLinesMode::Trail)]
    #[schemars(description = "Mode")]
    pub mode: SpeedLinesMode,
    #[default = 12]
    #[schemars(description = "Lines", range(min = 3, max = 32), extend("x-step" = 1))]
    pub count: u32,
    #[default = 2.0]
    #[schemars(description = "Stroke weight", range(min = 1, max = 8), extend("x-step" = 0.5))]
    pub weight: f64,
    #[default = 96]
    #[schemars(description = "Spread", range(min = 16, max = 320), extend("x-step" = 4))]
    pub spread: u32,
    #[default = 14]
    #[schemars(description = "Travel frames", range(min = 6, max = 40), extend("x-step" = 1))]
    pub duration: u32,
    #[default = 0.5]
    #[schemars(description = "Bounce", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bounce: f64,
    #[default = 0.5]
    #[schemars(description = "Point x", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Point y", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
    #[schemars(description = "Delay", range(min = 0, max = 60), extend("x-step" = 1))]
    pub delay: u32,
    #[default = "#ffffff"]
    #[schemars(description = "Line color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = 1]
    #[schemars(description = "Seed", range(min = 1), extend("x-step" = 1))]
    pub seed: u32,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<CrtScreen>(
            element(
                ElementKind::Wrapper,
                "crt-screen",
                "CRT screen",
                "Put a scene behind curved glass: the picture bulges out to the corners, an RGB phosphor mask and scanlines ride the curve, bright areas flood the mask, and the tube falls away into darkness at the rim.",
                C::Effect,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/crt-screen", "MIT")
                .look(NodeLook::of("lucide:tv-minimal")),
        ),
        meta::<VhsFilter>(
            element(
                ElementKind::Wrapper,
                "vhs-filter",
                "VHS filter",
                "Hold a scene under analog tape: chroma bleeds sideways off every edge, rows wobble out of line, the bottom of the frame tears on the head switch, and luma noise settles over all of it.",
                C::Effect,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/vhs-filter", "MIT")
                .look(NodeLook::of("lucide:cassette-tape")),
        ),
        meta::<HalftonePrint>(
            element(
                ElementKind::Wrapper,
                "halftone-print",
                "Halftone print",
                "Print a scene onto paper: each colour separates onto its own rotated screen of dots sized by the ink under them, slightly out of register the way a real press runs.",
                C::Effect,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/halftone-print", "MIT")
                .look(NodeLook::of("tabler:grid-dots")),
        ),
        meta::<CameraLens>(
            element(
                ElementKind::Wrapper,
                "camera-lens",
                "Camera lens",
                "Put a scene behind real glass: resolution falls off toward the corners, highlights bloom through the optics, colour fringes at the edges — a screencast that reads as filmed rather than rendered.",
                C::Effect,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/camera-lens", "MIT")
                .look(NodeLook::of("lucide:aperture").moves(NodeMotion::RackFocus)),
        ),
        meta::<Polaroid>(
            element(
                ElementKind::Wrapper,
                "polaroid",
                "Polaroid",
                "An instant-photo frame with a handwritten caption for video, images, or live compositions.",
                C::Effect,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/polaroid", "MIT")
                .look(NodeLook::of("tabler:polaroid")),
        ),
        meta::<TvPowerOff>(
            element(
                ElementKind::Wrapper,
                "tv-power-off",
                "TV power off",
                "End a scene the way a tube does: the picture collapses into a glowing horizontal line, the line pinches to a phosphor dot, the dot decays, and the frame is left dead black.",
                C::Effect,
                62,
            )
                .source("Remocn", "https://remocn.dev/docs/components/tv-power-off", "MIT")
                .look(NodeLook::of("lucide:tv").moves(NodeMotion::Collapse)),
        ),
        meta::<SpeedLines>(
            element(
                ElementKind::Component,
                "speed-lines",
                "Speed lines",
                "Streaks shoot across the frame toward a point and die as they land, or radial focus lines rush in from the edges and snap onto one point for an impact frame. Lay it over a clip with Layers.",
                C::Effect,
                55,
            )
                .source("Remocn", "https://remocn.dev/docs/components/speed-lines", "MIT")
                .authored_clock()
                .fixed("children", json!(null))
                .look(NodeLook::of("lucide:chevrons-right")),
        ),
    ]
}
