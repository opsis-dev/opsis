use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{FlickerOrder, FontWeight, MaskReveal};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct HandwritingText {
    #[default = "Signed by hand"]
    #[schemars(description = "Text", extend("x-help" = "The line that gets written."))]
    pub text: String,
    #[default = 0.14]
    #[schemars(description = "Pen size", extend("x-step" = 0.01, "x-help" = "Diameter of the nib, in em. 0 hides it.", "x-advanced" = true))]
    pub pen_size: f64,
    #[default = "#e8b86d"]
    #[schemars(description = "Pen color", extend("x-help" = "Nib colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub pen_color: String,
    #[default = 0.18]
    #[schemars(description = "Ink softness", extend("x-step" = 0.01, "x-help" = "Softness of the ink edge, as a share of one glyph.", "x-advanced" = true))]
    pub ink_softness: f64,
    #[default = 1.6]
    #[schemars(description = "Wobble", extend("x-step" = 0.1, "x-help" = "Per-character tilt and baseline drift, in degrees. Fixed, not animated.", "x-advanced" = true))]
    pub wobble: f64,
    #[default = 3.0]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between one character and the next. This is the writing speed.", "x-advanced" = true))]
    pub stagger_in_frames: f64,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = "handwriting"]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: String,
    #[default = "#f4f4f5"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Font size", range(min = 12, max = 600), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "x-help" = "Leave empty to size the text to the frame.", "type" = "number"))]
    pub font_size: Option<f64>,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NeonFlickerText {
    #[default = "Open all night"]
    #[schemars(description = "Text", extend("x-help" = "The sign."))]
    pub text: String,
    #[default = "#f472b6"]
    #[schemars(description = "Glow color", extend("x-help" = "Colour of the halo around the tube.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub glow_color: String,
    #[default = 26.0]
    #[schemars(description = "Glow size", extend("x-help" = "Four stacked shadows, so it stays soft."))]
    pub glow_size: f64,
    #[default = 0.14]
    #[schemars(description = "Unlit brightness", extend("x-step" = 0.01, "x-help" = "Brightness of an unlit tube, 0–1.", "x-advanced" = true))]
    pub off_level: f64,
    #[default = 0.12]
    #[schemars(description = "Hum", extend("x-step" = 0.01, "x-help" = "Depth of the mains hum once the sign has settled.", "x-advanced" = true))]
    pub hum: f64,
    #[default = 0.06]
    #[schemars(description = "Buzz", extend("x-step" = 0.01, "x-help" = "Chance per beat that a settled tube stutters. 0 disables it.", "x-advanced" = true))]
    pub buzz: f64,
    #[default = true]
    #[schemars(description = "Flicker on exit", extend("x-help" = "Stutter back off instead of fading cleanly."))]
    pub flicker_on_exit: bool,
    #[default(FlickerOrder::Start)]
    #[schemars(description = "Order", extend("x-help" = "Which tube strikes first."))]
    pub order: FlickerOrder,
    #[default = 2.0]
    #[schemars(description = "Stagger", extend("x-unit" = "frames", "x-help" = "Frames between one tube striking and the next.", "x-advanced" = true))]
    pub stagger_in_frames: f64,
    #[default = 1.0]
    #[schemars(description = "Variation", extend("x-advanced" = true))]
    pub seed: f64,
    #[default = "#fdf2f8"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Font size", range(min = 12, max = 600), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "x-help" = "Leave empty to size the text to the frame.", "type" = "number"))]
    pub font_size: Option<f64>,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[schemars(description = "Unlit color", extend("x-widget" = "color", "x-zero-label" = "From the glow", "x-help" = "The tube with no gas lit: cold glass, not invisible. Leave empty for a faint glow colour."))]
    pub off_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TextMaskVideo {
    #[default = "LAUNCH\nDAY"]
    #[schemars(description = "Text", extend("x-widget" = "textarea", "x-help" = "The letterforms the media is seen through."))]
    pub text: String,
    #[schemars(description = "Media", extend("x-widget" = "asset", "x-accept" = "video/*,image/*", "x-zero-label" = "Gradient", "x-help" = "A video or image to show through the letters. Leave empty for the gradient."))]
    pub src: String,
    #[default = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)"]
    #[schemars(description = "Gradient fill", extend("x-widget" = "gradient", "x-show-if" = {"src":[""]}))]
    pub gradient: String,
    #[default(MaskReveal::Left)]
    #[schemars(description = "Reveal", extend("x-help" = "Which way the letters are uncovered."))]
    pub reveal: MaskReveal,
    #[default = 26.0]
    #[schemars(description = "Drift", extend("x-advanced" = true))]
    pub drift: f64,
    #[default = 0.12]
    #[schemars(description = "Zoom", extend("x-step" = 0.01, "x-help" = "Extra scale the media breathes through. 0 holds it still.", "x-advanced" = true))]
    pub zoom: f64,
    #[default = 30.0]
    #[schemars(description = "Animation length", range(min = 1, max = 300), extend("x-step" = 1, "x-unit" = "frames", "x-help" = "Length of the reveal.", "x-advanced" = true))]
    pub duration_in_frames: f64,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 800.0]
    #[schemars(description = "Font weight")]
    pub font_weight: f64,
    #[schemars(description = "Letter spacing")]
    pub letter_spacing: f64,
    #[default = 0.98]
    #[schemars(description = "Line height", extend("x-step" = 0.01))]
    pub line_height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Font size", range(min = 12, max = 600), extend("x-zero-label" = "Fit", "x-unit" = "px", "x-help" = "Leave empty to fit the text to the frame.", "x-advanced" = true, "type" = "number"))]
    pub font_size: Option<f64>,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<HandwritingText>(
            element(
                ElementKind::Component,
                "handwriting-text",
                "Handwriting",
                "Writes the text out by hand, stroke by stroke.",
                C::Text,
                120,
            )
            .source(
                "RemotionUI",
                "https://remotionui.com/docs/components/handwriting-text",
                "MIT",
            )
            .look(NodeLook::of("lucide:signature").moves(NodeMotion::Draw)),
        ),
        meta::<NeonFlickerText>(
            element(
                ElementKind::Component,
                "neon-flicker-text",
                "Neon sign",
                "Text that flickers on like a neon sign, then hums gently.",
                C::Text,
                120,
            )
            .source(
                "RemotionUI",
                "https://remotionui.com/docs/components/neon-flicker-text",
                "MIT",
            )
            .look(NodeLook::of("lucide:type").moves(NodeMotion::Flicker)),
        ),
        meta::<TextMaskVideo>(
            element(
                ElementKind::Component,
                "text-mask-video",
                "Text mask",
                "Big letters with a video, image or gradient showing through them.",
                C::Text,
                120,
            )
            .source(
                "RemotionUI",
                "https://remotionui.com/docs/components/text-mask-video",
                "MIT",
            )
            .kind_prop("media", "src")
            .look(NodeLook::of("lucide:type-outline")),
        ),
    ]
}
