use opsis_macros::settings;
use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct SwirlDissolve {
    #[setting("Bands", default = 10, range(min = 1, max = 12), step = 1)]
    pub band_count: u32,
    #[setting("Softness", default = 0.35, range(min = 0, max = 1), step = 0.05)]
    pub softness: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#1f1d29".into(), "#413d56".into(), "#8f88ae".into()],
        length(min = 2, max = 10)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SwirlDissolve>(
        element(
            ElementKind::Presentation,
            "swirl-dissolve",
            "Swirl dissolve",
            "A swirl shader transition that unwinds open and winds shut to reveal the next scene.",
            ElementCategory::Transition,
            116,
        )
        .look(NodeLook::of("tabler:spiral").moves(NodeMotion::Dissolve)),
    )
}
