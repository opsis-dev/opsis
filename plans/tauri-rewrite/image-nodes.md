# Image nodes

New nodes that edit still images in Rust with the `image` crate: resize, crop, rotate and flip, adjust, blur and sharpen, place one image on another, and change the format. They run as executors in the Rust runner and pass `image` values like the other nodes.

## What it adds

- Logos, screenshots and photos fixed inside the flow (cropped, sized, straightened, sharpened, watermarked) without an outside editor.
- Photos from phones come out the right way up and in the right colours, which FFmpeg-based stills don't guarantee.

## Fit with Opsis

Already in place:

- An `image` port type, and Screenshot (`capture.still`) and Thumbnail (`media.thumbnail`, `src/shared/nodes/media.ts`) produce images; Import file gives `media` values with `kind: "image"`; an image converts to a file for any `file` input (`CONVERT`).
- Media nodes already treat images specially (still length in `media.toClip`, PNG output in `media.convert`), all through FFmpeg.
- Executors write into the node's cache folder and return values (`mediaResult()`, `projectFileRef`).

Missing:

- Any image library; nodes for still-image edits.

## The crates (checked 2026-10-25)

- `image` 0.25.10 (MIT OR Apache-2.0). Decodes PNG, JPEG, GIF, WebP, AVIF, TIFF, BMP, ICO, QOI, OpenEXR, HDR, PNM, TGA, DDS and farbfeld. Encodes PNG, JPEG, AVIF, TIFF, BMP, ICO and QOI, and WebP lossless only. AVIF decoding needs the `avif-native` feature, which uses the libdav1d C library. HEIC isn't supported.
  - `imageops` has `resize`, `crop`, `blur`, `fast_blur`, `unsharpen`, `brighten`, `contrast`, `huerotate`, `grayscale`, `invert`, `overlay`, `rotate90/180/270`, `flip_horizontal`, `flip_vertical`, `tile`, `dither`, `filter3x3` and `thumbnail`.
  - `DynamicImage::apply_orientation` rotates or flips by EXIF orientation.
  - Colour-space conversion is CICP-only (`apply_color_space`, `convert_color_space`). There is no ICC profile conversion.
- `moxcms` 0.9.1 (BSD-3-Clause OR Apache-2.0, "Simple Color Management in Rust") or `lcms2` 6.2.0 (MIT, wraps the Little CMS C library) for ICC profiles.
- `fast_image_resize` 6.1.0 (MIT OR Apache-2.0) for SIMD resizing.
- `imageproc` 0.27.0 (MIT) for rotation by any angle, drawing and filters `imageops` lacks.

## Work

1. **Loading.** One `load_image` in Rust: decode, read EXIF orientation and apply it, convert an embedded ICC profile to sRGB with `moxcms` (fall back to `lcms2` if a test profile fails), keep 16-bit and alpha where the source has them.
2. **Nodes** (`image → image`, in a new **Image** category, `CATEGORY_LABELS`):
   - **Resize image:** width, height or both, fit (inside, cover, exact), resampling quality; `fast_image_resize`.
   - **Crop image:** an area drawn on the image (the `boxes` widget over the input's frame), or margins.
   - **Rotate and flip:** 90° steps and flips, plus any angle with a background colour (`imageproc`).
   - **Adjust image:** brightness, contrast, hue, saturation, grayscale, invert.
   - **Blur and sharpen:** blur radius; unsharp amount and threshold.
   - **Place image:** a second `image` input placed on the first at a spot (the `spot` widget), with size and opacity (watermarks, logos on screenshots).
   - **Image format:** PNG, JPEG (quality), WebP (lossless), AVIF (quality, speed).
3. **Executors.** One per node in the Rust runner (`runner-and-executors.md`), writing into the node's cache folder; cache keys include the input's file stamp like every node.
4. **Cards.** Size in pixels and format, plus a warning when upscaling past 2× or when a crop is under 64 px.
5. **Copy.** Titles and descriptions in words, no library or format names beyond the file formats users choose.
6. **Tests.** Fixed inputs with EXIF orientations 1–8 and a Display P3 JPEG; compare outputs by hash, and colour samples against a reference conversion.

## Risks

- **ICC coverage.** `moxcms` may not handle every profile type; the fallback is Little CMS through `lcms2`.
- **Large photos.** A 48-megapixel photo is about 190 MB as 8-bit RGBA; resize before other operations where the node order allows.

## Sources

- crates.io API for `image`, `moxcms`, `lcms2`, `fast_image_resize`, `imageproc` (2026-10-25).
- [image codecs](https://docs.rs/image/latest/image/codecs/index.html), [imageops](https://docs.rs/image/latest/image/imageops/index.html), [`DynamicImage`](https://docs.rs/image/latest/image/enum.DynamicImage.html) (2026-10-25).
- `src/shared/nodes/media.ts` (read 2026-10-25).
