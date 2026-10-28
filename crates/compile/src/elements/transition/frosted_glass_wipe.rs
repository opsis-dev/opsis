use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum WipeFrom {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct FrostedGlassWipe {
    #[default = 20.0]
    #[schemars(description = "Blur", extend("x-help" = "Frost panel blur radius."))]
    pub blur: f64,
    #[default = 0.14]
    #[schemars(
        description = "Panel width",
        extend(
            "x-step" = 0.01,
            "x-help" = "Sweep panel width as fraction of frame.",
            "x-advanced" = true
        )
    )]
    pub panel_width: f64,
    #[default(WipeFrom::FromLeft)]
    #[schemars(description = "Direction", extend("x-help" = "Sweep direction."))]
    pub direction: WipeFrom,
    #[default = "rgba(255,255,255,0.12)"]
    #[schemars(
        description = "Frost color",
        extend(
            "x-help" = "Tint of the glass panel.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub frost_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<FrostedGlassWipe>(
        element(
            ElementKind::Presentation,
            "frosted-glass-wipe",
            "Frosted glass wipe",
            "A pane of frosted glass crosses the frame and leaves the next scene behind it.",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("tabler:blur").moves(NodeMotion::WipeReveal)),
    )
}
