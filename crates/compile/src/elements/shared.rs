use opsis_macros::{choice, settings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[choice(export)]
pub enum NumberFormat {
    #[label("Short (1.2K)")]
    Compact,
    #[label("Full (1,200)")]
    Full,
}

#[choice(export)]
pub enum GroupSeparator {
    #[label("None")]
    None,
    #[label("Space")]
    Space,
    #[label("Comma")]
    Comma,
    #[label("Dot")]
    Dot,
}

#[choice(export)]
pub enum Corner {
    #[label("Bottom left")]
    BottomLeft,
    #[label("Bottom right")]
    BottomRight,
    #[label("Top left")]
    TopLeft,
    #[label("Top right")]
    TopRight,
}

#[choice(export)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

#[choice(export)]
pub enum LegendValue {
    #[label("Share")]
    Percent,
    #[label("Value")]
    Value,
}

#[choice(export)]
pub enum BlendMode {
    #[label("Darken")]
    Multiply,
    #[label("Lighten")]
    Screen,
    #[label("Normal")]
    Normal,
}

#[choice(export)]
pub enum VerticalDirection {
    #[label("Up")]
    Up,
    #[label("Down")]
    Down,
}

#[choice(export)]
pub enum TextAlign {
    #[label("Centre")]
    Center,
    #[label("Left")]
    Left,
    #[label("Right")]
    Right,
}

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

#[choice(export)]
pub enum Fit {
    #[label("Show all")]
    Contain,
    #[label("Fill the frame")]
    Cover,
    #[label("Stretch")]
    Fill,
}

#[choice(export)]
pub enum Motion {
    #[label("Spin")]
    Spin,
    #[label("Tilt in")]
    TiltIn,
    #[label("Float")]
    Float,
    #[label("Orbit")]
    Orbit,
    #[label("Swing")]
    Swing,
    #[label("Still")]
    Still,
    #[label("Keyframes")]
    Keyframes,
}

#[choice(export)]
pub enum Lighting {
    #[label("Studio")]
    Studio,
    #[label("Soft")]
    Soft,
    #[label("Dramatic")]
    Dramatic,
}

#[choice(export)]
pub enum Direction {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

#[settings(export, eq)]
pub struct NoSettings {}

#[choice(export)]
pub enum FontWeight {
    #[serde(rename = "400")]
    W400,
    #[serde(rename = "500")]
    W500,
    #[serde(rename = "600")]
    W600,
    #[serde(rename = "700")]
    W700,
}

impl FontWeight {
    pub const fn value(&self) -> f64 {
        match self {
            Self::W400 => 400.0,
            Self::W500 => 500.0,
            Self::W600 => 600.0,
            Self::W700 => 700.0,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum LightOrDark {
    Light,
    Dark,
}

#[choice(export)]
pub enum DitherShape {
    Simplex,
    Warp,
    Dots,
    Wave,
    Ripple,
    Swirl,
    Sphere,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum DarkOrLight {
    Dark,
    Light,
}

#[choice(export)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
