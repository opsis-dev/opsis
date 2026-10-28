use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct HeroLaunch {
    #[default = "app:placeholders/screen-wide.svg"]
    #[schemars(description = "Left card (image/video)", extend("x-widget" = "asset"))]
    pub image1: String,
    #[default = "app:backdrops/aurora.jpg"]
    #[schemars(description = "Right card (image/video)", extend("x-widget" = "asset"))]
    pub image2: String,
    #[default = "Launch day, done right"]
    #[schemars(description = "Headline")]
    pub heading: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<HeroLaunch>(
            element(
                ElementKind::Component,
                "hero-launch",
                "Hero launch",
                "Product launch hero scene for a SaaS launch video intro. Cinematic product-launch hero — two cards slide into formation as the headline reveals above.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:presentation")),
        )
}
