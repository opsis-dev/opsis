#include <cstdint>
#include <memory>
#include <utility>
#include <vector>

#include "convert.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkData.h"
#include "include/core/SkImage.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkPaint.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkPathEffect.h"
#include "include/core/SkPathUtils.h"
#include "include/core/SkPixmap.h"
#include "include/core/SkSpan.h"
#include "include/core/SkStream.h"
#include "include/encode/SkPngEncoder.h"
#include "opsis_skia.h"

namespace opsis::skia {

Surface::Surface(sk_sp<SkSurface> surface) : surface_(std::move(surface)) {}

SkCanvas* Surface::canvas() const { return surface_->getCanvas(); }

void Surface::clear(Color4 color) { canvas()->clear(color_of(color)); }

void Surface::fill_rect(RectF rect, Color4 color) {
  SkPaint paint(color_of(color));
  paint.setAntiAlias(true);
  canvas()->drawRect(rect_of(rect), paint);
}

rust::Vec<uint8_t> Surface::read_rgba() const {
  SkImageInfo info =
      SkImageInfo::Make(surface_->width(), surface_->height(),
                        kRGBA_8888_SkColorType, kUnpremul_SkAlphaType);
  rust::Vec<uint8_t> out;
  size_t size = info.computeMinByteSize();
  out.reserve(size);
  for (size_t i = 0; i < size; ++i) {
    out.push_back(0);
  }
  SkPixmap target(info, out.data(), info.minRowBytes());
  surface_->readPixels(target, 0, 0);
  return out;
}

rust::Vec<uint8_t> Surface::encode_png() const {
  SkDynamicMemoryWStream stream;
  SkPixmap pixels;
  rust::Vec<uint8_t> out;
  sk_sp<SkImage> image = surface_->makeImageSnapshot();
  if (!image || !image->peekPixels(&pixels) ||
      !SkPngEncoder::Encode(&stream, pixels, {})) {
    return out;
  }
  sk_sp<SkData> data = stream.detachAsData();
  const uint8_t* bytes = data->bytes();
  out.reserve(data->size());
  for (size_t i = 0; i < data->size(); ++i) {
    out.push_back(bytes[i]);
  }
  return out;
}

int32_t Surface::width() const { return surface_->width(); }

int32_t Surface::height() const { return surface_->height(); }

std::unique_ptr<Image> Surface::snapshot() const {
  sk_sp<SkImage> image = surface_->makeImageSnapshot();
  if (!image) {
    return nullptr;
  }
  return std::make_unique<Image>(std::move(image));
}

int32_t Surface::save() { return canvas()->save(); }

void Surface::restore() { canvas()->restore(); }

void Surface::restore_to(int32_t count) { canvas()->restoreToCount(count); }

int32_t Surface::save_layer(RectF bounds, bool bounded, const Paint& paint) {
  SkRect rect = rect_of(bounds);
  return canvas()->saveLayer(bounded ? &rect : nullptr, &paint.get());
}

int32_t Surface::save_layer_backdrop(RectF bounds, bool bounded,
                                     const Paint& paint,
                                     const ImageFilter& backdrop) {
  SkRect rect = rect_of(bounds);
  SkCanvas::SaveLayerRec record(bounded ? &rect : nullptr, &paint.get(),
                                backdrop.get().get(), 0);
  return canvas()->saveLayer(record);
}

void Surface::concat(Matrix44 matrix) { canvas()->concat(matrix_of(matrix)); }

void Surface::reset_matrix() { canvas()->resetMatrix(); }

void Surface::clip_rect(RectF rect, bool anti_alias, bool outside) {
  canvas()->clipRect(rect_of(rect),
                     outside ? SkClipOp::kDifference : SkClipOp::kIntersect,
                     anti_alias);
}

void Surface::clip_round_rect(RoundRect rect, bool anti_alias) {
  canvas()->clipRRect(round_rect_of(rect), anti_alias);
}

void Surface::clip_path(const Path& path, bool anti_alias) {
  canvas()->clipPath(path.snapshot(), anti_alias);
}

void Surface::draw_rect(RectF rect, const Paint& paint) {
  canvas()->drawRect(rect_of(rect), paint.get());
}

void Surface::draw_round_rect(RoundRect rect, const Paint& paint) {
  canvas()->drawRRect(round_rect_of(rect), paint.get());
}

void Surface::draw_nested_round_rect(RoundRect outer, RoundRect inner,
                                     const Paint& paint) {
  canvas()->drawDRRect(round_rect_of(outer), round_rect_of(inner), paint.get());
}

namespace {

bool drawn_as_hairline(const SkPaint& paint, const SkMatrix& ctm) {
  const SkScalar width = paint.getStrokeWidth();
  if (paint.getStyle() != SkPaint::kStroke_Style || !paint.isAntiAlias() ||
      width <= 0) {
    return false;
  }
  const SkVector across = ctm.mapVector(width, 0);
  const SkVector down = ctm.mapVector(0, width);
  return across.length() <= 1 && down.length() <= 1;
}

}

void Surface::draw_path(const Path& path, const Paint& paint) {
  const SkPaint& stroke = paint.get();
  const SkMatrix ctm = canvas()->getTotalMatrix();
  if (!drawn_as_hairline(stroke, ctm)) {
    canvas()->drawPath(path.snapshot(), stroke);
    return;
  }
  SkPathBuilder outline;
  skpathutils::FillPathWithPaint(path.snapshot(), stroke, &outline, nullptr,
                                 ctm);
  SkPaint fill = stroke;
  fill.setStyle(SkPaint::kFill_Style);
  fill.setPathEffect(nullptr);
  canvas()->drawPath(outline.detach(), fill);
}

void Surface::draw_paint(const Paint& paint) {
  canvas()->drawPaint(paint.get());
}

void Surface::draw_glyphs(const Typeface& typeface, FontSpec font,
                          rust::Slice<const uint16_t> glyphs,
                          rust::Slice<const PointF> positions, PointF origin,
                          const Paint& paint) {
  size_t count =
      glyphs.size() < positions.size() ? glyphs.size() : positions.size();
  std::vector<SkPoint> points;
  points.reserve(count);
  for (size_t i = 0; i < count; ++i) {
    points.push_back(point_of(positions[i]));
  }
  canvas()->drawGlyphs(SkSpan<const SkGlyphID>(glyphs.data(), count),
                       SkSpan<const SkPoint>(points.data(), count),
                       point_of(origin), font_of(typeface, font), paint.get());
}

void Surface::draw_image_rect(const Image& image, RectF source, RectF target,
                              Sampling sampling, const Paint& paint) {
  canvas()->drawImageRect(image.get(), rect_of(source), rect_of(target),
                          sampling_of(sampling), &paint.get(),
                          SkCanvas::kFast_SrcRectConstraint);
}

std::unique_ptr<Surface> make_surface(int32_t width, int32_t height) {
  if (width <= 0 || height <= 0) {
    return nullptr;
  }
  sk_sp<SkSurface> surface =
      SkSurfaces::Raster(SkImageInfo::MakeN32Premul(width, height));
  if (!surface) {
    return nullptr;
  }
  return std::make_unique<Surface>(std::move(surface));
}

}
