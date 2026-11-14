use std::collections::HashMap;

use opsis_skia::{
    BlendMode, Channel, ImageFilter, Kernel, LightSource, Lighting, PointF, RectF, Rgba, Shader,
    TileMode,
};
use usvg::filter::{
    ColorChannel, ColorInterpolation, ColorMatrixKind, CompositeOperator, EdgeMode, Input, Kind,
    MorphologyOperator, TransferFunction, TurbulenceKind,
};

use crate::svg::blend_of;

const ALPHA_ONLY: [f32; 20] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    0.0,
];

const LUMINANCE_TO_ALPHA: [f32; 20] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.212_5, 0.715_4,
    0.072_1, 0.0, 0.0,
];

struct Made {
    filter: ImageFilter,
    linear: bool,
}

fn rect_of(rect: usvg::NonZeroRect) -> RectF {
    RectF::new(rect.x(), rect.y(), rect.width(), rect.height())
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
fn saturation(s: f32) -> [f32; 20] {
    [
        0.213 + 0.787 * s,
        0.715 - 0.715 * s,
        0.072 - 0.072 * s,
        0.0,
        0.0,
        0.213 - 0.213 * s,
        0.715 + 0.285 * s,
        0.072 - 0.072 * s,
        0.0,
        0.0,
        0.213 - 0.213 * s,
        0.715 - 0.715 * s,
        0.072 + 0.928 * s,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ]
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
fn hue_rotation(degrees: f32) -> [f32; 20] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        0.213 + cos * 0.787 - sin * 0.213,
        0.715 - cos * 0.715 - sin * 0.715,
        0.072 - cos * 0.072 + sin * 0.928,
        0.0,
        0.0,
        0.213 - cos * 0.213 + sin * 0.143,
        0.715 + cos * 0.285 + sin * 0.140,
        0.072 - cos * 0.072 - sin * 0.283,
        0.0,
        0.0,
        0.213 - cos * 0.213 - sin * 0.787,
        0.715 - cos * 0.715 + sin * 0.715,
        0.072 + cos * 0.928 + sin * 0.072,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ]
}

const fn channel_of(channel: ColorChannel) -> Channel {
    match channel {
        ColorChannel::R => Channel::Red,
        ColorChannel::G => Channel::Green,
        ColorChannel::B => Channel::Blue,
        ColorChannel::A => Channel::Alpha,
    }
}

fn color_of(color: usvg::Color, opacity: f32) -> Rgba {
    Rgba::new(
        f32::from(color.red) / 255.0,
        f32::from(color.green) / 255.0,
        f32::from(color.blue) / 255.0,
        opacity,
    )
}

struct Graph {
    results: HashMap<String, Made>,
    last: Option<Made>,
    rects: HashMap<String, RectF>,
    last_rect: Option<RectF>,
    region: RectF,
}

impl Graph {
    fn input(&self, input: &Input, linear: bool) -> ImageFilter {
        let made = match input {
            Input::SourceGraphic => Made {
                filter: ImageFilter::source(),
                linear: false,
            },
            Input::SourceAlpha => Made {
                filter: ImageFilter::color_matrix(&ALPHA_ONLY, &ImageFilter::source()),
                linear,
            },
            Input::Reference(name) => {
                let Some(found) = self.results.get(name).or(self.last.as_ref()) else {
                    return ImageFilter::source();
                };
                return converted(found, linear);
            }
        };
        converted(&made, linear)
    }
}

impl Graph {
    fn rect_of(&self, input: &Input) -> RectF {
        match input {
            Input::SourceGraphic | Input::SourceAlpha => self.region,
            Input::Reference(name) => self
                .rects
                .get(name)
                .copied()
                .or(self.last_rect)
                .unwrap_or(self.region),
        }
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the transfer functions follow the SVG specification's arithmetic, and the pictures have to match the browser's"
)]
fn transferred(function: &TransferFunction, c: f32) -> f32 {
    match function {
        TransferFunction::Table(values) if !values.is_empty() => {
            let n = opsis_types::numbers::narrow(opsis_types::numbers::float(values.len() - 1));
            if n <= 0.0 || c >= 1.0 {
                return values[values.len() - 1];
            }
            let k = (c * n).floor();
            let i = usize::try_from(opsis_types::numbers::whole(f64::from(k))).unwrap_or(0);
            let next = values.get(i + 1).copied().unwrap_or(values[i]);
            values[i] + (c - k / n) * n * (next - values[i])
        }
        TransferFunction::Discrete(values) if !values.is_empty() => {
            let n = opsis_types::numbers::narrow(opsis_types::numbers::float(values.len()));
            let k = usize::try_from(opsis_types::numbers::whole(f64::from((c * n).floor())))
                .unwrap_or(0)
                .min(values.len() - 1);
            values[k]
        }
        TransferFunction::Identity | TransferFunction::Table(_) | TransferFunction::Discrete(_) => {
            c
        }
        TransferFunction::Linear { slope, intercept } => slope * c + intercept,
        TransferFunction::Gamma {
            amplitude,
            exponent,
            offset,
        } => amplitude * c.powf(*exponent) + offset,
    }
}

fn transfer_tables(functions: [&TransferFunction; 4]) -> [u8; 1024] {
    let mut tables = [0u8; 1024];
    for (channel, function) in functions.iter().enumerate() {
        for i in 0..256usize {
            let c = opsis_types::numbers::narrow(opsis_types::numbers::float(i) / 255.0);
            let value = (transferred(function, c).clamp(0.0, 1.0) * 255.0).round();
            tables[channel * 256 + i] =
                u8::try_from(opsis_types::numbers::whole(f64::from(value))).unwrap_or(u8::MAX);
        }
    }
    tables
}

const fn edge_of(edge: EdgeMode) -> TileMode {
    match edge {
        EdgeMode::None => TileMode::Decal,
        EdgeMode::Duplicate => TileMode::Clamp,
        EdgeMode::Wrap => TileMode::Repeat,
    }
}

const NO_CONE: f32 = 90.0;

fn light_of(light: usvg::filter::LightSource) -> LightSource {
    match light {
        usvg::filter::LightSource::DistantLight(distant) => {
            let (azimuth, elevation) =
                (distant.azimuth.to_radians(), distant.elevation.to_radians());
            LightSource {
                kind: 0,
                position: [
                    azimuth.cos() * elevation.cos(),
                    azimuth.sin() * elevation.cos(),
                    elevation.sin(),
                ],
                target: [0.0; 3],
                exponent: 1.0,
                cone: NO_CONE,
            }
        }
        usvg::filter::LightSource::PointLight(point) => LightSource {
            kind: 1,
            position: [point.x, point.y, point.z],
            target: [0.0; 3],
            exponent: 1.0,
            cone: NO_CONE,
        },
        usvg::filter::LightSource::SpotLight(spot) => LightSource {
            kind: 2,
            position: [spot.x, spot.y, spot.z],
            target: [spot.points_at_x, spot.points_at_y, spot.points_at_z],
            exponent: spot.specular_exponent.get(),
            cone: spot.limiting_cone_angle.unwrap_or(NO_CONE),
        },
    }
}

fn lighting_color(color: usvg::Color, linear: bool) -> Rgba {
    let to_linear = |v: u8| {
        let c = f32::from(v) / 255.0;
        if !linear {
            c
        } else if c <= 0.040_45 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    Rgba::new(
        to_linear(color.red),
        to_linear(color.green),
        to_linear(color.blue),
        1.0,
    )
}

fn converted(made: &Made, linear: bool) -> ImageFilter {
    match (made.linear, linear) {
        (false, true) => ImageFilter::to_linear(&made.filter),
        (true, false) => ImageFilter::to_srgb(&made.filter),
        _ => ImageFilter::compose(&ImageFilter::source(), &made.filter),
    }
}

pub type Render<'r> = dyn FnMut(&usvg::Group, RectF) -> Option<opsis_skia::Image> + 'r;

fn primitive(
    graph: &Graph,
    kind: &Kind,
    (linear, rect): (bool, RectF),
    render: &mut Render<'_>,
) -> (ImageFilter, bool) {
    match kind {
        Kind::GaussianBlur(blur) => (
            ImageFilter::blur(
                blur.std_dev_x().get(),
                blur.std_dev_y().get(),
                TileMode::Decal,
                &graph.input(blur.input(), linear),
            ),
            linear,
        ),
        Kind::Offset(offset) => (
            ImageFilter::offset(
                PointF::new(offset.dx(), offset.dy()),
                &graph.input(offset.input(), linear),
            ),
            linear,
        ),
        Kind::Flood(flood) => {
            let shader = Shader::color(color_of(flood.color(), flood.opacity().get()));
            match shader {
                Some(shader) => (ImageFilter::shader(&shader), false),
                None => (ImageFilter::source(), false),
            }
        }
        Kind::Merge(merge) => {
            let mut inputs = merge.inputs().iter().map(|i| graph.input(i, linear));
            let first = inputs.next().unwrap_or_else(ImageFilter::source);
            (
                inputs.fold(first, |acc, next| ImageFilter::merge(&acc, &next)),
                linear,
            )
        }
        Kind::Blend(blend) => (
            ImageFilter::blend(
                blend_of(blend.mode()),
                &graph.input(blend.input2(), linear),
                &graph.input(blend.input1(), linear),
            ),
            linear,
        ),
        Kind::Composite(composite) => {
            let foreground = graph.input(composite.input1(), linear);
            let background = graph.input(composite.input2(), linear);
            let mode = match composite.operator() {
                CompositeOperator::Over => BlendMode::SrcOver,
                CompositeOperator::In => BlendMode::SrcIn,
                CompositeOperator::Out => BlendMode::SrcOut,
                CompositeOperator::Atop => BlendMode::SrcATop,
                CompositeOperator::Xor => BlendMode::Xor,
                CompositeOperator::Lighter => BlendMode::Plus,
                CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
                    return (
                        ImageFilter::arithmetic([k1, k2, k3, k4], &background, &foreground),
                        linear,
                    );
                }
            };
            (ImageFilter::blend(mode, &background, &foreground), linear)
        }
        Kind::ColorMatrix(matrix) => {
            let values: [f32; 20] = match matrix.kind() {
                ColorMatrixKind::Matrix(values) => {
                    let mut out = [0.0; 20];
                    for (slot, value) in out.iter_mut().zip(values) {
                        *slot = *value;
                    }
                    out
                }
                ColorMatrixKind::Saturate(s) => saturation(s.get()),
                ColorMatrixKind::HueRotate(degrees) => hue_rotation(*degrees),
                ColorMatrixKind::LuminanceToAlpha => LUMINANCE_TO_ALPHA,
            };
            (
                ImageFilter::color_matrix(&values, &graph.input(matrix.input(), linear)),
                linear,
            )
        }
        Kind::DisplacementMap(map) => (
            ImageFilter::displacement(
                (
                    channel_of(map.x_channel_selector()),
                    channel_of(map.y_channel_selector()),
                ),
                map.scale(),
                &graph.input(map.input2(), linear),
                &graph.input(map.input1(), linear),
            ),
            linear,
        ),
        Kind::DropShadow(shadow) => (
            ImageFilter::drop_shadow(
                PointF::new(shadow.dx(), shadow.dy()),
                shadow.std_dev_x().get(),
                color_of(shadow.color(), shadow.opacity().get()),
                false,
                &graph.input(shadow.input(), linear),
            ),
            linear,
        ),
        Kind::Turbulence(turbulence) => {
            let shader = Shader::noise(
                turbulence.kind() == TurbulenceKind::Turbulence,
                (
                    turbulence.base_frequency_x().get(),
                    turbulence.base_frequency_y().get(),
                ),
                i32::try_from(turbulence.num_octaves()).unwrap_or(i32::MAX),
                opsis_types::numbers::narrow(f64::from(turbulence.seed())),
                opsis_skia::Matrix33::IDENTITY,
            );
            match shader {
                Some(shader) => (ImageFilter::shader(&shader), linear),
                None => (ImageFilter::source(), linear),
            }
        }
        Kind::ComponentTransfer(transfer) => (
            ImageFilter::table(
                &transfer_tables([
                    transfer.func_r(),
                    transfer.func_g(),
                    transfer.func_b(),
                    transfer.func_a(),
                ]),
                &graph.input(transfer.input(), linear),
            ),
            linear,
        ),
        Kind::ConvolveMatrix(convolve) => {
            let matrix = convolve.matrix();
            (
                ImageFilter::convolution(
                    &Kernel {
                        columns: i32::try_from(matrix.columns()).unwrap_or(0),
                        rows: i32::try_from(matrix.rows()).unwrap_or(0),
                        values: matrix.data(),
                        gain: 1.0 / convolve.divisor().get(),
                        bias: convolve.bias(),
                        target: PointF::new(
                            opsis_types::numbers::narrow(f64::from(matrix.target_x())),
                            opsis_types::numbers::narrow(f64::from(matrix.target_y())),
                        ),
                        tile: edge_of(convolve.edge_mode()),
                        convolve_alpha: !convolve.preserve_alpha(),
                    },
                    &graph.input(convolve.input(), linear),
                ),
                linear,
            )
        }
        Kind::Morphology(morphology) => (
            ImageFilter::morphology(
                morphology.operator() == MorphologyOperator::Dilate,
                (morphology.radius_x().get(), morphology.radius_y().get()),
                &graph.input(morphology.input(), linear),
            ),
            linear,
        ),
        Kind::Tile(tile) => (
            ImageFilter::tile(
                graph.rect_of(tile.input()),
                rect,
                &graph.input(tile.input(), linear),
            ),
            linear,
        ),
        Kind::DiffuseLighting(light) => (
            ImageFilter::lighting(
                light_of(light.light_source()),
                Lighting {
                    specular: false,
                    color: lighting_color(light.lighting_color(), linear),
                    surface_scale: light.surface_scale(),
                    constant: light.diffuse_constant(),
                    shininess: 1.0,
                },
                &graph.input(light.input(), linear),
            ),
            linear,
        ),
        Kind::SpecularLighting(light) => (
            ImageFilter::lighting(
                light_of(light.light_source()),
                Lighting {
                    specular: true,
                    color: lighting_color(light.lighting_color(), linear),
                    surface_scale: light.surface_scale(),
                    constant: light.specular_constant(),
                    shininess: light.specular_exponent(),
                },
                &graph.input(light.input(), linear),
            ),
            linear,
        ),
        Kind::Image(image) => match render(image.root(), rect) {
            Some(picture) => (
                ImageFilter::image(
                    &picture,
                    RectF::new(0.0, 0.0, rect.width, rect.height),
                    rect,
                    opsis_skia::Sampling::Linear,
                ),
                false,
            ),
            None => (ImageFilter::source(), false),
        },
    }
}

pub fn chain(filter: &usvg::filter::Filter, render: &mut Render<'_>) -> ImageFilter {
    let mut graph = Graph {
        results: HashMap::new(),
        last: None,
        rects: HashMap::new(),
        last_rect: None,
        region: rect_of(filter.rect()),
    };
    for step in filter.primitives() {
        let linear = step.color_interpolation() == ColorInterpolation::LinearRGB;
        let (made, linear) = primitive(&graph, step.kind(), (linear, rect_of(step.rect())), render);
        let cropped = ImageFilter::crop(rect_of(step.rect()), TileMode::Decal, &made);
        let result = Made {
            filter: cropped,
            linear,
        };
        graph
            .rects
            .insert(step.result().to_owned(), rect_of(step.rect()));
        graph.last_rect = Some(rect_of(step.rect()));
        graph.results.insert(
            step.result().to_owned(),
            Made {
                filter: ImageFilter::compose(&ImageFilter::source(), &result.filter),
                linear,
            },
        );
        graph.last = Some(result);
    }
    let Some(last) = graph.last else {
        return ImageFilter::source();
    };
    let srgb = converted(&last, false);
    ImageFilter::crop(rect_of(filter.rect()), TileMode::Decal, &srgb)
}
