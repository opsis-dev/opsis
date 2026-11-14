use cxx::UniquePtr;

use crate::bridge::ffi;
use crate::image::Image;
use crate::values::{
    BlendMode, Channel, Gradient, LightSource, Lighting, Matrix33, PointF, RectF, Rgba, Sampling,
    StrokeCap, StrokeJoin, TileMode,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kernel<'k> {
    pub columns: i32,
    pub rows: i32,
    pub values: &'k [f32],
    pub gain: f32,
    pub bias: f32,
    pub target: PointF,
    pub tile: TileMode,
    pub convolve_alpha: bool,
}

pub struct Paint {
    pub(crate) inner: UniquePtr<ffi::Paint>,
}

impl std::fmt::Debug for Paint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Paint").finish_non_exhaustive()
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::new()
    }
}

impl Paint {
    pub fn new() -> Self {
        Self {
            inner: ffi::make_paint(),
        }
    }

    pub fn filled(color: Rgba) -> Self {
        let mut paint = Self::new();
        paint.set_color(color);
        paint
    }

    pub fn set_color(&mut self, color: Rgba) {
        self.inner.pin_mut().set_color(color);
    }

    pub fn set_anti_alias(&mut self, anti_alias: bool) {
        self.inner.pin_mut().set_anti_alias(anti_alias);
    }

    pub fn set_fill(&mut self) {
        self.inner.pin_mut().set_fill();
    }

    pub fn set_stroke(&mut self, width: f32, cap: StrokeCap, join: StrokeJoin, miter: f32) {
        self.inner.pin_mut().set_stroke(width, cap, join, miter);
    }

    pub fn set_blend(&mut self, mode: BlendMode) {
        self.inner.pin_mut().set_blend(mode);
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.inner.pin_mut().set_alpha(alpha);
    }

    pub fn set_shader(&mut self, shader: &Shader) {
        self.inner.pin_mut().set_shader(&shader.inner);
    }

    pub fn set_image_filter(&mut self, filter: &ImageFilter) {
        self.inner.pin_mut().set_image_filter(&filter.inner);
    }

    pub fn set_color_matrix(&mut self, matrix: &[f32; 20]) {
        self.inner.pin_mut().set_color_matrix(matrix);
    }

    pub fn set_luma(&mut self) {
        self.inner.pin_mut().set_luma();
    }

    pub fn set_mask_blur(&mut self, sigma: f32) {
        self.inner.pin_mut().set_mask_blur(sigma);
    }

    pub fn set_dither(&mut self, dither: bool) {
        self.inner.pin_mut().set_dither(dither);
    }

    pub fn set_dash(&mut self, intervals: &[f32], phase: f32) {
        self.inner.pin_mut().set_dash(intervals, phase);
    }
}

pub struct Shader {
    pub(crate) inner: UniquePtr<ffi::Shader>,
}

impl std::fmt::Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shader").finish_non_exhaustive()
    }
}

fn shader(inner: UniquePtr<ffi::Shader>) -> Option<Shader> {
    (!inner.is_null()).then_some(Shader { inner })
}

impl Shader {
    pub fn color(color: Rgba) -> Option<Self> {
        shader(ffi::color_shader(color))
    }

    pub fn linear(
        start: PointF,
        end: PointF,
        colors: &[Rgba],
        stops: &[f32],
        gradient: Gradient,
    ) -> Option<Self> {
        shader(ffi::linear_gradient(start, end, colors, stops, gradient))
    }

    pub fn radial(
        center: PointF,
        radius: f32,
        colors: &[Rgba],
        stops: &[f32],
        gradient: Gradient,
    ) -> Option<Self> {
        shader(ffi::radial_gradient(
            center, radius, colors, stops, gradient,
        ))
    }

    pub fn conical(
        (start, start_radius): (PointF, f32),
        (end, end_radius): (PointF, f32),
        colors: &[Rgba],
        stops: &[f32],
        gradient: Gradient,
    ) -> Option<Self> {
        shader(ffi::conical_gradient(
            start,
            start_radius,
            end,
            end_radius,
            colors,
            stops,
            gradient,
        ))
    }

    pub fn sweep(
        center: PointF,
        (start_angle, end_angle): (f32, f32),
        colors: &[Rgba],
        stops: &[f32],
        gradient: Gradient,
    ) -> Option<Self> {
        shader(ffi::sweep_gradient(
            center,
            start_angle,
            end_angle,
            colors,
            stops,
            gradient,
        ))
    }

    pub fn image(
        image: &Image,
        (tile_x, tile_y): (TileMode, TileMode),
        sampling: Sampling,
        local: Matrix33,
    ) -> Option<Self> {
        shader(ffi::image_shader(
            &image.inner,
            tile_x,
            tile_y,
            sampling,
            local,
        ))
    }

    pub fn noise(
        turbulence: bool,
        (frequency_x, frequency_y): (f32, f32),
        octaves: i32,
        seed: f32,
        local: Matrix33,
    ) -> Option<Self> {
        shader(ffi::noise_shader(
            turbulence,
            frequency_x,
            frequency_y,
            octaves,
            seed,
            local,
        ))
    }
}

pub struct ImageFilter {
    pub(crate) inner: UniquePtr<ffi::ImageFilter>,
}

impl std::fmt::Debug for ImageFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageFilter").finish_non_exhaustive()
    }
}

const fn filter(inner: UniquePtr<ffi::ImageFilter>) -> ImageFilter {
    ImageFilter { inner }
}

impl ImageFilter {
    pub fn source() -> Self {
        filter(ffi::source_filter())
    }

    pub fn blur(sigma_x: f32, sigma_y: f32, tile: TileMode, input: &Self) -> Self {
        filter(ffi::blur_filter(sigma_x, sigma_y, tile, &input.inner))
    }

    pub fn drop_shadow(
        offset: PointF,
        sigma: f32,
        color: Rgba,
        shadow_only: bool,
        input: &Self,
    ) -> Self {
        filter(ffi::drop_shadow_filter(
            offset,
            sigma,
            color,
            shadow_only,
            &input.inner,
        ))
    }

    pub fn color_matrix(matrix: &[f32; 20], input: &Self) -> Self {
        filter(ffi::color_matrix_filter(matrix, &input.inner))
    }

    pub fn offset(offset: PointF, input: &Self) -> Self {
        filter(ffi::offset_filter(offset, &input.inner))
    }

    pub fn merge(first: &Self, second: &Self) -> Self {
        filter(ffi::merge_filter(&first.inner, &second.inner))
    }

    pub fn blend(mode: BlendMode, background: &Self, foreground: &Self) -> Self {
        filter(ffi::blend_filter(
            mode,
            &background.inner,
            &foreground.inner,
        ))
    }

    pub fn arithmetic(k: [f32; 4], background: &Self, foreground: &Self) -> Self {
        filter(ffi::arithmetic_filter(
            k,
            &background.inner,
            &foreground.inner,
        ))
    }

    pub fn displacement(
        (x_channel, y_channel): (Channel, Channel),
        scale: f32,
        displacement: &Self,
        color: &Self,
    ) -> Self {
        filter(ffi::displacement_filter(
            x_channel,
            y_channel,
            scale,
            &displacement.inner,
            &color.inner,
        ))
    }

    pub fn image(image: &Image, source: RectF, target: RectF, sampling: Sampling) -> Self {
        filter(ffi::image_source_filter(
            &image.inner,
            source,
            target,
            sampling,
        ))
    }

    pub fn shader(shader: &Shader) -> Self {
        filter(ffi::shader_filter(&shader.inner))
    }

    pub fn to_linear(input: &Self) -> Self {
        filter(ffi::gamma_filter(true, &input.inner))
    }

    pub fn to_srgb(input: &Self) -> Self {
        filter(ffi::gamma_filter(false, &input.inner))
    }

    pub fn compose(outer: &Self, inner: &Self) -> Self {
        filter(ffi::compose_filter(&outer.inner, &inner.inner))
    }

    pub fn crop(rect: RectF, tile: TileMode, input: &Self) -> Self {
        filter(ffi::crop_filter(rect, tile, &input.inner))
    }

    pub fn transform(matrix: Matrix33, sampling: Sampling, input: &Self) -> Self {
        filter(ffi::matrix_filter(matrix, sampling, &input.inner))
    }

    pub fn table(tables: &[u8; 1024], input: &Self) -> Self {
        filter(ffi::table_filter(tables, &input.inner))
    }

    pub fn morphology(dilate: bool, (radius_x, radius_y): (f32, f32), input: &Self) -> Self {
        filter(ffi::morphology_filter(
            dilate,
            radius_x,
            radius_y,
            &input.inner,
        ))
    }

    pub fn convolution(kernel: &Kernel<'_>, input: &Self) -> Self {
        filter(ffi::convolution_filter(
            ffi::Convolution {
                columns: kernel.columns,
                rows: kernel.rows,
                gain: kernel.gain,
                bias: kernel.bias,
                target: kernel.target,
                tile: kernel.tile,
                convolve_alpha: kernel.convolve_alpha,
            },
            kernel.values,
            &input.inner,
        ))
    }

    pub fn tile(source: RectF, target: RectF, input: &Self) -> Self {
        filter(ffi::tile_filter(source, target, &input.inner))
    }

    pub fn lighting(light: LightSource, lighting: Lighting, input: &Self) -> Self {
        filter(ffi::lighting_filter(light, lighting, &input.inner))
    }
}
