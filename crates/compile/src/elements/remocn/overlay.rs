use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{FontWeight, KeyboardPlatform, LightOrDark};
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct KeystrokeStep {
    #[default = "mod+k"]
    #[schemars(description = "Keys")]
    pub keys: String,
    #[schemars(description = "At", range(min = 0, max = 120), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Hold", range(min = 0.0333, max = 20), extend("x-zero-label" = "Auto", "x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30, "type" = "number"))]
    pub hold: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MarkerHighlight {
    #[default = "Made for "]
    #[schemars(description = "Before")]
    pub before: String,
    #[default = "builders"]
    #[schemars(description = "Highlight")]
    pub highlight: String,
    #[default = "."]
    #[schemars(description = "After")]
    pub after: String,
    #[default = "#facc15"]
    #[schemars(description = "Marker color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub marker_color: String,
    #[default = "#171717"]
    #[schemars(description = "Base color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub base_color: String,
    #[default = "#171717"]
    #[schemars(description = "Highlighted text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub highlighted_text_color: String,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = "white"]
    #[schemars(description = "Background", extend("x-help" = "Set it to transparent to lay the text over other layers.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct InlineHighlight {
    #[default = "Ship faster with "]
    #[schemars(description = "Before")]
    pub before: String,
    #[default = "Acme"]
    #[schemars(description = "Highlight")]
    pub highlight: String,
    #[default = "."]
    #[schemars(description = "After")]
    pub after: String,
    #[default = "#171717"]
    #[schemars(description = "Base color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub base_color: String,
    #[default = "#ff5e3a"]
    #[schemars(description = "Highlight color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub highlight_color: String,
    #[default = 72]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: u32,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = "white"]
    #[schemars(description = "Background", extend("x-help" = "Set it to transparent to lay the text over other layers.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct InkUnderline {
    #[default = 420]
    #[schemars(description = "Width", range(min = 100, max = 800), extend("x-step" = 10))]
    pub width: u32,
    #[default = "#6f7f35"]
    #[schemars(description = "Ink color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = 9]
    #[schemars(description = "Brush width", range(min = 2, max = 24), extend("x-step" = 1))]
    pub thickness: u32,
    #[default = 1.0]
    #[schemars(description = "Opening pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub pressure: f64,
    #[default = 0.15]
    #[schemars(description = "Closing pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub release: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 5]
    #[schemars(description = "Poses to draw", range(min = 2, max = 10), extend("x-step" = 1))]
    pub duration_steps: u32,
    #[default = "ink"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default = 0.1]
    #[schemars(description = "Time per pose", range(min = 0.0333, max = 0.2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub step: f64,
    #[default = 0.5]
    #[schemars(description = "Where", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub x: f64,
    #[default = 0.62]
    #[schemars(description = "Where vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ScribbleCircle {
    #[default = 1.15]
    #[schemars(description = "Laps", range(min = 0.5, max = 3), extend("x-step" = 0.05))]
    pub laps: f64,
    #[default = 14]
    #[schemars(description = "Brush width", range(min = 4, max = 32), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = 0.2]
    #[schemars(description = "Opening pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub pressure: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 10]
    #[schemars(description = "Poses to draw", range(min = 2, max = 20), extend("x-step" = 1))]
    pub duration_steps: u32,
    #[default = "#6f7f35"]
    #[schemars(description = "Ink color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "scribble"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default = 0.1]
    #[schemars(description = "Time per pose", range(min = 0.0333, max = 0.2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub step: f64,
    #[default = 0.5]
    #[schemars(description = "Around", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Around vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub y: f64,
    #[default = 420]
    #[schemars(description = "Width", range(min = 20, max = 3840), extend("x-step" = 10, "x-unit" = "px"))]
    pub width: u32,
    #[default = 220]
    #[schemars(description = "Height", range(min = 20, max = 2160), extend("x-step" = 10, "x-unit" = "px"))]
    pub height: u32,
    #[default = 1.0]
    #[schemars(description = "Closing pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub release: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct InkArrow {
    #[default = 0.35]
    #[schemars(description = "Curvature", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub curvature: f64,
    #[default = 8]
    #[schemars(description = "Brush width", range(min = 2, max = 24), extend("x-step" = 1))]
    pub stroke_width: u32,
    #[default = 0.2]
    #[schemars(description = "Opening pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub pressure: f64,
    #[default = 1.0]
    #[schemars(description = "Closing pressure", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub release: f64,
    #[default = 1.0]
    #[schemars(description = "Grain", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub grain: f64,
    #[default = 24]
    #[schemars(description = "Head size", range(min = 8, max = 48), extend("x-step" = 1))]
    pub head_size: u32,
    #[default = 1.2]
    #[schemars(description = "Draw time", range(min = 0.4, max = 3), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub draw_dur: f64,
    #[default = 0.4]
    #[schemars(description = "Arrowhead time", range(min = 0.1333, max = 1.6), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub head_dur: f64,
    #[default = "#26242c"]
    #[schemars(description = "Ink color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "arrow"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default = 0.1]
    #[schemars(description = "Time per pose", range(min = 0.0333, max = 0.2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub step: f64,
    #[default = 0.3]
    #[schemars(description = "From", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub from_x: f64,
    #[default = 0.72]
    #[schemars(description = "From vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub from_y: f64,
    #[default = 0.48]
    #[schemars(description = "To", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub to_x: f64,
    #[default = 0.5]
    #[schemars(description = "To vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub to_y: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Keystroke {
    #[default(KeyboardPlatform::Mac)]
    #[schemars(description = "Platform")]
    pub platform: KeyboardPlatform,
    #[default(LightOrDark::Light)]
    #[schemars(description = "Theme")]
    pub theme: LightOrDark,
    #[default = "#0a84ff"]
    #[schemars(description = "Accent", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent: String,
    #[default = 64]
    #[schemars(description = "Key size", range(min = 32, max = 128), extend("x-step" = 1))]
    pub size: u32,
    #[default = 1.0]
    #[schemars(description = "Linger", range(min = 0, max = 3), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub linger: f64,
    #[default = 0.5]
    #[schemars(description = "Where", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub x: f64,
    #[default = 0.86]
    #[schemars(description = "Where vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub y: f64,
    #[default(vec![
        KeystrokeStep { at: 0.6667, ..Default::default() },
        KeystrokeStep { at: 2.6, keys: "g i".into(), ..Default::default() },
        KeystrokeStep { at: 3.8, keys: "shift+mod+p".into(), ..Default::default() },
    ])]
    #[schemars(description = "Keys", extend("x-help" = "Join keys pressed together with +, like mod+k. Separate keys pressed one after another with a space."))]
    pub steps: Vec<KeystrokeStep>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Confetti {
    #[default = 140]
    #[schemars(description = "Particles", range(min = 10, max = 400), extend("x-step" = 10))]
    pub particle_count: u32,
    #[default = 17]
    #[schemars(description = "Power", range(min = 4, max = 40), extend("x-step" = 1))]
    pub power: u32,
    #[default = 0.45]
    #[schemars(description = "Gravity", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub gravity: f64,
    #[default = 13]
    #[schemars(description = "Size", range(min = 4, max = 30), extend("x-step" = 1))]
    pub size: u32,
    #[default = 1]
    #[schemars(description = "Seed", range(min = 1), extend("x-step" = 1))]
    pub seed: u32,
    #[default(vec!["#1d9bf0".into(), "#ff5da2".into(), "#ffd23f".into(), "#22c55e".into(), "#a855f7".into(), "#ff7a45".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 12))]
    pub colors: Vec<Colour>,
    #[default = 0.5]
    #[schemars(description = "Burst from", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "Click the frame to choose the spot.", "x-widget" = "spot"))]
    pub origin_x: f64,
    #[default = 0.5]
    #[schemars(description = "Burst from vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub origin_y: f64,
    #[default = 3.0]
    #[schemars(description = "How long pieces last", range(min = 0.3333, max = 20), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub lifetime: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RadialBurst {
    #[default = 8]
    #[schemars(description = "Segments", range(min = 4, max = 16), extend("x-step" = 1))]
    pub segments: u32,
    #[default = 210]
    #[schemars(description = "Burst radius", range(min = 100, max = 280), extend("x-step" = 5))]
    pub radius: u32,
    #[default = 30]
    #[schemars(description = "Thickness", range(min = 8, max = 60), extend("x-step" = 1))]
    pub thickness: u32,
    #[default = 135]
    #[schemars(description = "Rotation", range(min = -360, max = 360), extend("x-step" = 15))]
    pub rotation: i32,
    #[default = 1.0]
    #[schemars(description = "Intensity", range(min = 0, max = 1.5), extend("x-step" = 0.1))]
    pub intensity: f64,
    #[default = 110]
    #[schemars(description = "Ribbon twist", range(min = -180, max = 180), extend("x-step" = 10))]
    pub twist: i32,
    #[default = 3]
    #[schemars(description = "Contour echoes", range(min = 0, max = 5), extend("x-step" = 1))]
    pub echoes: u32,
    #[default = "#ffffff"]
    #[schemars(description = "Shape color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "#f4e4a7"]
    #[schemars(description = "Accent color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = "#a800b7"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[schemars(description = "Loop animation")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<MarkerHighlight>(
            element(
                ElementKind::Component,
                "marker-highlight",
                "Marker highlight",
                "Colored marker block draws behind a phrase as the text color shifts.",
                C::Overlay,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/marker-highlight", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:highlighter")),
        ),
        meta::<InlineHighlight>(
            element(
                ElementKind::Component,
                "inline-highlight",
                "Inline highlight",
                "Animates one word inside a sentence from a base color to a brand color.",
                C::Overlay,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/inline-highlight", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:square-dashed-text")),
        ),
        meta::<InkUnderline>(
            element(
                ElementKind::Component,
                "ink-underline",
                "Ink underline",
                "A hand-dragged brush underline that sweeps itself across in a few stop-motion poses, thinning to a dry tail.",
                C::Overlay,
                120,
            )
                .source("Remocn", "https://remocn.dev/docs/components/ink-underline", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:underline").moves(NodeMotion::Draw)),
        ),
        meta::<ScribbleCircle>(
            element(
                ElementKind::Component,
                "scribble-circle",
                "Scribble circle",
                "A brush circle swept around a region of the frame, opening thin and closing thick, with grain breaking up the dry opening.",
                C::Overlay,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/scribble-circle", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:lasso").moves(NodeMotion::Draw)),
        ),
        meta::<InkArrow>(
            element(
                ElementKind::Component,
                "ink-arrow",
                "Ink arrow",
                "A hand-drawn brush arrow that sweeps itself from one point to another, thickening into the head that pops on the final poses.",
                C::Overlay,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/ink-arrow", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("tabler:arrow-curve-right").moves(NodeMotion::Draw)),
        ),
        meta::<Keystroke>(
            element(
                ElementKind::Component,
                "keystroke",
                "Keystroke",
                "Shows a keyboard shortcut as keycaps being pressed.",
                C::Overlay,
                180,
            )
                .source("Remocn", "https://remocn.dev/docs/components/keystroke", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:command")),
        ),
        meta::<Confetti>(
            element(
                ElementKind::Component,
                "confetti",
                "Confetti",
                "A burst of confetti to celebrate a moment.",
                C::Overlay,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/confetti", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:party-popper")),
        ),
        meta::<RadialBurst>(
            element(
                ElementKind::Component,
                "radial-burst",
                "Radial burst",
                "Sculptural ribbons twist into a tilted ring and sweep outward through contour echoes for an expressive product introduction.",
                C::Overlay,
                120,
            )
                .source("Remocn", "https://remocn.dev/docs/components/radial-burst", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:sparkle")),
        ),
    ]
}
