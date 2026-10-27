use indexmap::IndexMap;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::backdrops::Backdrop;
use crate::clip::PlaceholderProps;
use crate::elements::{ElementCategory, ElementKind, ElementMeta, ElementSource};
use crate::nodes::capture::StillProps;
use crate::params::{ParamSchema, Values, to_values};

pub const CAPTIONS_ELEMENT: &str = "core/captions";

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(transparent)]
#[schemars(extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
pub struct Colour(pub String);

impl From<&str> for Colour {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MarkLook {
    Underline,
    Circle,
    Box,
    Marker,
    CrossedOff,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum TitleAlign {
    Center,
    Left,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Entrance {
    Rise,
    Fade,
    Blur,
    Scale,
    None,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum CaptionStyle {
    WordByWord,
    Karaoke,
    Line,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum CaptionPosition {
    Bottom,
    Center,
    Top,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Contain,
    Cover,
    Fill,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum AnimationFit {
    Contain,
    Cover,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum Motion {
    Spin,
    TiltIn,
    Float,
    Orbit,
    Swing,
    Still,
    Keyframes,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ModelMotion {
    Spin,
    Orbit,
    Swing,
    Float,
    Still,
    Keyframes,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum Lighting {
    Studio,
    Soft,
    Dramatic,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Center,
    Left,
    Right,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ScreenDevice {
    Laptop,
    Monitor,
    Phone,
    Floating,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ScreenMove {
    PushIn,
    Orbit,
    TiltReveal,
    Rise,
    Keyframes,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum WipeDirection {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
    FromTopLeft,
    FromTopRight,
    FromBottomLeft,
    FromBottomRight,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Title {
    #[schemars(description = "Eyebrow")]
    pub eyebrow: String,
    #[default = "Sign up in 10 seconds"]
    #[schemars(description = "Title")]
    pub title: String,
    #[schemars(description = "Subtitle")]
    pub subtitle: String,
    #[schemars(
        description = "Mark a word",
        extend("x-help" = "Words from the title to draw a hand-drawn mark on, after the title comes in. Leave empty for none.")
    )]
    pub mark: String,
    #[default(MarkLook::Underline)]
    #[schemars(
        description = "Mark",
        extend("x-labels" = { "underline": "Underline", "circle": "Circle", "box": "Box", "marker": "Marker", "crossed-off": "Crossed out" })
    )]
    pub mark_look: MarkLook,
    #[default = "#facc15"]
    #[schemars(description = "Mark colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub mark_color: String,
    #[default(TitleAlign::Center)]
    #[schemars(description = "Alignment")]
    pub align: TitleAlign,
    #[default(Entrance::Rise)]
    #[schemars(description = "Entrance")]
    pub entrance: Entrance,
    #[default = "#fafafa"]
    #[schemars(description = "Text colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = "#a78bfa"]
    #[schemars(description = "Accent", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub accent: String,
    #[default = "transparent"]
    #[schemars(description = "Background (colour, gradient or 'transparent')")]
    pub background: String,
    #[default = 112.0]
    #[schemars(description = "Font size", range(min = 24, max = 260), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 760.0]
    #[schemars(description = "Font weight", range(min = 100, max = 900), extend("x-step" = 10))]
    pub font_weight: f64,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct KineticHeadline {
    #[default = "Ship *faster* with demos that *move*"]
    #[schemars(description = "Text — wrap words in *asterisks* to emphasise")]
    pub text: String,
    #[default = "#fafafa"]
    #[schemars(description = "Text colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = "#facc15"]
    #[schemars(description = "Emphasis colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub emphasis_color: String,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
    #[default = 104.0]
    #[schemars(description = "Font size", range(min = 24, max = 240), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 4.0]
    #[schemars(description = "Time between words", range(min = 1, max = 20), extend("x-step" = 1, "x-unit" = "frames"))]
    pub word_stagger: f64,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LowerThird {
    #[default = "Ada Lovelace"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Founder, Acme"]
    #[schemars(description = "Subtitle")]
    pub subtitle: String,
    #[default(Corner::BottomLeft)]
    #[schemars(description = "Position")]
    pub position: Corner,
    #[default = "#a78bfa"]
    #[schemars(description = "Accent", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub accent: String,
    #[default = "#fafafa"]
    #[schemars(description = "Text colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = "rgba(10,10,12,0.72)"]
    #[schemars(description = "Panel colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub panel: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Callout {
    #[default = "Click here"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = 0.5]
    #[schemars(description = "Target X (0–1)", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub x: f64,
    #[default = 0.5]
    #[schemars(description = "Target Y (0–1)", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub y: f64,
    #[default(Side::Top)]
    #[schemars(description = "Label side")]
    pub side: Side,
    #[default = "#fafafa"]
    #[schemars(description = "Label colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = "#f43f5e"]
    #[schemars(description = "Arrow colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub accent: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Captions {
    #[default(CaptionStyle::WordByWord)]
    #[schemars(description = "Style")]
    pub style: CaptionStyle,
    #[default(CaptionPosition::Bottom)]
    #[schemars(description = "Position")]
    pub position: CaptionPosition,
    #[default = 56.0]
    #[schemars(description = "Font size", range(min = 16, max = 140), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "#ffffff"]
    #[schemars(description = "Text colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = "#facc15"]
    #[schemars(description = "Highlight colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub highlight_color: String,
    #[default = "rgba(0,0,0,0.35)"]
    #[schemars(
        description = "Panel colour",
        extend(
            "x-help" = "The box behind the words. Lower its opacity to let the video show through.",
            "x-check" = { "rule": "colour", "message": "Pick a colour" }
        )
    )]
    pub panel: String,
    #[default = 800.0]
    #[schemars(description = "Font weight", range(min = 100, max = 900), extend("x-step" = 10))]
    pub font_weight: f64,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
}

impl crate::params::Params for Captions {}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Image {
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub src: String,
    #[default(Fit::Contain)]
    #[schemars(description = "Fit")]
    pub fit: Fit,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
    #[schemars(description = "Slow push-in")]
    pub ken_burns: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct MosaicBox {
    #[schemars(range(min = 0, max = 1))]
    pub x: f64,
    #[schemars(range(min = 0, max = 1))]
    pub y: f64,
    #[schemars(range(min = 0, max = 1))]
    pub width: f64,
    #[schemars(range(min = 0, max = 1))]
    pub height: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Mosaic {
    #[default(vec![MosaicBox { x: 0.62, y: 0.06, width: 0.3, height: 0.08 }])]
    #[schemars(
        description = "Areas to cover",
        length(max = 8),
        extend(
            "x-widget" = "boxes",
            "x-time-key" = "frameMs",
            "x-help" = "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Blur areas on the recording instead."
        )
    )]
    pub boxes: Vec<MosaicBox>,
    #[schemars(description = "Frame to draw on", range(min = 0, max = 3600000), extend("x-step" = 1, "x-unit" = "ms", "x-hidden" = true))]
    pub frame_ms: f64,
    #[default = 24.0]
    #[schemars(description = "Block size", range(min = 6, max = 72), extend("x-step" = 2))]
    pub block_size: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Lottie {
    #[schemars(description = "Animation file", extend("x-widget" = "asset", "x-accept" = ".json,.lottie"))]
    pub src: String,
    #[default = true]
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
    #[schemars(description = "Play backwards")]
    pub backwards: bool,
    #[default(AnimationFit::Contain)]
    #[schemars(description = "Fit", extend("x-labels" = { "contain": "Show all", "cover": "Fill the frame" }))]
    pub fit: AnimationFit,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Video {
    #[schemars(description = "Video", extend("x-widget" = "asset", "x-accept" = "video/*"))]
    pub src: String,
    #[default(Fit::Cover)]
    #[schemars(description = "Fit")]
    pub fit: Fit,
    #[schemars(description = "Loop")]
    pub r#loop: bool,
    #[default = true]
    #[schemars(description = "Muted")]
    pub muted: bool,
    #[default = 1.0]
    #[schemars(description = "Volume", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub volume: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.05))]
    pub playback_rate: f64,
    #[schemars(description = "Start at", range(min = 0, max = 3600000), extend("x-step" = 100, "x-unit" = "ms"))]
    pub trim_before_ms: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Solid {
    #[default = "#09090b"]
    #[schemars(description = "Colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Gradient {
    #[default = "#1e1b4b"]
    #[schemars(description = "From", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub from: String,
    #[default = "#0f172a"]
    #[schemars(description = "To", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub to: String,
    #[default = 135.0]
    #[schemars(description = "Angle", range(min = 0, max = 360), extend("x-step" = 1))]
    pub angle: f64,
    #[default = 0.08]
    #[schemars(description = "Grain", range(min = 0, max = 0.4), extend("x-step" = 0.01))]
    pub grain: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Noise {
    #[default = "#0b1020"]
    #[schemars(description = "Base", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color1: String,
    #[default = "#4f46e5"]
    #[schemars(description = "Highlight", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color2: String,
    #[default = 0.004]
    #[schemars(description = "Scale", range(min = 0.0005, max = 0.02), extend("x-step" = 0.0005))]
    pub scale: f64,
    #[default = 0.004]
    #[schemars(description = "Speed", range(min = 0, max = 0.05), extend("x-step" = 0.001))]
    pub speed: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Starburst {
    #[default = "#18181b"]
    #[schemars(description = "Ray A", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color1: String,
    #[default = "#27272a"]
    #[schemars(description = "Ray B", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color2: String,
    #[default = 24.0]
    #[schemars(description = "Rays", range(min = 6, max = 64), extend("x-step" = 1))]
    pub rays: f64,
    #[default = 0.15]
    #[schemars(description = "Rotation speed", range(min = 0, max = 2), extend("x-step" = 0.05, "x-unit" = "°/frame"))]
    pub rotation_speed: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Paper {
    #[default = "#f5f1e8"]
    #[schemars(description = "Paper colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = 0.18]
    #[schemars(description = "Grain", range(min = 0, max = 0.5), extend("x-step" = 0.01))]
    pub grain: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct StudioBackdrop {
    #[default(Backdrop::Glaze)]
    #[schemars(description = "Backdrop")]
    pub name: Backdrop,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct VideoLoop {
    #[schemars(description = "Video", extend("x-widget" = "asset", "x-accept" = "video/*"))]
    pub src: String,
    #[default = 0.3]
    #[schemars(description = "Dim", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub dim: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LightLeak {
    #[default = 3.0]
    #[schemars(description = "Seed", range(min = 0, max = 100), extend("x-step" = 1))]
    pub seed: f64,
    #[schemars(description = "Hue shift", range(min = 0, max = 360), extend("x-step" = 1))]
    pub hue_shift: f64,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Logo3d {
    #[schemars(
        description = "Logo file",
        extend(
            "x-widget" = "asset",
            "x-accept" = ".svg,.png,.jpg,.jpeg,.webp",
            "x-zero-label" = "Sample logo",
            "x-help" = "A drawn logo is given depth; a picture of a logo goes on a card."
        )
    )]
    pub src: String,
    #[default = 0.25]
    #[schemars(description = "Depth", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub depth: f64,
    #[schemars(description = "Colour", extend("x-widget" = "color", "x-zero-label" = "The logo's own colours"))]
    pub color: String,
    #[default = 0.3]
    #[schemars(description = "Metal", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub metalness: f64,
    #[default(Motion::Spin)]
    #[schemars(
        description = "Motion",
        extend("x-labels" = { "spin": "Spin", "tilt-in": "Tilt in", "float": "Float", "orbit": "Orbit", "swing": "Swing", "still": "Still", "keyframes": "Keyframes" })
    )]
    pub motion: Motion,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "motion": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[default(Lighting::Studio)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Card3d {
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Sample screen"))]
    pub src: String,
    #[default = 0.04]
    #[schemars(description = "Rounded corners", range(min = 0, max = 0.2), extend("x-step" = 0.01))]
    pub corner_radius: f64,
    #[default = 0.04]
    #[schemars(description = "Thickness", range(min = 0.01, max = 0.3), extend("x-step" = 0.01))]
    pub depth: f64,
    #[default = 40.0]
    #[schemars(
        description = "Tilt",
        range(min = 0, max = 80),
        extend("x-step" = 1, "x-unit" = "°", "x-help" = "How far the card is turned before it settles.")
    )]
    pub tilt: f64,
    #[default(Motion::TiltIn)]
    #[schemars(
        description = "Motion",
        extend("x-labels" = { "spin": "Spin", "tilt-in": "Tilt in", "float": "Float", "orbit": "Orbit", "swing": "Swing", "still": "Still", "keyframes": "Keyframes" })
    )]
    pub motion: Motion,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "motion": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[default(Lighting::Soft)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreparedTypeface {
    pub typeface: String,
    pub typeface_italic: bool,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Text3d {
    #[default = "Ship faster"]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Empty uses the project's heading font."))]
    pub font_family: String,
    #[default = 800.0]
    #[schemars(description = "Font weight", range(min = 100, max = 900), extend("x-step" = 10))]
    pub font_weight: f64,
    #[schemars(description = "Italic")]
    pub italic: bool,
    #[default = 0.35]
    #[schemars(description = "Depth", range(min = 0.05, max = 1), extend("x-step" = 0.05))]
    pub depth: f64,
    #[default = 0.4]
    #[schemars(description = "Rounded edges", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bevel: f64,
    #[default = "#fafafa"]
    #[schemars(description = "Colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub color: String,
    #[default = 0.2]
    #[schemars(description = "Metal", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub metalness: f64,
    #[default(TextAlign::Center)]
    #[schemars(description = "Alignment")]
    pub align: TextAlign,
    #[default = 16.0]
    #[schemars(
        description = "Line length",
        range(min = 4, max = 60),
        extend("x-step" = 1, "x-unit" = "letters", "x-help" = "Long text wraps onto a new line after about this many letters.")
    )]
    pub line_width: f64,
    #[default(Motion::TiltIn)]
    #[schemars(
        description = "Motion",
        extend("x-labels" = { "spin": "Spin", "tilt-in": "Tilt in", "float": "Float", "orbit": "Orbit", "swing": "Swing", "still": "Still", "keyframes": "Keyframes" })
    )]
    pub motion: Motion,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "motion": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[default(Lighting::Studio)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Model3d {
    #[schemars(
        description = "Model",
        extend(
            "x-widget" = "asset",
            "x-accept" = ".glb,.gltf",
            "x-help" = "A 3D model file. A model saved as several files keeps them next to it."
        )
    )]
    pub src: String,
    #[default(ModelMotion::Spin)]
    #[schemars(
        description = "Motion",
        extend("x-labels" = { "spin": "Turntable", "orbit": "Orbit", "swing": "Swing", "float": "Float", "still": "Still", "keyframes": "Keyframes" })
    )]
    pub motion: ModelMotion,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "motion": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[schemars(
        description = "Animation",
        extend("x-help" = "The name of the model's own animation to play. Leave empty for the first one.")
    )]
    pub animation: String,
    #[default = 1.0]
    #[schemars(description = "Animation speed", range(min = 0.1, max = 4), extend("x-step" = 0.05))]
    pub animation_speed: f64,
    #[default = true]
    #[schemars(description = "Loop the animation")]
    pub r#loop: bool,
    #[default = 1.0]
    #[schemars(description = "Zoom", range(min = 0.3, max = 3), extend("x-step" = 0.05))]
    pub zoom: f64,
    #[default(Lighting::Studio)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = true]
    #[schemars(description = "Floor shadow")]
    pub shadow: bool,
    #[default = "transparent"]
    #[schemars(description = "Background")]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Screen3d {
    #[default(ScreenDevice::Laptop)]
    #[schemars(
        description = "Device",
        extend("x-labels" = { "laptop": "Laptop", "monitor": "Monitor", "phone": "Phone", "floating": "Floating screen" })
    )]
    pub device: ScreenDevice,
    #[default = "#c9ced6"]
    #[schemars(description = "Device colour", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub device_color: String,
    #[default(ScreenMove::PushIn)]
    #[schemars(
        description = "Camera move",
        extend("x-labels" = { "push-in": "Push in", "orbit": "Orbit", "tilt-reveal": "Tilt reveal", "rise": "Rise", "keyframes": "Keyframes" })
    )]
    pub r#move: ScreenMove,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera",
            "x-show-if" = { "move": ["keyframes"] }
        )
    )]
    pub camera: CameraPath,
    #[default = true]
    #[schemars(
        description = "End facing the camera",
        extend("x-help" = "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device.")
    )]
    pub end_face_on: bool,
    #[default(Lighting::Studio)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = "#0b0b10"]
    #[schemars(description = "Background", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub background: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Shapes3d {
    #[default = 14.0]
    #[schemars(description = "Shapes", range(min = 3, max = 40), extend("x-step" = 1))]
    pub count: f64,
    #[default(vec!["#6366f1".into(), "#a78bfa".into(), "#f472b6".into(), "#38bdf8".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 8))]
    pub palette: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Arrangement", range(min = 1, max = 100), extend("x-step" = 1))]
    pub seed: f64,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0, max = 3), extend("x-step" = 0.05))]
    pub speed: f64,
    #[schemars(
        description = "Camera keyframes",
        extend(
            "x-help" = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
            "x-widget" = "camera"
        )
    )]
    pub camera: CameraPath,
    #[default(Lighting::Studio)]
    #[schemars(description = "Lighting", extend("x-labels" = { "studio": "Studio", "soft": "Soft", "dramatic": "Dramatic" }))]
    pub lighting: Lighting,
    #[default = "#0b1020"]
    #[schemars(description = "Background", extend("x-check" = { "rule": "colour", "message": "Pick a colour" }))]
    pub background: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct NoSettings {}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Slide {
    #[default(Direction::FromRight)]
    #[schemars(description = "Direction")]
    pub direction: Direction,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Wipe {
    #[default(WipeDirection::FromLeft)]
    #[schemars(description = "Direction")]
    pub direction: WipeDirection,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct Flip {
    #[default(Direction::FromRight)]
    #[schemars(description = "Direction")]
    pub direction: Direction,
}

const OPSIS_URL: &str = "https://github.com/bismitpanda/opsis";

struct Entry {
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    kind: ElementKind,
    frames: u32,
    look: Option<NodeLook>,
}

impl Entry {
    fn look(mut self, look: NodeLook) -> Self {
        self.look = Some(look);
        self
    }
}

fn meta<T: JsonSchema + Default + Serialize>(e: Entry) -> ElementMeta {
    let presentation = e.kind == ElementKind::Presentation;
    let slug = e.key.trim_start_matches("core/");
    let source = if presentation {
        ElementSource {
            registry: "@remotion/transitions".into(),
            url: format!("https://www.remotion.dev/docs/transitions/presentations/{slug}"),
            license: "Remotion".into(),
        }
    } else {
        ElementSource {
            registry: "Opsis".into(),
            url: OPSIS_URL.into(),
            license: "GPL-3.0-or-later".into(),
        }
    };
    ElementMeta {
        key: e.key.to_owned(),
        title: e.title.to_owned(),
        description: (!e.description.is_empty()).then(|| e.description.to_owned()),
        category: e.category,
        kind: e.kind,
        source,
        schema: ParamSchema::of::<T>(),
        default_props: to_values(&T::default()),
        fixed_props: Values::new(),
        kind_props: IndexMap::new(),
        default_duration_in_frames: e.frames,
        design: None,
        optional_clip: false,
        thumbnail: None,
        look: e.look,
    }
}

fn component(
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    frames: u32,
) -> Entry {
    Entry {
        key,
        title,
        description,
        category,
        kind: ElementKind::Component,
        frames,
        look: None,
    }
}

fn wrapper(
    key: &'static str,
    title: &'static str,
    description: &'static str,
    category: ElementCategory,
    frames: u32,
) -> Entry {
    Entry {
        key,
        title,
        description,
        category,
        kind: ElementKind::Wrapper,
        frames,
        look: None,
    }
}

fn presentation(key: &'static str, title: &'static str, frames: u32) -> Entry {
    Entry {
        key,
        title,
        description: "",
        category: ElementCategory::Transition,
        kind: ElementKind::Presentation,
        frames,
        look: None,
    }
}

pub fn core_elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<Title>(component(
            "core/title",
            "Title card",
            "A full-screen title with an optional eyebrow and subtitle.",
            C::Text,
            75,
        ).look(NodeLook::of("lucide:heading"))),
        meta::<KineticHeadline>(component(
            "core/kinetic-headline",
            "Animated headline",
            "A headline that animates in word by word.",
            C::Text,
            90,
        ).look(NodeLook::of("lucide:whole-word"))),
        meta::<LowerThird>(component(
            "core/lower-third",
            "Lower third",
            "A name or caption bar in the lower part of the screen.",
            C::Overlay,
            120,
        ).look(NodeLook::of("lucide:panel-bottom"))),
        meta::<Callout>(component(
            "core/callout",
            "Callout",
            "A short note that points something out.",
            C::Overlay,
            75,
        ).look(NodeLook::of("lucide:message-square-quote"))),
        meta::<Captions>(component(
            CAPTIONS_ELEMENT,
            "Captions",
            "Captions shown over the video.",
            C::Overlay,
            150,
        )),
        meta::<Image>(component(
            "core/image",
            "Image",
            "Shows an image from your project.",
            C::Core,
            90,
        ).look(NodeLook::of("lucide:image"))),
        meta::<Mosaic>(wrapper(
            "core/mosaic",
            "Mosaic",
            "Covers parts of the screen with coarse blocks, to hide keys, emails and personal details.",
            C::Effect,
            90,
        ).look(NodeLook::of("lucide:grid-3x3"))),
        meta::<Lottie>(component(
            "core/lottie",
            "Animation",
            "Plays an animation file from your project, looped or once, at any speed.",
            C::Overlay,
            90,
        ).look(NodeLook::of("lucide:shapes"))),
        meta::<Video>(component(
            "core/video",
            "Video",
            "Plays a video from your project.",
            C::Core,
            150,
        ).look(NodeLook::of("lucide:video"))),
        meta::<Solid>(component(
            "core/solid",
            "Solid colour",
            "A plain colour background.",
            C::Background,
            90,
        ).look(NodeLook::of("lucide:paint-bucket"))),
        meta::<Gradient>(component(
            "core/gradient",
            "Gradient",
            "A gradient background.",
            C::Background,
            90,
        ).look(NodeLook::of("lucide:rainbow"))),
        meta::<Noise>(component(
            "core/noise",
            "Noise",
            "A soft moving noise background.",
            C::Background,
            150,
        ).look(NodeLook::of("tabler:grain"))),
        meta::<Starburst>(component(
            "core/starburst",
            "Starburst",
            "Rays spreading from the centre.",
            C::Background,
            150,
        ).look(NodeLook::of("lucide:sun"))),
        meta::<Paper>(component(
            "core/paper",
            "Paper",
            "A paper texture background.",
            C::Background,
            150,
        ).look(NodeLook::of("lucide:file"))),
        meta::<StudioBackdrop>(component(
            "core/backdrop",
            "Studio backdrop",
            "A ready-made studio background image.",
            C::Background,
            150,
        ).look(NodeLook::of("lucide:wallpaper"))),
        meta::<VideoLoop>(component(
            "core/video-loop",
            "Looping video",
            "A video that loops for as long as needed.",
            C::Background,
            150,
        ).look(NodeLook::of("lucide:infinity"))),
        meta::<LightLeak>(component(
            "core/light-leak",
            "Light leak",
            "A warm light flare over the video.",
            C::Overlay,
            45,
        ).look(NodeLook::of("lucide:sun-dim"))),
        meta::<Logo3d>(component(
            "core/logo-3d",
            "3D logo",
            "Your logo in 3D, spinning, tilting in or floating, with studio lighting.",
            C::Scene,
            150,
        ).look(NodeLook::of("lucide:rotate-3d").moves(NodeMotion::Turn))),
        meta::<Card3d>(component(
            "core/card-3d",
            "3D card",
            "An image on a card with rounded corners and a soft shadow that tilts into place.",
            C::Scene,
            120,
        ).look(NodeLook::of("lucide:wallet-cards"))),
        meta::<Text3d>(component(
            "core/text-3d",
            "3D text",
            "Words in solid 3D letters with rounded edges, in any project font.",
            C::Text,
            120,
        ).look(NodeLook::of("lucide:case-sensitive").moves(NodeMotion::Turn))),
        meta::<Model3d>(component(
            "core/model-3d",
            "3D model",
            "A 3D model from your project on a turntable, with studio lighting and its own animation.",
            C::Scene,
            150,
        ).look(NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn))),
        meta::<Screen3d>(wrapper(
            "core/screen-3d",
            "3D screen",
            "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
            C::Device,
            150,
        ).look(NodeLook::of("lucide:monitor-smartphone").morphs("lucide:laptop-minimal"))),
        meta::<Shapes3d>(component(
            "core/shapes-3d",
            "Floating shapes",
            "Soft 3D spheres, rings and rounded boxes drifting slowly.",
            C::Background,
            150,
        ).look(NodeLook::of("lucide:torus"))),
        meta::<NoSettings>(presentation("core/fade", "Fade", 15).look(NodeLook::of("lucide:blend").moves(NodeMotion::FadeThrough))),
        meta::<Slide>(presentation("core/slide", "Slide", 20).look(NodeLook::of("tabler:transition-right").moves(NodeMotion::SlideThrough))),
        meta::<Wipe>(presentation("core/wipe", "Wipe", 20).look(NodeLook::of("lucide:square-split-horizontal").moves(NodeMotion::WipeReveal))),
        meta::<Flip>(presentation("core/flip", "Flip", 24).look(NodeLook::of("lucide:triangles-centerline-dashed-horizontal").moves(NodeMotion::Flip))),
        meta::<NoSettings>(presentation("core/clock-wipe", "Clock wipe", 24).look(NodeLook::of("lucide:clock-fading").moves(NodeMotion::ClockWipe))),
        meta::<NoSettings>(presentation("core/iris", "Iris", 24).look(NodeLook::of("lucide:aperture").moves(NodeMotion::Iris))),
    ]
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct CoreElementProps {
    #[serde(rename = "core/placeholder")]
    pub placeholder: PlaceholderProps,
    #[serde(rename = "core/still")]
    pub still: StillProps,
    #[serde(rename = "core/title")]
    pub title: Title,
    #[serde(rename = "core/kinetic-headline")]
    pub kinetic_headline: KineticHeadline,
    #[serde(rename = "core/lower-third")]
    pub lower_third: LowerThird,
    #[serde(rename = "core/callout")]
    pub callout: Callout,
    #[serde(rename = "core/captions")]
    pub captions: Captions,
    #[serde(rename = "core/image")]
    pub image: Image,
    #[serde(rename = "core/mosaic")]
    pub mosaic: Mosaic,
    #[serde(rename = "core/lottie")]
    pub lottie: Lottie,
    #[serde(rename = "core/video")]
    pub video: Video,
    #[serde(rename = "core/solid")]
    pub solid: Solid,
    #[serde(rename = "core/gradient")]
    pub gradient: Gradient,
    #[serde(rename = "core/noise")]
    pub noise: Noise,
    #[serde(rename = "core/starburst")]
    pub starburst: Starburst,
    #[serde(rename = "core/paper")]
    pub paper: Paper,
    #[serde(rename = "core/backdrop")]
    pub backdrop: StudioBackdrop,
    #[serde(rename = "core/video-loop")]
    pub video_loop: VideoLoop,
    #[serde(rename = "core/light-leak")]
    pub light_leak: LightLeak,
    #[serde(rename = "core/logo-3d")]
    pub logo_3d: Logo3d,
    #[serde(rename = "core/card-3d")]
    pub card_3d: Card3d,
    #[serde(rename = "core/text-3d")]
    pub text_3d: Text3d,
    #[serde(rename = "core/model-3d")]
    pub model_3d: Model3d,
    #[serde(rename = "core/screen-3d")]
    pub screen_3d: Screen3d,
    #[serde(rename = "core/shapes-3d")]
    pub shapes_3d: Shapes3d,
}
