use color::ColorSpaceTag;
use serde::{Deserialize, Serialize};

use crate::values::{Alpha, Num};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSpace {
    #[default]
    Srgb,
    SrgbLinear,
    DisplayP3,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    XyzD65,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HueMethod {
    #[default]
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Interpolation {
    pub space: ColorSpace,
    pub hue: HueMethod,
}

impl Interpolation {
    pub const DEFAULT: Self = Self {
        space: ColorSpace::Srgb,
        hue: HueMethod::Shorter,
    };
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Color {
    Components {
        space: ColorSpace,
        components: [Num; 3],
        alpha: Alpha,
    },
    CurrentColor,
    Mix(Box<ColorMix>),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ColorMix {
    pub interpolation: Interpolation,
    pub first: Color,
    pub first_share: Option<Num>,
    pub second: Color,
    pub second_share: Option<Num>,
}

impl Color {
    pub const TRANSPARENT: Self = Self::srgb_alpha(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::srgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::srgb(1.0, 1.0, 1.0);

    pub const fn srgb(r: f64, g: f64, b: f64) -> Self {
        Self::srgb_alpha(r, g, b, 1.0)
    }

    pub const fn srgb_alpha(r: f64, g: f64, b: f64, alpha: f64) -> Self {
        Self::Components {
            space: ColorSpace::Srgb,
            components: [Num(r), Num(g), Num(b)],
            alpha: Alpha(alpha),
        }
    }

    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::srgb(
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        )
    }

    pub const fn oklch(l: f64, c: f64, h: f64, alpha: f64) -> Self {
        Self::Components {
            space: ColorSpace::Oklch,
            components: [Num(l), Num(c), Num(h)],
            alpha: Alpha(alpha),
        }
    }

    pub fn mix(
        interpolation: Interpolation,
        first: (Self, Option<f64>),
        second: (Self, Option<f64>),
    ) -> Self {
        Self::Mix(Box::new(ColorMix {
            interpolation,
            first: first.0,
            first_share: first.1.map(Num),
            second: second.0,
            second_share: second.1.map(Num),
        }))
    }
}

const fn space_of(tag: ColorSpaceTag) -> Option<ColorSpace> {
    Some(match tag {
        ColorSpaceTag::Srgb => ColorSpace::Srgb,
        ColorSpaceTag::LinearSrgb => ColorSpace::SrgbLinear,
        ColorSpaceTag::DisplayP3 => ColorSpace::DisplayP3,
        ColorSpaceTag::Hsl => ColorSpace::Hsl,
        ColorSpaceTag::Hwb => ColorSpace::Hwb,
        ColorSpaceTag::Lab => ColorSpace::Lab,
        ColorSpaceTag::Lch => ColorSpace::Lch,
        ColorSpaceTag::Oklab => ColorSpace::Oklab,
        ColorSpaceTag::Oklch => ColorSpace::Oklch,
        ColorSpaceTag::XyzD65 => ColorSpace::XyzD65,
        _ => return None,
    })
}

fn as_written(component: f32) -> f64 {
    component
        .to_string()
        .parse()
        .unwrap_or_else(|_| f64::from(component))
}

fn function_body<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let open = text.get(..=name.len())?;
    if !open.eq_ignore_ascii_case(&format!("{name}(")) {
        return None;
    }
    text.get(name.len() + 1..)?.strip_suffix(')')
}

fn mix_part(text: &str) -> Option<(Color, Option<f64>)> {
    let parts = crate::css::words(text);
    let (share, color) = match parts.as_slice() {
        [color] => (None, *color),
        [color, share] | [share, color] if share.ends_with('%') => (
            Some(share.trim_end_matches('%').parse::<f64>().ok()?),
            *color,
        ),
        _ => return None,
    };
    Some((Color::css(color)?, share))
}

fn mixed(inner: &str) -> Option<Color> {
    let args = crate::css::split_top(inner, |c| c == ',');
    let [method, first, second] = args.as_slice() else {
        return None;
    };
    let (interpolation, rest) = crate::css::take_interpolation(&crate::css::words(method))?;
    if !rest.is_empty() {
        return None;
    }
    Some(Color::mix(
        interpolation,
        mix_part(first)?,
        mix_part(second)?,
    ))
}

impl Color {
    pub fn css(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("currentcolor") {
            return Some(Self::CurrentColor);
        }
        if let Some(inner) = function_body(text, "color-mix") {
            return mixed(inner);
        }
        let parsed = color::parse_color(text).ok()?;
        let (space, parsed) = space_of(parsed.cs).map_or_else(
            || (ColorSpace::XyzD65, parsed.convert(ColorSpaceTag::XyzD65)),
            |space| (space, parsed),
        );
        let [a, b, c, alpha] = parsed.components.map(as_written);
        Some(Self::Components {
            space,
            components: [Num(a), Num(b), Num(c)],
            alpha: Alpha(alpha),
        })
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}
