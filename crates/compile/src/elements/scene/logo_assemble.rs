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
pub struct LogoAssembleImage {
    #[default = "app:backdrops/aurora.jpg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoAssemble {
    #[default = "Acme"]
    #[schemars(description = "Brand name")]
    pub brand_name: String,
    #[default = "Product demos, made simple"]
    #[schemars(description = "Center text")]
    pub middle_text: String,
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Logo (simple, monochrome)", extend("x-widget" = "asset"))]
    pub logo_src: String,
    #[default = 10]
    #[schemars(description = "Ring images", range(min = 5, max = 16), extend("x-step" = 1))]
    pub count: u32,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub background: String,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = warm_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(vec![
        LogoAssembleImage::default(),
        LogoAssembleImage { image: "app:backdrops/bloom.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/canyon.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/chalk.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/cobalt.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/dusk.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/ember.jpg".into() },
        LogoAssembleImage { image: "app:backdrops/flare.jpg".into() },
    ])]
    #[schemars(description = "Images", length(min = 1))]
    pub images: Vec<LogoAssembleImage>,
    #[schemars(
        description = "Font",
        extend(
            "x-widget" = "font",
            "x-help" = "Project font follows the fonts set for the whole project."
        )
    )]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
