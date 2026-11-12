use color::{ColorSpaceTag, DynamicColor, Flags, HueDirection};
use opsis_scene::color::{Color, ColorMix, ColorSpace, HueMethod};
use opsis_skia::Rgba;
use opsis_types::numbers::narrow;

const WHOLE_SHARE: f64 = 100.0;
const EVEN_SHARE: f64 = 50.0;

#[derive(Debug, Clone, Copy)]
pub struct Resolved(pub DynamicColor);

impl Resolved {
    pub fn transparent() -> Self {
        Self(DynamicColor {
            cs: ColorSpaceTag::Srgb,
            flags: Flags::default(),
            components: [0.0; 4],
        })
    }

    pub fn srgb(self) -> [f32; 4] {
        self.0.convert(ColorSpaceTag::Srgb).components
    }

    pub const fn alpha(self) -> f32 {
        self.0.components[3]
    }

    pub fn rgba(self) -> Rgba {
        let [r, g, b, a] = self.srgb().map(|c| c.clamp(0.0, 1.0));
        Rgba::new(r, g, b, a)
    }

    pub fn extended(self) -> Rgba {
        let [r, g, b, a] = self.srgb();
        Rgba::new(r, g, b, a.clamp(0.0, 1.0))
    }

    #[must_use]
    pub const fn faded(self, by: f64) -> Self {
        Self(self.0.multiply_alpha(narrow(by)))
    }
}

pub const fn tag(space: ColorSpace) -> ColorSpaceTag {
    match space {
        ColorSpace::Srgb => ColorSpaceTag::Srgb,
        ColorSpace::SrgbLinear => ColorSpaceTag::LinearSrgb,
        ColorSpace::DisplayP3 => ColorSpaceTag::DisplayP3,
        ColorSpace::Hsl => ColorSpaceTag::Hsl,
        ColorSpace::Hwb => ColorSpaceTag::Hwb,
        ColorSpace::Lab => ColorSpaceTag::Lab,
        ColorSpace::Lch => ColorSpaceTag::Lch,
        ColorSpace::Oklab => ColorSpaceTag::Oklab,
        ColorSpace::Oklch => ColorSpaceTag::Oklch,
        ColorSpace::XyzD65 => ColorSpaceTag::XyzD65,
    }
}

pub const fn direction(hue: HueMethod) -> HueDirection {
    match hue {
        HueMethod::Shorter => HueDirection::Shorter,
        HueMethod::Longer => HueDirection::Longer,
        HueMethod::Increasing => HueDirection::Increasing,
        HueMethod::Decreasing => HueDirection::Decreasing,
    }
}

pub fn interpolate(
    space: ColorSpace,
    hue: HueMethod,
    (first, second): (Resolved, Resolved),
    t: f64,
) -> Resolved {
    Resolved(
        first
            .0
            .interpolate(second.0, tag(space), direction(hue))
            .eval(narrow(t)),
    )
}

fn mix(m: &ColorMix, current: &Color) -> Resolved {
    let (p1, p2) = match (m.first_share, m.second_share) {
        (None, None) => (EVEN_SHARE, EVEN_SHARE),
        (Some(p), None) => (p.0, WHOLE_SHARE - p.0),
        (None, Some(p)) => (WHOLE_SHARE - p.0, p.0),
        (Some(a), Some(b)) => (a.0, b.0),
    };
    let sum = p1 + p2;
    if sum <= 0.0 {
        return Resolved::transparent();
    }
    let mixed = interpolate(
        m.interpolation.space,
        m.interpolation.hue,
        (resolve(&m.first, current), resolve(&m.second, current)),
        p2 / sum,
    );
    if sum < WHOLE_SHARE {
        mixed.faded(sum / WHOLE_SHARE)
    } else {
        mixed
    }
}

pub fn resolve(color: &Color, current: &Color) -> Resolved {
    match color {
        Color::Components {
            space,
            components,
            alpha,
        } => {
            let [a, b, c] = components.map(|n| narrow(n.0));
            Resolved(DynamicColor {
                cs: tag(*space),
                flags: Flags::default(),
                components: [a, b, c, narrow(alpha.0)],
            })
        }
        Color::CurrentColor => match current {
            Color::CurrentColor => resolve(&Color::BLACK, &Color::BLACK),
            other => resolve(other, &Color::BLACK),
        },
        Color::Mix(m) => mix(m, current),
    }
}

pub const fn is_legacy(color: &Color) -> bool {
    match color {
        Color::Components { space, .. } => {
            matches!(space, ColorSpace::Srgb | ColorSpace::Hsl | ColorSpace::Hwb)
        }
        Color::CurrentColor => true,
        Color::Mix(_) => false,
    }
}
