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
pub struct LogoAssembleImage {
    #[setting(
        "Image",
        default = "app:backdrops/aurora.jpg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoAssemble {
    #[setting("Brand name", default = "Acme")]
    pub brand_name: String,
    #[setting("Center text", default = "Product demos, made simple")]
    pub middle_text: String,
    #[setting(
        "Logo (simple, monochrome)",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset"
    )]
    pub logo_src: String,
    #[setting("Ring images", default = 10, range(min = 5, max = 16), step = 1)]
    pub count: u32,
    #[setting("Background", widget = "color", zero_label = "Theme")]
    pub background: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Images",
        default = vec![
            LogoAssembleImage::default(),
            LogoAssembleImage { image: "app:backdrops/bloom.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/canyon.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/chalk.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/cobalt.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/dusk.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/ember.jpg".into() },
            LogoAssembleImage { image: "app:backdrops/flare.jpg".into() },
        ],
        length(min = 1)
    )]
    pub images: Vec<LogoAssembleImage>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoAssemble>(
            element(
                ElementKind::Component,
                "logo-assemble",
                "Logo assemble",
                "Logo reveal animation: image cards orbit and collapse into your logo and brand name. A single ring of image cards revolves around a centred title, then spins inward and drains to the middle — giving birth to a simple logo that slides left as the brand name reveals to its right.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:orbit")),
        )
}
