use std::fmt::Write;

use opsis_macros::{choice, settings};
use opsis_types::cards::{Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::numbers::whole;
use opsis_types::values::{CompileValue, Measured};
use opsis_types::workflow::PortType;

use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, FileNeed, NodeDef, Phase, PortFamily, RowExt, missing, port, row,
};
use crate::nodes::camera::BackdropStyle;
use crate::nodes::effects::Area;
use crate::params::Params;
use crate::ports::{IN_PREFIX, in_index};
use crate::text::{percent_text, plural};

pub const FRAME_TYPE: &str = "image.frame";

pub const GRID_TYPE: &str = "image.grid";

pub const LEVELS_TYPE: &str = "image.levels";

pub const TILT_TYPE: &str = "image.tilt";

pub const OUTLINE_TYPE: &str = "image.outline";

pub const CLEAR_COLOUR_TYPE: &str = "image.clearColour";

pub const TRIM_TYPE: &str = "image.trim";

pub const SPOTLIGHT_TYPE: &str = "image.spotlight";

pub const REDACT_TYPE: &str = "image.redact";

pub const FORMAT_TYPE: &str = "image.format";

pub const PLACE_TYPE: &str = "image.place";

pub const BEFORE_AFTER_TYPE: &str = "image.beforeAfter";

pub const MAGNIFY_TYPE: &str = "image.magnify";

pub const COMPARE_TYPE: &str = "image.compare";

pub const ANNOTATE_TYPE: &str = "image.annotate";

pub const BLUR_TYPE: &str = "image.blur";

pub const ADJUST_TYPE: &str = "image.adjust";

pub const ROTATE_TYPE: &str = "image.rotate";

pub const CROP_TYPE: &str = "image.crop";

pub const RESIZE_TYPE: &str = "image.resize";

pub const IMAGE_PORT: &str = "image";
pub const OVERLAY_PORT: &str = "overlay";
pub const AFTER_PORT: &str = "after";
pub const DIFFERENCES_PORT: &str = "differences";
pub const MAX_SIDE: u32 = 16384;
pub const SOFT_UPSCALE: f64 = 2.0;
pub const SMALL_CROP: f64 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[choice]
pub enum ResizeFit {
    #[default]
    #[label("Fit inside")]
    Inside,
    #[label("Fill and crop")]
    Cover,
    #[label("Stretch")]
    Exact,
}

#[choice]
pub enum PlaceAlign {
    #[label("Top left")]
    Start,
    #[default]
    #[label("Centre")]
    Center,
    #[label("Bottom right")]
    End,
}

impl PlaceAlign {
    pub fn share(self) -> f64 {
        match self {
            Self::Start => 0.0,
            Self::Center => 0.5,
            Self::End => 1.0,
        }
    }
}

#[choice]
pub enum LabelEdge {
    #[default]
    #[label("Top")]
    Top,
    #[label("Bottom")]
    Bottom,
}

#[choice]
pub enum ResizeBy {
    #[default]
    #[label("Pixels")]
    Pixels,
    #[label("Percent")]
    Percent,
}

#[choice]
pub enum Resample {
    #[label("Fast")]
    Fast,
    #[label("Smooth")]
    Smooth,
    #[default]
    #[label("Sharp")]
    Sharp,
}

#[settings]
pub struct ResizeImage {
    #[setting("Size by")]
    pub by: ResizeBy,
    #[setting(
        "Scale",
        default = 100,
        range(min = 1, max = 1000),
        unit = "%",
        show_if = { "by": ["percent"] }
    )]
    pub scale: u32,
    #[setting(
        "Width",
        default = 1920,
        range(min = 0, max = 16384),
        unit = "px",
        help = "0 follows the height and keeps the proportion.",
        show_if = { "by": ["pixels"] }
    )]
    pub width: u32,
    #[setting(
        "Height",
        range(min = 0, max = 16384),
        unit = "px",
        help = "0 follows the width and keeps the proportion.",
        show_if = { "by": ["pixels"] }
    )]
    pub height: u32,
    #[setting(
        "Fit",
        help = "When both sides are set: fit the whole image inside them, fill them and crop the rest, or stretch to exactly that size."
    )]
    pub fit: ResizeFit,
    #[setting(
        "Keep",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        help = "Click the picture to choose which part stays when it is filled and cropped.",
        show_if = { "fit": ["cover"] },
        advanced
    )]
    pub crop_at_x: f64,
    #[setting(
        "Keep vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub crop_at_y: f64,
    #[setting(
        "Never enlarge",
        help = "A picture smaller than the size asked keeps its own size.",
        advanced
    )]
    pub shrink_only: bool,
    #[setting("Quality", advanced)]
    pub quality: Resample,
}

#[choice]
pub enum CropMode {
    #[default]
    #[label("Area")]
    Area,
    #[label("Margins")]
    Margins,
}

fn whole_area() -> Vec<Area> {
    vec![Area {
        x: 0.1,
        y: 0.1,
        width: 0.8,
        height: 0.8,
    }]
}

#[settings]
pub struct CropImage {
    #[setting("Crop by")]
    pub mode: CropMode,
    #[setting(
        "Area to keep",
        default = whole_area(),
        length(min = 1, max = 1),
        widget = "boxes",
        show_if = { "mode": ["area"] },
        help = "Drag on the picture to choose the part to keep."
    )]
    pub area: Vec<Area>,
    #[setting("Top", range(min = 0, max = 16384), unit = "px", show_if = { "mode": ["margins"] })]
    pub top: u32,
    #[setting("Right", range(min = 0, max = 16384), unit = "px", show_if = { "mode": ["margins"] })]
    pub right: u32,
    #[setting(
        "Bottom",
        range(min = 0, max = 16384),
        unit = "px",
        show_if = { "mode": ["margins"] }
    )]
    pub bottom: u32,
    #[setting("Left", range(min = 0, max = 16384), unit = "px", show_if = { "mode": ["margins"] })]
    pub left: u32,
}

#[choice]
pub enum Turn {
    #[default]
    #[label("None")]
    None,
    #[label("90° right")]
    Right,
    #[label("180°")]
    Half,
    #[label("90° left")]
    Left,
}

#[settings]
pub struct RotateImage {
    #[setting(
        "Turn",
        default = Turn::Right
    )]
    pub turn: Turn,
    #[setting("Flip left to right")]
    pub flip_horizontal: bool,
    #[setting("Flip upside down")]
    pub flip_vertical: bool,
    #[setting(
        "Straighten",
        range(min = -45, max = 45),
        step = 0.5,
        unit = "°",
        help = "Tilts the picture by a small angle. The corners fill with the background colour."
    )]
    pub angle: f64,
    #[setting(
        "Cut off the corners",
        help = "Crops the straightened picture to the largest part with no empty corners.",
        advanced
    )]
    pub crop_corners: bool,
    #[setting(
        "Background",
        default = "#00000000",
        widget = "color",
        check = colour,
        help = "Fills the corners a tilt opens up.",
        advanced
    )]
    pub background: String,
}

#[settings]
pub struct AdjustImage {
    #[setting("Brightness", range(min = -1, max = 1), step = 0.05)]
    pub brightness: f64,
    #[setting("Contrast", range(min = -1, max = 1), step = 0.05)]
    pub contrast: f64,
    #[setting(
        "Saturation",
        range(min = -1, max = 1),
        step = 0.05,
        help = "-1 takes all the colour out."
    )]
    pub saturation: f64,
    #[setting("Hue", range(min = -180, max = 180), step = 1, unit = "°", advanced)]
    pub hue: f64,
    #[setting("Black and white")]
    pub grayscale: bool,
    #[setting("Invert colours", advanced)]
    pub invert: bool,
}

#[settings]
pub struct BlurImage {
    #[setting("Blur", range(min = 0, max = 100), step = 0.5, unit = "px")]
    pub blur: f64,
    #[setting("Sharpen", default = 1.0, range(min = 0, max = 5), step = 0.1)]
    pub sharpen: f64,
    #[setting(
        "Sharpen threshold",
        default = 4,
        range(min = 0, max = 255),
        advanced,
        help = "Differences smaller than this stay as they are, so flat areas don't get grainy."
    )]
    pub threshold: u32,
    #[setting(
        "Sharpen amount",
        default = 100,
        range(min = 0, max = 300),
        unit = "%",
        help = "How much of the sharpening is added.",
        advanced
    )]
    pub sharpen_amount: u32,
}

#[settings]
pub struct PlaceImage {
    #[setting(
        "Place at",
        default = 0.85,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        help = "Click the picture to choose where the middle of the placed image goes."
    )]
    pub x: f64,
    #[setting(
        "Place at vertical",
        default = 0.85,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Size",
        default = 20.0,
        range(min = 1, max = 100),
        step = 1,
        unit = "%",
        help = "The placed image's width, as a share of the picture's width."
    )]
    pub size: f64,
    #[setting("Opacity", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub opacity: f64,
}

#[choice]
pub enum StillFileFormat {
    #[default]
    #[label("PNG")]
    Png,
    #[label("JPEG")]
    Jpeg,
    #[label("WebP (lossless)")]
    Webp,
    #[label("AVIF")]
    Avif,
}

impl StillFileFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Avif => "avif",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
            Self::Avif => "AVIF",
        }
    }
}

#[settings(eq)]
pub struct ImageFormat {
    #[setting(
        "Format",
        default = StillFileFormat::Jpeg
    )]
    pub format: StillFileFormat,
    #[setting(
        "Quality",
        default = 85,
        range(min = 1, max = 100),
        show_if = { "format": ["jpeg", "avif"] },
        help = "Higher looks better and makes a larger file."
    )]
    pub quality: u32,
    #[setting(
        "Speed",
        default = 6,
        range(min = 1, max = 10),
        show_if = { "format": ["avif"] },
        advanced,
        help = "Higher saves faster and makes a larger file."
    )]
    pub speed: u32,
    #[setting(
        "Background",
        default = "#ffffff",
        widget = "color",
        check = colour,
        show_if = { "format": ["jpeg"] },
        help = "JPEG has no see-through parts, so they're filled with this colour."
    )]
    pub background: String,
}

#[choice]
pub enum RedactStyle {
    #[default]
    #[label("Large pixels")]
    Pixelate,
    #[label("Blur")]
    Blur,
    #[label("Solid colour")]
    Solid,
}

impl RedactStyle {
    pub fn word(self) -> &'static str {
        match self {
            Self::Pixelate => "pixelated",
            Self::Blur => "blurred",
            Self::Solid => "covered",
        }
    }
}

pub fn hidden_area() -> Vec<Area> {
    vec![Area {
        x: 0.35,
        y: 0.4,
        width: 0.3,
        height: 0.2,
    }]
}

#[settings]
pub struct RedactImage {
    #[setting(
        "Areas to hide",
        default = hidden_area(),
        length(min = 1, max = 8),
        widget = "boxes",
        help = "Drag on the picture to mark each part to hide."
    )]
    pub areas: Vec<Area>,
    #[setting("Hide with")]
    pub style: RedactStyle,
    #[setting(
        "Pixel size",
        default = 16,
        range(min = 2, max = 256),
        unit = "px",
        show_if = { "style": ["pixelate"] }
    )]
    pub block_size: u32,
    #[setting(
        "Blur",
        default = 16.0,
        range(min = 1, max = 100),
        step = 0.5,
        unit = "px",
        show_if = { "style": ["blur"] }
    )]
    pub blur: f64,
    #[setting(
        "Colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "style": ["solid"] }
    )]
    pub color: String,
    #[setting(
        "Corner radius",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "style": ["solid"] },
        advanced
    )]
    pub radius: f64,
}

fn lit_area() -> Vec<Area> {
    vec![Area {
        x: 0.3,
        y: 0.3,
        width: 0.4,
        height: 0.4,
    }]
}

#[settings]
pub struct SpotlightImage {
    #[setting(
        "Areas to light",
        default = lit_area(),
        length(min = 1, max = 8),
        widget = "boxes",
        help = "Drag on the picture to mark each part to keep bright."
    )]
    pub areas: Vec<Area>,
    #[setting("Darken the rest", default = 0.6, range(min = 0, max = 1), step = 0.05)]
    pub dim: f64,
    #[setting("Blur the rest", range(min = 0, max = 50), step = 0.5, unit = "px")]
    pub blur: f64,
    #[setting(
        "Corner radius",
        default = 12.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: f64,
    #[setting(
        "Soft edge",
        default = 8.0,
        range(min = 0, max = 100),
        step = 1,
        unit = "px",
        advanced
    )]
    pub softness: f64,
    #[setting("Darken toward", default = "#000000", widget = "color", check = colour, advanced)]
    pub dim_color: String,
}

#[settings(eq)]
pub struct TrimEdges {
    #[setting(
        "Tolerance",
        default = 8,
        range(min = 0, max = 255),
        help = "How far a pixel's colour may be from the corner's and still count as empty edge."
    )]
    pub tolerance: u32,
    #[setting("Keep around", range(min = 0, max = 2000), unit = "px")]
    pub padding: u32,
    #[setting(
        "Edge colour",
        widget = "color",
        zero_label = "Top-left pixel",
        check = colour_or_empty,
        help = "The colour of the empty edge to cut.",
        advanced
    )]
    pub edge_color: String,
}

#[settings]
pub struct ClearColour {
    #[setting(
        "Colour",
        widget = "color",
        zero_label = "The corner's colour",
        check = colour_or_empty
    )]
    pub color: String,
    #[setting(
        "Tolerance",
        default = 10.0,
        range(min = 0, max = 100),
        step = 1,
        unit = "%",
        help = "How different a colour may be and still be made see-through."
    )]
    pub tolerance: f64,
    #[setting(
        "Only areas touching the edges",
        default = true,
        help = "Keeps the same colour inside the picture, such as the white inside a logo."
    )]
    pub edges_only: bool,
    #[setting(
        "Soft edge",
        default = 10.0,
        range(min = 0, max = 100),
        step = 1,
        unit = "%",
        advanced
    )]
    pub softness: f64,
}

#[settings]
pub struct OutlineImage {
    #[setting(
        "Width",
        default = 8.0,
        range(min = 1, max = 200),
        step = 1,
        unit = "px"
    )]
    pub width: f64,
    #[setting("Colour", default = "#ffffff", widget = "color", check = colour)]
    pub color: String,
    #[setting("Soft edge", range(min = 0, max = 50), step = 1, unit = "px", advanced)]
    pub softness: f64,
    #[setting("Shadow")]
    pub shadow: bool,
    #[setting(
        "Shadow opacity",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_opacity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Shadow softness",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        zero_label = "Outline width",
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_softness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Shadow drop",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        zero_label = "Half the outline width",
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_drop: Option<f64>,
    #[setting(
        "Shadow colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_color: String,
}

impl OutlineImage {
    pub fn shadow_softness(&self) -> f64 {
        self.shadow_softness.unwrap_or(self.width)
    }

    pub fn shadow_drop(&self) -> f64 {
        self.shadow_drop.unwrap_or(self.width / 2.0)
    }
}

#[settings]
pub struct TiltImage {
    #[setting(
        "Turn",
        default = 18.0,
        range(min = -60, max = 60),
        step = 1,
        unit = "°",
        help = "Turns the picture away on its right side, or its left with a negative angle."
    )]
    pub turn: f64,
    #[setting(
        "Tilt",
        default = 8.0,
        range(min = -60, max = 60),
        step = 1,
        unit = "°",
        help = "Leans the top back, or the bottom with a negative angle."
    )]
    pub tilt: f64,
    #[setting("Lean", range(min = -45, max = 45), step = 1, unit = "°", advanced)]
    pub lean: f64,
    #[setting("Background", default = "#00000000", widget = "color", check = colour, advanced)]
    pub background: String,
    #[setting("Shadow", default = true)]
    pub shadow: bool,
    #[setting(
        "Viewing distance",
        default = 2.0,
        range(min = 0.5, max = 10),
        step = 0.1,
        help = "How far away the picture is seen from, in its long side. Closer looks more three-dimensional.",
        advanced
    )]
    pub perspective: f64,
    #[setting(
        "Shadow opacity",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_opacity: f64,
    #[setting(
        "Shadow softness",
        default = 2.0,
        range(min = 0, max = 10),
        step = 0.1,
        unit = "%",
        help = "Of the picture's long side.",
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_softness: f64,
    #[setting(
        "Shadow drop",
        default = 2.0,
        range(min = 0, max = 10),
        step = 0.1,
        unit = "%",
        help = "Of the picture's long side.",
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_drop: f64,
    #[setting(
        "Shadow colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "shadow": ["true"] },
        advanced
    )]
    pub shadow_color: String,
}

impl TiltImage {
    pub fn shadow_space(&self) -> f64 {
        if self.shadow {
            SHADOW_SPACE.max((2.0 * self.shadow_softness + self.shadow_drop) / 100.0)
        } else {
            0.0
        }
    }
}

#[choice]
pub enum LevelsMethod {
    #[default]
    #[label("Stretch")]
    Stretch,
    #[label("Even out")]
    Equalize,
}

#[settings]
pub struct AutoLevels {
    #[setting(
        "Method",
        help = "Stretch spreads the levels from black to white. Even out spreads them evenly, which is stronger."
    )]
    pub method: LevelsMethod,
    #[setting("Strength", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub strength: f64,
    #[setting(
        "Ignore the darkest and lightest",
        default = 0.5,
        range(min = 0, max = 5),
        step = 0.1,
        unit = "%",
        show_if = { "method": ["stretch"] },
        advanced
    )]
    pub clip: f64,
    #[setting(
        "Each colour on its own",
        help = "Also corrects a colour cast, but can shift colours."
    )]
    pub per_channel: bool,
}

#[choice]
pub enum GridLayout {
    #[label("One row")]
    Row,
    #[label("One column")]
    Column,
    #[default]
    #[label("Grid")]
    Grid,
}

#[choice]
pub enum GridFit {
    #[label("Same height")]
    SameHeight,
    #[label("Same width")]
    SameWidth,
    #[default]
    #[label("Equal cells")]
    Cells,
}

#[settings]
pub struct ImageGrid {
    #[setting("Layout")]
    pub layout: GridLayout,
    #[setting("Columns", default = 2, range(min = 1, max = 12), show_if = { "layout": ["grid"] })]
    pub columns: u32,
    #[setting(
        "Sizes",
        help = "Same height and same width shrink every image to the smallest one's. Equal cells makes every cell as big as the largest image and centres each image in its own; nothing is enlarged."
    )]
    pub fit: GridFit,
    #[setting("Gap", default = 24, range(min = 0, max = 1000), unit = "px")]
    pub gap: u32,
    #[setting(
        "Padding",
        default = 24,
        range(min = 0, max = 1000),
        unit = "px",
        advanced
    )]
    pub padding: u32,
    #[setting("Background", default = "#ffffff", widget = "color", check = colour)]
    pub background: String,
    #[setting(
        "Place in its cell",
        help = "Where a picture smaller than its cell sits.",
        advanced
    )]
    pub align: PlaceAlign,
    #[setting(
        "Corner radius",
        range(min = 0, max = 500),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: f64,
}

#[choice]
pub enum FrameShape {
    #[default]
    #[label("Keep")]
    Keep,
    #[label("16:9")]
    Wide,
    #[label("1:1")]
    Square,
    #[label("4:5")]
    Portrait,
    #[label("9:16")]
    Story,
}

impl FrameShape {
    fn ratio(self) -> Option<f64> {
        match self {
            Self::Keep => None,
            Self::Wide => Some(16.0 / 9.0),
            Self::Square => Some(1.0),
            Self::Portrait => Some(4.0 / 5.0),
            Self::Story => Some(9.0 / 16.0),
        }
    }
}

#[settings(check)]
pub struct FrameScreenshot {
    #[serde(flatten)]
    pub background: BackdropStyle,
    #[setting(
        "Padding",
        default = 8.0,
        range(min = 0, max = 50),
        step = 1,
        unit = "%",
        help = "Space around the picture, as a share of its shorter side."
    )]
    pub padding: f64,
    #[setting(
        "Corner radius",
        default = 16.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px"
    )]
    pub radius: f64,
    #[setting(
        "Shadow drop",
        default = 16.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub shadow_drop: f64,
    #[setting(
        "Shadow softness",
        default = 40.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub shadow_softness: f64,
    #[setting(
        "Shadow opacity",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub shadow_opacity: f64,
    #[setting(
        "Shape",
        help = "Grows the background around the picture to this shape; the picture is never cut."
    )]
    pub shape: FrameShape,
    #[setting(
        "Place on the shape",
        labels = { "start": "Top or left", "center": "Centre", "end": "Bottom or right" },
        help = "Where the picture sits when the shape grows the background.",
        advanced
    )]
    pub align: PlaceAlign,
    #[setting("Shadow colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub shadow_color: String,
}

impl Params for FrameScreenshot {
    fn check(&self) -> Result<(), SettingsError> {
        self.background.check()
    }
}

pub fn framed_placement(source: (f64, f64), p: &FrameScreenshot) -> ((f64, f64), (f64, f64)) {
    let pad = (source.0.min(source.1) * p.padding.clamp(0.0, 50.0) / 100.0).round();
    let (mut w, mut h) = (source.0 + 2.0 * pad, source.1 + 2.0 * pad);
    if let Some(ratio) = p.shape.ratio() {
        if w / h > ratio {
            h = (w / ratio).round();
        } else {
            w = (h * ratio).round();
        }
    }
    let free = (w - source.0 - 2.0 * pad, h - source.1 - 2.0 * pad);
    let at = (
        (pad + free.0 * p.align.share()).round(),
        (pad + free.1 * p.align.share()).round(),
    );
    ((w, h), at)
}

#[derive(Debug)]
pub struct GridPlan {
    pub canvas: (f64, f64),
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<Rect>,
}

fn grid_shape(n: usize, p: &ImageGrid) -> (usize, usize) {
    let columns = match p.layout {
        GridLayout::Row => n,
        GridLayout::Column => 1,
        GridLayout::Grid => (p.columns.max(1) as usize).min(n),
    }
    .max(1);
    (columns, n.div_ceil(columns).max(1))
}

pub fn grid_plan(sizes: &[(f64, f64)], p: &ImageGrid) -> GridPlan {
    let n = sizes.len();
    let (columns, rows) = grid_shape(n, p);
    let gap = f64::from(p.gap);
    let pad = f64::from(p.padding);
    let scaled: Vec<(f64, f64)> = match p.fit {
        GridFit::SameHeight => {
            let h = sizes.iter().map(|s| s.1).fold(f64::INFINITY, f64::min);
            sizes
                .iter()
                .map(|&(w, sh)| ((w * h / sh).round().max(1.0), h))
                .collect()
        }
        GridFit::SameWidth => {
            let w = sizes.iter().map(|s| s.0).fold(f64::INFINITY, f64::min);
            sizes
                .iter()
                .map(|&(sw, h)| (w, (h * w / sw).round().max(1.0)))
                .collect()
        }
        GridFit::Cells => sizes.to_vec(),
    };
    let cell = (
        scaled.iter().map(|s| s.0).fold(0.0, f64::max),
        scaled.iter().map(|s| s.1).fold(0.0, f64::max),
    );
    let place = |i: usize| (i % columns, i / columns);
    let column_widths: Vec<f64> = (0..columns)
        .map(|column| {
            (0..n)
                .filter(|&i| place(i).0 == column)
                .map(|i| {
                    if p.fit == GridFit::Cells {
                        cell.0
                    } else {
                        scaled[i].0
                    }
                })
                .fold(0.0, f64::max)
        })
        .collect();
    let row_heights: Vec<f64> = (0..rows)
        .map(|row| {
            (0..n)
                .filter(|&i| place(i).1 == row)
                .map(|i| {
                    if p.fit == GridFit::Cells {
                        cell.1
                    } else {
                        scaled[i].1
                    }
                })
                .fold(0.0, f64::max)
        })
        .collect();
    let left = |column: usize| {
        pad + column_widths[..column].iter().sum::<f64>() + gap * numbers::float(column)
    };
    let top = |row: usize| pad + row_heights[..row].iter().sum::<f64>() + gap * numbers::float(row);
    let cells = (0..n)
        .map(|i| {
            let (column, row) = place(i);
            let (w, h) = scaled[i];
            Rect {
                x: (left(column) + (column_widths[column] - w) * p.align.share()).round(),
                y: (top(row) + (row_heights[row] - h) * p.align.share()).round(),
                width: w,
                height: h,
            }
        })
        .collect();
    let canvas = (
        (left(columns) - gap + pad).max(1.0),
        (top(rows) - gap + pad).max(1.0),
    );
    GridPlan {
        canvas,
        columns,
        rows,
        cells,
    }
}

const SHADOW_SPACE: f64 = 0.06;

#[expect(
    clippy::many_single_char_names,
    reason = "w, h are the picture's size and x, y, z a corner's position in 3D"
)]
pub fn tilt_corners(size: (f64, f64), p: &TiltImage) -> [(f64, f64); 4] {
    let (w, h) = size;
    let distance = p.perspective * w.max(h);
    let (lean, tilt, turn) = (
        p.lean.to_radians(),
        p.tilt.to_radians(),
        p.turn.to_radians(),
    );
    let projected = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(x, y)| {
        let (x, y) = (x - w / 2.0, y - h / 2.0);
        let (x, y) = (
            x * lean.cos() - y * lean.sin(),
            x * lean.sin() + y * lean.cos(),
        );
        let (y, z) = (y * tilt.cos(), -y * tilt.sin());
        let (x, z) = (
            x * turn.cos() - z * turn.sin(),
            x * turn.sin() + z * turn.cos(),
        );
        let scale = distance / (distance + z);
        (x * scale, y * scale)
    });
    let margin = p.shadow_space() * w.max(h);
    let left = projected.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let top = projected.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    projected.map(|(x, y)| (x - left + margin, y - top + margin))
}

pub fn tilted_size(size: (f64, f64), p: &TiltImage) -> (f64, f64) {
    let corners = tilt_corners(size, p);
    let margin = p.shadow_space() * size.0.max(size.1);
    let right = corners.iter().map(|c| c.0).fold(0.0, f64::max);
    let bottom = corners.iter().map(|c| c.1).fold(0.0, f64::max);
    (
        (right + margin).ceil().max(1.0),
        (bottom + margin).ceil().max(1.0),
    )
}

pub fn outline_padding(p: &OutlineImage) -> f64 {
    let shadow = if p.shadow {
        (2.0 * p.shadow_softness()).max(p.shadow_softness() + 2.0 * p.shadow_drop())
    } else {
        0.0
    };
    (p.width + p.softness + shadow).ceil()
}

fn even(value: f64) -> f64 {
    value.round().max(1.0)
}

pub fn resized_size(source: (f64, f64), p: &ResizeImage) -> (f64, f64) {
    let asked = asked_size(source, p);
    if p.shrink_only && (asked.0 > source.0 || asked.1 > source.1) {
        let k = (source.0 / asked.0).min(source.1 / asked.1);
        return (even(asked.0 * k), even(asked.1 * k));
    }
    asked
}

fn asked_size(source: (f64, f64), p: &ResizeImage) -> (f64, f64) {
    let (sw, sh) = source;
    if p.by == ResizeBy::Percent {
        let k = f64::from(p.scale) / 100.0;
        return (even(sw * k), even(sh * k));
    }
    let (w, h) = (f64::from(p.width), f64::from(p.height));
    match (w > 0.0, h > 0.0) {
        (false, false) => (sw, sh),
        (true, false) => (w, even(sh * w / sw)),
        (false, true) => (even(sw * h / sh), h),
        (true, true) => match p.fit {
            ResizeFit::Inside => {
                let scale = (w / sw).min(h / sh);
                (even(sw * scale), even(sh * scale))
            }
            ResizeFit::Cover | ResizeFit::Exact => (w, h),
        },
    }
}

pub fn upscale(source: (f64, f64), out: (f64, f64)) -> f64 {
    (out.0 / source.0).max(out.1 / source.1)
}

fn inside(source: (f64, f64), rect: Rect) -> Rect {
    let (sw, sh) = source;
    let x = rect.x.clamp(0.0, sw - 1.0);
    let y = rect.y.clamp(0.0, sh - 1.0);
    Rect {
        x,
        y,
        width: rect.width.clamp(1.0, sw - x),
        height: rect.height.clamp(1.0, sh - y),
    }
}

pub fn area_rect(source: (f64, f64), area: &Area) -> Rect {
    let (sw, sh) = source;
    inside(
        source,
        Rect {
            x: (area.x * sw).round(),
            y: (area.y * sh).round(),
            width: (area.width * sw).round(),
            height: (area.height * sh).round(),
        },
    )
}

pub fn crop_rect(source: (f64, f64), p: &CropImage) -> Rect {
    let (sw, sh) = source;
    let rect = match p.mode {
        CropMode::Area => {
            let area = p.area.first().cloned().unwrap_or(Area {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            });
            return area_rect(source, &area);
        }
        CropMode::Margins => Rect {
            x: f64::from(p.left),
            y: f64::from(p.top),
            width: sw - f64::from(p.left + p.right),
            height: sh - f64::from(p.top + p.bottom),
        },
    };
    inside(source, rect)
}

pub fn rotated_size(source: (f64, f64), p: &RotateImage) -> (f64, f64) {
    let (w, h) = match p.turn {
        Turn::Right | Turn::Left => (source.1, source.0),
        Turn::None | Turn::Half => source,
    };
    if p.angle == 0.0 {
        return (w, h);
    }
    let (sin, cos) = p.angle.to_radians().sin_cos();
    let (sin, cos) = (sin.abs(), cos.abs());
    if p.crop_corners {
        let k = (w / (w * cos + h * sin)).min(h / (w * sin + h * cos));
        return ((w * k).floor().max(1.0), (h * k).floor().max(1.0));
    }
    ((w * cos + h * sin).ceil(), (w * sin + h * cos).ceil())
}

pub fn tilted_canvas(source: (f64, f64), p: &RotateImage) -> (f64, f64) {
    rotated_size(
        source,
        &RotateImage {
            crop_corners: false,
            ..p.clone()
        },
    )
}

fn pixels(width: f64, height: f64) -> String {
    format!(
        "{} × {} px",
        numbers::truncate::<i64>(width),
        numbers::truncate::<i64>(height)
    )
}

fn result_size<P>(ctx: &CardCtx<'_, P>) -> Option<(f64, f64)> {
    match ctx.result?.output(IMAGE_PORT)? {
        CompileValue::Image(image) => Some((f64::from(image.width?), f64::from(image.height?))),
        _ => None,
    }
}

fn input_size<P>(ctx: &CardCtx<'_, P>, port_name: &str) -> Option<(f64, f64)> {
    let (width, height) = ctx.inputs.pixels.get(port_name).copied()?;
    Some((f64::from(width), f64::from(height)))
}

fn size_row<P>(
    ctx: &CardCtx<'_, P>,
    planned: impl Fn((f64, f64)) -> (f64, f64),
) -> Option<opsis_types::cards::CardRow> {
    let size = result_size(ctx).or_else(|| input_size(ctx, IMAGE_PORT).map(planned))?;
    Some(row(Icon::Ruler, pixels(size.0, size.1)))
}

fn image_node<P: Params>(kind: &'static str, title: &str, description: &str) -> NodeDef {
    NodeDef::of::<P>(kind, title, Category::Image, Phase::Media)
        .describe(format!("{description} Makes a new file."))
        .executes()
        .inputs(vec![(IMAGE_PORT, port(PortType::Media, "Image"))])
        .outputs(vec![(IMAGE_PORT, port(PortType::Image, "Image"))])
        .compile(|ctx: &Ctx<'_, P>| {
            let image = ctx
                .result
                .and_then(|r| r.output(IMAGE_PORT))
                .filter(|v| matches!(v, CompileValue::Image(_)));
            Ok(Some(
                image
                    .map(|v| (IMAGE_PORT.to_owned(), v.clone()))
                    .into_iter()
                    .collect(),
            ))
        })
}

fn resize() -> NodeDef {
    image_node::<ResizeImage>(
        RESIZE_TYPE,
        "Resize image",
        "Makes an image a chosen width, height or both.",
    )
    .card(|ctx: &CardCtx<'_, ResizeImage>| {
        let p = &ctx.params;
        let side = |v: u32| {
            if v > 0 {
                v.to_string()
            } else {
                "auto".to_owned()
            }
        };
        let mut rows = vec![size_row(ctx, |s| resized_size(s, p)).or_else(|| {
            Some(row(
                Icon::Ruler,
                format!("{} × {} px", side(p.width), side(p.height)),
            ))
        })];
        if let Some(source) = input_size(ctx, IMAGE_PORT) {
            let factor = upscale(source, resized_size(source, p));
            if factor > SOFT_UPSCALE {
                rows.push(Some(missing(
                    Icon::TriangleAlert,
                    format!("Enlarges {factor:.1}×, so it may look soft"),
                )));
            }
        }
        rows
    })
}

fn crop() -> NodeDef {
    image_node::<CropImage>(
        CROP_TYPE,
        "Crop image",
        "Keeps one part of an image: an area you draw on it, or what's inside margins.",
    )
    .card(|ctx: &CardCtx<'_, CropImage>| {
        let p = &ctx.params;
        let mut rows = vec![
            size_row(ctx, |s| {
                let r = crop_rect(s, p);
                (r.width, r.height)
            })
            .or_else(|| {
                Some(row(
                    Icon::Crop,
                    match p.mode {
                        CropMode::Area => "An area you draw".to_owned(),
                        CropMode::Margins => {
                            format!("Margins {} {} {} {} px", p.top, p.right, p.bottom, p.left)
                        }
                    },
                ))
            }),
        ];
        if let Some(source) = input_size(ctx, IMAGE_PORT) {
            let r = crop_rect(source, p);
            if r.width.min(r.height) < SMALL_CROP {
                rows.push(Some(missing(
                    Icon::TriangleAlert,
                    format!(
                        "Only {} px is left",
                        numbers::truncate::<i64>(r.width.min(r.height))
                    ),
                )));
            }
        }
        rows
    })
}

fn rotate() -> NodeDef {
    image_node::<RotateImage>(
        ROTATE_TYPE,
        "Rotate and flip",
        "Turns an image in quarter turns, flips it, or straightens it by a small angle.",
    )
    .card(|ctx: &CardCtx<'_, RotateImage>| {
        let p = &ctx.params;
        let mut parts: Vec<String> = Vec::new();
        match p.turn {
            Turn::Right => parts.push("90° right".into()),
            Turn::Half => parts.push("180°".into()),
            Turn::Left => parts.push("90° left".into()),
            Turn::None => {}
        }
        if p.flip_horizontal {
            parts.push("flipped left to right".into());
        }
        if p.flip_vertical {
            parts.push("flipped upside down".into());
        }
        if p.angle != 0.0 {
            parts.push(format!("tilted {}°", p.angle));
        }
        let what = if parts.is_empty() {
            row(Icon::RefreshCw, "No change").muted_if(true)
        } else {
            row(
                Icon::RefreshCw,
                crate::card_rows::sentence(&parts.join(", ")),
            )
        };
        vec![Some(what), size_row(ctx, |s| rotated_size(s, p))]
    })
}

fn adjust() -> NodeDef {
    image_node::<AdjustImage>(
        ADJUST_TYPE,
        "Adjust image",
        "Changes an image's brightness, contrast, colour and hue.",
    )
    .card(|ctx: &CardCtx<'_, AdjustImage>| {
        let p = &ctx.params;
        let signed = |v: f64| {
            format!(
                "{}{}",
                if v > 0.0 { "+" } else { "" },
                numbers::truncate::<i64>((v * 100.0).round())
            )
        };
        let mut parts = Vec::new();
        if p.brightness != 0.0 {
            parts.push(format!("brightness {}", signed(p.brightness)));
        }
        if p.contrast != 0.0 {
            parts.push(format!("contrast {}", signed(p.contrast)));
        }
        if p.saturation != 0.0 {
            parts.push(format!("colour {}", signed(p.saturation)));
        }
        if p.hue != 0.0 {
            parts.push(format!("hue {}°", p.hue));
        }
        if p.grayscale {
            parts.push("black and white".into());
        }
        if p.invert {
            parts.push("inverted".into());
        }
        let what = if parts.is_empty() {
            row(Icon::Sun, "No change").muted_if(true)
        } else {
            row(Icon::Sun, crate::card_rows::sentence(&parts.join(", ")))
        };
        vec![Some(what), size_row(ctx, |s| s)]
    })
}

fn blur() -> NodeDef {
    image_node::<BlurImage>(
        BLUR_TYPE,
        "Blur and sharpen",
        "Softens an image or makes its edges crisper.",
    )
    .card(|ctx: &CardCtx<'_, BlurImage>| {
        let p = &ctx.params;
        let mut parts = Vec::new();
        if p.blur > 0.0 {
            parts.push(format!("blur {} px", p.blur));
        }
        if p.sharpen > 0.0 {
            parts.push(format!("sharpen {}", p.sharpen));
        }
        let what = if parts.is_empty() {
            row(Icon::Wind, "No change").muted_if(true)
        } else {
            row(Icon::Wind, crate::card_rows::sentence(&parts.join(", ")))
        };
        vec![Some(what), size_row(ctx, |s| s)]
    })
}

fn place() -> NodeDef {
    image_node::<PlaceImage>(
        PLACE_TYPE,
        "Place image",
        "Puts a second image, such as a logo or a watermark, on top of an image.",
    )
    .inputs(vec![
        (IMAGE_PORT, port(PortType::Media, "Image")),
        (OVERLAY_PORT, port(PortType::Media, "On top")),
    ])
    .card(|ctx: &CardCtx<'_, PlaceImage>| {
        let p = &ctx.params;
        let mut rows = vec![Some(row(
            Icon::Layers,
            format!(
                "{} wide at {}, {}{}",
                percent_text(p.size / 100.0, f64::INFINITY),
                percent_text(p.x, f64::INFINITY),
                percent_text(p.y, f64::INFINITY),
                if p.opacity < 1.0 {
                    format!(" · {} opaque", percent_text(p.opacity, f64::INFINITY))
                } else {
                    String::new()
                }
            ),
        ))];
        if !ctx.connected(OVERLAY_PORT) {
            rows.push(Some(missing(Icon::Image, "Connect an image to place")));
        }
        rows.push(size_row(ctx, |s| s));
        rows
    })
}

fn format() -> NodeDef {
    image_node::<ImageFormat>(
        FORMAT_TYPE,
        "Image format",
        "Saves an image as PNG, JPEG, WebP or AVIF.",
    )
    .card(|ctx: &CardCtx<'_, ImageFormat>| {
        let p = &ctx.params;
        let text = match p.format {
            StillFileFormat::Jpeg | StillFileFormat::Avif => {
                format!("{} · quality {}", p.format.word(), p.quality)
            }
            StillFileFormat::Png | StillFileFormat::Webp => p.format.word().to_owned(),
        };
        vec![Some(row(Icon::Image, text)), size_row(ctx, |s| s)]
    })
}

fn redact() -> NodeDef {
    image_node::<RedactImage>(
        REDACT_TYPE,
        "Redact",
        "Hides parts of an image, such as names, email addresses and keys, behind large pixels, a blur or a solid colour.",
    )
    .card(|ctx: &CardCtx<'_, RedactImage>| {
        let p = &ctx.params;
        vec![
            Some(row(
                Icon::EyeOff,
                format!(
                    "{} · {}",
                    crate::text::plural(numbers::float(p.areas.len()), "area"),
                    p.style.word()
                ),
            )),
            size_row(ctx, |s| s),
        ]
    })
}

fn spotlight() -> NodeDef {
    image_node::<SpotlightImage>(
        SPOTLIGHT_TYPE,
        "Spotlight",
        "Darkens or blurs everything outside the areas you draw, so the eye goes to them.",
    )
    .card(|ctx: &CardCtx<'_, SpotlightImage>| {
        let p = &ctx.params;
        let mut text = format!(
            "{} · darkens {}",
            crate::text::plural(numbers::float(p.areas.len()), "area"),
            percent_text(p.dim, f64::INFINITY)
        );
        if p.blur > 0.0 {
            let _ = write!(text, " · blurs {} px", p.blur);
        }
        vec![Some(row(Icon::Flashlight, text)), size_row(ctx, |s| s)]
    })
}

fn trim() -> NodeDef {
    image_node::<TrimEdges>(
        TRIM_TYPE,
        "Trim empty edges",
        "Cuts away even-coloured margins, so the content sits tight.",
    )
    .card(|ctx: &CardCtx<'_, TrimEdges>| {
        let p = &ctx.params;
        vec![
            result_size(ctx)
                .map(|(w, h)| row(Icon::Crop, pixels(w, h)))
                .or_else(|| Some(row(Icon::Crop, "Cuts even-coloured edges"))),
            (p.padding > 0).then(|| row(Icon::Ruler, format!("Keeps {} px around", p.padding))),
        ]
    })
}

fn clear_colour() -> NodeDef {
    image_node::<ClearColour>(
        CLEAR_COLOUR_TYPE,
        "Make a colour transparent",
        "Makes a flat background, such as a logo's white, see-through.",
    )
    .card(|ctx: &CardCtx<'_, ClearColour>| {
        let p = &ctx.params;
        let colour = p.color.trim();
        let what = if colour.is_empty() {
            row(Icon::Palette, "The corner's colour made see-through")
        } else {
            row(Icon::Palette, format!("{colour} made see-through"))
                .swatches(vec![colour.to_owned()])
        };
        vec![
            Some(what),
            Some(row(
                Icon::Scan,
                if p.edges_only {
                    "Only areas touching the edges"
                } else {
                    "Everywhere in the picture"
                },
            )),
            size_row(ctx, |s| s),
        ]
    })
}

fn outline() -> NodeDef {
    image_node::<OutlineImage>(
        OUTLINE_TYPE,
        "Outline",
        "Draws a coloured outline around the visible part of a see-through image, for cutouts and logos.",
    )
    .card(|ctx: &CardCtx<'_, OutlineImage>| {
        let p = &ctx.params;
        let colour = p.color.trim().to_owned();
        let mut text = format!("{} px outline", p.width);
        if p.shadow {
            text.push_str(" · shadow");
        }
        vec![
            Some(row(Icon::Shapes, text).swatches(vec![colour])),
            size_row(ctx, |(w, h)| {
                let pad = outline_padding(p) * 2.0;
                (w + pad, h + pad)
            }),
        ]
    })
}

fn tilt() -> NodeDef {
    image_node::<TiltImage>(
        TILT_TYPE,
        "Tilt",
        "Shows an image on a turned and tilted plane, for a quick mock-up.",
    )
    .card(|ctx: &CardCtx<'_, TiltImage>| {
        let p = &ctx.params;
        let mut parts = Vec::new();
        if p.turn != 0.0 {
            parts.push(format!("turned {}°", p.turn));
        }
        if p.tilt != 0.0 {
            parts.push(format!("tilted {}°", p.tilt));
        }
        if p.lean != 0.0 {
            parts.push(format!("leaning {}°", p.lean));
        }
        let what = if parts.is_empty() {
            row(Icon::RefreshCw, "Flat").muted_if(true)
        } else {
            row(
                Icon::RefreshCw,
                crate::card_rows::sentence(&parts.join(", ")),
            )
        };
        vec![Some(what), size_row(ctx, |s| tilted_size(s, p))]
    })
}

fn levels() -> NodeDef {
    image_node::<AutoLevels>(
        LEVELS_TYPE,
        "Auto levels",
        "Stretches a dull or washed-out image's levels from black to white.",
    )
    .card(|ctx: &CardCtx<'_, AutoLevels>| {
        let p = &ctx.params;
        let mut text = match p.method {
            LevelsMethod::Stretch => "Levels stretched".to_owned(),
            LevelsMethod::Equalize => "Levels evened out".to_owned(),
        };
        if p.per_channel {
            text.push_str(" · each colour");
        }
        if p.strength < 1.0 {
            let _ = write!(text, " · {}", percent_text(p.strength, f64::INFINITY));
        }
        vec![
            Some(row(Icon::Sun, text).muted_if(p.strength <= 0.0)),
            size_row(ctx, |s| s),
        ]
    })
}

fn picture_label(i: usize, _last: Option<usize>) -> String {
    format!("Image {}", i + 1)
}

fn grid() -> NodeDef {
    image_node::<ImageGrid>(
        GRID_TYPE,
        "Grid",
        "Puts any number of images in a row, a column or a grid, with gaps and a background.",
    )
    .inputs(vec![])
    .family(PortFamily::new(
        IN_PREFIX,
        PortType::Media,
        2,
        picture_label,
    ))
    .card(|ctx: &CardCtx<'_, ImageGrid>| {
        let p = &ctx.params;
        let mut ports: Vec<(usize, (u32, u32))> = ctx
            .inputs
            .pixels
            .iter()
            .filter_map(|(port, size)| Some((in_index(port)?, *size)))
            .collect();
        ports.sort_by_key(|(i, _)| *i);
        let connected = ctx
            .inputs
            .connected
            .iter()
            .filter(|c| in_index(c).is_some())
            .count();
        let count = match connected {
            0 => missing(Icon::Columns2, "Connect two images"),
            1 => missing(Icon::Columns2, "Connect one more image"),
            n => {
                let (columns, rows) = grid_shape(n, p);
                row(
                    Icon::Columns2,
                    format!(
                        "{} · {columns} × {rows}",
                        crate::text::plural(numbers::float(n), "image")
                    ),
                )
            }
        };
        let sizes: Vec<(f64, f64)> = ports
            .iter()
            .map(|(_, (w, h))| (f64::from(*w), f64::from(*h)))
            .collect();
        let size = result_size(ctx).or_else(|| {
            (sizes.len() == connected && connected >= 2).then(|| grid_plan(&sizes, p).canvas)
        });
        vec![
            Some(count),
            size.map(|(w, h)| row(Icon::Ruler, pixels(w, h))),
        ]
    })
}

fn frame() -> NodeDef {
    image_node::<FrameScreenshot>(
        FRAME_TYPE,
        "Frame screenshot",
        "Gives a screenshot rounded corners and a soft shadow on a padded background, ready for a post.",
    )
    .card(|ctx: &CardCtx<'_, FrameScreenshot>| {
        let p = &ctx.params;
        vec![
            Some(p.background.card_row()),
            size_row(ctx, |s| framed_placement(s, p).0),
        ]
    })
}

#[choice]
pub enum BeforeAfterLayout {
    #[default]
    #[label("One picture, split")]
    Split,
    #[label("Side by side")]
    SideBySide,
    #[label("Stacked")]
    Stacked,
}

#[settings]
pub struct BeforeAfter {
    #[setting(
        "Layout",
        help = "Split shows the before on one side of a divider and the after on the other, both at the before's size."
    )]
    pub layout: BeforeAfterLayout,
    #[setting(
        "Divider at",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        help = "How far across the picture the divider sits.",
        show_if = { "layout": ["split"] }
    )]
    pub position: f64,
    #[setting(
        "Divider angle",
        range(min = -45, max = 45),
        step = 1,
        unit = "°",
        show_if = { "layout": ["split"] }
    )]
    pub angle: f64,
    #[setting(
        "Divider width",
        default = 4,
        range(min = 0, max = 80),
        unit = "px",
        advanced
    )]
    pub divider_width: u32,
    #[setting("Divider colour", default = "#ffffff".to_owned(), widget = "color", advanced)]
    pub divider_color: String,
    #[setting("Labels", default = true)]
    pub labels: bool,
    #[setting("Before label", default = "Before".to_owned(), show_if = { "labels": ["true"] })]
    pub before_label: String,
    #[setting("After label", default = "After".to_owned(), show_if = { "labels": ["true"] })]
    pub after_label: String,
    #[setting("Font", widget = "font", zero_label = "Main font", show_if = { "labels": ["true"] }, advanced)]
    pub font: String,
    #[setting(
        "Weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_weight: u32,
    #[setting(
        "Text size",
        range(min = 0, max = 400),
        unit = "px",
        zero_label = "Fit the picture",
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_size: u32,
    #[setting(
        "Text colour",
        default = "#ffffff".to_owned(),
        widget = "color",
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_color: String,
    #[setting(
        "Label background",
        default = "rgba(0, 0, 0, 0.6)".to_owned(),
        widget = "color",
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_background: String,
    #[setting(
        "Label corners",
        default = 30,
        range(min = 0, max = 50),
        unit = "%",
        help = "Of the label's height; 50 makes a pill.",
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_radius: u32,
    #[setting(
        "Labels at",
        show_if = { "labels": ["true"] },
        advanced
    )]
    pub label_position: LabelEdge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeforeAfterPlan {
    pub size: (u32, u32),
    pub before: Rect,
    pub after: Rect,
}

const LABEL_SHARE: f64 = 0.045;
const SMALLEST_LABEL: f64 = 12.0;

pub fn before_after_plan(
    before: (f64, f64),
    after: (f64, f64),
    p: &BeforeAfter,
) -> BeforeAfterPlan {
    let gap = f64::from(p.divider_width);
    let (bw, bh) = (
        f64::from(whole(before.0).max(1)),
        f64::from(whole(before.1).max(1)),
    );
    let (aw, ah) = (after.0.max(1.0), after.1.max(1.0));
    let place = |x: f64, y: f64, width: f64, height: f64| Rect {
        x,
        y,
        width,
        height,
    };
    match p.layout {
        BeforeAfterLayout::Split => BeforeAfterPlan {
            size: (whole(bw), whole(bh)),
            before: place(0.0, 0.0, bw, bh),
            after: place(0.0, 0.0, bw, bh),
        },
        BeforeAfterLayout::SideBySide => {
            let width = f64::from(whole(aw * bh / ah).max(1));
            BeforeAfterPlan {
                size: (whole(bw + gap + width), whole(bh)),
                before: place(0.0, 0.0, bw, bh),
                after: place(bw + gap, 0.0, width, bh),
            }
        }
        BeforeAfterLayout::Stacked => {
            let height = f64::from(whole(ah * bw / aw).max(1));
            BeforeAfterPlan {
                size: (whole(bw), whole(bh + gap + height)),
                before: place(0.0, 0.0, bw, bh),
                after: place(0.0, bh + gap, bw, height),
            }
        }
    }
}

pub fn label_size(plan: &BeforeAfterPlan, p: &BeforeAfter) -> f64 {
    if p.label_size > 0 {
        return f64::from(p.label_size);
    }
    let short = plan.before.width.min(plan.before.height);
    (short * LABEL_SHARE).round().max(SMALLEST_LABEL)
}

fn before_after() -> NodeDef {
    image_node::<BeforeAfter>(
        BEFORE_AFTER_TYPE,
        "Before and after",
        "Shows two pictures side by side, stacked, or split by a divider, with Before and After labels.",
    )
    .uses_project_fonts()
    .inputs(vec![
        (IMAGE_PORT, port(PortType::Media, "Before")),
        (AFTER_PORT, port(PortType::Media, "After")),
    ])
    .card(|ctx: &CardCtx<'_, BeforeAfter>| {
        let p = &ctx.params;
        let layout = match p.layout {
            BeforeAfterLayout::Split => {
                let at = format!("Split at {}", percent_text(p.position, f64::INFINITY));
                if p.angle == 0.0 {
                    at
                } else {
                    format!("{at}, turned {}°", numbers::truncate::<i64>(p.angle.round()))
                }
            }
            BeforeAfterLayout::SideBySide => "Side by side".to_owned(),
            BeforeAfterLayout::Stacked => "Stacked".to_owned(),
        };
        let labels = if p.labels {
            format!("{layout} · “{}” and “{}”", p.before_label, p.after_label)
        } else {
            layout
        };
        let planned = match (input_size(ctx, IMAGE_PORT), input_size(ctx, AFTER_PORT)) {
            (Some(before), Some(after)) => {
                let size = before_after_plan(before, after, p).size;
                Some(row(Icon::Ruler, pixels(f64::from(size.0), f64::from(size.1))))
            }
            _ => None,
        };
        vec![
            Some(row(Icon::Columns2, labels)),
            (!ctx.connected(AFTER_PORT)).then(|| missing(Icon::Image, "Connect the after picture")),
            result_size(ctx)
                .map(|(w, h)| row(Icon::Ruler, pixels(w, h)))
                .or(planned),
        ]
    })
}

#[choice]
pub enum MagnifySize {
    #[default]
    #[label("The two boxes")]
    TwoBoxes,
    #[label("The area and a zoom")]
    AreaAndZoom,
    #[label("Where it shows and a zoom")]
    ShownAndZoom,
}

#[choice]
pub enum MagnifyShape {
    #[default]
    #[label("Circle")]
    Circle,
    #[label("Rounded box")]
    Rounded,
}

fn magnify_area() -> Vec<Area> {
    vec![Area {
        x: 0.15,
        y: 0.2,
        width: 0.12,
        height: 0.12,
    }]
}

fn magnify_shown() -> Vec<Area> {
    vec![Area {
        x: 0.55,
        y: 0.35,
        width: 0.3,
        height: 0.3,
    }]
}

#[settings]
pub struct Magnify {
    #[setting(
        "Size from",
        help = "Any two of the area, where it shows and the zoom decide the third. It never shrinks: a view drawn smaller than the area grows to fit it.",
        advanced
    )]
    pub size_from: MagnifySize,
    #[setting(
        "Area to enlarge",
        default = magnify_area(),
        length(min = 1, max = 1),
        widget = "boxes",
        help = "Drag on the picture over the part to enlarge. With Where it shows and a zoom, only its middle counts."
    )]
    pub area: Vec<Area>,
    #[setting(
        "Where it shows",
        default = magnify_shown(),
        length(min = 1, max = 1),
        widget = "boxes",
        help = "Drag where the enlarged view goes. With The area and a zoom, only its middle counts."
    )]
    pub shown: Vec<Area>,
    #[setting(
        "Zoom",
        default = 2.5,
        range(min = 1, max = 10),
        step = 0.1,
        unit = "×",
        show_if = { "sizeFrom": ["area-and-zoom", "shown-and-zoom"] },
        advanced
    )]
    pub zoom: f64,
    #[setting("Shape")]
    pub shape: MagnifyShape,
    #[setting(
        "Corner radius",
        default = 16,
        range(min = 0, max = 400),
        unit = "px",
        show_if = { "shape": ["rounded"] },
        advanced
    )]
    pub radius: u32,
    #[setting(
        "Border width",
        default = 4,
        range(min = 0, max = 40),
        unit = "px",
        advanced
    )]
    pub border_width: u32,
    #[setting("Border colour", default = "#ffffff".to_owned(), widget = "color")]
    pub border_color: String,
    #[setting(
        "Line to the spot",
        default = true,
        help = "Outlines the enlarged part and joins it to the view."
    )]
    pub line: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Line width",
        range(min = 1, max = 40),
        unit = "px",
        zero_label = "Half the border",
        show_if = { "line": ["true"] },
        "type" = "integer",
        advanced
    )]
    pub line_width: Option<u32>,
    #[setting(
        "Quality",
        default = Resample::Smooth,
        advanced
    )]
    pub quality: Resample,
    #[setting("Shadow", help = "A soft shadow under the enlarged view.", advanced)]
    pub shadow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnifyPlan {
    pub area: Rect,
    pub shown: Rect,
    pub zoom: f64,
}

fn middle(rect: Rect) -> (f64, f64) {
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn around(centre: (f64, f64), size: (f64, f64)) -> Rect {
    Rect {
        x: centre.0 - size.0 / 2.0,
        y: centre.1 - size.1 / 2.0,
        width: size.0,
        height: size.1,
    }
}

fn kept_in(source: (f64, f64), rect: Rect) -> Rect {
    let (width, height) = (rect.width.min(source.0), rect.height.min(source.1));
    Rect {
        x: rect.x.clamp(0.0, source.0 - width),
        y: rect.y.clamp(0.0, source.1 - height),
        width,
        height,
    }
}

fn shaped(rect: Rect, shape: MagnifyShape, covering: bool) -> Rect {
    match shape {
        MagnifyShape::Circle => {
            let side = if covering {
                rect.width.max(rect.height)
            } else {
                rect.width.min(rect.height)
            };
            around(middle(rect), (side, side))
        }
        MagnifyShape::Rounded => rect,
    }
}

fn first_box(source: (f64, f64), areas: &[Area]) -> Rect {
    areas.first().map_or(
        Rect {
            x: 0.0,
            y: 0.0,
            width: source.0,
            height: source.1,
        },
        |area| area_rect(source, area),
    )
}

pub fn magnify_plan(source: (f64, f64), p: &Magnify) -> MagnifyPlan {
    let area_box = shaped(first_box(source, &p.area), p.shape, true);
    let shown_box = shaped(first_box(source, &p.shown), p.shape, false);
    let zoom = p.zoom.max(1.0);
    let area_box = kept_in(source, area_box);
    let (area, shown) = match p.size_from {
        MagnifySize::TwoBoxes => {
            let drawn = (shown_box.width / area_box.width).min(shown_box.height / area_box.height);
            let shown = if drawn < 1.0 {
                let grown = (shown_box.width / drawn, shown_box.height / drawn);
                kept_in(source, around(middle(shown_box), grown))
            } else {
                shown_box
            };
            let scale = (shown.width / area_box.width).min(shown.height / area_box.height);
            let size = (shown.width / scale, shown.height / scale);
            (kept_in(source, around(middle(area_box), size)), shown)
        }
        MagnifySize::AreaAndZoom => {
            let size = (area_box.width * zoom, area_box.height * zoom);
            (area_box, kept_in(source, around(middle(shown_box), size)))
        }
        MagnifySize::ShownAndZoom => {
            let size = (shown_box.width / zoom, shown_box.height / zoom);
            (kept_in(source, around(middle(area_box), size)), shown_box)
        }
    };
    MagnifyPlan {
        area,
        shown,
        zoom: shown.width / area.width.max(1.0),
    }
}

fn magnify() -> NodeDef {
    image_node::<Magnify>(
        MAGNIFY_TYPE,
        "Magnify",
        "Shows one part of an image enlarged in a circle or a rounded box beside it, joined to the spot by a line, so small text reads.",
    )
    .card(|ctx: &CardCtx<'_, Magnify>| {
        let p = &ctx.params;
        let shape = match p.shape {
            MagnifyShape::Circle => "circle",
            MagnifyShape::Rounded => "rounded box",
        };
        let zoom = match (input_size(ctx, IMAGE_PORT), p.size_from) {
            (Some(source), _) => Some(magnify_plan(source, p).zoom),
            (None, MagnifySize::TwoBoxes) => None,
            (None, _) => Some(p.zoom),
        };
        let text = zoom.map_or_else(
            || format!("Two boxes · {shape}"),
            |z| format!("{}× · {shape}", (z * 10.0).round() / 10.0),
        );
        vec![Some(row(Icon::ZoomIn, text)), size_row(ctx, |s| s)]
    })
}

#[settings]
pub struct CompareScreenshots {
    #[setting(
        "Sensitivity",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        help = "Higher marks fainter changes. Lower it when compressed pictures show changes everywhere."
    )]
    pub sensitivity: f64,
    #[setting(
        "Smallest change",
        default = 4,
        range(min = 1, max = 400),
        unit = "px",
        help = "Changes smaller than this across are left unmarked.",
        advanced
    )]
    pub smallest: u32,
    #[setting(
        "Padding",
        default = 8,
        range(min = 0, max = 100),
        unit = "px",
        help = "Space around each change. Changes whose boxes touch become one box.",
        advanced
    )]
    pub padding: u32,
    #[setting(
        "Box width",
        default = 4,
        range(min = 1, max = 40),
        unit = "px",
        advanced
    )]
    pub box_width: u32,
    #[setting("Box colour", default = "#ef4444".to_owned(), widget = "color")]
    pub box_color: String,
    #[setting(
        "Unchanged pixels",
        default = "#00000000",
        widget = "color",
        check = colour,
        help = "What the Differences picture shows where nothing changed.",
        advanced
    )]
    pub differences_background: String,
}

const DIFFERENT_SIZES: &str = "The two pictures are different sizes";

pub fn compare_threshold(sensitivity: f64) -> u8 {
    numbers::truncate::<u8>((2.0 + 64.0 * (1.0 - sensitivity.clamp(0.0, 1.0))).round())
}

fn compare() -> NodeDef {
    image_node::<CompareScreenshots>(
        COMPARE_TYPE,
        "Compare screenshots",
        "Boxes what changed between two captures of the same page, and makes a picture of only the differences.",
    )
    .inputs(vec![
        (IMAGE_PORT, port(PortType::Media, "Before")),
        (AFTER_PORT, port(PortType::Media, "After")),
    ])
    .outputs(vec![
        (IMAGE_PORT, port(PortType::Image, "Marked")),
        (DIFFERENCES_PORT, port(PortType::Image, "Differences")),
    ])
    .card(|ctx: &CardCtx<'_, CompareScreenshots>| {
        let changes = ctx.result.and_then(|r| {
            r.measured.iter().find_map(|m| match m {
                Measured::Changes { count } => Some(*count),
                Measured::NoiseLevel { .. } | Measured::File { .. } | Measured::Fitted { .. } => None,
            })
        });
        let found = match changes {
            Some(0) => row(Icon::CircleCheck, "No change"),
            Some(count) => row(Icon::ScanSearch, plural(f64::from(count), "change")),
            None => row(Icon::ScanSearch, "Marks what changed").tone(Tone::Muted),
        };
        let sizes = (input_size(ctx, IMAGE_PORT), input_size(ctx, AFTER_PORT));
        vec![
            Some(found),
            (!ctx.connected(AFTER_PORT)).then(|| missing(Icon::Image, "Connect the after picture")),
            matches!(sizes, (Some(a), Some(b)) if a != b)
                .then(|| missing(Icon::TriangleAlert, DIFFERENT_SIZES)),
        ]
    })
    .compile(|ctx: &Ctx<'_, CompareScreenshots>| {
        let outputs = [IMAGE_PORT, DIFFERENCES_PORT]
            .into_iter()
            .filter_map(|port| {
                let value = ctx.result?.output(port)?;
                matches!(value, CompileValue::Image(_)).then(|| (port.to_owned(), value.clone()))
            })
            .collect();
        Ok(Some(outputs))
    })
}

#[choice]
pub enum MarkKind {
    #[default]
    #[label("Arrow")]
    Arrow,
    #[label("Box")]
    Rectangle,
    #[label("Oval")]
    Oval,
    #[label("Step number")]
    Step,
    #[label("Text")]
    Text,
}

impl MarkKind {
    pub fn has_text(self) -> bool {
        matches!(self, Self::Step | Self::Text)
    }
}

#[settings]
pub struct Mark {
    #[setting("Kind")]
    pub kind: MarkKind,
    #[setting(default = 0.3, range(min = 0, max = 1), hidden)]
    pub x: f64,
    #[setting(default = 0.3, range(min = 0, max = 1), hidden)]
    pub y: f64,
    #[setting(default = 0.5, range(min = 0, max = 1), hidden)]
    pub x2: f64,
    #[setting(default = 0.5, range(min = 0, max = 1), hidden)]
    pub y2: f64,
    #[setting("Colour", default = "#ef4444".to_owned(), widget = "color")]
    pub color: String,
    #[setting(
        "Line width",
        default = 6,
        range(min = 1, max = 60),
        unit = "px",
        show_if = { "kind": ["arrow", "rectangle", "oval"] }
    )]
    pub line_width: u32,
    #[setting("Text", show_if = { "kind": ["text"] })]
    pub text: String,
    #[setting(
        "Size",
        default = 32,
        range(min = 8, max = 400),
        unit = "px",
        show_if = { "kind": ["step", "text"] }
    )]
    pub size: u32,
    #[setting(
        "Font",
        widget = "font",
        zero_label = "Main font",
        show_if = { "kind": ["step", "text"] }
    )]
    pub font: String,
    #[setting(
        "Weight",
        default = 700,
        range(min = 100, max = 900),
        step = 100,
        show_if = { "kind": ["step", "text"] }
    )]
    pub weight: u32,
    #[setting(
        "Number colour",
        default = "#ffffff".to_owned(),
        widget = "color",
        show_if = { "kind": ["step"] }
    )]
    pub text_color: String,
    #[setting(
        "Head size",
        default = 1.0,
        range(min = 0.3, max = 3),
        step = 0.1,
        show_if = { "kind": ["arrow"] }
    )]
    pub head_size: f64,
    #[setting(
        "Corner radius",
        range(min = 0, max = 200),
        unit = "px",
        show_if = { "kind": ["rectangle"] }
    )]
    pub radius: u32,
    #[setting(
        "Fill",
        range(min = 0, max = 1),
        step = 0.05,
        help = "How strongly the colour fills the inside.",
        show_if = { "kind": ["rectangle", "oval"] }
    )]
    pub fill: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        show_if = { "kind": ["text"] }
    )]
    pub text_background: String,
}

#[settings]
pub struct Annotate {
    #[setting(
        "Marks",
        length(max = 50),
        widget = "marks",
        help = "Pick a kind and drag on the picture. Step numbers count up in the order they're listed."
    )]
    pub marks: Vec<Mark>,
    #[setting("First step number", default = 1, range(min = 0, max = 999), advanced)]
    pub first_step: u32,
}

fn annotate() -> NodeDef {
    image_node::<Annotate>(
        ANNOTATE_TYPE,
        "Annotate",
        "Draws arrows, boxes, ovals, numbered steps and text on a picture, so a still explains itself.",
    )
    .uses_project_fonts()
    .card(|ctx: &CardCtx<'_, Annotate>| {
        let marks = &ctx.params.marks;
        let steps = marks.iter().filter(|m| m.kind == MarkKind::Step).count();
        let summary = match (marks.len(), steps) {
            (0, _) => row(Icon::Shapes, "Draw marks on the picture").tone(Tone::Muted),
            (n, 0) => row(Icon::Shapes, plural(numbers::float(n), "mark")),
            (n, s) => row(
                Icon::Shapes,
                format!("{} · {}", plural(numbers::float(n), "mark"), plural(numbers::float(s), "step")),
            ),
        };
        let empty = marks
            .iter()
            .any(|m| m.kind == MarkKind::Text && m.text.trim().is_empty());
        vec![
            Some(summary),
            empty.then(|| missing(Icon::TriangleAlert, "A text mark has no text")),
            size_row(ctx, |s| s),
        ]
    })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        resize().look(NodeLook::of("lucide:image-upscale")),
        crop().look(NodeLook::of("hugeicons:image-crop")),
        rotate().look(NodeLook::of("lucide:rotate-cw-square").moves(NodeMotion::Turn)),
        adjust().look(NodeLook::of("lucide:sliders-vertical")),
        blur().look(NodeLook::of("lucide:droplet").moves(NodeMotion::RackFocus)),
        place().look(NodeLook::of("lucide:stamp")),
        format().look(NodeLook::of("lucide:file-image")),
        redact().look(NodeLook::of("lucide:rectangle-ellipsis")),
        spotlight().look(NodeLook::of("lucide:flashlight")),
        trim().look(NodeLook::of("lucide:shrink")),
        clear_colour().look(NodeLook::of("lucide:eraser")),
        outline().look(NodeLook::of("tabler:sticker")),
        tilt().look(NodeLook::of("tabler:perspective").moves(NodeMotion::Open)),
        levels().look(NodeLook::of("tabler:chart-histogram")),
        grid().look(NodeLook::of("tabler:layout-collage")),
        frame().look(NodeLook::of("hugeicons:image-composition")),
        before_after().look(NodeLook::of("lucide:columns-2")),
        magnify().look(NodeLook::of("tabler:zoom-in-area")),
        compare().look(NodeLook::of("tabler:layers-difference")),
        annotate().look(NodeLook::of("tabler:photo-edit")),
    ]
    .into_iter()
    .map(|def| def.needs(FileNeed::Image))
    .collect()
}
