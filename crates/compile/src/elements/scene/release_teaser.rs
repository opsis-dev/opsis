use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ReleaseTeaser {
    #[setting("Name in the final release lockup", default = "Orvio")]
    pub brand_name: String,
    #[setting("Version suffix; leave empty to hide", default = "2")]
    pub release: String,
    #[setting("Final supporting line", default = "Your next chapter starts here.")]
    pub tagline: String,
    #[setting("Mark and dimensional edge lighting", default = "#9BC8DD", check = colour)]
    pub accent_color: String,
    #[setting(
        "Static mark and complete statements, without camera motion or blur",
        advanced
    )]
    pub reduced_motion: bool,
    #[setting(
        "Statements",
        default = vec![
            "Start with an idea".into(),
            "Make room\nfor better work".into(),
            "Bring every detail to life".into(),
            "Move together.\nGo further.".into(),
            "A fresh chapter.\nNo starting over.".into(),
        ],
        length(min = 5, max = 5),
        help = "The five lines, in order."
    )]
    pub statements: Vec<String>,
    #[setting("Logo", widget = "asset", accept = "image/*", zero_label = "Name only")]
    pub logo_src: String,
    #[setting("Background", default = "#050709", check = colour)]
    pub theme_background: String,
    #[setting("Panel colour", default = "#151C22", check = colour)]
    pub theme_surface: String,
    #[setting("Text colour", default = "#F2F5F7", check = colour)]
    pub theme_foreground: String,
    #[setting("Soft text colour", default = "#A4ADB5", check = colour)]
    pub theme_muted: String,
    #[setting("Light", default = 1.0, range(min = 0, max = 2), step = 0.05, advanced)]
    pub light_intensity: f64,
    #[setting(
        "Music",
        zero_label = "None",
        help = "Plays through the Mixer like any other music.",
        widget = "sound"
    )]
    pub music: String,
    #[setting(
        "Music volume",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub music_volume: f64,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Text size",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        help = "Scales every line of text together.",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Statement weight",
        default = 500,
        range(min = 100, max = 800),
        step = 100,
        help = "The name is one step bolder.",
        advanced
    )]
    pub statement_weight: u32,
    #[setting(
        "Logo size",
        default = 34,
        range(min = 12, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub logo_size: u32,
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
                .look(NodeLook::of("lucide:rocket")),
        )
}
