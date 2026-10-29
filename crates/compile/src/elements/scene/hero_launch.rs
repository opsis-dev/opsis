use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct HeroLaunch {
    #[setting(
        "Left card (image/video)",
        default = "app:placeholders/screen-wide.svg",
        widget = "asset"
    )]
    pub image1: String,
    #[setting(
        "Right card (image/video)",
        default = "app:backdrops/aurora.jpg",
        widget = "asset"
    )]
    pub image2: String,
    #[setting("Headline", default = "Launch day, done right")]
    pub heading: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
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
