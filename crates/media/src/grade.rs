use std::f64::consts::PI;
use std::path::Path;

use opsis_compile::color::{color_or, hue_and_saturation};
use opsis_compile::errors::SettingsError;
use opsis_compile::nodes::grade::{
    AUTO_FIX_TYPE, AutoFix, COLOUR_TYPE, Colour, DEFAULT_TINT, LENS_TYPE, LOOK_TYPE, Lens,
    LookFile, MONO_TYPE, Mono, MonoKind, TEXTURE_TYPE, Texture, ToneCurve, VignetteShape,
    tone_offsets,
};
use opsis_compile::registry::node_def;
use opsis_compile::text::round_to;
use opsis_types::numbers::exact_eq;
use opsis_types::values::StyleEntry;
use serde::de::DeserializeOwned;

use crate::codecs::serde_name;
use crate::filters::ffmpeg_color;

const DESIGN_WIDTH: f64 = 1920.0;
pub const COLOUR_STEPS: [&str; 3] = [COLOUR_TYPE, LOOK_TYPE, MONO_TYPE];
pub const PAGE_STEPS: [&str; 2] = [TEXTURE_TYPE, LENS_TYPE];

pub fn colour_steps(entries: &[StyleEntry]) -> Vec<StyleEntry> {
    entries
        .iter()
        .filter(|e| COLOUR_STEPS.contains(&e.kind.as_str()))
        .cloned()
        .collect()
}
const NEUTRAL_KELVIN: f64 = 6500.0;
const KELVIN_SWING: f64 = 3500.0;
const TINT_LIGHTNESS: &str = "0.5";
const EVEN_SIZE: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2";
const SEPIA: &str = "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub struct GradeSize {
    pub width: u32,
    pub height: u32,
    pub still: bool,
}

pub type Resolve<'a, E> = &'a dyn Fn(&str) -> Result<String, E>;

#[derive(Debug, thiserror::Error)]
pub enum GradeError {
    #[error("A look file ends in .cube, .3dl or .png.")]
    LookKind,
    #[error("\"{0}\" isn't a grade step.")]
    NotAStep(String),
    #[error(transparent)]
    Settings(#[from] SettingsError),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeChain {
    pub chain: String,
    pub extra_inputs: Vec<String>,
}

#[derive(Default)]
struct Piece {
    filters: Vec<String>,
    strength: Option<f64>,
    look_image: Option<String>,
}

fn num(value: f64) -> String {
    (round_to(value, 3)).to_string()
}

fn option_value(text: &str) -> String {
    escaped(text, &['\\', '\'', ':'])
}

fn graph_text(text: &str) -> String {
    escaped(text, &['\\', '\'', ',', ';', '[', ']'])
}

fn escaped(text: &str, special: &[char]) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if special.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

pub fn filter_path(file: &str) -> String {
    graph_text(&option_value(file))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LookKind {
    Table,
    Image,
}

pub fn look_kind_of(file: &str) -> Option<LookKind> {
    let ext = Path::new(file)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    match ext.as_deref() {
        Some("cube" | "3dl") => Some(LookKind::Table),
        Some("png") => Some(LookKind::Image),
        _ => None,
    }
}

fn fade_of(strength: f64) -> Option<f64> {
    (strength < 1.0).then_some(strength)
}

enum Opt {
    Off,
    On(String),
}

fn on(value: impl Into<String>) -> Opt {
    Opt::On(value.into())
}

fn when(test: bool, value: impl Into<String>) -> Opt {
    if test {
        Opt::On(value.into())
    } else {
        Opt::Off
    }
}

fn options(name: &str, values: Vec<(&str, Opt)>) -> Vec<String> {
    let used: Vec<String> = values
        .into_iter()
        .filter_map(|(key, value)| match value {
            Opt::On(v) => Some(format!("{key}={v}")),
            Opt::Off => None,
        })
        .collect();
    if used.is_empty() {
        Vec::new()
    } else {
        vec![format!("{name}={}", used.join(":"))]
    }
}

fn balance(tones: Option<[f64; 3]>, keys: [&'static str; 3]) -> Vec<(&'static str, Opt)> {
    keys.into_iter()
        .enumerate()
        .map(|(i, key)| (key, tones.map_or(Opt::Off, |t| on((t[i]).to_string()))))
        .collect()
}

fn colour_filters(p: &Colour) -> Vec<String> {
    let shadows = tone_offsets(&p.shadows, p.shadows_amount);
    let midtones = tone_offsets(&p.midtones, p.midtones_amount);
    let highlights = tone_offsets(&p.highlights, p.highlights_amount);
    let mut tones = balance(shadows, ["rs", "gs", "bs"]);
    tones.extend(balance(midtones, ["rm", "gm", "bm"]));
    tones.extend(balance(highlights, ["rh", "gh", "bh"]));
    [
        options(
            "exposure",
            vec![("exposure", when(p.exposure != 0.0, num(p.exposure)))],
        ),
        options(
            "eq",
            vec![
                ("brightness", when(p.brightness != 0.0, num(p.brightness))),
                (
                    "contrast",
                    when(!exact_eq(p.contrast, 1.0), num(p.contrast)),
                ),
                (
                    "saturation",
                    when(!exact_eq(p.saturation, 1.0), num(p.saturation)),
                ),
                ("gamma", when(!exact_eq(p.gamma, 1.0), num(p.gamma))),
            ],
        ),
        options(
            "colortemperature",
            vec![
                (
                    "temperature",
                    when(
                        p.warmth != 0.0,
                        (p.warmth.mul_add(-KELVIN_SWING, NEUTRAL_KELVIN).round()).to_string(),
                    ),
                ),
                ("mix", when(p.warmth != 0.0, "1")),
            ],
        ),
        options(
            "vibrance",
            vec![("intensity", when(p.vibrance != 0.0, num(p.vibrance)))],
        ),
        options("hue", vec![("h", when(p.hue != 0.0, num(p.hue)))]),
        options("colorbalance", tones),
        options(
            "curves",
            vec![(
                "preset",
                when(p.curve != ToneCurve::None, serde_name(&p.curve)),
            )],
        ),
    ]
    .concat()
}

fn mono_piece(p: &Mono) -> Option<Piece> {
    if p.strength == 0.0 {
        return None;
    }
    let filters = match p.kind {
        MonoKind::Tint => {
            let (hue, saturation) =
                hue_and_saturation(&color_or(&p.tint, DEFAULT_TINT)).unwrap_or((0.0, 0.5));
            options(
                "colorize",
                vec![
                    ("hue", on(num(hue))),
                    ("saturation", on(num(saturation))),
                    ("lightness", on(TINT_LIGHTNESS)),
                    ("mix", on("1")),
                ],
            )
        }
        MonoKind::Sepia => vec![SEPIA.to_owned()],
        MonoKind::Bw => {
            let (cb, cr) = p.filter.chroma();
            vec![format!("monochrome=cb={}:cr={}:size=1", num(cb), num(cr))]
        }
    };
    Some(Piece {
        filters,
        strength: fade_of(p.strength),
        look_image: None,
    })
}

fn texture_filters(p: &Texture, size: GradeSize) -> Vec<String> {
    let mut filters = [
        options(
            "cas",
            vec![("strength", when(p.sharpen > 0.0, num(p.sharpen)))],
        ),
        options(
            "gblur",
            vec![(
                "sigma",
                when(
                    p.soften > 0.0,
                    num(f64::max(
                        0.01,
                        p.soften * 8.0 * (f64::from(size.width) / DESIGN_WIDTH),
                    )),
                ),
            )],
        ),
        options(
            "vignette",
            vec![
                ("angle", when(p.vignette > 0.0, num(p.vignette * PI / 2.0))),
                (
                    "x0",
                    when(p.vignette > 0.0, format!("w*{}", num(p.vignette_x))),
                ),
                (
                    "y0",
                    when(p.vignette > 0.0, format!("h*{}", num(p.vignette_y))),
                ),
                (
                    "aspect",
                    when(
                        p.vignette > 0.0 && p.vignette_shape == VignetteShape::Frame,
                        format!("{}/{}", size.width, size.height),
                    ),
                ),
            ],
        ),
    ]
    .concat();
    if !size.still {
        let (amount, motion) = if p.colour_grain {
            ("alls", "allf")
        } else {
            ("c0s", "c0f")
        };
        filters.extend(options(
            "noise",
            vec![
                (
                    amount,
                    when(
                        p.grain > 0.0,
                        (f64::max(1.0, (p.grain * 40.0).round())).to_string(),
                    ),
                ),
                (motion, when(p.grain > 0.0, "t+u")),
            ],
        ));
    }
    filters
}

fn lens_filters(p: &Lens, size: GradeSize) -> Vec<String> {
    let shift = f64::max(
        1.0,
        (p.fringe * 8.0 * (f64::from(size.width) / DESIGN_WIDTH)).round(),
    );
    let (sin, cos) = p.fringe_angle.to_radians().sin_cos();
    let along = |part: f64| (shift * part).round();
    let (across, down) = (along(cos), along(sin));
    let centred = exact_eq(p.bulge_x, 0.5) && exact_eq(p.bulge_y, 0.5);
    let mut filters = [
        options(
            "rgbashift",
            vec![
                (
                    "rh",
                    when(p.fringe > 0.0 && across != 0.0, across.to_string()),
                ),
                ("rv", when(p.fringe > 0.0 && down != 0.0, down.to_string())),
                (
                    "bh",
                    when(p.fringe > 0.0 && across != 0.0, (-across).to_string()),
                ),
                (
                    "bv",
                    when(p.fringe > 0.0 && down != 0.0, (-down).to_string()),
                ),
            ],
        ),
        options(
            "lenscorrection",
            vec![
                ("cx", when(p.bulge != 0.0 && !centred, num(p.bulge_x))),
                ("cy", when(p.bulge != 0.0 && !centred, num(p.bulge_y))),
                ("k1", when(p.bulge != 0.0, num(p.bulge * 0.5))),
                ("k2", when(p.bulge != 0.0, "0")),
                ("i", when(p.bulge != 0.0, "bilinear")),
                (
                    "fc",
                    when(
                        p.bulge != 0.0 && ffmpeg_color(&p.corner_color) != ffmpeg_color("#000000"),
                        ffmpeg_color(&p.corner_color),
                    ),
                ),
            ],
        ),
    ]
    .concat();
    if !size.still {
        filters.extend(options(
            "lagfun",
            vec![(
                "decay",
                when(p.trails > 0.0, num(p.trails.mul_add(0.48, 0.5))),
            )],
        ));
    }
    filters
}

fn auto_fix_filters(p: &AutoFix, size: GradeSize) -> Vec<String> {
    let mut filters = Vec::new();
    if p.white_balance {
        let back = if size.still { "rgb24" } else { "yuv420p" };
        filters.extend([
            "format=gbrpf32le".to_owned(),
            "grayworld".to_owned(),
            format!("format={back}"),
        ]);
    }
    if p.levels && p.levels_strength > 0.0 {
        filters.extend(options(
            "normalize",
            vec![
                ("smoothing", on(p.levels_smoothing.to_string())),
                (
                    "independence",
                    on(if p.levels_each_colour { "1" } else { "0" }),
                ),
                ("strength", on(num(p.levels_strength))),
            ],
        ));
    }
    if p.deband {
        let default = AutoFix::default().deband_strength;
        let threshold = num(p.deband_strength);
        let tuned = options(
            "deband",
            ["1thr", "2thr", "3thr", "4thr"]
                .into_iter()
                .map(|key| {
                    (
                        key,
                        when(!exact_eq(p.deband_strength, default), threshold.clone()),
                    )
                })
                .collect(),
        );
        if tuned.is_empty() {
            filters.push("deband".to_owned());
        } else {
            filters.extend(tuned);
        }
    }
    filters
}

fn look_piece<E: From<GradeError>>(
    p: &LookFile,
    resolve: Resolve<'_, E>,
) -> Result<Option<Piece>, E> {
    if p.file.is_empty() || p.strength == 0.0 {
        return Ok(None);
    }
    let strength = fade_of(p.strength);
    Ok(Some(
        match look_kind_of(&p.file).ok_or(GradeError::LookKind)? {
            LookKind::Image => Piece {
                filters: Vec::new(),
                look_image: Some(resolve(&p.file)?),
                strength,
            },
            LookKind::Table => Piece {
                filters: options(
                    "lut3d",
                    vec![
                        ("file", on(filter_path(&resolve(&p.file)?))),
                        ("interp", on("tetrahedral")),
                    ],
                ),
                strength,
                look_image: None,
            },
        },
    ))
}

fn params<P: DeserializeOwned>(entry: &StyleEntry) -> Result<P, GradeError> {
    let def = node_def(&entry.kind).ok_or_else(|| GradeError::NotAStep(entry.kind.clone()))?;
    Ok(def.typed_params(&entry.params)?)
}

fn filters_only(filters: Vec<String>) -> Piece {
    Piece {
        filters,
        ..Piece::default()
    }
}

fn piece_of<E: From<GradeError>>(
    entry: &StyleEntry,
    size: GradeSize,
    resolve: Resolve<'_, E>,
) -> Result<Option<Piece>, E> {
    Ok(match entry.kind.as_str() {
        COLOUR_TYPE => Some(filters_only(colour_filters(&params(entry)?))),
        LOOK_TYPE => look_piece(&params(entry)?, resolve)?,
        MONO_TYPE => mono_piece(&params(entry)?),
        TEXTURE_TYPE => Some(filters_only(texture_filters(&params(entry)?, size))),
        LENS_TYPE => Some(filters_only(lens_filters(&params(entry)?, size))),
        AUTO_FIX_TYPE => Some(filters_only(auto_fix_filters(&params(entry)?, size))),
        other => return Err(GradeError::NotAStep(other.to_owned()).into()),
    })
}

struct Chain {
    statements: Vec<String>,
    extra_inputs: Vec<String>,
    current: String,
    pending: Vec<String>,
    labels: usize,
}

impl Chain {
    fn label(&mut self) -> String {
        self.labels += 1;
        format!("g{}", self.labels - 1)
    }

    fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let out = self.label();
        self.statements.push(format!(
            "[{}]{}[{out}]",
            self.current,
            self.pending.join(",")
        ));
        self.current = out;
        self.pending.clear();
    }

    fn add(&mut self, piece: Piece) {
        if piece.strength.is_none() && piece.look_image.is_none() {
            self.pending.extend(piece.filters);
            return;
        }
        self.flush();
        let mut branch = self.current.clone();
        let mut original = String::new();
        if piece.strength.is_some() {
            original = self.label();
            branch = self.label();
            self.statements
                .push(format!("[{}]split[{original}][{branch}]", self.current));
        }
        let graded = self.label();
        if let Some(image) = piece.look_image {
            self.extra_inputs.push(image);
            self.statements.push(format!(
                "[{branch}][{}:v]haldclut=repeatlast=1[{graded}]",
                self.extra_inputs.len()
            ));
        } else {
            self.statements
                .push(format!("[{branch}]{}[{graded}]", piece.filters.join(",")));
        }
        match piece.strength {
            None => self.current = graded,
            Some(strength) => {
                let out = self.label();
                self.statements.push(format!(
                    "[{graded}][{original}]blend=all_mode=normal:all_opacity={}[{out}]",
                    num(strength)
                ));
                self.current = out;
            }
        }
    }
}

pub fn grade_filters<E: From<GradeError>>(
    entries: &[StyleEntry],
    size: GradeSize,
    resolve: Resolve<'_, E>,
) -> Result<GradeChain, E> {
    let mut chain = Chain {
        statements: Vec::new(),
        extra_inputs: Vec::new(),
        current: "0:v".into(),
        pending: Vec::new(),
        labels: 0,
    };
    for entry in entries {
        match piece_of(entry, size, resolve)? {
            Some(piece) if !piece.filters.is_empty() || piece.look_image.is_some() => {
                chain.add(piece);
            }
            _ => {}
        }
    }
    let mut last = std::mem::take(&mut chain.pending);
    if !size.still {
        last.push(EVEN_SIZE.to_owned());
    }
    let tail = if last.is_empty() {
        "null".to_owned()
    } else {
        last.join(",")
    };
    chain
        .statements
        .push(format!("[{}]{tail}[out]", chain.current));
    Ok(GradeChain {
        chain: chain.statements.join(";"),
        extra_inputs: chain.extra_inputs,
    })
}
