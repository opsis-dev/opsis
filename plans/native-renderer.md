# Native renderer

What is left of the native renderer: Skia and Filament on every platform but Linux x64. The renderer itself is built and cut over; its design lives in [scene.md](../agent-docs/scene.md), [film.md](../agent-docs/film.md), [elements.md](../agent-docs/elements.md) and [toolchain.md](../agent-docs/toolchain.md) (Setup, clang and libc++, Skia and Filament).

Status: built and checked on Linux x64. Linux arm64, Windows and macOS have no Skia or Filament build yet, and can't be checked on the machine the renderer was built on.

## How to read this plan

- One part, **Wrappers on the other platforms**, with the sections **In place**, **Missing**, **Work**, **Risks** and **Sources**.
- Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.

## Wrappers on the other platforms

### In place

- **`opsis-skia` and `opsis-filament` are the only way Rust reaches Skia and Filament**: C++ behind the `cxx` bridge for Skia, plain C functions (`opsis_filament.h`) for Filament.
- **Skia `chrome/m157` (`fef60e46`) is built by `bun run setup`** (`buildSkia`, `scripts/lib/skia.ts`): on Linux inside the `debian:trixie` container with clang and libc++, elsewhere natively with clang (`buildHere`). The build writes `consumer.json`, which `crates/skia/build.rs` compiles and links with.
- **Filament v1.80.1 is pinned per platform** in setup's pins (`scripts/lib/setup/pins.generated.json`: Linux x64 and arm64, macOS Arm and Intel, Windows x64), and `bun run setup` unpacks this platform's archive. `crates/filament/build.rs` compiles `crates/filament/cpp`, links the archive's libraries (`LIBRARIES`, with `-lX11` on Linux) and compiles `materials/*.mat` with the archive's `matc`.
- **The static Qt links its image plugins on every platform** (`LINUX_PLUGINS`, `MACOS_PLUGINS`, `WINDOWS_PLUGINS`, `crates/qt/build.rs`).
- **Measured on Linux x64**: a full-screen Filament pass at 1920×1080 read back with `readPixels` every frame takes 6.21 ms on an RTX 3060, 10.32 ms on Intel Mesa and 74.64 ms on lavapipe (Filament 1.80.1, Vulkan, no window).

### Missing

- Skia built and a test picture drawn through `opsis-skia` on Linux arm64, Windows and macOS.
- Filament drawing without a window on macOS (Metal) and Windows, and a software driver on Windows.
- **Skia's own libraries kept private on macOS and Windows.** `makePrivate` renames the symbols of Skia's libjpeg, libpng, libwebp and FreeType only on Linux (`objcopy`); the static Qt and Skia share one program on every platform, so without it Qt's JPEG plugin and text there would bind to Skia's copies.
- Filament's archives linked with each platform's C++ runtime: on Linux they share the app's libc++; the system libc++ on macOS and MSVC's runtime on Windows are unchecked.

### Work

1. Build Skia on Linux arm64, macOS and Windows through `bun run setup`, and run `crates/skia/tests/surface.rs` on each.
2. Make Skia's own libraries private on macOS (Mach-O symbols carry a leading `_`) and Windows (COFF archives), or find that the platform's linker keeps them apart.
3. Link `opsis-filament` on macOS and Windows and run `crates/filament/tests/pass.rs` without a window; on Windows, choose and pin a software Vulkan driver.
4. Run the goldens (`crates/render/tests/goldens.rs`) on each platform on the software driver, and record what differs.

### Risks

- **Filament's `readPixels` is "intended for debugging and testing"** (`Renderer.h`), and every frame with a shader effect or a 3D layer needs one; the Linux measurement above shows it affordable there, and other drivers may differ.
- **Filament's archives and the app's C++ share one C++ runtime**: on Linux the link relies on libc++ keeping its ABI in `std::__1` across versions; on macOS and Windows the archives' runtime has to match the app's.
- **Skia's C++ API moves between milestones** (at m159 the gradient factories live only in `include/effects/SkGradient.h`); the pinned tree is m157.
- **Linking the system FreeType** can crash when its version differs from the one Skia was built with (rust-skia #1345); Skia's FreeType stays its own.

### Sources

- Filament v1.80.1 (Apache-2.0, https://github.com/google/filament): `README.md` (backends: OpenGL, Metal, Vulkan, WebGPU), `BUILDING.md` (clang, libc++, software rendering through `VK_ICD_FILENAMES`), `filament/include/filament/{Engine.h,Renderer.h,SwapChain.h}`.
- Skia (https://github.com/google/skia): `include/core/SkMilestone.h`, `gn/skia.gni`. rust-skia issue #1345.
