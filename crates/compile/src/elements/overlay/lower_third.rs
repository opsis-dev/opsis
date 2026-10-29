use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Corner;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LowerThird {
    #[setting("Title", default = "Ada Lovelace")]
    pub title: String,
    #[setting("Subtitle", default = "Founder, Acme")]
    pub subtitle: String,
    #[setting("Position", default = Corner::BottomLeft)]
    pub position: Corner,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Panel colour", default = "rgba(10,10,12,0.72)", check = colour)]
    pub panel: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Exit time",
        default = 0.4667,
        range(min = 0, max = 2),
        unit = "s",
        advanced
    )]
    pub exit_time: f64,
    #[setting(
        "Slide distance",
        default = 120,
        range(min = 0, max = 600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub slide: u32,
    #[setting(
        "Margin",
        default = 80,
        range(min = 0, max = 300),
        step = 1,
        unit = "px",
        advanced
    )]
    pub margin: u32,
    #[setting(
        "Bar width",
        default = 8,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub bar_width: u32,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting(
        "Background blur",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_blur: u32,
    #[setting(
        "Title size",
        default = 44,
        range(min = 16, max = 120),
        step = 1,
        unit = "px"
    )]
    pub title_size: u32,
    #[setting(
        "Title weight",
        default = 720,
        range(min = 100, max = 900),
        step = 10,
        advanced
    )]
    pub title_weight: u32,
    #[setting(
        "Subtitle size",
        default = 28,
        range(min = 10, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub subtitle_size: u32,
    #[setting("Subtitle opacity", default = 0.7, range(min = 0, max = 1), advanced)]
    pub subtitle_opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LowerThird>(
        element(
            ElementKind::Component,
            "lower-third",
            "Lower third",
            "A name or caption bar in the lower part of the screen.",
            ElementCategory::Overlay,
            120,
        )
        .look(NodeLook::of("lucide:panel-bottom")),
    )
}
