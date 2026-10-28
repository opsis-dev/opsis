use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::FontWeight;
use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoAssembleImage {
    #[default = "app:backdrops/aurora.jpg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoDriftTile {
    #[default = "Ab"]
    #[schemars(description = "Mark", extend("x-help" = "One to three characters."))]
    pub glyph: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Label", extend("type" = "string"))]
    pub label: Option<String>,
    #[default = "#1f2430"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
    #[schemars(description = "Text colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub color: String,
    #[schemars(description = "Start across", range(min = -1200, max = 1200), extend("x-step" = 1, "x-advanced" = true))]
    pub x: i32,
    #[schemars(description = "Start down", range(min = -800, max = 800), extend("x-step" = 1, "x-advanced" = true))]
    pub y: i32,
    #[schemars(description = "Drift across", range(min = -200, max = 200), extend("x-step" = 1, "x-advanced" = true))]
    pub vx: i32,
    #[schemars(description = "Drift down", range(min = -200, max = 200), extend("x-step" = 1, "x-advanced" = true))]
    pub vy: i32,
    #[default = 140]
    #[schemars(description = "Size", range(min = 20, max = 400), extend("x-step" = 1, "x-advanced" = true))]
    pub size: u32,
    #[default = 1.0]
    #[schemars(description = "Appears at", range(min = 0, max = 10), extend("x-step" = 0.01, "x-advanced" = true))]
    pub at: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(transform = super::snapcn_themes)]
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
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoDrift {
    #[default = "Built for the stack you already use."]
    #[schemars(description = "Headline")]
    pub headline: String,
    #[default = 0.1762]
    #[schemars(description = "Pull-back / sec", range(min = 0, max = 0.4), extend("x-step" = 0.001))]
    pub pullback: f64,
    #[default = 1.0]
    #[schemars(description = "Drift", range(min = 0, max = 3), extend("x-step" = 0.05))]
    pub tile_speed: f64,
    #[default = 1.0]
    #[schemars(description = "Tile size", range(min = 0.3, max = 2), extend("x-step" = 0.05))]
    pub tile_scale: f64,
    #[default = 0.4]
    #[schemars(description = "Tile fade τ (s)", range(min = 0.05, max = 2), extend("x-step" = 0.01))]
    pub tile_fade: f64,
    #[default = 0.02]
    #[schemars(description = "Tile radius", range(min = 0, max = 0.5), extend("x-step" = 0.01))]
    pub tile_radius: f64,
    #[default = 0.34]
    #[schemars(description = "Glyph size", range(min = 0.1, max = 0.7), extend("x-step" = 0.01))]
    pub glyph_scale: f64,
    #[default = 0.15]
    #[schemars(description = "Word stagger (s)", range(min = 0, max = 0.6), extend("x-step" = 0.005))]
    pub word_stagger: f64,
    #[default = 0.14]
    #[schemars(description = "Word lands in (s)", range(min = 0.02, max = 1), extend("x-step" = 0.005))]
    pub word_duration: f64,
    #[default = 1.216]
    #[schemars(description = "Word arrives at", range(min = 0.5, max = 2), extend("x-step" = 0.001))]
    pub word_scale: f64,
    #[default = 4.3]
    #[schemars(description = "Word blur (px)", range(min = 0, max = 20), extend("x-step" = 0.1))]
    pub word_blur: f64,
    #[default = 3.13]
    #[schemars(description = "Exit at (s)", range(min = 0.5, max = 10), extend("x-step" = 0.01))]
    pub exit_at: f64,
    #[default = 0.34]
    #[schemars(description = "Exit over (s)", range(min = 0.05, max = 2), extend("x-step" = 0.01))]
    pub exit_duration: f64,
    #[default = 20.0]
    #[schemars(description = "Headline size", range(min = 8, max = 64), extend("x-step" = 0.5))]
    pub font_size: f64,
    #[default = true]
    #[schemars(description = "Wash")]
    pub glow: bool,
    #[default = 0.07]
    #[schemars(description = "Wash strength", range(min = 0, max = 0.5), extend("x-step" = 0.005))]
    pub glow_opacity: f64,
    #[schemars(description = "Accent", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub accent_color: String,
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(FontWeight::W500)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[schemars(description = "Glow colour", extend("x-widget" = "color", "x-zero-label" = "Accent"))]
    pub glow_color: String,
    #[default(vec![
        LogoDriftTile { glyph: "Rc".into(), label: Some("Record".into()), color: "#ffffff".into(), x: 163, y: 232, vx: -54, vy: 12, size: 109, at: 0.28, ..Default::default() },
        LogoDriftTile { glyph: "Ed".into(), label: Some("Edit".into()), background: "#f2f4f7".into(), color: "#16181d".into(), x: 128, y: -209, vx: 81, vy: 32, size: 122, at: 0.48 },
        LogoDriftTile { glyph: "Zm".into(), label: Some("Zoom".into()), background: "#10131a".into(), color: "#7dd3fc".into(), x: -384, y: 344, vx: -27, vy: -68, size: 200, at: 0.95 },
        LogoDriftTile { glyph: "Cc".into(), label: Some("Captions".into()), background: "#e7e8ea".into(), color: "#16181d".into(), x: 519, y: 45, vx: -31, vy: 64, size: 144, at: 1.51 },
        LogoDriftTile { glyph: "Mx".into(), label: Some("Mix".into()), background: "#f1ece2".into(), color: "#8a5a2b".into(), x: -692, y: -125, vx: 41, vy: -71, size: 214, at: 1.55 },
    ])]
    #[schemars(description = "Tiles")]
    pub tiles: Vec<LogoDriftTile>,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<LogoAssemble>(
            element(
                ElementKind::Component,
                "logo-assemble",
                "Logo assemble",
                "Logo reveal animation: image cards orbit and collapse into your logo and brand name. A single ring of image cards revolves around a centred title, then spins inward and drains to the middle — giving birth to a simple logo that slides left as the brand name reveals to its right.",
                C::Scene,
                180,
            )
                .source("snapcn", "https://snapcn.dev/docs/logo-assemble", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:orbit")),
        ),
        meta::<LogoDrift>(
            element(
                ElementKind::Component,
                "logo-drift",
                "Logo drift",
                "Logo wall animation: a logo cloud of integrations drifting behind a headline. A field of tiles drifting past a headline while the camera pulls steadily back, so the wall of things-it-works-with keeps arriving from every edge.",
                C::Scene,
                180,
            )
                .source("snapcn", "https://snapcn.dev/docs/logo-drift", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:cloud")),
        ),
        meta::<HeroLaunch>(
            element(
                ElementKind::Component,
                "hero-launch",
                "Hero launch",
                "Product launch hero scene for a SaaS launch video intro. Cinematic product-launch hero — two cards slide into formation as the headline reveals above.",
                C::Scene,
                180,
            )
                .source("snapcn", "https://snapcn.dev/docs/hero-launch", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:presentation")),
        ),
    ]
}
