use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnterLogo {
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
    #[default = "#2563eb"]
    #[schemars(
        description = "Background",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnter {
    #[default = 118]
    #[schemars(
        description = "Diameter",
        range(min = 48, max = 320),
        extend("x-step" = 1, "x-unit" = "px")
    )]
    pub diameter: u32,
    #[default = 38]
    #[schemars(
        description = "Overlap",
        range(min = 0, max = 200),
        extend("x-step" = 1, "x-unit" = "px")
    )]
    pub overlap: u32,
    #[default = "#fff"]
    #[schemars(
        description = "Ring",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ring_color: String,
    #[default(Orientation::Horizontal)]
    #[schemars(description = "Orientation")]
    pub orientation: Orientation,
    #[default = 0.2333]
    #[schemars(
        description = "Stagger",
        range(min = 0),
        extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30)
    )]
    pub stagger: f64,
    #[default(vec![
        LogoEnterLogo::default(),
        LogoEnterLogo { background: "#0f766e".into(), ..Default::default() },
        LogoEnterLogo { background: "#b45309".into(), ..Default::default() },
        LogoEnterLogo { background: "#7c3aed".into(), ..Default::default() },
    ])]
    #[schemars(description = "Logos", length(min = 1, max = 16))]
    pub logos: Vec<LogoEnterLogo>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoEnter>(
            element(
                ElementKind::Component,
                "logo-enter",
                "Logo enter",
                "A stacked group of round brand chips that spring in one-by-one, then hold. Swap the logos to feature any set of marks.",
                ElementCategory::Ui,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:layout-grid-circles")),
        )
}
