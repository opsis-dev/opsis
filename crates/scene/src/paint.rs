use serde::{Deserialize, Serialize};

use crate::color::{Color, Interpolation};
use crate::values::{Angle, Length, Num, Position2, Px};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetRef(pub String);

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ColorStop {
    pub color: Color,
    pub at: Option<Length>,
    pub until: Option<Length>,
}

impl ColorStop {
    pub const fn new(color: Color) -> Self {
        Self {
            color,
            at: None,
            until: None,
        }
    }

    pub fn at(color: Color, at: impl Into<Length>) -> Self {
        Self {
            color,
            at: Some(at.into()),
            until: None,
        }
    }

    pub fn span(color: Color, at: impl Into<Length>, until: impl Into<Length>) -> Self {
        Self {
            color,
            at: Some(at.into()),
            until: Some(until.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinearDirection {
    Angle(Angle),
    ToSide {
        horizontal: Option<Side>,
        vertical: Option<Side>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadialShape {
    #[default]
    Ellipse,
    Circle,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadialSize {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    #[default]
    FarthestCorner,
    Explicit(Length, Length),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gradient {
    Linear {
        direction: LinearDirection,
        stops: Vec<ColorStop>,
        repeating: bool,
        interpolation: Interpolation,
    },
    Radial {
        shape: RadialShape,
        size: RadialSize,
        center: Position2,
        stops: Vec<ColorStop>,
        repeating: bool,
        interpolation: Interpolation,
    },
    Conic {
        from: Angle,
        center: Position2,
        stops: Vec<ColorStop>,
        repeating: bool,
        interpolation: Interpolation,
    },
}

impl Gradient {
    pub const fn linear(angle: Angle, stops: Vec<ColorStop>) -> Self {
        Self::Linear {
            direction: LinearDirection::Angle(angle),
            stops,
            repeating: false,
            interpolation: Interpolation::DEFAULT,
        }
    }

    pub const fn radial(shape: RadialShape, center: Position2, stops: Vec<ColorStop>) -> Self {
        Self::Radial {
            shape,
            size: RadialSize::FarthestCorner,
            center,
            stops,
            repeating: false,
            interpolation: Interpolation::DEFAULT,
        }
    }

    pub const fn conic(from: Angle, center: Position2, stops: Vec<ColorStop>) -> Self {
        Self::Conic {
            from,
            center,
            stops,
            repeating: false,
            interpolation: Interpolation::DEFAULT,
        }
    }

    #[must_use]
    pub const fn repeating(mut self) -> Self {
        match &mut self {
            Self::Linear { repeating, .. }
            | Self::Radial { repeating, .. }
            | Self::Conic { repeating, .. } => *repeating = true,
        }
        self
    }

    pub const fn layer(self) -> ImageLayer {
        ImageLayer::of(Image::Gradient(self))
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Image {
    Gradient(Gradient),
    Url(AssetRef),
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayerSize {
    #[default]
    Auto,
    Cover,
    Contain,
    Explicit(Length, Length),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Repeat {
    #[default]
    Repeat,
    NoRepeat,
    Space,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoxArea {
    #[default]
    BorderBox,
    PaddingBox,
    ContentBox,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    PlusLighter,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ImageLayer {
    pub image: Image,
    pub size: LayerSize,
    pub position: Position2,
    pub repeat: (Repeat, Repeat),
    pub clip: BoxArea,
    pub origin: BoxArea,
    pub blend: BlendMode,
}

impl ImageLayer {
    pub const fn of(image: Image) -> Self {
        Self {
            image,
            size: LayerSize::Auto,
            position: Position2 {
                x: Length::ZERO,
                y: Length::ZERO,
            },
            repeat: (Repeat::Repeat, Repeat::Repeat),
            clip: BoxArea::BorderBox,
            origin: BoxArea::PaddingBox,
            blend: BlendMode::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Shadow {
    pub inset: bool,
    pub x: Length,
    pub y: Length,
    pub blur: Length,
    pub spread: Length,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Filter {
    Blur(Length),
    Brightness(Num),
    Contrast(Num),
    Saturate(Num),
    Grayscale(Num),
    Sepia(Num),
    Invert(Num),
    Opacity(Num),
    HueRotate(Angle),
    DropShadow(Shadow),
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transform {
    Translate(Length, Length, Length),
    Scale(Num, Num, Num),
    Rotate(Angle),
    Rotate3d(Num, Num, Num, Angle),
    RotateX(Angle),
    RotateY(Angle),
    Skew(Angle, Angle),
    Matrix([Num; 6]),
    Matrix3d([Num; 16]),
    Perspective(Option<Px>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FillRule {
    #[default]
    Nonzero,
    Evenodd,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShapeRadius {
    ClosestSide,
    FarthestSide,
    Length(Length),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClipPath {
    Inset {
        edges: crate::values::Edges<Length>,
        round: Option<crate::values::Corners<(Length, Length)>>,
    },
    Circle {
        radius: ShapeRadius,
        center: Position2,
    },
    Ellipse {
        radii: (ShapeRadius, ShapeRadius),
        center: Position2,
    },
    Polygon {
        fill: FillRule,
        points: Vec<(Length, Length)>,
    },
    Path {
        fill: FillRule,
        data: String,
    },
    Url(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaskMode {
    #[default]
    Alpha,
    Luminance,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct MaskLayer {
    pub layer: ImageLayer,
    pub mode: MaskMode,
}
