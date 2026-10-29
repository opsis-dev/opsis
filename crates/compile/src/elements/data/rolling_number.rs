use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct RollingNumber {
    #[setting("From", range(min = 0, max = 999_999_999_999_u64), step = 1)]
    #[specta(type = f64)]
    pub from: u64,
    #[setting(
        "To",
        default = 24813,
        range(min = 0, max = 999_999_999_999_u64),
        step = 1
    )]
    #[specta(type = f64)]
    pub to: u64,
    #[setting("Font size", default = 120, range(min = 24, max = 280), step = 4)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting(
        "Background",
        default = "white",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RollingNumber>(
            element(
                ElementKind::Component,
                "rolling-number",
                "Rolling number",
                "Odometer where every place rolls at its own speed, new places rising into view, settling exactly on the target value.",
                ElementCategory::Data,
                150,
            )
                .authored_clock()
                .design(1920, 1080)
                .fonts(&["JetBrains Mono"])
                .look(NodeLook::of("lucide:hash").moves(NodeMotion::Roll)),
        )
}
