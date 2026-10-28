use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::DarkOrLight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct DragDropFlow {
    #[default = "hero-take.mp4"]
    #[schemars(description = "File name", extend("x-help" = "File the cursor picks up."))]
    pub file_name: String,
    #[default = "48.2 MB"]
    #[schemars(
        description = "File size",
        extend("x-help" = "Size shown once the upload completes.")
    )]
    pub file_size: String,
    #[default(vec!["b-roll-rooftop.mov".into(), "interview-cam-b.mp4".into()])]
    #[schemars(description = "Other files", extend("x-help" = "Other rows in the source list."))]
    pub siblings: Vec<String>,
    #[default = "Media library"]
    #[schemars(description = "Library title", extend("x-help" = "Heading on the source panel."))]
    pub source_label: String,
    #[default = "Drop your clip"]
    #[schemars(description = "Label", extend("x-help" = "Idle prompt in the drop zone."))]
    pub label: String,
    #[default = "MP4 or MOV, up to 2 GB"]
    #[schemars(description = "Hint", extend("x-help" = "Second line under the prompt."))]
    pub hint: String,
    #[default = "Release to upload"]
    #[schemars(
        description = "Label while dragging",
        extend("x-help" = "Prompt while the file is held over the zone.")
    )]
    pub active_label: String,
    #[default = "Uploaded"]
    #[schemars(
        description = "Label when done",
        extend("x-help" = "Label once the upload finishes.")
    )]
    pub done_label: String,
    #[default = "#E8B86D"]
    #[schemars(
        description = "Accent color",
        extend(
            "x-help" = "Zone, cursor ring, and progress colour.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(
        description = "Speed",
        range(min = 0.25, max = 4),
        extend("x-step" = 0.25, "x-help" = "Animation speed multiplier.")
    )]
    pub speed: f64,
    #[default = "Uploading"]
    #[schemars(
        description = "Label while uploading",
        extend("x-help" = "Label while the file uploads.")
    )]
    pub uploading_label: String,
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Theme",
            "x-help" = "Leave empty to follow the theme."
        )
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
        .look(NodeLook::of("lucide:cloud-upload")),
    )
}
