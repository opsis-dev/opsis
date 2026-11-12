use std::collections::HashMap;

use opsis_skia::{BlendMode, Channel, ImageFilter, PointF, RectF, Rgba, Shader, TileMode};
use usvg::filter::{
    ColorChannel, ColorInterpolation, ColorMatrixKind, CompositeOperator, Input, Kind,
    TurbulenceKind,
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
        Kind::ComponentTransfer(transfer) => (graph.input(transfer.input(), linear), linear),
        Kind::ConvolveMatrix(convolve) => (graph.input(convolve.input(), linear), linear),
        Kind::Morphology(morphology) => (graph.input(morphology.input(), linear), linear),
        Kind::Tile(tile) => (graph.input(tile.input(), linear), linear),
        Kind::DiffuseLighting(light) => (graph.input(light.input(), linear), linear),
        Kind::SpecularLighting(light) => (graph.input(light.input(), linear), linear),
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
    };
    for step in filter.primitives() {
        let linear = step.color_interpolation() == ColorInterpolation::LinearRGB;
        let (made, linear) = primitive(&graph, step.kind(), (linear, rect_of(step.rect())), render);
        let cropped = ImageFilter::crop(rect_of(step.rect()), TileMode::Decal, &made);
        let result = Made {
            filter: cropped,
            linear,
        };
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
