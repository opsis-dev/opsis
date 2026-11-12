#pragma once

#include "include/core/SkBlendMode.h"
#include "include/core/SkColor.h"
#include "include/core/SkM44.h"
#include "include/core/SkMatrix.h"
#include "include/core/SkPoint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"
#include "include/core/SkSamplingOptions.h"
#include "include/core/SkTileMode.h"
#include "opsis-skia/src/bridge.rs.h"

namespace opsis::skia {

inline SkColor4f color_of(Color4 color) {
  return SkColor4f{color.red, color.green, color.blue, color.alpha};
}

inline SkRect rect_of(RectF rect) {
  return SkRect::MakeXYWH(rect.left, rect.top, rect.width, rect.height);
}

inline RectF rect_from(const SkRect& rect) {
  return RectF{rect.left(), rect.top(), rect.width(), rect.height()};
}

inline SkPoint point_of(PointF point) {
  return SkPoint::Make(point.x, point.y);
}

inline SkRRect round_rect_of(RoundRect rect) {
  SkVector radii[4] = {
      {rect.radii[0], rect.radii[1]},
      {rect.radii[2], rect.radii[3]},
      {rect.radii[4], rect.radii[5]},
      {rect.radii[6], rect.radii[7]},
  };
  SkRRect out;
  out.setRectRadii(rect_of(rect.rect), radii);
  return out;
}

inline SkMatrix matrix_of(Matrix33 matrix) {
  const auto& v = matrix.values;
  return SkMatrix::MakeAll(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7],
                           v[8]);
}

inline SkM44 matrix_of(Matrix44 matrix) {
  return SkM44::RowMajor(matrix.values.data());
}

inline SkBlendMode blend_of(BlendMode mode) {
  return static_cast<SkBlendMode>(static_cast<uint8_t>(mode));
}

inline SkTileMode tile_of(TileMode mode) {
  return static_cast<SkTileMode>(static_cast<uint8_t>(mode));
}

inline SkColorChannel channel_of(Channel channel) {
  return static_cast<SkColorChannel>(static_cast<uint8_t>(channel));
}

inline SkSamplingOptions sampling_of(Sampling sampling) {
  switch (sampling) {
    case Sampling::Nearest:
      return SkSamplingOptions(SkFilterMode::kNearest);
    case Sampling::Linear:
      return SkSamplingOptions(SkFilterMode::kLinear);
    case Sampling::LinearMipmap:
      return SkSamplingOptions(SkFilterMode::kLinear, SkMipmapMode::kLinear);
    case Sampling::Cubic:
      return SkSamplingOptions(SkCubicResampler::Mitchell());
    case Sampling::LinearMipmapNearest:
      return SkSamplingOptions(SkFilterMode::kLinear, SkMipmapMode::kNearest);
    case Sampling::CatmullRom:
      return SkSamplingOptions(SkCubicResampler::CatmullRom());
  }
  return SkSamplingOptions(SkFilterMode::kLinear);
}

}
