#include <utility>
#include <vector>

#include "convert.h"
#include "include/core/SkBlurTypes.h"
#include "include/core/SkColorFilter.h"
#include "include/core/SkMaskFilter.h"
#include "include/core/SkPathEffect.h"
#include "include/core/SkSpan.h"
#include "include/effects/SkDashPathEffect.h"
#include "include/effects/SkGradient.h"
#include "include/effects/SkImageFilters.h"
#include "include/effects/SkLumaColorFilter.h"
#include "include/effects/SkPerlinNoiseShader.h"
#include "opsis_skia.h"

namespace opsis::skia {

Paint::Paint() { paint_.setAntiAlias(true); }

void Paint::set_color(Color4 color) { paint_.setColor4f(color_of(color)); }

void Paint::set_anti_alias(bool anti_alias) { paint_.setAntiAlias(anti_alias); }

void Paint::set_fill() { paint_.setStroke(false); }

void Paint::set_stroke(float width, StrokeCap cap, StrokeJoin join,
                       float miter) {
  paint_.setStroke(true);
  paint_.setStrokeWidth(width);
  paint_.setStrokeCap(static_cast<SkPaint::Cap>(static_cast<uint8_t>(cap)));
  paint_.setStrokeJoin(static_cast<SkPaint::Join>(static_cast<uint8_t>(join)));
  paint_.setStrokeMiter(miter);
}

void Paint::set_blend(BlendMode mode) { paint_.setBlendMode(blend_of(mode)); }

void Paint::set_alpha(float alpha) { paint_.setAlphaf(alpha); }

void Paint::set_shader(const Shader& shader) { paint_.setShader(shader.get()); }

void Paint::set_image_filter(const ImageFilter& filter) {
  paint_.setImageFilter(filter.get());
}

void Paint::set_color_matrix(rust::Slice<const float> matrix) {
  if (matrix.size() != 20) {
    return;
  }
  paint_.setColorFilter(SkColorFilters::Matrix(matrix.data()));
}

void Paint::set_luma() { paint_.setColorFilter(SkLumaColorFilter::Make()); }

void Paint::set_mask_blur(float sigma) {
  paint_.setMaskFilter(
      sigma > 0 ? SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, sigma) : nullptr);
}

void Paint::set_dither(bool dither) { paint_.setDither(dither); }

void Paint::set_dash(rust::Slice<const float> intervals, float phase) {
  paint_.setPathEffect(SkDashPathEffect::Make(
      SkSpan<const float>(intervals.data(), intervals.size()), phase));
}

const SkPaint& Paint::get() const { return paint_; }

std::unique_ptr<Paint> make_paint() { return std::make_unique<Paint>(); }

Shader::Shader(sk_sp<SkShader> shader) : shader_(std::move(shader)) {}

const sk_sp<SkShader>& Shader::get() const { return shader_; }

ImageFilter::ImageFilter(sk_sp<SkImageFilter> filter)
    : filter_(std::move(filter)) {}

const sk_sp<SkImageFilter>& ImageFilter::get() const { return filter_; }

namespace {

struct Stops {
  std::vector<SkColor4f> colors;
  std::vector<float> positions;
};

Stops stops_of(rust::Slice<const Color4> colors,
               rust::Slice<const float> stops) {
  Stops out;
  for (const Color4& color : colors) {
    out.colors.push_back(color_of(color));
  }
  if (stops.size() == colors.size()) {
    out.positions.assign(stops.begin(), stops.end());
  }
  return out;
}

SkGradient gradient_of(const Stops& stops, Gradient gradient) {
  SkGradient::Interpolation interpolation;
  interpolation.fInPremul = gradient.premul
                                ? SkGradient::Interpolation::InPremul::kYes
                                : SkGradient::Interpolation::InPremul::kNo;
  interpolation.fColorSpace =
      static_cast<SkGradient::Interpolation::ColorSpace>(
          static_cast<uint8_t>(gradient.space));
  interpolation.fHueMethod = static_cast<SkGradient::Interpolation::HueMethod>(
      static_cast<uint8_t>(gradient.hue));
  return SkGradient(
      SkGradient::Colors(
          SkSpan<const SkColor4f>(stops.colors.data(), stops.colors.size()),
          SkSpan<const float>(stops.positions.data(), stops.positions.size()),
          tile_of(gradient.tile)),
      interpolation);
}

std::unique_ptr<Shader> wrap(sk_sp<SkShader> shader) {
  if (!shader) {
    return nullptr;
  }
  return std::make_unique<Shader>(std::move(shader));
}

std::unique_ptr<ImageFilter> wrap(sk_sp<SkImageFilter> filter) {
  return std::make_unique<ImageFilter>(std::move(filter));
}

}

std::unique_ptr<Shader> color_shader(Color4 color) {
  return wrap(SkShaders::Color(color_of(color), nullptr));
}

std::unique_ptr<Shader> linear_gradient(PointF start, PointF end,
                                        rust::Slice<const Color4> colors,
                                        rust::Slice<const float> stops,
                                        Gradient gradient) {
  Stops held = stops_of(colors, stops);
  SkPoint points[2] = {point_of(start), point_of(end)};
  SkMatrix local = matrix_of(gradient.local);
  return wrap(
      SkShaders::LinearGradient(points, gradient_of(held, gradient), &local));
}

std::unique_ptr<Shader> radial_gradient(PointF center, float radius,
                                        rust::Slice<const Color4> colors,
                                        rust::Slice<const float> stops,
                                        Gradient gradient) {
  Stops held = stops_of(colors, stops);
  SkMatrix local = matrix_of(gradient.local);
  return wrap(SkShaders::RadialGradient(point_of(center), radius,
                                        gradient_of(held, gradient), &local));
}

std::unique_ptr<Shader> conical_gradient(PointF start, float start_radius,
                                         PointF end, float end_radius,
                                         rust::Slice<const Color4> colors,
                                         rust::Slice<const float> stops,
                                         Gradient gradient) {
  Stops held = stops_of(colors, stops);
  SkMatrix local = matrix_of(gradient.local);
  return wrap(SkShaders::TwoPointConicalGradient(
      point_of(start), start_radius, point_of(end), end_radius,
      gradient_of(held, gradient), &local));
}

std::unique_ptr<Shader> sweep_gradient(PointF center, float start_angle,
                                       float end_angle,
                                       rust::Slice<const Color4> colors,
                                       rust::Slice<const float> stops,
                                       Gradient gradient) {
  Stops held = stops_of(colors, stops);
  SkMatrix local = matrix_of(gradient.local);
  return wrap(SkShaders::SweepGradient(point_of(center), start_angle, end_angle,
                                       gradient_of(held, gradient), &local));
}

std::unique_ptr<Shader> image_shader(const Image& image, TileMode tile_x,
                                     TileMode tile_y, Sampling sampling,
                                     Matrix33 local) {
  SkMatrix matrix = matrix_of(local);
  return wrap(image.get()->makeShader(tile_of(tile_x), tile_of(tile_y),
                                      sampling_of(sampling), &matrix));
}

std::unique_ptr<Shader> noise_shader(bool turbulence, float frequency_x,
                                     float frequency_y, int32_t octaves,
                                     float seed, Matrix33 local) {
  sk_sp<SkShader> noise =
      turbulence
          ? SkShaders::MakeTurbulence(frequency_x, frequency_y, octaves, seed)
          : SkShaders::MakeFractalNoise(frequency_x, frequency_y, octaves,
                                        seed);
  if (!noise) {
    return nullptr;
  }
  return wrap(noise->makeWithLocalMatrix(matrix_of(local)));
}

std::unique_ptr<ImageFilter> source_filter() {
  return wrap(sk_sp<SkImageFilter>());
}

std::unique_ptr<ImageFilter> blur_filter(float sigma_x, float sigma_y,
                                         TileMode tile,
                                         const ImageFilter& input) {
  return wrap(
      SkImageFilters::Blur(sigma_x, sigma_y, tile_of(tile), input.get()));
}

std::unique_ptr<ImageFilter> drop_shadow_filter(PointF offset, float sigma,
                                                Color4 color, bool shadow_only,
                                                const ImageFilter& input) {
  if (shadow_only) {
    return wrap(SkImageFilters::DropShadowOnly(offset.x, offset.y, sigma, sigma,
                                               color_of(color), nullptr,
                                               input.get()));
  }
  return wrap(SkImageFilters::DropShadow(
      offset.x, offset.y, sigma, sigma, color_of(color), nullptr, input.get()));
}

std::unique_ptr<ImageFilter> color_matrix_filter(
    rust::Slice<const float> matrix, const ImageFilter& input) {
  if (matrix.size() != 20) {
    return wrap(input.get());
  }
  return wrap(SkImageFilters::ColorFilter(SkColorFilters::Matrix(matrix.data()),
                                          input.get()));
}

std::unique_ptr<ImageFilter> offset_filter(PointF offset,
                                           const ImageFilter& input) {
  return wrap(SkImageFilters::Offset(offset.x, offset.y, input.get()));
}

std::unique_ptr<ImageFilter> merge_filter(const ImageFilter& first,
                                          const ImageFilter& second) {
  return wrap(SkImageFilters::Merge(first.get(), second.get()));
}

std::unique_ptr<ImageFilter> blend_filter(BlendMode mode,
                                          const ImageFilter& background,
                                          const ImageFilter& foreground) {
  return wrap(SkImageFilters::Blend(blend_of(mode), background.get(),
                                    foreground.get()));
}

std::unique_ptr<ImageFilter> arithmetic_filter(std::array<float, 4> k,
                                               const ImageFilter& background,
                                               const ImageFilter& foreground) {
  return wrap(SkImageFilters::Arithmetic(k[0], k[1], k[2], k[3], true,
                                         background.get(), foreground.get()));
}

std::unique_ptr<ImageFilter> displacement_filter(
    Channel x_channel, Channel y_channel, float scale,
    const ImageFilter& displacement, const ImageFilter& color) {
  return wrap(SkImageFilters::DisplacementMap(channel_of(x_channel),
                                              channel_of(y_channel), scale,
                                              displacement.get(), color.get()));
}

std::unique_ptr<ImageFilter> image_source_filter(const Image& image,
                                                 RectF source, RectF target,
                                                 Sampling sampling) {
  return wrap(SkImageFilters::Image(image.get(), rect_of(source),
                                    rect_of(target), sampling_of(sampling)));
}

std::unique_ptr<ImageFilter> shader_filter(const Shader& shader) {
  return wrap(SkImageFilters::Shader(shader.get()));
}

std::unique_ptr<ImageFilter> gamma_filter(bool to_linear,
                                          const ImageFilter& input) {
  return wrap(SkImageFilters::ColorFilter(
      to_linear ? SkColorFilters::SRGBToLinearGamma()
                : SkColorFilters::LinearToSRGBGamma(),
      input.get()));
}

std::unique_ptr<ImageFilter> compose_filter(const ImageFilter& outer,
                                            const ImageFilter& inner) {
  return wrap(SkImageFilters::Compose(outer.get(), inner.get()));
}

std::unique_ptr<ImageFilter> crop_filter(RectF rect, TileMode tile,
                                         const ImageFilter& input) {
  return wrap(SkImageFilters::Crop(rect_of(rect), tile_of(tile), input.get()));
}

std::unique_ptr<ImageFilter> matrix_filter(Matrix33 matrix, Sampling sampling,
                                           const ImageFilter& input) {
  return wrap(SkImageFilters::MatrixTransform(
      matrix_of(matrix), sampling_of(sampling), input.get()));
}

}
