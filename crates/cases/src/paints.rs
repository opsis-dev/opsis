use opsis_scene::color::{Color, ColorSpace, HueMethod, Interpolation};
use opsis_scene::paint::{
    ColorStop, Gradient, Image, ImageLayer, LinearDirection, RadialShape, RadialSize, Side,
};
use opsis_scene::values::{Alpha, Angle, Length, Num, Position2};

pub fn hex(rgb: u32) -> Color {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color::rgb8(r, g, b)
}

pub fn hexa(rgba: u32) -> Color {
    let [r, g, b, a] = rgba.to_be_bytes();
    rgba_of(r, g, b, f64::from(a) / 255.0)
}

pub fn rgba_of(r: u8, g: u8, b: u8, alpha: f64) -> Color {
    Color::srgb_alpha(
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
        alpha,
    )
}

pub const fn colour(space: ColorSpace, components: [f64; 3], alpha: f64) -> Color {
    Color::Components {
        space,
        components: [Num(components[0]), Num(components[1]), Num(components[2])],
        alpha: Alpha(alpha),
    }
}

pub const fn transparent() -> Color {
    Color::TRANSPARENT
}

pub const fn stop(color: Color) -> ColorStop {
    ColorStop {
        color,
        at: None,
        until: None,
    }
}

pub const fn stop_at(color: Color, at: Length) -> ColorStop {
    ColorStop {
        color,
        at: Some(at),
        until: None,
    }
}

pub const fn stop_span(color: Color, at: Length, until: Length) -> ColorStop {
    ColorStop {
        color,
        at: Some(at),
        until: Some(until),
    }
}

pub fn degrees(value: f64) -> Length {
    Length::percent(value / 3.6)
}

pub const fn space(space: ColorSpace) -> Interpolation {
    Interpolation {
        space,
        hue: HueMethod::Shorter,
    }
}

pub const fn linear(degrees: f64, stops: Vec<ColorStop>) -> Gradient {
    linear_in(Interpolation::DEFAULT, degrees, stops)
}

pub const fn linear_in(
    interpolation: Interpolation,
    degrees: f64,
    stops: Vec<ColorStop>,
) -> Gradient {
    Gradient::Linear {
        direction: LinearDirection::Angle(Angle::Deg(Num(degrees))),
        stops,
        repeating: false,
        interpolation,
    }
}

pub const fn linear_to(
    horizontal: Option<Side>,
    vertical: Option<Side>,
    stops: Vec<ColorStop>,
) -> Gradient {
    Gradient::Linear {
        direction: LinearDirection::ToSide {
            horizontal,
            vertical,
        },
        stops,
        repeating: false,
        interpolation: Interpolation::DEFAULT,
    }
}

pub const fn radial(
    shape: RadialShape,
    size: RadialSize,
    center: Position2,
    stops: Vec<ColorStop>,
) -> Gradient {
    Gradient::Radial {
        shape,
        size,
        center,
        stops,
        repeating: false,
        interpolation: Interpolation::DEFAULT,
    }
}

pub const fn conic(from: f64, center: Position2, stops: Vec<ColorStop>) -> Gradient {
    Gradient::Conic {
        from: Angle::Deg(Num(from)),
        center,
        stops,
        repeating: false,
        interpolation: Interpolation::DEFAULT,
    }
}

#[must_use]
pub fn repeating(gradient: Gradient) -> Gradient {
    match gradient {
        Gradient::Linear {
            direction,
            stops,
            interpolation,
            ..
        } => Gradient::Linear {
            direction,
            stops,
            repeating: true,
            interpolation,
        },
        Gradient::Radial {
            shape,
            size,
            center,
            stops,
            interpolation,
            ..
        } => Gradient::Radial {
            shape,
            size,
            center,
            stops,
            repeating: true,
            interpolation,
        },
        Gradient::Conic {
            from,
            center,
            stops,
            interpolation,
            ..
        } => Gradient::Conic {
            from,
            center,
            stops,
            repeating: true,
            interpolation,
        },
    }
}

#[must_use]
pub fn interpolated(gradient: Gradient, with: Interpolation) -> Gradient {
    match gradient {
        Gradient::Linear {
            direction,
            stops,
            repeating,
            ..
        } => Gradient::Linear {
            direction,
            stops,
            repeating,
            interpolation: with,
        },
        Gradient::Radial {
            shape,
            size,
            center,
            stops,
            repeating,
            ..
        } => Gradient::Radial {
            shape,
            size,
            center,
            stops,
            repeating,
            interpolation: with,
        },
        Gradient::Conic {
            from,
            center,
            stops,
            repeating,
            ..
        } => Gradient::Conic {
            from,
            center,
            stops,
            repeating,
            interpolation: with,
        },
    }
}

pub const fn layer(gradient: Gradient) -> ImageLayer {
    ImageLayer::of(Image::Gradient(gradient))
}

pub const fn at(x: f64, y: f64) -> Position2 {
    Position2 {
        x: Length::percent(x),
        y: Length::percent(y),
    }
}
