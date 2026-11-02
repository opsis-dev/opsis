use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::numbers::exact_eq;
use opsis_types::values::StyleEntry;

use crate::asset_ref::base_name;
use crate::card_rows::{file_row, truncate_card};
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
                plural(numbers::float(names.len()), "change"),
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

#[choice(rename_all = "snake_case")]
pub enum ToneCurve {
    #[label("None")]
    None,
    #[label("Vintage")]
    Vintage,
    #[label("Cross-process")]
    CrossProcess,
    #[label("Lighter")]
    Lighter,
    #[label("Darker")]
    Darker,
    #[label("More contrast")]
    MediumContrast,
    #[label("Strong contrast")]
    StrongContrast,
    #[label("Negative")]
    Negative,
}

#[settings]
pub struct Colour {
    #[setting(
        "Brightness",
        range(min = -1, max = 1),
        step = 0.05,
        presets_label = "Look",
        presets = [
            { "label": "Natural", "values": { "brightness": 0, "contrast": 1, "saturation": 1, "warmth": 0 } },
            { "label": "Warm", "values": { "brightness": 0, "contrast": 1, "saturation": 1.1, "warmth": 0.35 } },
            { "label": "Cool", "values": { "brightness": 0, "contrast": 1.05, "saturation": 1, "warmth": -0.3 } },
            { "label": "Punchy", "values": { "brightness": 0, "contrast": 1.2, "saturation": 1.25, "warmth": 0 } },
            { "label": "Soft", "values": { "brightness": 0.1, "contrast": 0.88, "saturation": 0.9, "warmth": 0 } },
            { "label": "Black and white", "values": { "brightness": 0, "contrast": 1.15, "saturation": 0, "warmth": 0 } },
            { "label": "Old film", "values": { "brightness": 0, "contrast": 1, "saturation": 0.75, "warmth": 0.25 } }
        ]
    )]
    pub brightness: f64,
    #[setting(
        "Contrast",
        default = 1.0,
        range(min = 0.5, max = 1.5),
        step = 0.05,
        unit = "×"
    )]
    pub contrast: f64,
    #[setting(
        "Colour",
        default = 1.0,
        range(min = 0, max = 2),
        step = 0.05,
        unit = "×"
    )]
    pub saturation: f64,
    #[setting(
        "Gamma",
        default = 1.0,
        range(min = 0.5, max = 2),
        step = 0.05,
        advanced
    )]
    pub gamma: f64,
    #[setting("Exposure", range(min = -2, max = 2), step = 0.05, unit = "stops")]
    pub exposure: f64,
    #[setting("Warmth", range(min = -1, max = 1), step = 0.05)]
    pub warmth: f64,
    #[setting("Vibrance", range(min = -1, max = 1), step = 0.05)]
    pub vibrance: f64,
    #[setting("Hue shift", range(min = -180, max = 180), step = 1, unit = "°", advanced)]
    pub hue: f64,
    #[setting(
        "Colour for shadows",
        default = "#3b6ea5",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty,
        advanced
    )]
    pub shadows: String,
    #[setting("Shadows amount", range(min = 0, max = 1), step = 0.05, advanced)]
    pub shadows_amount: f64,
    #[setting(
        "Colour for midtones",
        default = "#c9a27a",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty,
        advanced
    )]
    pub midtones: String,
    #[setting("Midtones amount", range(min = 0, max = 1), step = 0.05, advanced)]
    pub midtones_amount: f64,
    #[setting(
        "Colour for highlights",
        default = "#ffd9a0",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty,
        advanced
    )]
    pub highlights: String,
    #[setting("Highlights amount", range(min = 0, max = 1), step = 0.05, advanced)]
    pub highlights_amount: f64,
    #[setting(
        "Tone curve",
        default = ToneCurve::None
    )]
    pub curve: ToneCurve,
}

impl Colour {
    fn changes(&self) -> [Change; 12] {
        [
            ("Brightness", self.brightness != 0.0),
            ("Contrast", !exact_eq(self.contrast, 1.0)),
            ("Colour", !exact_eq(self.saturation, 1.0)),
            ("Gamma", !exact_eq(self.gamma, 1.0)),
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

#[settings]
pub struct LookFile {
    #[setting(
        "Look file",
        widget = "asset",
        accept = ".cube,.3dl,.png",
        help = "A colour look saved from another video editor, or downloaded. Files ending in .cube or .3dl, or a look image."
    )]
    pub file: String,
    #[setting("Strength", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub strength: f64,
}

#[choice]
pub enum MonoKind {
    #[label("Black and white")]
    Bw,
    #[label("Sepia")]
    Sepia,
    #[label("Tint")]
    Tint,
}

pub const DEFAULT_TINT: &str = "#d8a15b";

#[settings]
pub struct Mono {
    #[setting(
        "Kind",
        default = MonoKind::Bw
    )]
    pub kind: MonoKind,
    #[setting(
        "Tint colour",
        default = "#d8a15b",
        widget = "color",
        zero_label = "Default",
        show_if = { "kind": ["tint"] },
        check = colour_or_empty
    )]
    pub tint: String,
    #[setting("Strength", default = 1.0, range(min = 0, max = 1), step = 0.05)]
    pub strength: f64,
    #[setting(
        "Colour filter",
        help = "Like a coloured filter on a black-and-white camera: things of that colour come out lighter.",
        show_if = { "kind": ["bw"] },
        advanced
    )]
    pub filter: MonoFilter,
}

#[choice]
pub enum MonoFilter {
    #[default]
    #[label("None")]
    None,
    #[label("Red")]
    Red,
    #[label("Orange")]
    Orange,
    #[label("Yellow")]
    Yellow,
    #[label("Green")]
    Green,
    #[label("Blue")]
    Blue,
}

impl MonoFilter {
    pub const fn chroma(self) -> (f64, f64) {
        match self {
            Self::None => (0.0, 0.0),
            Self::Red => (-0.2, 0.6),
            Self::Orange => (-0.45, 0.45),
            Self::Yellow => (-0.6, 0.15),
            Self::Green => (-0.35, -0.5),
            Self::Blue => (0.6, -0.15),
        }
    }
}

#[settings]
pub struct Texture {
    #[setting("Sharpen", range(min = 0, max = 1), step = 0.05)]
    pub sharpen: f64,
    #[setting("Soften", range(min = 0, max = 1), step = 0.05)]
    pub soften: f64,
    #[setting("Dark edges", range(min = 0, max = 1), step = 0.05)]
    pub vignette: f64,
    #[setting(
        "Dark edges centre",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        help = "Click the frame to choose the middle of the light.",
        advanced
    )]
    pub vignette_x: f64,
    #[setting(
        "Dark edges centre vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub vignette_y: f64,
    #[setting("Film grain", range(min = 0, max = 1), step = 0.05)]
    pub grain: f64,
    #[setting("Dark edges shape", advanced)]
    pub vignette_shape: VignetteShape,
    #[setting(
        "Colour grain",
        default = true,
        help = "Off, the grain is only in the light, not the colour.",
        advanced
    )]
    pub colour_grain: bool,
}

#[choice]
pub enum VignetteShape {
    #[default]
    #[label("Round")]
    Round,
    #[label("Follows the frame")]
    Frame,
}

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

#[settings]
pub struct Lens {
    #[setting("Colour fringing", range(min = 0, max = 1), step = 0.05)]
    pub fringe: f64,
    #[setting("Lens bulge", range(min = -1, max = 1), step = 0.05)]
    pub bulge: f64,
    #[setting("Motion trails", range(min = 0, max = 1), step = 0.05)]
    pub trails: f64,
    #[setting("Fringe angle", range(min = -180, max = 180), step = 5, unit = "°", advanced)]
    pub fringe_angle: f64,
    #[setting(
        "Bulge centre",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        help = "Click the frame to choose the middle of the bulge.",
        advanced
    )]
    pub bulge_x: f64,
    #[setting(
        "Bulge centre vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub bulge_y: f64,
    #[setting(
        "Corner colour",
        default = "#000000",
        widget = "color",
        check = colour,
        help = "Fills corners a bulge pulls in.",
        advanced
    )]
    pub corner_color: String,
}

impl Lens {
    fn changes(&self) -> [Change; 3] {
        [
            ("Colour fringing", self.fringe > 0.0),
            ("Lens bulge", self.bulge != 0.0),
            ("Motion trails", self.trails > 0.0),
        ]
    }
}

#[settings]
pub struct AutoFix {
    #[setting("White balance")]
    pub white_balance: bool,
    #[setting("Stretch the levels")]
    pub levels: bool,
    #[setting(
        "Levels strength",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "levels": ["true"] }
    )]
    pub levels_strength: f64,
    #[setting(
        "Levels smoothing",
        default = 24,
        range(min = 0, max = 300),
        unit = "frames",
        advanced,
        show_if = { "levels": ["true"] }
    )]
    pub levels_smoothing: u32,
    #[setting("Remove banding")]
    pub deband: bool,
    #[setting(
        "Each colour apart",
        help = "Stretches red, green and blue each on its own, which also corrects a colour cast.",
        show_if = { "levels": ["true"] },
        advanced
    )]
    pub levels_each_colour: bool,
    #[setting(
        "Banding strength",
        default = 0.02,
        range(min = 0.003, max = 0.5),
        step = 0.005,
        show_if = { "deband": ["true"] },
        advanced
    )]
    pub deband_strength: f64,
}

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
        .neutral(|p: &Colour| active(&p.changes()).is_empty()).look(NodeLook::of("tabler:exposure")),
        step::<LookFile>(LOOK_TYPE, "Look file", "Applies a colour look saved from another video editor. Used by Apply grade.")
            .card(|ctx: &CardCtx<'_, LookFile>| {
                vec![Some(look_file_row(&ctx.params.file, &ctx.inputs.missing_files)), strength_row(ctx.params.strength)]
            })
            .neutral(|p: &LookFile| p.strength == 0.0).look(NodeLook::of("lucide:file-sliders")),
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
        .neutral(|p: &Mono| p.strength == 0.0).look(NodeLook::of("iconoir:droplet-half")),
        step::<Texture>(
            TEXTURE_TYPE,
            "Texture",
            "Sharpens or softens the picture and adds dark edges and film grain. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Texture>| vec![Some(changes_row(Icon::Sparkles, &ctx.params.changes()))])
        .neutral(|p: &Texture| active(&p.changes()).is_empty()).look(NodeLook::of("tabler:texture")),
        step::<Lens>(
            LENS_TYPE,
            "Lens",
            "Adds colour fringing, bends the picture like a lens and leaves trails behind motion. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, Lens>| vec![Some(changes_row(Icon::Scan, &ctx.params.changes()))])
        .neutral(|p: &Lens| active(&p.changes()).is_empty()).look(NodeLook::of("tabler:prism-light")),
        step::<AutoFix>(
            AUTO_FIX_TYPE,
            "Auto fix",
            "Fixes a colour cast, stretches dull levels and smooths banding in skies. Used by Apply grade.",
        )
        .card(|ctx: &CardCtx<'_, AutoFix>| vec![Some(changes_row(Icon::Check, &ctx.params.changes()))])
        .neutral(|p: &AutoFix| active(&p.changes()).is_empty()).look(NodeLook::of("lucide:wand")),
    ]
}

pub fn grade_step_title(kind: &str) -> String {
    node_def(kind)
        .filter(|d| d.category == Category::Grade)
        .map_or_else(|| kind.to_owned(), |d| d.title.clone())
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
