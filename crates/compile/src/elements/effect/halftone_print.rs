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
pub struct HalftonePrint {
    #[default = 10]
    #[schemars(description = "Dot size", range(min = 4, max = 32), extend("x-step" = 1))]
    pub dot_size: u32,
    #[schemars(description = "Screen angle", range(min = 0, max = 90), extend("x-step" = 1))]
    pub angle: u32,
    #[default = 1.2]
    #[schemars(description = "Misregistration", range(min = 0, max = 6), extend("x-step" = 0.2))]
    pub misregistration: f64,
    #[default = "#f4efe4"]
    #[schemars(
        description = "Paper",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub paper_tint: String,
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
