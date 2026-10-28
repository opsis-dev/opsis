use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct ReleaseTeaser {
    #[default = "Orvio"]
    #[schemars(description = "Name in the final release lockup")]
    pub brand_name: String,
    #[default = "2"]
    #[schemars(description = "Version suffix; leave empty to hide")]
    pub release: String,
    #[default = "Your next chapter starts here."]
    #[schemars(description = "Final supporting line")]
    pub tagline: String,
    #[default = "#9BC8DD"]
    #[schemars(
        description = "Mark and dimensional edge lighting",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[schemars(description = "Static mark and complete statements, without camera motion or blur")]
    pub reduced_motion: bool,
    #[default(vec![
        "Start with an idea".into(),
        "Make room\nfor better work".into(),
        "Bring every detail to life".into(),
        "Move together.\nGo further.".into(),
        "A fresh chapter.\nNo starting over.".into(),
    ])]
    #[schemars(
        description = "Statements",
        length(min = 5, max = 5),
        extend("x-help" = "The five lines, in order.")
    )]
    pub statements: Vec<String>,
    #[schemars(
        description = "Logo",
        extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Name only")
    )]
    pub logo_src: String,
    #[schemars(
        description = "Background",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub theme_background: String,
    #[schemars(
        description = "Panel colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub theme_surface: String,
    #[schemars(
        description = "Text colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub theme_foreground: String,
    #[schemars(
        description = "Soft text colour",
        extend("x-widget" = "color", "x-zero-label" = "Default")
    )]
    pub theme_muted: String,
    #[default = 1.0]
    #[schemars(
        description = "Light",
        range(min = 0, max = 2),
        extend("x-step" = 0.05, "x-advanced" = true)
    )]
    pub light_intensity: f64,
    #[schemars(
        description = "Music",
        extend(
            "x-zero-label" = "None",
            "x-help" = "Plays through the Mixer like any other music.",
            "x-widget" = "sound"
        )
    )]
    pub music: String,
    #[default = 1.0]
    #[schemars(description = "Music volume", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub music_volume: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ReleaseTeaser>(
            element(
                ElementKind::Component,
                "release-teaser",
                "Release teaser",
                "A 16-second cinematic release teaser with center-out typography, a procedural dimensional mark, moving edge light, and an independent fictional identity.",
                ElementCategory::Scene,
                300,
            )
                .fonts(&["Inter"])
                .look(NodeLook::of("lucide:rocket")),
        )
}
