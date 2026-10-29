use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoReveal {
    #[setting(
        "Logo outline",
        default = "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z",
        widget = "code",
        help = "Logo path, or paths drawn in sequence."
    )]
    pub path_d: String,
    #[setting(
        "Name",
        default = "Acme",
        help = "Brand name rising after the mark completes."
    )]
    pub wordmark: String,
    #[setting(
        "Tagline",
        default = "Ship your next release faster",
        help = "Supporting line, entering last."
    )]
    pub tagline: String,
    #[setting("Line color", default = "#e8b86d", help = "Mark stroke color.", check = colour)]
    pub stroke: String,
    #[setting("Background color", default = "#080810", help = "Scene background.", check = colour)]
    pub background_color: String,
    #[setting("Text color", default = "#f4f4f5", check = colour)]
    pub text_color: String,
    #[setting(
        "Outline area",
        zero_label = "Fit",
        help = "Leave empty to fit the outline.",
        advanced
    )]
    pub view_box: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Logo size",
        range(min = 20, max = 2000),
        zero_label = "Auto",
        step = 10,
        unit = "px",
        "type" = "integer"
    )]
    pub size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Line width",
        range(min = 1, max = 40),
        zero_label = "Auto",
        step = 0.5,
        unit = "px",
        advanced,
        "type" = "number"
    )]
    pub stroke_width: Option<f64>,
    #[setting("Fill colour", widget = "color", zero_label = "None")]
    pub fill: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Draw time",
        default = 2.4,
        range(min = 0.2, max = 10),
        step = 0.1,
        unit = "s",
        advanced
    )]
    pub draw_seconds: f64,
    #[setting(
        "Time between lines",
        default = 0.27,
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        help = "When the logo has several lines, each starts this long after the one before.",
        advanced
    )]
    pub path_stagger: f64,
    #[setting(
        "Name at",
        default = 1.37,
        range(min = 0, max = 20),
        step = 0.01,
        unit = "s",
        help = "The tagline follows 0.4 s later.",
        advanced
    )]
    pub name_at: f64,
    #[setting("Glow", default = true, advanced)]
    pub glow: bool,
    #[setting(
        "Pen tip",
        default = true,
        help = "A dot at the end of the line while it draws.",
        advanced
    )]
    pub pen_tip: bool,
    #[setting(
        "Name size",
        default = 84,
        range(min = 16, max = 240),
        step = 1,
        unit = "px"
    )]
    pub name_size: u32,
    #[setting(
        "Name weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Tagline size",
        default = 40,
        range(min = 10, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tagline_size: u32,
    #[setting(
        "Tagline color",
        widget = "color",
        zero_label = "Softer text color",
        check = colour_or_empty,
        advanced
    )]
    pub tagline_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<LogoReveal>(
        element(
            ElementKind::Component,
            "logo-reveal",
            "Logo reveal",
            "Draws your logo outline, then brings in the name and a tagline.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw)),
    )
}
