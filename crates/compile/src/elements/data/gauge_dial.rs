use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GaugeDial {
    #[setting("Value", default = 78.0, help = "Target the needle sweeps to.")]
    pub value: f64,
    #[setting("Min", help = "Value at the start of the arc.", advanced)]
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
    #[setting("Track color", default = "rgba(250,250,250,0.10)", check = colour, advanced)]
    pub track_color: String,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.58)", check = colour, advanced)]
    pub label_color: String,
    #[setting("Rounded ends", default = true, advanced)]
    pub round_ends: bool,
    #[setting("Show needle", default = true, advanced)]
    pub show_needle: bool,
    #[setting("Capitalise label", default = true, advanced)]
    pub uppercase_label: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Decimals",
        range(min = 0, max = 3),
        zero_label = "From the value",
        step = 1,
        "type" = "integer",
        advanced
    )]
    pub decimals: Option<u32>,
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
