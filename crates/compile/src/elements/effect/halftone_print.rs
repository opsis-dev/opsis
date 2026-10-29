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
pub struct HalftonePrint {
    #[setting("Dot size", default = 10, range(min = 4, max = 32), step = 1)]
    pub dot_size: u32,
    #[setting("Screen angle", range(min = 0, max = 90), step = 1, advanced)]
    pub angle: u32,
    #[setting(
        "Misregistration",
        default = 1.2,
        range(min = 0, max = 6),
        step = 0.2,
        advanced
    )]
    pub misregistration: f64,
    #[setting("Paper", default = "#f4efe4", check = colour)]
    pub paper_tint: String,
    #[setting("Cyan ink", default = "#14e0fa", widget = "color", check = colour, advanced)]
    pub cyan_ink: String,
    #[setting("Magenta ink", default = "#e61ff0", widget = "color", check = colour, advanced)]
    pub magenta_ink: String,
    #[setting("Yellow ink", default = "#f7e61a", widget = "color", check = colour, advanced)]
    pub yellow_ink: String,
}

pub fn definition() -> ElementMeta {
    meta::<HalftonePrint>(
            element(
                ElementKind::Wrapper,
                "halftone-print",
                "Halftone print",
                "Print a scene onto paper: each colour separates onto its own rotated screen of dots sized by the ink under them, slightly out of register the way a real press runs.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("tabler:grid-dots")),
        )
}
