use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DragDropFlow {
    #[setting(
        "File name",
        default = "hero-take.mp4",
        help = "File the cursor picks up."
    )]
    pub file_name: String,
    #[setting(
        "File size",
        default = "48.2 MB",
        help = "Size shown once the upload completes."
    )]
    pub file_size: String,
    #[setting(
        "Other files",
        default = vec!["b-roll-rooftop.mov".into(), "interview-cam-b.mp4".into()],
        help = "Other rows in the source list."
    )]
    pub siblings: Vec<String>,
    #[setting(
        "Library title",
        default = "Media library",
        help = "Heading on the source panel."
    )]
    pub source_label: String,
    #[setting(
        "Label",
        default = "Drop your clip",
        help = "Idle prompt in the drop zone."
    )]
    pub label: String,
    #[setting(
        "Hint",
        default = "MP4 or MOV, up to 2 GB",
        help = "Second line under the prompt."
    )]
    pub hint: String,
    #[setting(
        "Label while dragging",
        default = "Release to upload",
        help = "Prompt while the file is held over the zone."
    )]
    pub active_label: String,
    #[setting(
        "Label when done",
        default = "Uploaded",
        help = "Label once the upload finishes."
    )]
    pub done_label: String,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Zone, cursor ring, and progress colour.",
        check = colour
    )]
    pub accent_color: String,
    #[setting("Theme", default = DarkOrLight::Dark, help = "Surface palette.")]
    pub theme: DarkOrLight,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Label while uploading",
        default = "Uploading",
        help = "Label while the file uploads."
    )]
    pub uploading_label: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<DragDropFlow>(
        element(
            ElementKind::Component,
            "drag-drop-flow",
            "File upload",
            "A pointer drags a file into an upload area, which catches it and uploads it.",
            ElementCategory::Ui,
            150,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:cloud-upload")),
    )
}
