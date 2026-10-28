use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{BlendMode, FontWeight};
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SoftBlurIn {
    #[default = "Made for every take."]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 12.0]
    #[schemars(description = "Blur", range(min = 1, max = 30), extend("x-step" = 1))]
    pub blur: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
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
pub struct Typewriter {
    #[default = "console.log('hello, world')"]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[default = true]
    #[schemars(description = "Show cursor")]
    pub cursor: bool,
    #[default = 22.0]
    #[schemars(description = "Chars / sec", range(min = 4, max = 60), extend("x-step" = 1))]
    pub chars_per_second: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "#171717"]
    #[schemars(description = "Cursor color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub cursor_color: String,
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
pub struct ShimmerSweep {
    #[default = "Generating"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = "#3f3f46"]
    #[schemars(description = "Base color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub base_color: String,
    #[default = "#fafafa"]
    #[schemars(description = "Shine color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub shine_color: String,
    #[default = 96.0]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: f64,
    #[default(FontWeight::W700)]
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
pub struct MatrixDecode {
    #[default = "DECRYPTED"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = "!@#$%^&*()_+-=<>?/\\|"]
    #[schemars(description = "Scramble characters", extend("x-zero-label" = "Built-in", "x-help" = "Characters the text flickers through before it settles."))]
    pub charset: String,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#22c55e"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 2.0]
    #[schemars(description = "Reveal time", range(min = 0.3333, max = 8), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub reveal_duration: f64,
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
pub struct SlotMachineRoll {
    #[default = "$99"]
    #[schemars(description = "From")]
    pub from: String,
    #[default = "$199"]
    #[schemars(description = "To")]
    pub to: String,
    #[default = 120.0]
    #[schemars(description = "Font size", range(min = 12, max = 240), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W700)]
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
pub struct StaggeredFadeUp {
    #[default = "Ship your demo in minutes"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 0.1333]
    #[schemars(description = "Stagger", range(min = 0, max = 1), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub stagger_delay: f64,
    #[default = 20.0]
    #[schemars(description = "Distance", range(min = 0, max = 120), extend("x-step" = 1))]
    pub distance: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
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
pub struct KineticCenterBuild {
    #[default = "Words push left."]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 88.0]
    #[schemars(description = "Entry offset", range(min = 20, max = 160), extend("x-step" = 1))]
    pub entry_offset: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
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
pub struct MaskRevealUp {
    #[default = "Designed to move.\nBuilt to focus."]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[default = 30.0]
    #[schemars(description = "Distance", range(min = 0, max = 120), extend("x-step" = 1))]
    pub distance: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
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
pub struct TrackingIn {
    #[default = "tracking in"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 0.5]
    #[schemars(description = "Start tracking (em)", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub start_tracking: f64,
    #[default = 12.0]
    #[schemars(description = "Start blur", range(min = 0, max = 40), extend("x-step" = 1))]
    pub start_blur: f64,
    #[default = 96.0]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W700)]
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
pub struct WordPush {
    #[default = "move at your own pace"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 10.0]
    #[schemars(description = "Frames / word", range(min = 3, max = 30), extend("x-step" = 1))]
    pub word_gap: f64,
    #[default = 0.8]
    #[schemars(description = "Gap shrink / word", range(min = 0.5, max = 1), extend("x-step" = 0.05))]
    pub accel: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W400)]
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
pub struct RgbGlitchText {
    #[default = "GLITCH"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 96.0]
    #[schemars(description = "Font size", range(min = 12, max = 200), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W700)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 0.6667]
    #[schemars(description = "Glitch at", range(min = 0, max = 4), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub glitch_at: f64,
    #[default = 0.2667]
    #[schemars(description = "Glitch length", range(min = 0.0333, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub glitch_duration: f64,
    #[default = 6.0]
    #[schemars(description = "Intensity (px)", range(min = 0, max = 30), extend("x-step" = 1))]
    pub intensity: f64,
    #[default = "glitch"]
    #[schemars(description = "Seed")]
    pub seed: String,
    #[default(vec!["#ff0040".into(), "#00ff80".into(), "#0080ff".into()])]
    #[schemars(description = "Glitch colours", length(min = 3, max = 3), extend("x-help" = "The three copies that split off while the text glitches."))]
    pub glitch_colors: Vec<Colour>,
    #[default(BlendMode::Multiply)]
    #[schemars(description = "Glitch blend", extend("x-help" = "How the split copies mix with what is behind them.", "x-labels" = {"multiply":"On a light background","screen":"On a dark background","normal":"On any background"}))]
    pub blend: BlendMode,
    #[default = "#fafafa"]
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
pub struct GradientScaleCutText {
    #[default = "Introducing"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 520.0]
    #[schemars(description = "Giant font size", range(min = 160, max = 720), extend("x-step" = 4))]
    pub giant_font_size: f64,
    #[default = 132.0]
    #[schemars(description = "Compact font size", range(min = 40, max = 240), extend("x-step" = 2))]
    pub compact_font_size: f64,
    #[default(-16.0)]
    #[schemars(description = "Final horizontal anchor offset", range(min = -400, max = 400), extend("x-step" = 4))]
    pub anchor_offset_x: f64,
    #[default = 400.0]
    #[schemars(description = "Giant phase left travel", range(min = 0, max = 800), extend("x-step" = 4))]
    pub giant_travel: f64,
    #[default = 160.0]
    #[schemars(description = "Compact entrance travel", range(min = 0, max = 400), extend("x-step" = 4))]
    pub settle_travel: f64,
    #[default = 0.4333]
    #[schemars(description = "Hard cut at", range(min = 0.1333, max = 0.8), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub cut_frame: f64,
    #[default = 14.0]
    #[schemars(description = "Reveal edge softness", range(min = 0, max = 40), extend("x-step" = 1))]
    pub reveal_softness: f64,
    #[default = 12.0]
    #[schemars(description = "Blur after the cut", range(min = 0, max = 40), extend("x-step" = 1))]
    pub compact_blur: f64,
    #[default = "#f04a14"]
    #[schemars(description = "Gradient start", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub gradient_start: String,
    #[default = "#f3f1f1"]
    #[schemars(description = "Gradient end", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub gradient_end: String,
    #[default = "#170b0a"]
    #[schemars(description = "Unrevealed text color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ghost_color: String,
    #[default = "#000000"]
    #[schemars(description = "Background color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[default(FontWeight::W700)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct StrikethroughReplace {
    #[default = "$49/mo"]
    #[schemars(description = "From")]
    pub from: String,
    #[default = "Free"]
    #[schemars(description = "To")]
    pub to: String,
    #[default = "#ff5e3a"]
    #[schemars(description = "Line color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub line_color: String,
    #[default = 96.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default(FontWeight::W700)]
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
pub struct LineByLineSlide {
    #[default = "Record once.\nShare everywhere."]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[default = 48.0]
    #[schemars(description = "Distance", range(min = 0, max = 160), extend("x-step" = 1))]
    pub distance: f64,
    #[default = 72.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#171717"]
    #[schemars(description = "Color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
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

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<SoftBlurIn>(
            element(
                ElementKind::Component,
                "soft-blur-in",
                "Soft blur in",
                "Per-character fade-in with a gentle blur and upward motion.",
                C::Text,
                60,
            )
                .source("Remocn", "https://remocn.dev/docs/components/soft-blur-in", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::BlurSwap)),
        ),
        meta::<Typewriter>(
            element(
                ElementKind::Component,
                "typewriter",
                "Typewriter",
                "Types the text out letter by letter with a blinking cursor.",
                C::Text,
                120,
            )
                .source("Remocn", "https://remocn.dev/docs/components/typewriter", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:text-cursor").moves(NodeMotion::Caret)),
        ),
        meta::<ShimmerSweep>(
            element(
                ElementKind::Component,
                "shimmer-sweep",
                "Shimmer sweep",
                "A band of light sweeps across the text.",
                C::Text,
                120,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shimmer-sweep", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Shimmer)),
        ),
        meta::<MatrixDecode>(
            element(
                ElementKind::Component,
                "matrix-decode",
                "Matrix decode",
                "Random scramble resolves left-to-right into the target text.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/matrix-decode", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:binary").morphs("lucide:type")),
        ),
        meta::<SlotMachineRoll>(
            element(
                ElementKind::Component,
                "slot-machine-roll",
                "Slot machine roll",
                "Vertical reel scrolls characters from one value to another.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/slot-machine-roll", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Roll)),
        ),
        meta::<StaggeredFadeUp>(
            element(
                ElementKind::Component,
                "staggered-fade-up",
                "Staggered fade up",
                "Words fade in and slide up one after another with a configurable delay.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/staggered-fade-up", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Rise)),
        ),
        meta::<KineticCenterBuild>(
            element(
                ElementKind::Component,
                "kinetic-center-build",
                "Kinetic center build",
                "A word appears in the center; each new word enters from the right and pushes the line until the full phrase locks centered.",
                C::Text,
                60,
            )
                .source("Remocn", "https://remocn.dev/docs/components/kinetic-center-build", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:text-align-center")),
        ),
        meta::<MaskRevealUp>(
            element(
                ElementKind::Component,
                "mask-reveal-up",
                "Mask reveal up",
                "Lines reveal upward with a soft masked feel and compact stagger.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/mask-reveal-up", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::MaskUp)),
        ),
        meta::<TrackingIn>(
            element(
                ElementKind::Component,
                "tracking-in",
                "Tracking in",
                "Letter-spacing collapses and blur clears with a springy motion.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/tracking-in", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Tracking)),
        ),
        meta::<WordPush>(
            element(
                ElementKind::Component,
                "word-push",
                "Word push",
                "A phrase builds from its first word centered: each next word drives in from the right and pushes the line left, easing into the new center.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/word-push", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Push)),
        ),
        meta::<RgbGlitchText>(
            element(
                ElementKind::Component,
                "rgb-glitch-text",
                "RGB glitch text",
                "Three RGB-offset text copies jitter for a brief glitch window.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/rgb-glitch-text", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Glitch)),
        ),
        meta::<GradientScaleCutText>(
            element(
                ElementKind::Component,
                "gradient-scale-cut-text",
                "Gradient scale cut text",
                "An oversized gradient reveal snaps to a separate compact blurred line that settles left while the reveal continues.",
                C::Text,
                36,
            )
                .source("Remocn", "https://remocn.dev/docs/components/gradient-scale-cut-text", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type").moves(NodeMotion::Snap)),
        ),
        meta::<StrikethroughReplace>(
            element(
                ElementKind::Component,
                "strikethrough-replace",
                "Strikethrough replace",
                "Draws a strike line across old text then reveals new text in its place.",
                C::Text,
                120,
            )
                .source("Remocn", "https://remocn.dev/docs/components/strikethrough-replace", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:strikethrough").morphs("lucide:type")),
        ),
        meta::<LineByLineSlide>(
            element(
                ElementKind::Component,
                "line-by-line-slide",
                "Line by line slide",
                "Each line enters from the left with a staggered slide and exits to the right for a flowing paragraph reveal.",
                C::Text,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/line-by-line-slide", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:text-align-start")),
        ),
    ]
}
