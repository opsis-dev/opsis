use opsis_layout::computed::{Context, TextStyle};
use opsis_layout::output::Rect;
use opsis_scene::color::{Color, ColorSpace, Interpolation};
use opsis_scene::paint::{ColorStop, Gradient, LinearDirection, RadialShape, RadialSize, Side};
use opsis_scene::values::Length;
use opsis_skia::{
    Gradient as SkiaGradient, GradientSpace, HueMethod, Matrix33, Rgba, Shader, TileMode,
};
use opsis_types::numbers::narrow;

use crate::color::{is_legacy, resolve};
use crate::geometry::{point, px};

struct Stops {
    colors: Vec<Rgba>,
    positions: Vec<f64>,
}

fn stop_positions(
    stops: &[ColorStop],
    length: f64,
    text: &TextStyle,
    context: &Context,
    current: &Color,
) -> Stops {
    let mut colors = Vec::new();
    let mut positions: Vec<Option<f64>> = Vec::new();
    let at = |l: &Length| px(l, text, context, length) / length.max(f64::MIN_POSITIVE);
    for stop in stops {
        let color = resolve(&stop.color, current).extended();
        colors.push(color);
        positions.push(stop.at.as_ref().map(at));
        if let Some(until) = &stop.until {
            colors.push(color);
            positions.push(Some(at(until)));
        }
    }
    let count = positions.len();
    if count == 0 {
        return Stops {
            colors,
            positions: Vec::new(),
        };
    }
    if positions[0].is_none() {
        positions[0] = Some(0.0);
    }
    if positions[count - 1].is_none() {
        positions[count - 1] = Some(1.0);
    }
    let mut highest = f64::NEG_INFINITY;
    for p in positions.iter_mut().flatten() {
        if *p < highest {
            *p = highest;
        }
        highest = *p;
    }
    let mut i = 0;
    while i < count {
        if positions[i].is_some() {
            i += 1;
            continue;
        }
        let start = i - 1;
        let mut end = i;
        while positions[end].is_none() {
            end += 1;
        }
        let low = positions[start].unwrap_or(0.0);
        let high = positions[end].unwrap_or(1.0);
        let gaps = opsis_types::numbers::float(end - start);
        for (k, slot) in positions.iter_mut().enumerate().take(end).skip(start + 1) {
            *slot = Some(low + (high - low) * opsis_types::numbers::float(k - start) / gaps);
        }
        i = end;
    }
    Stops {
        colors,
        positions: positions.into_iter().map(|p| p.unwrap_or(0.0)).collect(),
    }
}

const fn space_of(space: ColorSpace) -> GradientSpace {
    match space {
        ColorSpace::Srgb => GradientSpace::Srgb,
        ColorSpace::SrgbLinear => GradientSpace::SrgbLinear,
        ColorSpace::DisplayP3 | ColorSpace::XyzD65 => GradientSpace::Destination,
        ColorSpace::Hsl => GradientSpace::Hsl,
        ColorSpace::Hwb => GradientSpace::Hwb,
        ColorSpace::Lab => GradientSpace::Lab,
        ColorSpace::Lch => GradientSpace::Lch,
        ColorSpace::Oklab => GradientSpace::Oklab,
        ColorSpace::Oklch => GradientSpace::Oklch,
    }
}

const fn hue_of(hue: opsis_scene::color::HueMethod) -> HueMethod {
    match hue {
        opsis_scene::color::HueMethod::Shorter => HueMethod::Shorter,
        opsis_scene::color::HueMethod::Longer => HueMethod::Longer,
        opsis_scene::color::HueMethod::Increasing => HueMethod::Increasing,
        opsis_scene::color::HueMethod::Decreasing => HueMethod::Decreasing,
    }
}

fn settings(interpolation: Interpolation, stops: &[ColorStop], repeating: bool) -> SkiaGradient {
    let legacy = stops.iter().all(|s| is_legacy(&s.color));
    let space = if interpolation.space == ColorSpace::Srgb && !legacy {
        ColorSpace::Oklab
    } else {
        interpolation.space
    };
    let mut gradient = SkiaGradient::new(space_of(space));
    gradient.hue = hue_of(interpolation.hue);
    gradient.tile = if repeating {
        TileMode::Repeat
    } else {
        TileMode::Clamp
    };
    gradient
}

fn normalized(stops: &Stops, repeating: bool) -> (f64, f64, Vec<f32>) {
    let first = stops.positions.first().copied().unwrap_or(0.0);
    let last = stops.positions.last().copied().unwrap_or(1.0);
    let (lo, hi) = if repeating {
        (first, last)
    } else {
        (first.min(0.0), last.max(1.0))
    };
    let span = hi - lo;
    if span <= 0.0 {
        return (lo, lo + 1.0, stops.positions.iter().map(|_| 0.0).collect());
    }
    let positions = stops
        .positions
        .iter()
        .map(|p| narrow((p - lo) / span))
        .collect();
    (lo, hi, positions)
}

fn css_angle(direction: &LinearDirection, width: f64, height: f64) -> f64 {
    match direction {
        LinearDirection::Angle(angle) => angle.degrees().to_radians(),
        LinearDirection::ToSide {
            horizontal,
            vertical,
        } => {
            let dx = match horizontal {
                Some(Side::Right) => height,
                Some(Side::Left) => -height,
                _ => 0.0,
            };
            let dy = match vertical {
                Some(Side::Bottom) => width,
                Some(Side::Top) => -width,
                _ => 0.0,
            };
            if horizontal.is_none() && vertical.is_none() {
                std::f64::consts::PI
            } else if horizontal.is_none() || vertical.is_none() {
                let (sx, sy) = (dx.signum(), dy.signum());
                sx.atan2(-sy)
            } else {
                dx.atan2(-dy)
            }
        }
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
pub fn shader(
    gradient: &Gradient,
    tile: Rect,
    text: &TextStyle,
    context: &Context,
    current: &Color,
) -> Option<Shader> {
    match gradient {
        Gradient::Linear {
            direction,
            stops,
            repeating,
            interpolation,
        } => {
            let angle = css_angle(direction, tile.width, tile.height);
            let (sin, cos) = angle.sin_cos();
            let length = (tile.width * sin).abs() + (tile.height * cos).abs();
            let resolved = stop_positions(stops, length, text, context, current);
            let (lo, hi, positions) = normalized(&resolved, *repeating);
            let (cx, cy) = (tile.x + tile.width / 2.0, tile.y + tile.height / 2.0);
            let (dx, dy) = (sin, -cos);
            let along = |t: f64| {
                let d = (t - 0.5) * length;
                point(cx + dx * d, cy + dy * d)
            };
            Shader::linear(
                along(lo),
                along(hi),
                &resolved.colors,
                &positions,
                settings(*interpolation, stops, *repeating),
            )
        }
        Gradient::Radial {
            shape,
            size,
            center,
            stops,
            repeating,
            interpolation,
        } => {
            let cx = tile.x + px(&center.x, text, context, tile.width);
            let cy = tile.y + px(&center.y, text, context, tile.height);
            let (rx, ry) = radial_size(
                *shape,
                size,
                (cx - tile.x, cy - tile.y),
                tile,
                text,
                context,
            );
            let rx = rx.max(f64::MIN_POSITIVE);
            let ry = ry.max(f64::MIN_POSITIVE);
            let resolved = stop_positions(stops, rx, text, context, current);
            let (lo, hi, positions) = normalized(&resolved, *repeating);
            let lo = lo.max(0.0);
            let mut settings = settings(*interpolation, stops, *repeating);
            let scale_y = ry / rx;
            settings.local = Matrix33::new([
                1.0,
                0.0,
                0.0,
                0.0,
                narrow(scale_y),
                narrow(cy - cy * scale_y),
                0.0,
                0.0,
                1.0,
            ]);
            if lo > 0.0 {
                Shader::conical(
                    (point(cx, cy), narrow(rx * lo)),
                    (point(cx, cy), narrow(rx * hi)),
                    &resolved.colors,
                    &positions,
                    settings,
                )
            } else {
                Shader::radial(
                    point(cx, cy),
                    narrow(rx * hi),
                    &resolved.colors,
                    &positions,
                    settings,
                )
            }
        }
        Gradient::Conic {
            from,
            center,
            stops,
            repeating,
            interpolation,
        } => {
            let cx = tile.x + px(&center.x, text, context, tile.width);
            let cy = tile.y + px(&center.y, text, context, tile.height);
            let resolved = stop_positions(stops, 1.0, text, context, current);
            let (lo, hi, positions) = normalized(&resolved, *repeating);
            let mut settings = settings(*interpolation, stops, *repeating);
            let turn = (from.degrees() - 90.0).to_radians();
            let (sin, cos) = turn.sin_cos();
            settings.local = Matrix33::new([
                narrow(cos),
                narrow(-sin),
                narrow(cx - cx * cos + cy * sin),
                narrow(sin),
                narrow(cos),
                narrow(cy - cx * sin - cy * cos),
                0.0,
                0.0,
                1.0,
            ]);
            Shader::sweep(
                point(cx, cy),
                (narrow(lo * 360.0), narrow(hi * 360.0)),
                &resolved.colors,
                &positions,
                settings,
            )
        }
    }
}

fn radial_size(
    shape: RadialShape,
    size: &RadialSize,
    (cx, cy): (f64, f64),
    tile: Rect,
    text: &TextStyle,
    context: &Context,
) -> (f64, f64) {
    let (left, right) = (cx.abs(), (tile.width - cx).abs());
    let (top, bottom) = (cy.abs(), (tile.height - cy).abs());
    let closest = (left.min(right), top.min(bottom));
    let farthest = (left.max(right), top.max(bottom));
    let corners = [(left, top), (right, top), (left, bottom), (right, bottom)];
    let distance = |(x, y): (f64, f64)| x.hypot(y);
    match (shape, size) {
        (_, RadialSize::Explicit(x, y)) => {
            let rx = px(x, text, context, tile.width);
            let ry = match shape {
                RadialShape::Circle => rx,
                RadialShape::Ellipse => px(y, text, context, tile.height),
            };
            (rx, ry)
        }
        (RadialShape::Circle, RadialSize::ClosestSide) => {
            let r = closest.0.min(closest.1);
            (r, r)
        }
        (RadialShape::Circle, RadialSize::FarthestSide) => {
            let r = farthest.0.max(farthest.1);
            (r, r)
        }
        (RadialShape::Circle, RadialSize::ClosestCorner) => {
            let r = corners
                .map(distance)
                .into_iter()
                .fold(f64::INFINITY, f64::min);
            (r, r)
        }
        (RadialShape::Circle, RadialSize::FarthestCorner) => {
            let r = corners.map(distance).into_iter().fold(0.0, f64::max);
            (r, r)
        }
        (RadialShape::Ellipse, RadialSize::ClosestSide) => closest,
        (RadialShape::Ellipse, RadialSize::FarthestSide) => farthest,
        (RadialShape::Ellipse, RadialSize::ClosestCorner) => (
            closest.0 * std::f64::consts::SQRT_2,
            closest.1 * std::f64::consts::SQRT_2,
        ),
        (RadialShape::Ellipse, RadialSize::FarthestCorner) => (
            farthest.0 * std::f64::consts::SQRT_2,
            farthest.1 * std::f64::consts::SQRT_2,
        ),
    }
}
