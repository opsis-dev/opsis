#include "convert.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathMeasure.h"
#include "opsis_skia.h"

namespace opsis::skia {

Path::Path() = default;

void Path::move_to(PointF point) { builder_.moveTo(point_of(point)); }

void Path::line_to(PointF point) { builder_.lineTo(point_of(point)); }

void Path::quad_to(PointF control, PointF point) {
  builder_.quadTo(point_of(control), point_of(point));
}

void Path::cubic_to(PointF first, PointF second, PointF point) {
  builder_.cubicTo(point_of(first), point_of(second), point_of(point));
}

void Path::close() { builder_.close(); }

void Path::add_rect(RectF rect) { builder_.addRect(rect_of(rect)); }

void Path::add_round_rect(RoundRect rect) {
  builder_.addRRect(round_rect_of(rect), SkPathDirection::kCW, 0);
}

void Path::add_oval(RectF rect) { builder_.addOval(rect_of(rect)); }

void Path::add_path(const Path& other, Matrix33 matrix) {
  builder_.addPath(other.snapshot().makeTransform(matrix_of(matrix)));
}

void Path::set_even_odd(bool even_odd) {
  builder_.setFillType(even_odd ? SkPathFillType::kEvenOdd
                                : SkPathFillType::kWinding);
}

RectF Path::bounds() const { return rect_from(snapshot().getBounds()); }

float Path::length() const {
  SkPathMeasure measure(snapshot(), false);
  float total = 0;
  do {
    total += measure.getLength();
  } while (measure.nextContour());
  return total;
}

SkPath Path::snapshot() const { return builder_.snapshot(); }

SkPathBuilder& Path::builder() { return builder_; }

std::unique_ptr<Path> make_path() { return std::make_unique<Path>(); }

}
