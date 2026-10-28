use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWallLogo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Image",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Name and glyph",
            "type" = "string"
        )
    )]
    pub image: Option<String>,
    #[default = "Marrow"]
    #[schemars(description = "Name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Color",
        extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string")
    )]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Glyph", extend("type" = "string"))]
    pub glyph: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoWall {
    #[default(vec![
        LogoWallLogo {
            name: "Northstar".into(),
            color: Some("#E8B86D".into()),
            ..Default::default()
        },
        LogoWallLogo {
            name: "Halcyon".into(),
            color: Some("#7DD3E8".into()),
            ..Default::default()
        },
        LogoWallLogo {
            name: "Fernweh".into(),
            color: Some("#9BD4A0".into()),
            ..Default::default()
        },
        LogoWallLogo {
            name: "Orbital".into(),
            color: Some("#C99BE8".into()),
            ..Default::default()
        },
        LogoWallLogo { name: "Lumen".into(), color: Some("#8FB8F0".into()), ..Default::default() },
        LogoWallLogo {
            name: "Tidewater".into(),
            color: Some("#E89B9B".into()),
            ..Default::default()
        },
        LogoWallLogo {
            name: "Kestrel".into(),
            color: Some("#E8B86D".into()),
            ..Default::default()
        },
        LogoWallLogo { color: Some("#7DD3E8".into()), ..Default::default() },
    ])]
    #[schemars(
        description = "Logos",
        extend(
            "x-help" = "Wordmark, brand colour and the glyph in the mark beside it, or a logo image in their place."
        )
    )]
    pub logos: Vec<LogoWallLogo>,
    #[default = "Trusted by"]
    #[schemars(
        description = "Small heading",
        extend("x-help" = "Small line above the heading. Omit to drop it.")
    )]
    pub eyebrow: String,
    #[default = "Teams that ship with us"]
    #[schemars(
        description = "Title",
        extend("x-help" = "Heading above the wall. Omit to drop it.")
    )]
    pub title: String,
    #[default = 4]
    #[schemars(
        description = "Columns",
        range(min = 1),
        extend("x-step" = 1, "x-help" = "Tiles per row.")
    )]
    pub columns: u32,
    #[default = 0.32]
    #[schemars(
        description = "Start at (seconds)",
        extend("x-step" = 0.01, "x-help" = "Second the first tile arrives.", "x-advanced" = true)
    )]
    pub start_at_seconds: f64,
    #[default = 0.06]
    #[schemars(
        description = "Arrive stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between tiles arriving, in reading order. Deliberately fast.",
            "x-advanced" = true
        )
    )]
    pub arrive_stagger_seconds: f64,
    #[default = 0.24]
    #[schemars(
        description = "Stagger (seconds)",
        extend(
            "x-step" = 0.01,
            "x-help" = "Seconds between diagonals of the colour sweep.",
            "x-advanced" = true
        )
    )]
    pub stagger_seconds: f64,
    #[default = 0.6]
    #[schemars(
        description = "Colour fade (seconds)",
        range(min = 0.05, max = 10),
        extend(
            "x-step" = 0.01,
            "x-help" = "How long one tile takes to go from grey to brand colour.",
            "x-advanced" = true
        )
    )]
    pub warm_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Eyebrow, and the fallback for logos with no colour.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-help" = "Animation speed multiplier.")
    )]
    pub speed: f64,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<LogoWall>(
        element(
            ElementKind::Component,
            "logo-wall",
            "Logo wall",
            "A grid of customer logos that light up from grey to colour.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:layout-grid")),
    )
}
