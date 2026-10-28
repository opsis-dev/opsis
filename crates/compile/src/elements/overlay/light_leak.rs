use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LightLeak {
    #[default = 3]
    #[schemars(description = "Seed", range(min = 0, max = 100), extend("x-step" = 1))]
    pub seed: u32,
    #[schemars(description = "Hue shift", range(min = 0, max = 360), extend("x-step" = 1))]
    pub hue_shift: u32,
}

pub fn definition() -> ElementMeta {
    meta::<LightLeak>(
        element(
            ElementKind::Component,
            "light-leak",
            "Light leak",
            "A warm light flare over the video.",
            ElementCategory::Overlay,
            45,
        )
        .look(NodeLook::of("lucide:sun-dim")),
    )
}
