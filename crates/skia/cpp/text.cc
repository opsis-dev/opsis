#include <utility>
#include <vector>

#include "convert.h"
#include "include/core/SkData.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontArguments.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkFontTypes.h"
#include "include/core/SkPath.h"
#include "include/core/SkStream.h"
#include "include/ports/SkFontMgr_empty.h"
#include "opsis_skia.h"

namespace opsis::skia {

Typeface::Typeface(sk_sp<SkTypeface> typeface)
    : typeface_(std::move(typeface)) {}

const sk_sp<SkTypeface>& Typeface::get() const { return typeface_; }

namespace {

const sk_sp<SkFontMgr>& font_manager() {
  static const sk_sp<SkFontMgr> manager = SkFontMgr_New_Custom_Empty();
  return manager;
}

}

SkFont font_of(const Typeface& typeface, FontSpec spec) {
  SkFont font(typeface.get(), spec.size);
  font.setEdging(SkFont::Edging::kAntiAlias);
  font.setHinting(SkFontHinting::kNone);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  font.setEmbolden(spec.embolden);
  font.setSkewX(spec.skew_x);
  return font;
}

std::unique_ptr<Typeface> make_typeface(rust::Slice<const uint8_t> bytes,
                                        uint32_t index,
                                        rust::Slice<const Axis> axes) {
  std::vector<SkFontArguments::VariationPosition::Coordinate> coordinates;
  for (const Axis& axis : axes) {
    coordinates.push_back({axis.tag, axis.value});
  }
  SkFontArguments arguments;
  arguments.setCollectionIndex(static_cast<int>(index));
  arguments.setVariationDesignPosition(
      {coordinates.data(), static_cast<int>(coordinates.size())});
  sk_sp<SkTypeface> typeface = font_manager()->makeFromStream(
      SkMemoryStream::MakeCopy(bytes.data(), bytes.size()), arguments);
  if (!typeface) {
    return nullptr;
  }
  return std::make_unique<Typeface>(std::move(typeface));
}

std::unique_ptr<Path> glyph_outlines(const Typeface& typeface, FontSpec spec,
                                     rust::Slice<const uint16_t> glyphs,
                                     rust::Slice<const PointF> positions,
                                     PointF origin) {
  auto out = std::make_unique<Path>();
  SkFont font = font_of(typeface, spec);
  size_t count =
      glyphs.size() < positions.size() ? glyphs.size() : positions.size();
  for (size_t i = 0; i < count; ++i) {
    std::optional<SkPath> outline = font.getPath(glyphs[i]);
    if (!outline) {
      continue;
    }
    SkMatrix place = SkMatrix::Translate(origin.x + positions[i].x,
                                         origin.y + positions[i].y);
    out->builder().addPath(outline->makeTransform(place));
  }
  return out;
}

}
