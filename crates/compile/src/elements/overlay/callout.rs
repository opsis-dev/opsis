use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Callout {
    #[setting("Text", default = "Click here")]
    pub text: String,
    #[setting("Target X (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Target Y (0–1)", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Label side", default = Side::Top)]
    pub side: Side,
    #[setting("Label colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Arrow colour", default = "#f43f5e", check = colour)]
    pub accent: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Draw time",
        default = 0.6,
        range(min = 0.1, max = 3),
        unit = "s",
        advanced
    )]
    pub draw_time: f64,
    #[setting(
        "Label distance",
        default = 170,
        range(min = 40, max = 600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_distance: u32,
    #[setting("Arrow bend", default = 40, range(min = -200, max = 200), step = 1, unit = "px", advanced)]
    pub bend: i32,
    #[setting(
        "Arrow width",
        default = 7,
        range(min = 1, max = 30),
        step = 1,
        unit = "px",
        advanced
    )]
    pub arrow_width: u32,
    #[setting(
        "Dot size",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub dot_size: u32,
    #[setting(
        "Label corner radius",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub label_radius: u32,
    #[setting(
        "Text size",
        default = 34,
        range(min = 12, max = 120),
        step = 1,
        unit = "px"
    )]
    pub font_size: u32,
    #[setting(
        "Font weight",
        default = 700,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Arrow slant",
        default = 19.29,
        range(min = 0, max = 60),
        step = 0.5,
        unit = "°",
        advanced
    )]
    pub slant: f64,
    #[setting(
        "Fade out",
        default = 0.3333,
        range(min = 0, max = 2),
        unit = "s",
        advanced
    )]
    pub fade_out: f64,
    #[setting(
        "Label background",
        widget = "color",
        zero_label = "Accent",
        check = colour_or_empty,
        advanced
    )]
    pub label_background: String,
}

pub fn definition() -> ElementMeta {
    meta::<Callout>(
        element(
            ElementKind::Component,
            "callout",
            "Callout",
            "A short note that points something out.",
            ElementCategory::Overlay,
            75,
        )
        .look(NodeLook::of("lucide:message-square-quote")),
    )
}
