use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum NumberFormat {
    Compact,
    Full,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum GroupSeparator {
    None,
    Space,
    Comma,
    Dot,
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
pub enum LegendValue {
    Percent,
    Value,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    Multiply,
    Screen,
    Normal,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum VerticalDirection {
    Up,
    Down,
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
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Contain,
    Cover,
    Fill,
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
pub enum Lighting {
    Studio,
    Soft,
    Dramatic,
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct NoSettings {}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum LightOrDark {
    Light,
    Dark,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    Horizontal,
    Vertical,
}
