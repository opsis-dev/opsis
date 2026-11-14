#pragma once

#include <array>
#include <cstdint>
#include <memory>

#include "include/core/SkFont.h"
#include "include/core/SkImage.h"
#include "include/core/SkImageFilter.h"
#include "include/core/SkPaint.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkRefCnt.h"
#include "include/core/SkShader.h"
#include "include/core/SkSurface.h"
#include "include/core/SkTypeface.h"
#include "modules/skottie/include/Skottie.h"
#include "rust/cxx.h"

namespace opsis::skia {
struct Color4;
struct RectF;
struct PointF;
struct LightSource;
struct Convolution;
struct Lighting;
struct RoundRect;
struct Matrix33;
struct Matrix44;
struct FontSpec;
struct Axis;
struct Gradient;
enum class BlendMode : uint8_t;
enum class TileMode : uint8_t;
enum class Sampling : uint8_t;
enum class Channel : uint8_t;
enum class StrokeCap : uint8_t;
enum class StrokeJoin : uint8_t;
}

namespace opsis::skia {

class Image {
 public:
  explicit Image(sk_sp<SkImage> image);

  int32_t image_width() const;
  int32_t image_height() const;
  const sk_sp<SkImage>& get() const;

 private:
  sk_sp<SkImage> image_;
};

class Shader {
 public:
  explicit Shader(sk_sp<SkShader> shader);

  const sk_sp<SkShader>& get() const;

 private:
  sk_sp<SkShader> shader_;
};

class ImageFilter {
 public:
  explicit ImageFilter(sk_sp<SkImageFilter> filter);

  const sk_sp<SkImageFilter>& get() const;

 private:
  sk_sp<SkImageFilter> filter_;
};

class Paint {
 public:
  Paint();

  void set_color(Color4 color);
  void set_anti_alias(bool anti_alias);
  void set_fill();
  void set_stroke(float width, StrokeCap cap, StrokeJoin join, float miter);
  void set_blend(BlendMode mode);
  void set_alpha(float alpha);
  void set_shader(const Shader& shader);
  void set_image_filter(const ImageFilter& filter);
  void set_color_matrix(rust::Slice<const float> matrix);
  void set_luma();
  void set_mask_blur(float sigma);
  void set_dither(bool dither);
  void set_dash(rust::Slice<const float> intervals, float phase);
  const SkPaint& get() const;

 private:
  SkPaint paint_;
};

class Path {
 public:
  Path();

  void move_to(PointF point);
  void line_to(PointF point);
  void quad_to(PointF control, PointF point);
  void cubic_to(PointF first, PointF second, PointF point);
  void close();
  void add_rect(RectF rect);
  void add_round_rect(RoundRect rect);
  void add_oval(RectF rect);
  void add_path(const Path& other, Matrix33 matrix);
  void set_even_odd(bool even_odd);
  RectF bounds() const;
  float length() const;
  SkPath snapshot() const;
  SkPathBuilder& builder();

 private:
  SkPathBuilder builder_;
};

class Typeface {
 public:
  explicit Typeface(sk_sp<SkTypeface> typeface);

  const sk_sp<SkTypeface>& get() const;

 private:
  sk_sp<SkTypeface> typeface_;
};

class Lottie {
 public:
  explicit Lottie(sk_sp<skottie::Animation> animation);

  float lottie_width() const;
  float lottie_height() const;
  double lottie_fps() const;
  double lottie_in_point() const;
  double lottie_out_point() const;
  void seek(double frame);
  void render(SkCanvas* canvas, const SkRect& target) const;

 private:
  sk_sp<skottie::Animation> animation_;
};

class Surface {
 public:
  explicit Surface(sk_sp<SkSurface> surface);

  void clear(Color4 color);
  void fill_rect(RectF rect, Color4 color);
  rust::Vec<uint8_t> read_rgba() const;
  rust::Vec<uint8_t> encode_png() const;
  int32_t width() const;
  int32_t height() const;
  std::unique_ptr<Image> snapshot() const;

  int32_t save();
  void restore();
  void restore_to(int32_t count);
  int32_t save_layer(RectF bounds, bool bounded, const Paint& paint);
  int32_t save_layer_backdrop(RectF bounds, bool bounded, const Paint& paint,
                              const ImageFilter& backdrop);
  void concat(Matrix44 matrix);
  void reset_matrix();
  void clip_rect(RectF rect, bool anti_alias, bool outside);
  void clip_round_rect(RoundRect rect, bool anti_alias);
  void clip_path(const Path& path, bool anti_alias);
  void draw_rect(RectF rect, const Paint& paint);
  void draw_round_rect(RoundRect rect, const Paint& paint);
  void draw_nested_round_rect(RoundRect outer, RoundRect inner,
                              const Paint& paint);
  void draw_path(const Path& path, const Paint& paint);
  void draw_paint(const Paint& paint);
  void draw_glyphs(const Typeface& typeface, FontSpec font,
                   rust::Slice<const uint16_t> glyphs,
                   rust::Slice<const PointF> positions, PointF origin,
                   const Paint& paint);
  void draw_image_rect(const Image& image, RectF source, RectF target,
                       Sampling sampling, const Paint& paint);
  void draw_lottie(const Lottie& lottie, RectF target);

 private:
  SkCanvas* canvas() const;

  sk_sp<SkSurface> surface_;
};

std::unique_ptr<Surface> make_surface(int32_t width, int32_t height);
std::unique_ptr<Paint> make_paint();

std::unique_ptr<Shader> color_shader(Color4 color);
std::unique_ptr<Shader> linear_gradient(PointF start, PointF end,
                                        rust::Slice<const Color4> colors,
                                        rust::Slice<const float> stops,
                                        Gradient gradient);
std::unique_ptr<Shader> radial_gradient(PointF center, float radius,
                                        rust::Slice<const Color4> colors,
                                        rust::Slice<const float> stops,
                                        Gradient gradient);
std::unique_ptr<Shader> conical_gradient(PointF start, float start_radius,
                                         PointF end, float end_radius,
                                         rust::Slice<const Color4> colors,
                                         rust::Slice<const float> stops,
                                         Gradient gradient);
std::unique_ptr<Shader> sweep_gradient(PointF center, float start_angle,
                                       float end_angle,
                                       rust::Slice<const Color4> colors,
                                       rust::Slice<const float> stops,
                                       Gradient gradient);
std::unique_ptr<Shader> image_shader(const Image& image, TileMode tile_x,
                                     TileMode tile_y, Sampling sampling,
                                     Matrix33 local);
std::unique_ptr<Shader> noise_shader(bool turbulence, float frequency_x,
                                     float frequency_y, int32_t octaves,
                                     float seed, Matrix33 local);

std::unique_ptr<ImageFilter> source_filter();
std::unique_ptr<ImageFilter> blur_filter(float sigma_x, float sigma_y,
                                         TileMode tile,
                                         const ImageFilter& input);
std::unique_ptr<ImageFilter> drop_shadow_filter(PointF offset, float sigma,
                                                Color4 color, bool shadow_only,
                                                const ImageFilter& input);
std::unique_ptr<ImageFilter> color_matrix_filter(
    rust::Slice<const float> matrix, const ImageFilter& input);
std::unique_ptr<ImageFilter> offset_filter(PointF offset,
                                           const ImageFilter& input);
std::unique_ptr<ImageFilter> merge_filter(const ImageFilter& first,
                                          const ImageFilter& second);
std::unique_ptr<ImageFilter> blend_filter(BlendMode mode,
                                          const ImageFilter& background,
                                          const ImageFilter& foreground);
std::unique_ptr<ImageFilter> arithmetic_filter(std::array<float, 4> k,
                                               const ImageFilter& background,
                                               const ImageFilter& foreground);
std::unique_ptr<ImageFilter> displacement_filter(
    Channel x_channel, Channel y_channel, float scale,
    const ImageFilter& displacement, const ImageFilter& color);
std::unique_ptr<ImageFilter> image_source_filter(const Image& image,
                                                 RectF source, RectF target,
                                                 Sampling sampling);
std::unique_ptr<ImageFilter> shader_filter(const Shader& shader);
std::unique_ptr<ImageFilter> gamma_filter(bool to_linear,
                                          const ImageFilter& input);
std::unique_ptr<ImageFilter> compose_filter(const ImageFilter& outer,
                                            const ImageFilter& inner);
std::unique_ptr<ImageFilter> crop_filter(RectF rect, TileMode tile,
                                         const ImageFilter& input);
std::unique_ptr<ImageFilter> matrix_filter(Matrix33 matrix, Sampling sampling,
                                           const ImageFilter& input);
std::unique_ptr<ImageFilter> table_filter(rust::Slice<const uint8_t> tables,
                                          const ImageFilter& input);
std::unique_ptr<ImageFilter> morphology_filter(bool dilate, float radius_x,
                                               float radius_y,
                                               const ImageFilter& input);
std::unique_ptr<ImageFilter> convolution_filter(Convolution convolution,
                                                rust::Slice<const float> kernel,
                                                const ImageFilter& input);
std::unique_ptr<ImageFilter> tile_filter(RectF source, RectF target,
                                         const ImageFilter& input);
std::unique_ptr<ImageFilter> lighting_filter(LightSource light,
                                             Lighting lighting,
                                             const ImageFilter& input);

std::unique_ptr<Path> make_path();

std::unique_ptr<Typeface> make_typeface(rust::Slice<const uint8_t> bytes,
                                        uint32_t index,
                                        rust::Slice<const Axis> axes);
std::unique_ptr<Path> glyph_outlines(const Typeface& typeface, FontSpec font,
                                     rust::Slice<const uint16_t> glyphs,
                                     rust::Slice<const PointF> positions,
                                     PointF origin);

SkFont font_of(const Typeface& typeface, FontSpec spec);

std::unique_ptr<Lottie> make_lottie(rust::Slice<const uint8_t> json);

std::unique_ptr<Image> decode_image(rust::Slice<const uint8_t> bytes);
std::unique_ptr<Image> image_from_rgba(int32_t width, int32_t height,
                                       rust::Slice<const uint8_t> pixels);

}
