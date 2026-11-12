#[cxx::bridge(namespace = "opsis::skia")]
pub mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Color4 {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct RectF {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct PointF {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct RoundRect {
        rect: RectF,
        radii: [f32; 8],
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Matrix33 {
        values: [f32; 9],
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Matrix44 {
        values: [f32; 16],
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct FontSpec {
        size: f32,
        embolden: bool,
        skew_x: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Axis {
        tag: u32,
        value: f32,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum BlendMode {
        Clear,
        Src,
        Dst,
        SrcOver,
        DstOver,
        SrcIn,
        DstIn,
        SrcOut,
        DstOut,
        SrcATop,
        DstATop,
        Xor,
        Plus,
        Modulate,
        Screen,
        Overlay,
        Darken,
        Lighten,
        ColorDodge,
        ColorBurn,
        HardLight,
        SoftLight,
        Difference,
        Exclusion,
        Multiply,
        Hue,
        Saturation,
        Color,
        Luminosity,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum TileMode {
        Clamp,
        Repeat,
        Mirror,
        Decal,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum GradientSpace {
        Destination,
        SrgbLinear,
        Lab,
        Oklab,
        OklabGamutMap,
        Lch,
        Oklch,
        OklchGamutMap,
        Srgb,
        Hsl,
        Hwb,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum HueMethod {
        Shorter,
        Longer,
        Increasing,
        Decreasing,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum Sampling {
        Nearest,
        Linear,
        LinearMipmap,
        Cubic,
        LinearMipmapNearest,
        CatmullRom,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum Channel {
        Red,
        Green,
        Blue,
        Alpha,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum StrokeCap {
        Butt,
        Round,
        Square,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum StrokeJoin {
        Miter,
        Round,
        Bevel,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Gradient {
        tile: TileMode,
        space: GradientSpace,
        hue: HueMethod,
        premul: bool,
        local: Matrix33,
    }

    unsafe extern "C++" {
        include!("opsis_skia.h");

        type Surface;
        type Paint;
        type Shader;
        type ImageFilter;
        type Path;
        type Typeface;
        type Image;
        type Lottie;

        fn make_surface(width: i32, height: i32) -> UniquePtr<Surface>;
        fn clear(self: Pin<&mut Surface>, color: Color4);
        fn fill_rect(self: Pin<&mut Surface>, rect: RectF, color: Color4);
        fn read_rgba(self: &Surface) -> Vec<u8>;
        fn encode_png(self: &Surface) -> Vec<u8>;
        fn width(self: &Surface) -> i32;
        fn height(self: &Surface) -> i32;
        fn snapshot(self: &Surface) -> UniquePtr<Image>;

        fn save(self: Pin<&mut Surface>) -> i32;
        fn restore(self: Pin<&mut Surface>);
        fn restore_to(self: Pin<&mut Surface>, count: i32);
        fn save_layer(self: Pin<&mut Surface>, bounds: RectF, bounded: bool, paint: &Paint) -> i32;
        fn save_layer_backdrop(
            self: Pin<&mut Surface>,
            bounds: RectF,
            bounded: bool,
            paint: &Paint,
            backdrop: &ImageFilter,
        ) -> i32;
        fn concat(self: Pin<&mut Surface>, matrix: Matrix44);
        fn reset_matrix(self: Pin<&mut Surface>);
        fn clip_rect(self: Pin<&mut Surface>, rect: RectF, anti_alias: bool, outside: bool);
        fn clip_round_rect(self: Pin<&mut Surface>, rect: RoundRect, anti_alias: bool);
        fn clip_path(self: Pin<&mut Surface>, path: &Path, anti_alias: bool);
        fn draw_rect(self: Pin<&mut Surface>, rect: RectF, paint: &Paint);
        fn draw_round_rect(self: Pin<&mut Surface>, rect: RoundRect, paint: &Paint);
        fn draw_nested_round_rect(
            self: Pin<&mut Surface>,
            outer: RoundRect,
            inner: RoundRect,
            paint: &Paint,
        );
        fn draw_path(self: Pin<&mut Surface>, path: &Path, paint: &Paint);
        fn draw_paint(self: Pin<&mut Surface>, paint: &Paint);
        fn draw_glyphs(
            self: Pin<&mut Surface>,
            typeface: &Typeface,
            font: FontSpec,
            glyphs: &[u16],
            positions: &[PointF],
            origin: PointF,
            paint: &Paint,
        );
        fn draw_image_rect(
            self: Pin<&mut Surface>,
            image: &Image,
            source: RectF,
            target: RectF,
            sampling: Sampling,
            paint: &Paint,
        );

        fn draw_lottie(self: Pin<&mut Surface>, lottie: &Lottie, target: RectF);

        fn make_lottie(json: &[u8]) -> UniquePtr<Lottie>;
        fn lottie_width(self: &Lottie) -> f32;
        fn lottie_height(self: &Lottie) -> f32;
        fn lottie_fps(self: &Lottie) -> f64;
        fn lottie_in_point(self: &Lottie) -> f64;
        fn lottie_out_point(self: &Lottie) -> f64;
        fn seek(self: Pin<&mut Lottie>, frame: f64);

        fn make_paint() -> UniquePtr<Paint>;
        fn set_color(self: Pin<&mut Paint>, color: Color4);
        fn set_anti_alias(self: Pin<&mut Paint>, anti_alias: bool);
        fn set_fill(self: Pin<&mut Paint>);
        fn set_stroke(
            self: Pin<&mut Paint>,
            width: f32,
            cap: StrokeCap,
            join: StrokeJoin,
            miter: f32,
        );
        fn set_blend(self: Pin<&mut Paint>, mode: BlendMode);
        fn set_alpha(self: Pin<&mut Paint>, alpha: f32);
        fn set_shader(self: Pin<&mut Paint>, shader: &Shader);
        fn set_image_filter(self: Pin<&mut Paint>, filter: &ImageFilter);
        fn set_color_matrix(self: Pin<&mut Paint>, matrix: &[f32]);
        fn set_luma(self: Pin<&mut Paint>);
        fn set_mask_blur(self: Pin<&mut Paint>, sigma: f32);
        fn set_dither(self: Pin<&mut Paint>, dither: bool);
        fn set_dash(self: Pin<&mut Paint>, intervals: &[f32], phase: f32);

        fn color_shader(color: Color4) -> UniquePtr<Shader>;
        fn linear_gradient(
            start: PointF,
            end: PointF,
            colors: &[Color4],
            stops: &[f32],
            gradient: Gradient,
        ) -> UniquePtr<Shader>;
        fn radial_gradient(
            center: PointF,
            radius: f32,
            colors: &[Color4],
            stops: &[f32],
            gradient: Gradient,
        ) -> UniquePtr<Shader>;
        fn conical_gradient(
            start: PointF,
            start_radius: f32,
            end: PointF,
            end_radius: f32,
            colors: &[Color4],
            stops: &[f32],
            gradient: Gradient,
        ) -> UniquePtr<Shader>;
        fn sweep_gradient(
            center: PointF,
            start_angle: f32,
            end_angle: f32,
            colors: &[Color4],
            stops: &[f32],
            gradient: Gradient,
        ) -> UniquePtr<Shader>;
        fn image_shader(
            image: &Image,
            tile_x: TileMode,
            tile_y: TileMode,
            sampling: Sampling,
            local: Matrix33,
        ) -> UniquePtr<Shader>;
        fn noise_shader(
            turbulence: bool,
            frequency_x: f32,
            frequency_y: f32,
            octaves: i32,
            seed: f32,
            local: Matrix33,
        ) -> UniquePtr<Shader>;

        fn source_filter() -> UniquePtr<ImageFilter>;
        fn blur_filter(
            sigma_x: f32,
            sigma_y: f32,
            tile: TileMode,
            input: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;
        fn drop_shadow_filter(
            offset: PointF,
            sigma: f32,
            color: Color4,
            shadow_only: bool,
            input: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;
        fn color_matrix_filter(matrix: &[f32], input: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn offset_filter(offset: PointF, input: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn merge_filter(first: &ImageFilter, second: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn blend_filter(
            mode: BlendMode,
            background: &ImageFilter,
            foreground: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;
        fn arithmetic_filter(
            k: [f32; 4],
            background: &ImageFilter,
            foreground: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;
        fn displacement_filter(
            x_channel: Channel,
            y_channel: Channel,
            scale: f32,
            displacement: &ImageFilter,
            color: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;
        fn image_source_filter(
            image: &Image,
            source: RectF,
            target: RectF,
            sampling: Sampling,
        ) -> UniquePtr<ImageFilter>;
        fn shader_filter(shader: &Shader) -> UniquePtr<ImageFilter>;
        fn gamma_filter(to_linear: bool, input: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn compose_filter(outer: &ImageFilter, inner: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn crop_filter(rect: RectF, tile: TileMode, input: &ImageFilter) -> UniquePtr<ImageFilter>;
        fn matrix_filter(
            matrix: Matrix33,
            sampling: Sampling,
            input: &ImageFilter,
        ) -> UniquePtr<ImageFilter>;

        fn make_path() -> UniquePtr<Path>;
        fn move_to(self: Pin<&mut Path>, point: PointF);
        fn line_to(self: Pin<&mut Path>, point: PointF);
        fn quad_to(self: Pin<&mut Path>, control: PointF, point: PointF);
        fn cubic_to(self: Pin<&mut Path>, first: PointF, second: PointF, point: PointF);
        fn close(self: Pin<&mut Path>);
        fn add_rect(self: Pin<&mut Path>, rect: RectF);
        fn add_round_rect(self: Pin<&mut Path>, rect: RoundRect);
        fn add_oval(self: Pin<&mut Path>, rect: RectF);
        fn add_path(self: Pin<&mut Path>, other: &Path, matrix: Matrix33);
        fn set_even_odd(self: Pin<&mut Path>, even_odd: bool);
        fn bounds(self: &Path) -> RectF;

        fn make_typeface(bytes: &[u8], index: u32, axes: &[Axis]) -> UniquePtr<Typeface>;
        fn glyph_outlines(
            typeface: &Typeface,
            font: FontSpec,
            glyphs: &[u16],
            positions: &[PointF],
            origin: PointF,
        ) -> UniquePtr<Path>;

        fn decode_image(bytes: &[u8]) -> UniquePtr<Image>;
        fn image_from_rgba(width: i32, height: i32, pixels: &[u8]) -> UniquePtr<Image>;
        fn image_width(self: &Image) -> i32;
        fn image_height(self: &Image) -> i32;
    }
}
