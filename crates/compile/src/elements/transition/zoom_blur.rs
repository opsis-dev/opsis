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
pub struct ZoomBlur {
    #[default = 16]
    #[schemars(description = "Blur", range(min = 0, max = 40), extend("x-step" = 2))]
    pub blur: u32,
    #[schemars(description = "Rise", range(min = 0, max = 160), extend("x-step" = 10))]
    pub rise: u32,
}

pub fn definition() -> ElementMeta {
    meta::<ZoomBlur>(
            element(
                ElementKind::Presentation,
                "zoom-blur",
                "Zoom blur",
                "A depth punch-in transition that pushes the outgoing scene through the viewer while the incoming one resolves out of blur.",
                ElementCategory::Transition,
                114,
            )
                .look(NodeLook::of("lucide:expand").moves(NodeMotion::ZoomBlur)),
        )
}
