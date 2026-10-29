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
pub struct Solid {
    #[setting("Colour", default = "#09090b", check = colour)]
    pub color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Solid>(
        element(
            ElementKind::Component,
            "solid",
            "Solid colour",
            "A plain colour background.",
            ElementCategory::Background,
            90,
        )
        .look(NodeLook::of("lucide:paint-bucket")),
    )
}
