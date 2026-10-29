use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
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
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw)),
    )
}
