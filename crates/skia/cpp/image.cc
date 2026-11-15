#include <utility>

#include "convert.h"
#include "include/core/SkData.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkPixmap.h"
#include "opsis_skia.h"

namespace opsis::skia {

Image::Image(sk_sp<SkImage> image) : image_(std::move(image)) {}

int32_t Image::image_width() const { return image_->width(); }

int32_t Image::image_height() const { return image_->height(); }

const sk_sp<SkImage>& Image::get() const { return image_; }

std::unique_ptr<Image> decode_image(rust::Slice<const uint8_t> bytes) {
  sk_sp<SkData> data = SkData::MakeWithCopy(bytes.data(), bytes.size());
  sk_sp<SkImage> lazy = SkImages::DeferredFromEncodedData(std::move(data));
  if (!lazy) {
    return nullptr;
  }
  sk_sp<SkImage> image = lazy->makeRasterImage();
  if (!image) {
    return nullptr;
  }
  return std::make_unique<Image>(std::move(image));
}

std::unique_ptr<Image> image_from_rgba(int32_t width, int32_t height,
                                       rust::Slice<const uint8_t> pixels) {
  if (width <= 0 || height <= 0) {
    return nullptr;
  }
  SkImageInfo info = SkImageInfo::Make(width, height, kRGBA_8888_SkColorType,
                                       kUnpremul_SkAlphaType);
  if (pixels.size() < info.computeMinByteSize()) {
    return nullptr;
  }
  SkPixmap source(info, pixels.data(), info.minRowBytes());
  sk_sp<SkImage> image = SkImages::RasterFromPixmapCopy(source);
  if (!image) {
    return nullptr;
  }
  return std::make_unique<Image>(std::move(image));
}

std::unique_ptr<Image> image_wrap_bgra(int32_t width, int32_t height,
                                       rust::Box<HeldPixels> pixels) {
  if (width <= 0 || height <= 0) {
    return nullptr;
  }
  SkImageInfo info = SkImageInfo::Make(width, height, kBGRA_8888_SkColorType,
                                       kOpaque_SkAlphaType);
  rust::Slice<const uint8_t> bytes = pixels->bytes();
  if (bytes.size() < info.computeMinByteSize()) {
    return nullptr;
  }
  auto* held = new rust::Box<HeldPixels>(std::move(pixels));
  SkPixmap source(info, bytes.data(), info.minRowBytes());
  sk_sp<SkImage> image = SkImages::RasterFromPixmap(
      source,
      [](const void*, void* context) {
        delete static_cast<rust::Box<HeldPixels>*>(context);
      },
      held);
  if (!image) {
    delete held;
    return nullptr;
  }
  return std::make_unique<Image>(std::move(image));
}

std::unique_ptr<Image> share_image(const Image& image) {
  return std::make_unique<Image>(image.get());
}

}
