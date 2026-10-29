use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoAssembleImage {
    #[setting(
        "Image",
        default = "app:backdrops/aurora.jpg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
}

#[settings(export)]
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
    #[setting(
        "Ring images",
        default = 10,
        range(min = 5, max = 16),
        step = 1,
        advanced
    )]
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
    #[setting(
        "Orbit time",
        default = 1.67,
        range(min = 0.3, max = 10),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        help = "How long the cards circle before they gather into the logo.",
        advanced
    )]
    pub orbit_seconds: f64,
    #[setting(
        "Orbit speed",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.05,
        advanced
    )]
    pub orbit_speed: f64,
    #[setting(
        "Card shape",
        default = 1.22,
        range(min = 0.5, max = 2),
        step = 0.01,
        help = "Card height divided by its width.",
        advanced
    )]
    pub card_ratio: f64,
    #[setting(
        "Card shadow",
        default = 0.4,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub card_shadow: f64,
    #[setting(
        "Card corners",
        default = 4,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_radius: u32,
    #[setting(
        "Ring size",
        default = 36.0,
        range(min = 10, max = 48),
        step = 0.5,
        unit = "%",
        help = "As a share of the frame's short side.",
        advanced
    )]
    pub ring_size: f64,
    #[setting(
        "Card size",
        default = 15.0,
        range(min = 4, max = 30),
        step = 0.5,
        unit = "%",
        advanced
    )]
    pub card_size: f64,
    #[setting(
        "Logo size",
        default = 12.5,
        range(min = 4, max = 40),
        step = 0.5,
        unit = "%"
    )]
    pub logo_size: f64,
    #[setting(
        "Logo gap",
        default = 3.5,
        range(min = 0, max = 15),
        step = 0.5,
        unit = "%",
        advanced
    )]
    pub logo_gap: f64,
    #[setting(
        "Name size",
        default = 5.5,
        range(min = 2, max = 15),
        step = 0.1,
        unit = "%"
    )]
    pub name_size: f64,
    #[setting(
        "Center text size",
        default = 4.0,
        range(min = 1, max = 10),
        step = 0.1,
        unit = "%",
        advanced
    )]
    pub middle_text_size: f64,
    #[setting(
        "Font weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
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
