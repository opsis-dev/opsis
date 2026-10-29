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
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Card tilt",
        default = 17,
        range(min = 0, max = 60),
        step = 1,
        unit = "°",
        advanced
    )]
    pub card_tilt: u32,
    #[setting(
        "Card gap",
        default = 92,
        range(min = 0, max = 400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_gap: u32,
    #[setting(
        "Card shape",
        default = 1.613,
        range(min = 0.5, max = 3),
        step = 0.01,
        help = "Card width divided by its height.",
        advanced
    )]
    pub card_aspect: f64,
    #[setting(
        "Card corners",
        default = 30,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_radius: u32,
    #[setting(
        "Headline size",
        default = 42,
        range(min = 12, max = 140),
        step = 1,
        unit = "px",
        advanced
    )]
    pub headline_size: u32,
    #[setting(
        "Headline weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub headline_weight: u32,
    #[setting(
        "Fade the edges",
        default = true,
        help = "The cards fade into the background at the sides and bottom.",
        advanced
    )]
    pub edge_fade: bool,
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
