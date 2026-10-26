use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::values::StyleEntry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::asset_ref::base_name;
use crate::card::{file_row, truncate_card};
use crate::color::{color_or, rgb_of};
use crate::node::{CardCtx, Category, NodeDef, RowExt, missing, row};
use crate::params::Params;
use crate::registry::node_def;
use crate::text::{percent_text, plural, round_to};

type Change = (&'static str, bool);

fn active(changes: &[Change]) -> Vec<&'static str> {
    changes
        .iter()
        .filter(|(_, on)| *on)
        .map(|(label, _)| *label)
        .collect()
}

fn changes_row(icon: Icon, changes: &[Change]) -> CardRow {
    let names = active(changes);
    if names.is_empty() {
        row(icon, "No change").tone(Tone::Muted)
    } else {
        row(
            icon,
            truncate_card(&format!(
                "{} · {}",
                plural(names.len() as f64, "change"),
                names.join(", ")
            )),
        )
    }
}

fn strength_row(strength: f64) -> Option<CardRow> {
    (strength < 1.0).then(|| {
        row(
            Icon::Sparkles,
            format!("{} strength", percent_text(strength, f64::INFINITY)),
        )
    })
}

pub fn tone_offsets(color: &str, amount: f64) -> Option<[f64; 3]> {
    let rgb = rgb_of(color).filter(|_| amount > 0.0)?;
    let mean = (rgb.r + rgb.g + rgb.b) / 3.0;
    let offset = |channel: f64| round_to(((channel - mean) * amount).clamp(-1.0, 1.0), 3);
    let offsets = [offset(rgb.r), offset(rgb.g), offset(rgb.b)];
    offsets.iter().any(|o| *o != 0.0).then_some(offsets)
}

fn tone_amount(color: &str, amount: f64) -> bool {
    tone_offsets(color, amount).is_some()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToneCurve {
    None,
    Vintage,
    CrossProcess,
    Lighter,
    Darker,
    MediumContrast,
    StrongContrast,
    Negative,
}

impl ToneCurve {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Vintage => "Vintage",
            Self::CrossProcess => "Cross-process",
            Self::Lighter => "Lighter",
            Self::Darker => "Darker",
            Self::MediumContrast => "More contrast",
            Self::StrongContrast => "Strong contrast",
            Self::Negative => "Negative",
        }
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Colour {
    #[schemars(
        description = "Brightness",
        range(min = -1, max = 1),
        extend(
            "x-step" = 0.05,
            "x-presets-label" = "Look",
            "x-presets" = [
                { "label": "Natural", "values": { "brightness": 0, "contrast": 1, "saturation": 1, "warmth": 0 } },
                { "label": "Warm", "values": { "brightness": 0, "contrast": 1, "saturation": 1.1, "warmth": 0.35 } },
                { "label": "Cool", "values": { "brightness": 0, "contrast": 1.05, "saturation": 1, "warmth": -0.3 } },
                { "label": "Punchy", "values": { "brightness": 0, "contrast": 1.2, "saturation": 1.25, "warmth": 0 } },
                { "label": "Soft", "values": { "brightness": 0.1, "contrast": 0.88, "saturation": 0.9, "warmth": 0 } },
                { "label": "Black and white", "values": { "brightness": 0, "contrast": 1.15, "saturation": 0, "warmth": 0 } },
                { "label": "Old film", "values": { "brightness": 0, "contrast": 1, "saturation": 0.75, "warmth": 0.25 } }
            ]
        )
    )]
    pub brightness: f64,
    #[default = 1.0]
    #[schemars(description = "Contrast", range(min = 0.5, max = 1.5), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub contrast: f64,
    #[default = 1.0]
    #[schemars(description = "Colour", range(min = 0, max = 2), extend("x-step" = 0.05, "x-unit" = "×"))]
    pub saturation: f64,
    #[default = 1.0]
    #[schemars(description = "Gamma", range(min = 0.5, max = 2), extend("x-step" = 0.05))]
    pub gamma: f64,
    #[schemars(description = "Exposure", range(min = -2, max = 2), extend("x-step" = 0.05, "x-unit" = "stops"))]
    pub exposure: f64,
    #[schemars(description = "Warmth", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub warmth: f64,
    #[schemars(description = "Vibrance", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub vibrance: f64,
    #[schemars(description = "Hue shift", range(min = -180, max = 180), extend("x-step" = 1, "x-unit" = "°"))]
    pub hue: f64,
    #[default = "#3b6ea5"]
    #[schemars(
        description = "Colour for shadows",
        extend("x-widget" = "color", "x-zero-label" = "Default", "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" })
    )]
    pub shadows: String,
    #[schemars(description = "Shadows amount", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub shadows_amount: f64,
    #[default = "#c9a27a"]
    #[schemars(
        description = "Colour for midtones",
        extend("x-widget" = "color", "x-zero-label" = "Default", "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" })
    )]
    pub midtones: String,
    #[schemars(description = "Midtones amount", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub midtones_amount: f64,
    #[default = "#ffd9a0"]
    #[schemars(
        description = "Colour for highlights",
        extend("x-widget" = "color", "x-zero-label" = "Default", "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" })
    )]
    pub highlights: String,
    #[schemars(description = "Highlights amount", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub highlights_amount: f64,
    #[default(ToneCurve::None)]
    #[schemars(
        description = "Tone curve",
        extend("x-labels" = {
            "none": "None",
            "vintage": "Vintage",
            "cross_process": "Cross-process",
            "lighter": "Lighter",
            "darker": "Darker",
            "medium_contrast": "More contrast",
            "strong_contrast": "Strong contrast",
            "negative": "Negative"
        })
    )]
    pub curve: ToneCurve,
}

impl Params for Colour {}

impl Colour {
    fn changes(&self) -> [Change; 12] {
        [
            ("Brightness", self.brightness != 0.0),
            ("Contrast", self.contrast != 1.0),
            ("Colour", self.saturation != 1.0),
            ("Gamma", self.gamma != 1.0),
            ("Exposure", self.exposure != 0.0),
            ("Warmth", self.warmth != 0.0),
            ("Vibrance", self.vibrance != 0.0),
            ("Hue", self.hue != 0.0),
            ("Shadows", tone_amount(&self.shadows, self.shadows_amount)),
            (
                "Midtones",
                tone_amount(&self.midtones, self.midtones_amount),
            ),
            (
                "Highlights",
                tone_amount(&self.highlights, self.highlights_amount),
            ),
            ("Tone curve", self.curve != ToneCurve::None),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LookFile {
    #[schemars(
        description = "Look file",
        extend(
            "x-widget" = "asset",
            "x-accept" = ".cube,.3dl,.png",
            "x-help" = "A colour look saved from another video editor, or downloaded. Files ending in .cube or .3dl, or a look image."
        )
    )]
    pub file: String,
    #[default = 1.0]
    #[schemars(description = "Strength", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub strength: f64,
}

impl Params for LookFile {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MonoKind {
    Bw,
    Sepia,
    Tint,
}

impl MonoKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bw => "Black and white",
            Self::Sepia => "Sepia",
            Self::Tint => "Tint",
        }
    }
}

pub const DEFAULT_TINT: &str = "#d8a15b";

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Mono {
    #[default(MonoKind::Bw)]
    #[schemars(description = "Kind", extend("x-labels" = { "bw": "Black and white", "sepia": "Sepia", "tint": "Tint" }))]
    pub kind: MonoKind,
    #[default = "#d8a15b"]
    #[schemars(
        description = "Tint colour",
        extend(
            "x-widget" = "color",
            "x-zero-label" = "Default",
            "x-show-if" = { "kind": ["tint"] },
            "x-check" = { "rule": "colour-or-empty", "message": "Pick a colour" }
        )
    )]
    pub tint: String,
    #[default = 1.0]
    #[schemars(description = "Strength", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub strength: f64,
}

impl Params for Mono {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Texture {
    #[schemars(description = "Sharpen", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub sharpen: f64,
    #[schemars(description = "Soften", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub soften: f64,
    #[schemars(description = "Dark edges", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub vignette: f64,
    #[default = 0.5]
    #[schemars(
        description = "Dark edges centre",
        range(min = 0, max = 1),
        extend("x-step" = 0.01, "x-widget" = "spot", "x-help" = "Click the frame to choose the middle of the light.")
    )]
    pub vignette_x: f64,
    #[default = 0.5]
    #[schemars(description = "Dark edges centre vertical", range(min = 0, max = 1), extend("x-step" = 0.01, "x-hidden" = true))]
    pub vignette_y: f64,
    #[schemars(description = "Film grain", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub grain: f64,
}

impl Params for Texture {}

impl Texture {
    fn changes(&self) -> [Change; 4] {
        [
            ("Sharpen", self.sharpen > 0.0),
            ("Soften", self.soften > 0.0),
            ("Dark edges", self.vignette > 0.0),
            ("Film grain", self.grain > 0.0),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Lens {
    #[schemars(description = "Colour fringing", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub fringe: f64,
    #[schemars(description = "Lens bulge", range(min = -1, max = 1), extend("x-step" = 0.05))]
    pub bulge: f64,
    #[schemars(description = "Motion trails", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub trails: f64,
}

impl Params for Lens {}

impl Lens {
    fn changes(&self) -> [Change; 3] {
        [
            ("Colour fringing", self.fringe > 0.0),
            ("Lens bulge", self.bulge != 0.0),
            ("Motion trails", self.trails > 0.0),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFix {
    #[schemars(description = "White balance")]
    pub white_balance: bool,
    #[schemars(description = "Stretch the levels")]
    pub levels: bool,
    #[default = 0.5]
    #[schemars(
        description = "Levels strength",
        range(min = 0, max = 1),
        extend("x-step" = 0.05, "x-show-if" = { "levels": ["true"] })
    )]
    pub levels_strength: f64,
    #[default = 24]
    #[schemars(
        description = "Levels smoothing",
        range(min = 0, max = 300),
        extend("x-unit" = "frames", "x-advanced" = true, "x-show-if" = { "levels": ["true"] })
    )]
    pub levels_smoothing: u32,
    #[schemars(description = "Remove banding")]
    pub deband: bool,
}

impl Params for AutoFix {}

impl AutoFix {
    fn changes(&self) -> [Change; 3] {
        [
            ("White balance", self.white_balance),
            (
                "Stretch the levels",
                self.levels && self.levels_strength > 0.0,
            ),
            ("Remove banding", self.deband),
        ]
    }
}

pub const COLOUR_TYPE: &str = "grade.colour";
pub const LOOK_TYPE: &str = "grade.lookFile";
pub const MONO_TYPE: &str = "grade.mono";
pub const TEXTURE_TYPE: &str = "grade.texture";
pub const LENS_TYPE: &str = "grade.lens";
pub const AUTO_FIX_TYPE: &str = "grade.autoFix";

fn look_file_row(file: &str, missing_files: &[String]) -> CardRow {
    if file.is_empty() {
        return missing(Icon::File, "Choose a look file");
    }
    if missing_files.iter().any(|m| m == file) {
        return missing(
            Icon::File,
            format!("{} isn't in the project", base_name(file)),
        );
    }
    file_row(Icon::File, file)
}

fn step<P: Params>(kind: &'static str, title: &str, description: &str) -> NodeDef {
    NodeDef::grade_modifier::<P>(kind, title, Category::Grade).describe(description)
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        step::<Colour>(
            COLOUR_TYPE,
            "Colour and light",
            "Changes brightness, contrast, colour, warmth and the colour of shadows and highlights. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Colour>| {
            let p = &ctx.params;
            vec![Some(changes_row(Icon::Sun, &p.changes())), (p.curve != ToneCurve::None).then(|| row(Icon::Spline, p.curve.label()))]
        })
        .neutral(|p: &Colour| active(&p.changes()).is_empty()),
        step::<LookFile>(LOOK_TYPE, "Look file", "Applies a colour look saved from another video editor. Used by Apply grade.")
            .card(|ctx: &CardCtx<'_, LookFile>| {
                vec![Some(look_file_row(&ctx.params.file, &ctx.inputs.missing_files)), strength_row(ctx.params.strength)]
            })
            .neutral(|p: &LookFile| p.strength == 0.0),
        step::<Mono>(
            MONO_TYPE,
            "Black and white and tint",
            "Turns the picture black and white, sepia or one tint of colour. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Mono>| {
            let p = &ctx.params;
            let kind = row(Icon::Moon, p.kind.label());
            let kind = if p.kind == MonoKind::Tint { kind.swatches(vec![color_or(&p.tint, DEFAULT_TINT)]) } else { kind };
            vec![Some(kind), strength_row(p.strength)]
        })
        .neutral(|p: &Mono| p.strength == 0.0),
        step::<Texture>(
            TEXTURE_TYPE,
            "Texture",
            "Sharpens or softens the picture and adds dark edges and film grain. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Texture>| vec![Some(changes_row(Icon::Sparkles, &ctx.params.changes()))])
        .neutral(|p: &Texture| active(&p.changes()).is_empty()),
        step::<Lens>(
            LENS_TYPE,
            "Lens",
            "Adds colour fringing, bends the picture like a lens and leaves trails behind motion. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Lens>| vec![Some(changes_row(Icon::Scan, &ctx.params.changes()))])
        .neutral(|p: &Lens| active(&p.changes()).is_empty()),
        step::<AutoFix>(
            AUTO_FIX_TYPE,
            "Auto fix",
            "Fixes a colour cast, stretches dull levels and smooths banding in skies. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, AutoFix>| vec![Some(changes_row(Icon::Check, &ctx.params.changes()))])
        .neutral(|p: &AutoFix| active(&p.changes()).is_empty()),
    ]
}

pub fn grade_step_title(kind: &str) -> String {
    node_def(kind)
        .filter(|d| d.category == Category::Grade)
        .map(|d| d.title.clone())
        .unwrap_or_else(|| kind.to_owned())
}

pub fn is_neutral_entry(entry: &StyleEntry) -> bool {
    node_def(&entry.kind)
        .and_then(|d| d.is_neutral(&entry.params))
        .unwrap_or(true)
}

fn needs_motion(entry: &StyleEntry) -> bool {
    let Some(def) = node_def(&entry.kind) else {
        return false;
    };
    match entry.kind.as_str() {
        LENS_TYPE => def
            .typed_params::<Lens>(&entry.params)
            .is_ok_and(|p| p.trails > 0.0),
        TEXTURE_TYPE => def
            .typed_params::<Texture>(&entry.params)
            .is_ok_and(|p| p.grain > 0.0),
        _ => false,
    }
}

pub fn grade_needs_video(entries: &[StyleEntry]) -> bool {
    entries.iter().any(needs_motion)
}

pub fn grade_changes_nothing(entries: &[StyleEntry]) -> bool {
    entries.iter().all(is_neutral_entry)
}
