use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AuroraBg {
    #[setting(
        "Background color",
        default = "#050505",
        help = "Plate behind the curtains.",
        check = colour
    )]
    pub background_color: String,
    #[setting(
        "Colors",
        default = vec!["#e4ac59".into(), "#e07a5f".into(), "#c2557a".into()],
        length(min = 1),
        help = "Curtain colours, cycled."
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Ribbon count",
        default = 4,
        range(min = 1, max = 12),
        step = 1,
        help = "How many curtains."
    )]
    pub ribbon_count: u32,
    #[setting(
        "Amplitude",
        default = 9.0,
        help = "Vertical travel of a fold, in percent of the frame.",
        advanced
    )]
    pub amplitude: f64,
    #[setting(
        "Thickness",
        default = 15.0,
        help = "Curtain height at its thickest point, in percent.",
        advanced
    )]
    pub thickness: f64,
    #[setting(
        "Vertical position",
        default = 46.0,
        help = "Where the band sits, in percent of the frame height."
    )]
    pub center_y: f64,
    #[setting("Spread", default = 34.0, advanced)]
    pub spread: f64,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.25,
        help = "Drift speed. 0 freezes the sky."
    )]
    pub speed: f64,
    #[setting(
        "Blur",
        default = 14.0,
        help = "Much above 20 and the folds become blobs.",
        advanced
    )]
    pub blur: f64,
    #[setting("Intensity", default = 1.3, step = 0.1, help = "Overall brightness.")]
    pub intensity: f64,
    #[setting(
        "Horizon glow",
        default = 0.5,
        step = 0.01,
        help = "Ground glow under the band. 0 removes it.",
        advanced
    )]
    pub horizon_glow: f64,
    #[setting(
        "Striation",
        default = 0.55,
        step = 0.01,
        help = "Vertical ray structure through the curtains, 0-1.",
        advanced
    )]
    pub striation: f64,
    #[setting("Variation", default = 1, advanced)]
    pub seed: u32,
    #[setting("Folds", default = 2.0, range(min = 0.5, max = 6), advanced)]
    pub folds: f64,
    #[setting(
        "Sideways drift",
        default = 13.0,
        range(min = 0, max = 40),
        unit = "%/s",
        advanced
    )]
    pub sideways_drift: f64,
    #[setting(
        "Ray spacing",
        default = 72,
        range(min = 16, max = 240),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ray_spacing: u32,
    #[setting("Vignette", default = 0.55, range(min = 0, max = 1), advanced)]
    pub vignette: f64,
    #[setting(
        "Horizon glow colour",
        widget = "color",
        zero_label = "First colour",
        check = colour_or_empty,
        advanced
    )]
    pub horizon_glow_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<AuroraBg>(
        element(
            ElementKind::Component,
            "aurora-bg",
            "Aurora",
            "Soft ribbons of light drifting across a dark sky.",
            ElementCategory::Background,
            120,
        )
        .look(NodeLook::of("tabler:wave-sine")),
    )
}
