use color::{ColorSpaceTag, parse_color};
use opsis_scene::color::Color;
use opsis_scene::interpolate::{InterpolateError, Interpolation, interpolate};
use opsis_types::numbers::{exact_eq, float, truncate};

const CHROMA_RANGE: f64 = 0.4;
const BISECTION_STEPS: f64 = 8192.0;
const BLACK: &str = "#000";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: Option<f64>,
    pub alpha: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub alpha: Option<f64>,
}

fn normalize_hue(hue: f64) -> f64 {
    let h = hue % 360.0;
    if h < 0.0 { h + 360.0 } else { h }
}

fn to_linear(c: f64) -> f64 {
    let abs = c.abs();
    if abs <= 0.04045 {
        return c / 12.92;
    }
    sign_or_one(c) * ((abs + 0.055) / 1.055).powf(2.4)
}

fn from_linear(c: f64) -> f64 {
    let abs = c.abs();
    if abs > 0.003_130_8 {
        return sign_or_one(c) * 1.055_f64.mul_add(abs.powf(1.0 / 2.4), -0.055);
    }
    c * 12.92
}

fn sign_or_one(c: f64) -> f64 {
    if c > 0.0 {
        1.0
    } else if c < 0.0 {
        -1.0
    } else {
        1.0
    }
}

#[expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "culori rounds every step and names its channels so, and the colours have to match it"
)]
pub fn rgb_to_oklch(rgb: Rgb) -> Oklch {
    let (r, g, b) = (to_linear(rgb.r), to_linear(rgb.g), to_linear(rgb.b));
    let l_ =
        (0.412_221_469_470_763 * r + 0.536_332_537_261_734_8 * g + 0.051_445_993_267_502_2 * b)
            .cbrt();
    let m_ =
        (0.211_903_495_817_825_2 * r + 0.680_699_550_645_234_4 * g + 0.107_396_953_536_940_6 * b)
            .cbrt();
    let s_ =
        (0.088_302_459_190_056_4 * r + 0.281_718_839_136_121_5 * g + 0.629_978_701_673_822_2 * b)
            .cbrt();
    let l =
        0.210_454_268_309_314 * l_ + 0.793_617_774_702_305_4 * m_ - 0.004_072_043_011_619_3 * s_;
    let (mut a, mut lab_b) = (
        1.977_998_532_431_168_4 * l_ - 2.428_592_242_048_58 * m_ + 0.450_593_709_617_411 * s_,
        0.025_904_042_465_547_8 * l_ + 0.782_771_712_457_529_6 * m_ - 0.808_675_754_923_077_4 * s_,
    );
    if exact_eq(rgb.r, rgb.b) && exact_eq(rgb.b, rgb.g) {
        a = 0.0;
        lab_b = 0.0;
    }
    lab_to_lch(l, a, lab_b, rgb.alpha)
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the colour formula's own"
)]
fn lab_to_lch(l: f64, a: f64, b: f64, alpha: Option<f64>) -> Oklch {
    let c = a.hypot(b);
    let h = (!exact_eq(c, 0.0) && !c.is_nan()).then(|| normalize_hue(b.atan2(a).to_degrees()));
    Oklch { l, c, h, alpha }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "culori rounds every step, and the colours have to match it"
)]
pub fn oklch_to_rgb(color: Oklch) -> Rgb {
    let h = color.h.unwrap_or(0.0);
    let chroma = !exact_eq(color.c, 0.0) && !color.c.is_nan();
    let a = if chroma {
        color.c * (h / 180.0 * std::f64::consts::PI).cos()
    } else {
        0.0
    };
    let b = if chroma {
        color.c * (h / 180.0 * std::f64::consts::PI).sin()
    } else {
        0.0
    };
    let l = color.l;
    let l_ = (l + 0.396_337_777_376_174_9 * a + 0.215_803_757_309_913_6 * b).powi(3);
    let m_ = (l - 0.105_561_345_815_658_6 * a - 0.063_854_172_825_813_3 * b).powi(3);
    let s_ = (l - 0.089_484_177_529_811_9 * a - 1.291_485_548_019_409_2 * b).powi(3);
    Rgb {
        r: from_linear(
            4.076_741_636_075_957 * l_ - 3.307_711_539_258_061_6 * m_
                + 0.230_969_903_182_104_4 * s_,
        ),
        g: from_linear(
            -1.268_437_973_285_031_7 * l_ + 2.609_757_349_287_688_7 * m_
                - 0.341_319_376_002_657_3 * s_,
        ),
        b: from_linear(
            -0.004_196_076_138_675_6 * l_ - 0.703_418_617_935_936_2 * m_
                + 1.707_614_694_074_611_7 * s_,
        ),
        alpha: color.alpha,
    }
}

fn parsed_rgb(css: &str) -> Option<Rgb> {
    let parsed = parse_color(css.trim()).ok()?;
    let alpha = Some(f64::from(parsed.components[3]));
    let srgb = parsed.convert(ColorSpaceTag::Srgb).components;
    Some(Rgb {
        r: f64::from(srgb[0]),
        g: f64::from(srgb[1]),
        b: f64::from(srgb[2]),
        alpha,
    })
}

pub fn css_oklch(css: &str) -> Oklch {
    let css = if css.trim().starts_with("var(") || parse_color(css.trim()).is_err() {
        BLACK
    } else {
        css
    };
    let Ok(parsed) = parse_color(css.trim()) else {
        return Oklch {
            l: 0.0,
            c: 0.0,
            h: None,
            alpha: Some(1.0),
        };
    };
    let alpha = Some(f64::from(parsed.components[3]));
    match parsed.cs {
        ColorSpaceTag::Oklch => {
            let [l, c, h, _] = parsed.components.map(f64::from);
            Oklch {
                l,
                c,
                h: Some(h),
                alpha,
            }
        }
        ColorSpaceTag::Oklab => {
            let [l, a, b, _] = parsed.components.map(f64::from);
            lab_to_lch(l, a, b, alpha)
        }
        _ => parsed_rgb(css).map_or(
            Oklch {
                l: 0.0,
                c: 0.0,
                h: None,
                alpha: Some(1.0),
            },
            rgb_to_oklch,
        ),
    }
}

fn shorter_hues(a: Option<f64>, b: Option<f64>) -> (Option<f64>, Option<f64>) {
    match (a, b) {
        (Some(a), Some(b)) => {
            let first = normalize_hue(a);
            let d = normalize_hue(b) - first;
            let step = if d.abs() <= 180.0 {
                d
            } else {
                360.0f64.mul_add(-d.signum(), d)
            };
            (Some(first), Some(step + first))
        }
        (a, b) => (a.map(normalize_hue), b.map(normalize_hue)),
    }
}

fn lerp_pair(a: Option<f64>, b: Option<f64>, t: f64) -> Option<f64> {
    match (a, b) {
        (None, None) => None,
        (Some(a), Some(b)) => Some(t.mul_add(b - a, a)),
        (Some(v), None) | (None, Some(v)) => Some(v),
    }
}

pub fn mix(first: Oklch, second: Oklch, t: f64) -> Oklch {
    let t = t.clamp(0.0, 1.0);
    let (ha, hb) = shorter_hues(first.h, second.h);
    let (aa, ab) = match (first.alpha, second.alpha) {
        (None, None) => (None, None),
        (a, b) => (Some(a.unwrap_or(1.0)), Some(b.unwrap_or(1.0))),
    };
    Oklch {
        l: lerp_pair(Some(first.l), Some(second.l), t).unwrap_or(0.0),
        c: lerp_pair(Some(first.c), Some(second.c), t).unwrap_or(0.0),
        h: lerp_pair(ha, hb, t),
        alpha: lerp_pair(aa, ab, t),
    }
}

fn displayable(rgb: Rgb) -> bool {
    [rgb.r, rgb.g, rgb.b]
        .iter()
        .all(|v| (0.0..=1.0).contains(v))
}

const fn clipped(rgb: Rgb) -> Rgb {
    Rgb {
        r: rgb.r.clamp(0.0, 1.0),
        g: rgb.g.clamp(0.0, 1.0),
        b: rgb.b.clamp(0.0, 1.0),
        alpha: rgb.alpha,
    }
}

pub fn clamp_chroma(color: Oklch) -> Rgb {
    let rgb = oklch_to_rgb(color);
    if displayable(rgb) {
        return rgb;
    }
    let mut clamped = Oklch { c: 0.0, ..color };
    if !displayable(oklch_to_rgb(clamped)) {
        return clipped(oklch_to_rgb(clamped));
    }
    let (mut start, mut end) = (0.0, color.c);
    let resolution = CHROMA_RANGE / BISECTION_STEPS;
    let mut last_good = clamped.c;
    loop {
        if end - start <= resolution {
            break;
        }
        clamped.c = f64::mul_add(end - start, 0.5, start);
        if displayable(oklch_to_rgb(clamped)) {
            last_good = clamped.c;
            start = clamped.c;
        } else {
            end = clamped.c;
        }
    }
    if displayable(oklch_to_rgb(clamped)) {
        oklch_to_rgb(clamped)
    } else {
        oklch_to_rgb(Oklch {
            c: last_good,
            ..clamped
        })
    }
}

fn channel(value: f64) -> u8 {
    let scaled = (if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    } * 255.0)
        .round();
    truncate(scaled)
}

pub fn rgb_text(rgb: Rgb) -> String {
    let (r, g, b) = (channel(rgb.r), channel(rgb.g), channel(rgb.b));
    match rgb.alpha {
        Some(alpha) if !exact_eq(alpha, 1.0) => {
            let alpha = (alpha.clamp(0.0, 1.0) * 100.0).round() / 100.0;
            format!(
                "rgba({r}, {g}, {b}, {})",
                opsis_scene::random::js_number_text(alpha)
            )
        }
        _ => format!("rgb({r}, {g}, {b})"),
    }
}

pub fn mix_oklch(a: &str, b: &str, t: f64) -> String {
    rgb_text(clamp_chroma(mix(css_oklch(a), css_oklch(b), t)))
}

pub fn interpolate_colors(
    input: f64,
    range: &[f64],
    colors: &[&str],
) -> Result<String, InterpolateError> {
    let indices: Vec<f64> = (0..range.len()).map(float).collect();
    let position = interpolate(input, range, &indices, &Interpolation::clamped())?;
    let last = colors.len().saturating_sub(2);
    let index: usize = truncate(position.floor().clamp(0.0, float(last)));
    let from = colors.get(index).copied().unwrap_or(BLACK);
    let to = colors.get(index + 1).copied().unwrap_or(from);
    Ok(mix_oklch(from, to, position - float(index)))
}

pub fn interpolated_color(
    input: f64,
    range: &[f64],
    colors: &[&str],
) -> Result<Color, InterpolateError> {
    interpolate_colors(input, range, colors).map(|text| Color::css(&text).unwrap_or_default())
}

fn parsed_or_black(css: &str) -> Rgb {
    let text = css.trim();
    if text.starts_with("var(") {
        return Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha: Some(1.0),
        };
    }
    parsed_rgb(text).unwrap_or(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        alpha: Some(1.0),
    })
}

pub fn with_alpha(css: &str, alpha: f64) -> String {
    let rgb = parsed_or_black(css);
    rgb_text(Rgb {
        alpha: Some(alpha),
        ..rgb
    })
}

pub fn lightness(css: &str) -> f64 {
    rgb_to_oklch(parsed_or_black(css)).l
}
