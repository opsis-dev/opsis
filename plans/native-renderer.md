# Native renderer

What is left of the native renderer: Skia and Filament on every platform but Linux x64. The renderer itself is built and cut over; its design lives in [scene.md](../agent-docs/scene.md), [film.md](../agent-docs/film.md), [elements.md](../agent-docs/elements.md) and [toolchain.md](../agent-docs/toolchain.md) (Setup, clang and libc++, Skia and Filament).

Status: built and checked on Linux x64 and Linux arm64. Skia builds and passes its tests on both Macs; Windows has no Skia build yet, and Filament built from source has run on no platform yet.

## How to read this plan

- One part, **Wrappers on the other platforms**, with the sections **In place**, **Missing**, **Work**, **Risks** and **Sources**.
- Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.

## Wrappers on the other platforms

### In place

- **CI builds and runs the Rust tests on every platform but macOS Apple Silicon on every push** (`build.yml`), and runs the goldens, the Filament pass and the e2e scenarios on the same machines when started by hand (`test.yml`); Apple Silicon waits on Opsis's own FFmpeg ([plans/ffmpeg.md](ffmpeg.md)) ([packaging.md](../agent-docs/packaging.md)).
- **The first runs on the other platforms (2026-11-22)**: at `1730d190` Linux arm64 passed setup, the Skia tests, the Filament pass on lavapipe and the goldens; both Macs built Skia (its own libraries renamed by `llvm-objcopy`) and passed the Skia tests, then failed to link Filament's release archives; Windows stopped in setup unpacking with GNU tar. At `c0791ab5`, with Filament built from source, Linux arm64 passed again; both Macs failed compiling Filament's Metal backend for macOS 14.4 and Windows failed in Skia's dependency sync on long paths. Filament keeps its own macOS target and the sync runs with long paths for the next run.

- **`opsis-skia` and `opsis-filament` are the only way Rust reaches Skia and Filament**: C++ behind the `cxx` bridge for Skia, plain C functions (`opsis_filament.h`) for Filament.
- **Skia `chrome/m157` (`fef60e46`) is built by `bun run setup`** (`buildSkia`, `scripts/lib/skia.ts`): natively with clang (`buildHere`), or on Linux inside the `debian:trixie` container when `setup` is given `--container`. The build writes `consumer.json`, which `crates/skia/build.rs` compiles and links with.
- **Filament v1.80.1 is built from source by `bun run setup`** (`buildFilament`, `scripts/lib/filament.ts`) on every platform, natively or in the trixie container ([toolchain.md](../agent-docs/toolchain.md)); `crates/filament/build.rs` links its libraries from `lib/opsis` with each platform's system libraries.
- **The static Qt links its image plugins on every platform** (`LINUX_PLUGINS`, `MACOS_PLUGINS`, `WINDOWS_PLUGINS`, `crates/qt/build.rs`).
- **Measured on Linux x64**: a full-screen Filament pass at 1920×1080 read back with `readPixels` every frame takes 6.21 ms on an RTX 3060, 10.32 ms on Intel Mesa and 74.64 ms on lavapipe (Filament 1.80.1, Vulkan, no window).

### Missing

- Skia built and a test picture drawn through `opsis-skia` on Windows.
- Filament built from source on any platform; only its configure step has run, on Linux x64.
- Filament drawing without a window on macOS and Windows, on the browser's SwiftShader through its own Vulkan loader; drawing on a Mac's graphics card through Metal.
- **Skia's own libraries checked private on macOS and Windows.** `makePrivate` renames them on every platform with `llvm-objcopy` ([toolchain.md](../agent-docs/toolchain.md)), checked against GNU `objcopy` on Linux's archives and on small Mach-O and COFF archives, but never on a real Skia build there.
- Filament's libraries linked with each platform's C++ runtime: on Linux they share the app's libc++; the system libc++ on macOS and MSVC's dynamic runtime on Windows are unchecked.

### Work

1. Build Skia on Windows through `bun run setup`, and run `crates/skia/tests/surface.rs` there.
2. Check on macOS and Windows that the renamed Skia links beside the static Qt and that a JPEG opens in the editor.
3. Build Filament from source on Linux arm64, macOS and Windows, link `opsis-filament` and run `crates/filament/tests/pass.rs` without a window on the browser's SwiftShader.
4. Run the goldens (`crates/render/tests/goldens.rs`) on each platform on the software driver, and record what differs.

### Risks

- **Filament's `readPixels` is "intended for debugging and testing"** (`Renderer.h`), and every frame with a shader effect or a 3D layer needs one; the Linux measurement above shows it affordable there, and other drivers may differ.
- **Filament's libraries and the app's C++ share one C++ runtime**: on Linux the link relies on libc++ keeping its ABI in `std::__1` across versions; on macOS and Windows their runtime has to match the app's.
- **Skia's C++ API moves between milestones** (at m159 the gradient factories live only in `include/effects/SkGradient.h`); the pinned tree is m157.
- **Linking the system FreeType** can crash when its version differs from the one Skia was built with (rust-skia #1345); Skia's FreeType stays its own.

### Sources

- Filament v1.80.1 (Apache-2.0, https://github.com/google/filament): `README.md` (backends: OpenGL, Metal, Vulkan, WebGPU), `BUILDING.md` (clang, libc++, software rendering through `VK_ICD_FILENAMES`), `filament/include/filament/{Engine.h,Renderer.h,SwapChain.h}`.
- Skia (https://github.com/google/skia): `include/core/SkMilestone.h`, `gn/skia.gni`. rust-skia issue #1345.
