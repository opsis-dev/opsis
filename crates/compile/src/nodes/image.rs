use opsis_types::cards::Icon;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::values::CompileValue;
use opsis_types::workflow::PortType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::node::{CardCtx, Category, Ctx, NodeDef, Phase, PortFamily, RowExt, missing, port, row};
use crate::nodes::effects::Area;
use crate::params::Params;
use crate::ports::{IN_PREFIX, in_index};

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

pub const BLUR_TYPE: &str = "image.blur";

pub const ADJUST_TYPE: &str = "image.adjust";

pub const ROTATE_TYPE: &str = "image.rotate";

pub const CROP_TYPE: &str = "image.crop";

pub const RESIZE_TYPE: &str = "image.resize";

pub const IMAGE_PORT: &str = "image";
pub const OVERLAY_PORT: &str = "overlay";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFit {
    #[default]
    Inside,
    Cover,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Resample {
    Fast,
    Smooth,
    #[default]
    Sharp,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ResizeImage {
    #[default = 1920]
    #[schemars(
        description = "Width",
        range(min = 0, max = 16384),
        extend("x-unit" = "px", "x-help" = "0 follows the height and keeps the proportion.")
    )]
    pub width: u32,
    #[schemars(
        description = "Height",
        range(min = 0, max = 16384),
        extend("x-unit" = "px", "x-help" = "0 follows the width and keeps the proportion.")
    )]
    pub height: u32,
    #[schemars(
        description = "Fit",
        extend(
            "x-labels" = { "inside": "Fit inside", "cover": "Fill and crop", "exact": "Stretch" },
            "x-help" = "When both sides are set: fit the whole image inside them, fill them and crop the rest, or stretch to exactly that size."
        )
    )]
    pub fit: ResizeFit,
    #[schemars(
        description = "Quality",
        extend("x-labels" = { "fast": "Fast", "smooth": "Smooth", "sharp": "Sharp" }, "x-advanced" = true)
    )]
    pub quality: Resample,
}

impl Params for ResizeImage {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CropMode {
    #[default]
    Area,
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CropImage {
    #[schemars(description = "Crop by", extend("x-labels" = { "area": "Area", "margins": "Margins" }))]
    pub mode: CropMode,
    #[default(whole_area())]
    #[schemars(
        description = "Area to keep",
        length(min = 1, max = 1),
        extend(
            "x-widget" = "boxes",
            "x-show-if" = { "mode": ["area"] },
            "x-help" = "Drag on the picture to choose the part to keep."
        )
    )]
    pub area: Vec<Area>,
    #[schemars(description = "Top", range(min = 0, max = 16384), extend("x-unit" = "px", "x-show-if" = { "mode": ["margins"] }))]
    pub top: u32,
    #[schemars(description = "Right", range(min = 0, max = 16384), extend("x-unit" = "px", "x-show-if" = { "mode": ["margins"] }))]
    pub right: u32,
    #[schemars(description = "Bottom", range(min = 0, max = 16384), extend("x-unit" = "px", "x-show-if" = { "mode": ["margins"] }))]
    pub bottom: u32,
    #[schemars(description = "Left", range(min = 0, max = 16384), extend("x-unit" = "px", "x-show-if" = { "mode": ["margins"] }))]
    pub left: u32,
}

impl Params for CropImage {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Turn {
    #[default]
    None,
    Right,
    Half,
    Left,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RotateImage {
    #[default(Turn::Right)]
    #[schemars(
        description = "Turn",
        extend("x-labels" = { "none": "None", "right": "90° right", "half": "180°", "left": "90° left" })
    )]
    pub turn: Turn,
    #[schemars(description = "Flip left to right")]
    pub flip_horizontal: bool,
    #[schemars(description = "Flip upside down")]
    pub flip_vertical: bool,
    #[schemars(
        description = "Straighten",
        range(min = -45, max = 45),
        extend("x-step" = 0.5, "x-unit" = "°", "x-help" = "Tilts the picture by a small angle. The corners fill with the background colour.")
    )]
    pub angle: f64,
    #[default = "#00000000"]
    #[schemars(
        description = "Background",
        extend("x-widget" = "color", "x-check" = { "rule": "colour", "message": "Pick a colour" }, "x-help" = "Fills the corners a tilt opens up.")
    )]
    pub background: String,
}

impl Params for RotateImage {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AdjustImage {
    #[schemars(description = "Brightness", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub brightness: f64,
    #[schemars(description = "Contrast", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub contrast: f64,
    #[schemars(description = "Saturation", range(min = -1, max = 1), extend("x-step" = 0.05, "x-help" = "-1 takes all the colour out."))]
    pub saturation: f64,
    #[schemars(description = "Hue", range(min = -180, max = 180), extend("x-step" = 1, "x-unit" = "°"))]
    pub hue: f64,
    #[schemars(description = "Black and white")]
    pub grayscale: bool,
    #[schemars(description = "Invert colours")]
    pub invert: bool,
}

impl Params for AdjustImage {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BlurImage {
    #[schemars(description = "Blur", range(min = 0, max = 100), extend("x-step" = 0.5, "x-unit" = "px"))]
    pub blur: f64,
    #[default = 1.0]
    #[schemars(description = "Sharpen", range(min = 0, max = 5), extend("x-step" = 0.1))]
    pub sharpen: f64,
    #[default = 4]
    #[schemars(
        description = "Sharpen threshold",
        range(min = 0, max = 255),
        extend("x-advanced" = true, "x-help" = "Differences smaller than this stay as they are, so flat areas don't get grainy.")
    )]
    pub threshold: u32,
}

impl Params for BlurImage {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaceImage {
    #[default = 0.85]
    #[schemars(
        description = "Place at",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-widget" = "spot", "x-help" = "Click the picture to choose where the middle of the placed image goes.")
    )]
    pub x: f64,
    #[default = 0.85]
    #[schemars(description = "Place at vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub y: f64,
    #[default = 20.0]
    #[schemars(
        description = "Size",
        range(min = 1, max = 100),
        extend("x-step" = 1, "x-unit" = "%", "x-help" = "The placed image's width, as a share of the picture's width.")
    )]
    pub size: f64,
    #[default = 1.0]
    #[schemars(description = "Opacity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub opacity: f64,
}

impl Params for PlaceImage {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StillFileFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
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

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageFormat {
    #[default(StillFileFormat::Jpeg)]
    #[schemars(
        description = "Format",
        extend("x-labels" = { "png": "PNG", "jpeg": "JPEG", "webp": "WebP (lossless)", "avif": "AVIF" })
    )]
    pub format: StillFileFormat,
    #[default = 85]
    #[schemars(
        description = "Quality",
        range(min = 1, max = 100),
        extend("x-show-if" = { "format": ["jpeg", "avif"] }, "x-help" = "Higher looks better and makes a larger file.")
    )]
    pub quality: u32,
    #[default = 6]
    #[schemars(
        description = "Speed",
        range(min = 1, max = 10),
        extend("x-show-if" = { "format": ["avif"] }, "x-advanced" = true, "x-help" = "Higher saves faster and makes a larger file.")
    )]
    pub speed: u32,
    #[default = "#ffffff"]
    #[schemars(
        description = "Background",
        extend(
            "x-widget" = "color",
            "x-check" = { "rule": "colour", "message": "Pick a colour" },
            "x-show-if" = { "format": ["jpeg"] },
            "x-help" = "JPEG has no see-through parts, so they're filled with this colour."
        )
    )]
    pub background: String,
}

impl Params for ImageFormat {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RedactStyle {
    #[default]
    Pixelate,
    Blur,
    Solid,
}

impl RedactStyle {
    fn word(self) -> &'static str {
        match self {
            Self::Pixelate => "pixelated",
            Self::Blur => "blurred",
            Self::Solid => "covered",
        }
    }
}

fn hidden_area() -> Vec<Area> {
    vec![Area {
        x: 0.35,
        y: 0.4,
        width: 0.3,
        height: 0.2,
    }]
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RedactImage {
    #[default(hidden_area())]
    #[schemars(
        description = "Areas to hide",
        length(min = 1, max = 8),
        extend("x-widget" = "boxes", "x-help" = "Drag on the picture to mark each part to hide.")
    )]
    pub areas: Vec<Area>,
    #[schemars(
        description = "Hide with",
        extend("x-labels" = { "pixelate": "Large pixels", "blur": "Blur", "solid": "Solid colour" })
    )]
    pub style: RedactStyle,
    #[default = 16]
    #[schemars(
        description = "Pixel size",
        range(min = 2, max = 256),
        extend("x-unit" = "px", "x-show-if" = { "style": ["pixelate"] })
    )]
    pub block_size: u32,
    #[default = 16.0]
    #[schemars(
        description = "Blur",
        range(min = 1, max = 100),
        extend("x-step" = 0.5, "x-unit" = "px", "x-show-if" = { "style": ["blur"] })
    )]
    pub blur: f64,
    #[default = "#000000"]
    #[schemars(
        description = "Colour",
        extend(
            "x-widget" = "color",
            "x-check" = { "rule": "colour", "message": "Pick a colour" },
            "x-show-if" = { "style": ["solid"] }
        )
    )]
    pub color: String,
}

impl Params for RedactImage {}

fn lit_area() -> Vec<Area> {
    vec![Area {
        x: 0.3,
        y: 0.3,
        width: 0.4,
        height: 0.4,
    }]
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpotlightImage {
    #[default(lit_area())]
    #[schemars(
        description = "Areas to light",
        length(min = 1, max = 8),
        extend("x-widget" = "boxes", "x-help" = "Drag on the picture to mark each part to keep bright.")
    )]
    pub areas: Vec<Area>,
    #[default = 0.6]
    #[schemars(description = "Darken the rest", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub dim: f64,
    #[schemars(description = "Blur the rest", range(min = 0, max = 50), extend("x-step" = 0.5, "x-unit" = "px"))]
    pub blur: f64,
    #[default = 12.0]
    #[schemars(description = "Corner radius", range(min = 0, max = 200), extend("x-step" = 1, "x-unit" = "px"))]
    pub radius: f64,
    #[default = 8.0]
    #[schemars(description = "Soft edge", range(min = 0, max = 100), extend("x-step" = 1, "x-unit" = "px"))]
    pub softness: f64,
}

impl Params for SpotlightImage {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TrimEdges {
    #[default = 8]
    #[schemars(
        description = "Tolerance",
        range(min = 0, max = 255),
        extend("x-help" = "How far a pixel's colour may be from the corner's and still count as empty edge.")
    )]
    pub tolerance: u32,
    #[schemars(description = "Keep around", range(min = 0, max = 2000), extend("x-unit" = "px"))]
    pub padding: u32,
}

impl Params for TrimEdges {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ClearColour {
    #[schemars(
        description = "Colour",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "The corner's colour",
            "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" }
        )
    )]
    pub color: String,
    #[default = 10.0]
    #[schemars(
        description = "Tolerance",
        range(min = 0, max = 100),
        extend("x-step" = 1, "x-unit" = "%", "x-help" = "How different a colour may be and still be made see-through.")
    )]
    pub tolerance: f64,
    #[default = true]
    #[schemars(
        description = "Only areas touching the edges",
        extend("x-help" = "Keeps the same colour inside the picture, such as the white inside a logo.")
    )]
    pub edges_only: bool,
    #[default = 10.0]
    #[schemars(description = "Soft edge", range(min = 0, max = 100), extend("x-step" = 1, "x-unit" = "%"))]
    pub softness: f64,
}

impl Params for ClearColour {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OutlineImage {
    #[default = 8.0]
    #[schemars(description = "Width", range(min = 1, max = 200), extend("x-step" = 1, "x-unit" = "px"))]
    pub width: f64,
    #[default = "#ffffff"]
    #[schemars(
        description = "Colour",
        extend("x-widget" = "color", "x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub color: String,
    #[schemars(description = "Soft edge", range(min = 0, max = 50), extend("x-step" = 1, "x-unit" = "px"))]
    pub softness: f64,
    #[schemars(description = "Shadow")]
    pub shadow: bool,
}

impl Params for OutlineImage {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TiltImage {
    #[default = 18.0]
    #[schemars(
        description = "Turn",
        range(min = -60, max = 60),
        extend("x-step" = 1, "x-unit" = "°", "x-help" = "Turns the picture away on its right side, or its left with a negative angle.")
    )]
    pub turn: f64,
    #[default = 8.0]
    #[schemars(
        description = "Tilt",
        range(min = -60, max = 60),
        extend("x-step" = 1, "x-unit" = "°", "x-help" = "Leans the top back, or the bottom with a negative angle.")
    )]
    pub tilt: f64,
    #[schemars(description = "Lean", range(min = -45, max = 45), extend("x-step" = 1, "x-unit" = "°"))]
    pub lean: f64,
    #[default = "#00000000"]
    #[schemars(
        description = "Background",
        extend("x-widget" = "color", "x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
    #[default = true]
    #[schemars(description = "Shadow")]
    pub shadow: bool,
}

impl Params for TiltImage {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LevelsMethod {
    #[default]
    Stretch,
    Equalize,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoLevels {
    #[schemars(
        description = "Method",
        extend(
            "x-labels" = { "stretch": "Stretch", "equalize": "Even out" },
            "x-help" = "Stretch spreads the levels from black to white. Even out spreads them evenly, which is stronger."
        )
    )]
    pub method: LevelsMethod,
    #[default = 1.0]
    #[schemars(description = "Strength", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub strength: f64,
    #[default = 0.5]
    #[schemars(
        description = "Ignore the darkest and lightest",
        range(min = 0, max = 5),
        extend("x-step" = 0.1, "x-unit" = "%", "x-show-if" = { "method": ["stretch"] })
    )]
    pub clip: f64,
    #[schemars(description = "Each colour on its own", extend("x-help" = "Also corrects a colour cast, but can shift colours."))]
    pub per_channel: bool,
}

impl Params for AutoLevels {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GridLayout {
    Row,
    Column,
    #[default]
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GridFit {
    SameHeight,
    SameWidth,
    #[default]
    Cells,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageGrid {
    #[schemars(description = "Layout", extend("x-labels" = { "row": "One row", "column": "One column", "grid": "Grid" }))]
    pub layout: GridLayout,
    #[default = 2]
    #[schemars(description = "Columns", range(min = 1, max = 12), extend("x-show-if" = { "layout": ["grid"] }))]
    pub columns: u32,
    #[schemars(
        description = "Sizes",
        extend(
            "x-labels" = { "same-height": "Same height", "same-width": "Same width", "cells": "Equal cells" },
            "x-help" = "Same height and same width shrink every image to the smallest one's. Equal cells makes every cell as big as the largest image and centres each image in its own; nothing is enlarged."
        )
    )]
    pub fit: GridFit,
    #[default = 24]
    #[schemars(description = "Gap", range(min = 0, max = 1000), extend("x-unit" = "px"))]
    pub gap: u32,
    #[default = 24]
    #[schemars(description = "Padding", range(min = 0, max = 1000), extend("x-unit" = "px"))]
    pub padding: u32,
    #[default = "#ffffff"]
    #[schemars(
        description = "Background",
        extend("x-widget" = "color", "x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub background: String,
}

impl Params for ImageGrid {}

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
        .map(|c| {
            (0..n)
                .filter(|&i| place(i).0 == c)
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
        .map(|r| {
            (0..n)
                .filter(|&i| place(i).1 == r)
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
    let left = |c: usize| pad + column_widths[..c].iter().sum::<f64>() + gap * c as f64;
    let top = |r: usize| pad + row_heights[..r].iter().sum::<f64>() + gap * r as f64;
    let cells = (0..n)
        .map(|i| {
            let (c, r) = place(i);
            let (w, h) = scaled[i];
            Rect {
                x: (left(c) + (column_widths[c] - w) / 2.0).round(),
                y: (top(r) + (row_heights[r] - h) / 2.0).round(),
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

const VIEW_DISTANCE: f64 = 2.0;
const SHADOW_SPACE: f64 = 0.06;

pub fn tilt_corners(size: (f64, f64), p: &TiltImage) -> [(f64, f64); 4] {
    let (w, h) = size;
    let distance = VIEW_DISTANCE * w.max(h);
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
    let margin = if p.shadow {
        SHADOW_SPACE * w.max(h)
    } else {
        0.0
    };
    let left = projected.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let top = projected.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    projected.map(|(x, y)| (x - left + margin, y - top + margin))
}

pub fn tilted_size(size: (f64, f64), p: &TiltImage) -> (f64, f64) {
    let corners = tilt_corners(size, p);
    let margin = if p.shadow {
        SHADOW_SPACE * size.0.max(size.1)
    } else {
        0.0
    };
    let right = corners.iter().map(|c| c.0).fold(0.0, f64::max);
    let bottom = corners.iter().map(|c| c.1).fold(0.0, f64::max);
    (
        (right + margin).ceil().max(1.0),
        (bottom + margin).ceil().max(1.0),
    )
}

pub fn outline_padding(p: &OutlineImage) -> f64 {
    let shadow = if p.shadow { p.width * 2.0 } else { 0.0 };
    (p.width + p.softness + shadow).ceil()
}

fn even(value: f64) -> f64 {
    value.round().max(1.0)
}

pub fn resized_size(source: (f64, f64), p: &ResizeImage) -> (f64, f64) {
    let (sw, sh) = source;
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
            width: sw - (p.left + p.right) as f64,
            height: sh - (p.top + p.bottom) as f64,
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
    (
        (w * cos.abs() + h * sin.abs()).ceil(),
        (w * sin.abs() + h * cos.abs()).ceil(),
    )
}

fn pixels(width: f64, height: f64) -> String {
    format!("{} × {} px", width as i64, height as i64)
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
                    format!("Enlarges {:.1}×, so it may look soft", factor),
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
                    format!("Only {} px is left", r.width.min(r.height) as i64),
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
            row(Icon::RefreshCw, crate::card::sentence(&parts.join(", ")))
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
                (v * 100.0).round() as i64
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
            row(Icon::Sun, crate::card::sentence(&parts.join(", ")))
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
            row(Icon::Wind, crate::card::sentence(&parts.join(", ")))
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
                "{} % wide at {} %, {} %{}",
                p.size.round() as i64,
                (p.x * 100.0).round() as i64,
                (p.y * 100.0).round() as i64,
                if p.opacity < 1.0 {
                    format!(" · {} % opaque", (p.opacity * 100.0).round() as i64)
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
                    crate::text::plural(p.areas.len() as f64, "area"),
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
            "{} · darkens {} %",
            crate::text::plural(p.areas.len() as f64, "area"),
            (p.dim * 100.0).round() as i64
        );
        if p.blur > 0.0 {
            text.push_str(&format!(" · blurs {} px", p.blur));
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
            row(Icon::RefreshCw, crate::card::sentence(&parts.join(", ")))
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
            text.push_str(&format!(" · {} %", (p.strength * 100.0).round() as i64));
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
                        crate::text::plural(n as f64, "image")
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

pub fn nodes() -> Vec<NodeDef> {
    vec![
        resize().look(NodeLook::of("lucide:image-upscale")),
        crop().look(NodeLook::of("tabler:crop")),
        rotate().look(NodeLook::of("lucide:rotate-cw-square").moves(NodeMotion::Turn)),
        adjust().look(NodeLook::of("lucide:sliders-vertical")),
        blur().look(NodeLook::of("lucide:droplet")),
        place().look(NodeLook::of("lucide:stamp")),
        format().look(NodeLook::of("lucide:file-image")),
        redact().look(NodeLook::of("lucide:eye-off")),
        spotlight().look(NodeLook::of("lucide:flashlight")),
        trim().look(NodeLook::of("lucide:shrink")),
        clear_colour().look(NodeLook::of("lucide:eraser")),
        outline().look(NodeLook::of("lucide:square-dashed-bottom")),
        tilt().look(NodeLook::of("tabler:perspective")),
        levels().look(NodeLook::of("lucide:chart-column-increasing")),
        grid().look(NodeLook::of("lucide:grid-2x2")),
    ]
}
