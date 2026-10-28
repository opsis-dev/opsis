use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoReveal {
    #[default = "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z"]
    #[schemars(
        description = "Logo outline",
        extend("x-widget" = "code", "x-help" = "Logo path, or paths drawn in sequence.")
    )]
    pub path_d: String,
    #[default = "Acme"]
    #[schemars(
        description = "Name",
        extend("x-help" = "Brand name rising after the mark completes.")
    )]
    pub wordmark: String,
    #[default = "Ship your next release faster"]
    #[schemars(description = "Tagline", extend("x-help" = "Supporting line, entering last."))]
    pub tagline: String,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Line color",
        extend(
            "x-help" = "Mark stroke color.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub stroke: String,
    #[default = "#080810"]
    #[schemars(
        description = "Background color",
        extend(
            "x-help" = "Scene background.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background_color: String,
    #[default = "#f4f4f5"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub text_color: String,
    #[schemars(
        description = "Outline area",
        extend(
            "x-zero-label" = "Fit",
            "x-help" = "Leave empty to fit the outline.",
            "x-advanced" = true
        )
    )]
    pub view_box: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Logo size",
        range(min = 20, max = 2000),
        extend("x-zero-label" = "Auto", "x-step" = 10, "x-unit" = "px", "type" = "integer")
    )]
    pub size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Line width",
        range(min = 1, max = 40),
        extend(
            "x-zero-label" = "Auto",
            "x-step" = 0.5,
            "x-unit" = "px",
            "x-advanced" = true,
            "type" = "number"
        )
    )]
    pub stroke_width: Option<f64>,
    #[schemars(description = "Fill colour", extend("x-widget" = "color", "x-zero-label" = "None"))]
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
        .look(NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw)),
    )
}
