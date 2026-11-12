use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::paint::AssetRef;
use crate::values::Num;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Drawing {
    pub width: u32,
    pub height: u32,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fill {
    Color(Color),
    Linear {
        from: (Num, Num),
        to: (Num, Num),
        stops: Vec<(Num, Color)>,
    },
    Radial {
        from: (Num, Num, Num),
        to: (Num, Num, Num),
        stops: Vec<(Num, Color)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Join {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Composite {
    #[default]
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Baseline {
    Top,
    Hanging,
    Middle,
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Font {
    pub families: Vec<String>,
    pub size: Num,
    pub weight: Num,
    pub italic: bool,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            families: vec!["sans-serif".to_owned()],
            size: Num(10.0),
            weight: Num(400.0),
            italic: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Save,
    Restore,
    Translate(Num, Num),
    Rotate(Num),
    Scale(Num, Num),
    Transform([Num; 6]),
    SetTransform([Num; 6]),
    FillStyle(Fill),
    StrokeStyle(Fill),
    LineWidth(Num),
    LineCap(Cap),
    LineJoin(Join),
    LineDash(Vec<Num>, Num),
    GlobalAlpha(Num),
    Composite(Composite),
    Shadow {
        color: Color,
        blur: Num,
        x: Num,
        y: Num,
    },
    Font(Font),
    TextAlign(TextAlign),
    TextBaseline(Baseline),
    BeginPath,
    MoveTo(Num, Num),
    LineTo(Num, Num),
    QuadraticTo(Num, Num, Num, Num),
    BezierTo([Num; 6]),
    Arc {
        center: (Num, Num),
        radius: Num,
        start: Num,
        end: Num,
        anticlockwise: bool,
    },
    Rect([Num; 4]),
    ClosePath,
    Fill,
    Stroke,
    Clip,
    FillRect([Num; 4]),
    StrokeRect([Num; 4]),
    ClearRect([Num; 4]),
    FillText {
        text: String,
        at: (Num, Num),
    },
    StrokeText {
        text: String,
        at: (Num, Num),
    },
    PutImageData {
        at: (i32, i32),
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    },
    DrawImage {
        source: AssetRef,
        rect: [Num; 4],
    },
}
