use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::backdrops::Backdrop;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StudioBackdrop {
    #[setting("Backdrop", default = Backdrop::Glaze)]
    pub name: Backdrop,
}

pub fn definition() -> ElementMeta {
    meta::<StudioBackdrop>(
        element(
            ElementKind::Component,
            "backdrop",
            "Studio backdrop",
            "A ready-made studio background image.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:wallpaper")),
    )
}
