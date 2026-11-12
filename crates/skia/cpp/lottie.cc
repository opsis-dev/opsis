#include <utility>

#include "convert.h"
#include "include/codec/SkCodec.h"
#include "include/codec/SkJpegDecoder.h"
#include "include/codec/SkPngDecoder.h"
#include "include/codec/SkWebpDecoder.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkFontMgr.h"
#include "include/ports/SkFontMgr_empty.h"
#include "modules/skottie/include/Skottie.h"
#include "modules/skresources/include/SkResources.h"
#include "opsis_skia.h"

namespace opsis::skia {

namespace {

void register_codecs() {
  static const bool registered = [] {
    SkCodecs::Register(SkPngDecoder::Decoder());
    SkCodecs::Register(SkJpegDecoder::Decoder());
    SkCodecs::Register(SkWebpDecoder::Decoder());
    return true;
  }();
  (void)registered;
}

}

Lottie::Lottie(sk_sp<skottie::Animation> animation)
    : animation_(std::move(animation)) {}

float Lottie::lottie_width() const { return animation_->size().width(); }

float Lottie::lottie_height() const { return animation_->size().height(); }

double Lottie::lottie_fps() const { return animation_->fps(); }

double Lottie::lottie_in_point() const { return animation_->inPoint(); }

double Lottie::lottie_out_point() const { return animation_->outPoint(); }

void Lottie::seek(double frame) { animation_->seekFrame(frame); }

void Lottie::render(SkCanvas* canvas, const SkRect& target) const {
  animation_->render(canvas, &target);
}

std::unique_ptr<Lottie> make_lottie(rust::Slice<const uint8_t> json) {
  register_codecs();
  auto fonts = SkFontMgr_New_Custom_Empty();
  auto resources = skresources::DataURIResourceProviderProxy::Make(
      nullptr, skresources::ImageDecodeStrategy::kPreDecode, fonts);
  sk_sp<skottie::Animation> animation =
      skottie::Animation::Builder()
          .setFontManager(fonts)
          .setResourceProvider(resources)
          .make(reinterpret_cast<const char*>(json.data()), json.size());
  if (!animation) {
    return nullptr;
  }
  return std::make_unique<Lottie>(std::move(animation));
}

void Surface::draw_lottie(const Lottie& lottie, RectF target) {
  lottie.render(canvas(), rect_of(target));
}

}
