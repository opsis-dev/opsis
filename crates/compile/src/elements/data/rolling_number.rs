use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct RollingNumber {
    #[schemars(
        description = "From",
        range(min = 0, max = 999_999_999_999_u64),
        extend("x-step" = 1)
    )]
    pub from: u64,
    #[default = 24813]
    #[schemars(description = "To", range(min = 0, max = 999_999_999_999_u64), extend("x-step" = 1))]
    pub to: u64,
    #[default = 120]
    #[schemars(description = "Font size", range(min = 24, max = 280), extend("x-step" = 4))]
    pub font_size: u32,
    #[default = "#171717"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "white"]
    #[schemars(
        description = "Background",
        extend(
            "x-help" = "Set it to transparent to lay the text over other layers.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub background: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
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
                .look(NodeLook::of("lucide:hash").moves(NodeMotion::Roll)),
        )
}
