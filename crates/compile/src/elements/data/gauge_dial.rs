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
pub struct GaugeDial {
    #[default = 78.0]
    #[schemars(description = "Value", extend("x-help" = "Target the needle sweeps to."))]
    pub value: f64,
    #[schemars(description = "Min", extend("x-help" = "Value at the start of the arc."))]
    pub min: f64,
    #[default = 100.0]
    #[schemars(description = "Max", extend("x-help" = "Value at the end of the arc."))]
    pub max: f64,
    #[default = 480.0]
    #[schemars(description = "Size", extend("x-help" = "Type scales off it."))]
    pub size: f64,
    #[default = 26.0]
    #[schemars(description = "Thickness", extend("x-help" = "Track and fill thickness."))]
    pub thickness: f64,
    #[default = 250]
    #[schemars(
        description = "Sweep angle",
        range(min = 10, max = 360),
        extend(
            "x-step" = 1,
            "x-help" = "Total travel, centred on twelve o'clock. 180 gives a half-moon meter.",
            "x-advanced" = true
        )
    )]
    pub sweep_in_degrees: u32,
    #[default = "Weekly goal"]
    #[schemars(description = "Label", extend("x-help" = "Caption under the readout."))]
    pub label: String,
    #[default = "%"]
    #[schemars(description = "Unit", extend("x-help" = "Suffix on the readout."))]
    pub unit: String,
    #[default = 9]
    #[schemars(
        description = "Tick count",
        range(min = 0, max = 60),
        extend(
            "x-step" = 1,
            "x-help" = "Tick marks around the arc. 0 hides them.",
            "x-advanced" = true
        )
    )]
    pub tick_count: u32,
    #[default = 44]
    #[schemars(
        description = "Animation length",
        range(min = 1, max = 300),
        extend(
            "x-step" = 1,
            "x-unit" = "frames",
            "x-help" = "Length of the sweep.",
            "x-advanced" = true
        )
    )]
    pub duration_in_frames: u32,
    #[default = "#e8b86d"]
    #[schemars(
        description = "Color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[default = "rgba(250,250,250,0.10)"]
    #[schemars(
        description = "Track color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub track_color: String,
    #[default = "#fafafa"]
    #[schemars(
        description = "Text color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub ink_color: String,
    #[default = "rgba(250,250,250,0.58)"]
    #[schemars(
        description = "Label color",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub label_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<GaugeDial>(
        element(
            ElementKind::Component,
            "gauge-dial",
            "Gauge",
            "A dial whose needle sweeps up to a value.",
            ElementCategory::Data,
            120,
        )
        .look(NodeLook::of("lucide:gauge")),
    )
}
