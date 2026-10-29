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
pub struct GaugeDial {
    #[setting("Value", default = 78.0, help = "Target the needle sweeps to.")]
    pub value: f64,
    #[setting("Min", help = "Value at the start of the arc.")]
    pub min: f64,
    #[setting("Max", default = 100.0, help = "Value at the end of the arc.")]
    pub max: f64,
    #[setting("Size", default = 480.0, help = "Type scales off it.")]
    pub size: f64,
    #[setting("Thickness", default = 26.0, help = "Track and fill thickness.")]
    pub thickness: f64,
    #[setting(
        "Sweep angle",
        default = 250,
        range(min = 10, max = 360),
        step = 1,
        help = "Total travel, centred on twelve o'clock. 180 gives a half-moon meter.",
        advanced
    )]
    pub sweep_in_degrees: u32,
    #[setting("Label", default = "Weekly goal", help = "Caption under the readout.")]
    pub label: String,
    #[setting("Unit", default = "%", help = "Suffix on the readout.")]
    pub unit: String,
    #[setting(
        "Tick count",
        default = 9,
        range(min = 0, max = 60),
        step = 1,
        help = "Tick marks around the arc. 0 hides them.",
        advanced
    )]
    pub tick_count: u32,
    #[setting(
        "Animation length",
        default = 44,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of the sweep.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting("Color", default = "#e8b86d", check = colour)]
    pub color: String,
    #[setting("Track color", default = "rgba(250,250,250,0.10)", check = colour)]
    pub track_color: String,
    #[setting("Text color", default = "#fafafa", check = colour)]
    pub ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.58)", check = colour)]
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
