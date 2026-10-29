use opsis_macros::settings;
use opsis_types::looks::NodeLook;
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
pub struct Gradient {
    #[setting("From", default = "#1e1b4b", check = colour)]
    pub from: String,
    #[setting("To", default = "#0f172a", check = colour)]
    pub to: String,
    #[setting("Angle", default = 135, range(min = 0, max = 360), step = 1)]
    pub angle: u32,
    #[setting("Grain", default = 0.08, range(min = 0, max = 0.4), step = 0.01)]
    pub grain: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Gradient>(
        element(
            ElementKind::Component,
            "gradient",
            "Gradient",
            "A gradient background.",
            ElementCategory::Background,
            90,
        )
        .look(NodeLook::of("lucide:rainbow")),
    )
}
